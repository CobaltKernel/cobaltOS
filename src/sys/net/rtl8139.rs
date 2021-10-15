use core::convert::TryInto;

use alloc::{collections::BTreeMap, vec::Vec};
use array_macro::array;

use smoltcp::{iface::{EthernetInterfaceBuilder, NeighborCache, Routes}, phy::{self, Device, DeviceCapabilities}, wire::{EthernetAddress, IpCidr, Ipv4Address, Ipv4Packet}};
use x86_64::instructions::port::Port;

use crate::{breakpoint, debug, log, sys::{self, mem::allocator::PhysBuf}};


//const CRS: u32 = 1 << 31; // Carrier Sense Lost
//const TAB: u32 = 1 << 30; // Transmit Abort
//const OWC: u32 = 1 << 29; // Out of Window Collision
//const CDH: u32 = 1 << 28; // CD Heart Beat
const TOK: u32 = 1 << 15; // Transmit OK
//const TUN: u32 = 1 << 14; // Transmit FIFO Underrun
const OWN: u32 = 1 << 13; // DMA operation completed


// 00 = 8K + 16 bytes
// 01 = 16K + 16 bytes
// 10 = 32K + 16 bytes
// 11 = 64K + 16 bytes
const RX_BUFFER_IDX: usize = 0b00;

const MTU: usize = 1500;

const RX_BUFFER_PAD: usize = 16;
const RX_BUFFER_LEN: usize = (8129 << RX_BUFFER_IDX) + RX_BUFFER_PAD;

const TX_BUFFER_LEN: usize = 4096;
const TX_BUFFERS_COUNT: usize = 4;
const ROK: u16 = 0x01;

const CR_RST: u8 = 1 << 4; // Reset
const CR_RE: u8 = 1 << 3; // Receiver Enable
const CR_TE: u8 = 1 << 2; // Transmitter Enable
const CR_BUFE: u8 = 1 << 0; // Buffer Empty

// Rx Buffer Length
const RCR_RBLEN: u32 = (RX_BUFFER_IDX << 11) as u32;

// When the WRAP bit is set, the nic will keep moving the rest
// of the packet data into the memory immediately after the
// end of the Rx buffer instead of going back to the begining
// of the buffer. So the buffer must have an additionnal 1500 bytes.
const RCR_WRAP: u32 = 1 << 7;

const RCR_AB: u32 = 1 << 3; // Accept Broadcast packets
const RCR_AM: u32 = 1 << 2; // Accept Multicast packets
const RCR_APM: u32 = 1 << 1; // Accept Physical Match packets
const RCR_AAP: u32 = 1 << 0; // Accept All Packets

// Interframe Gap Time
const TCR_IFG: u32 = 3 << 24;

// Max DMA Burst Size per Tx DMA Burst
// 000 = 16 bytes
// 001 = 32 bytes
// 010 = 64 bytes
// 011 = 128 bytes
// 100 = 256 bytes
// 101 = 512 bytes
// 110 = 1024 bytes
// 111 = 2048 bytes
const TCR_MXDMA0: u32 = 1 << 8;
const TCR_MXDMA1: u32 = 1 << 9;
const TCR_MXDMA2: u32 = 1 << 10;

// Interrupt Mask Register
const IMR_TOK: u16 = 1 << 2; // Transmit OK Interrupt
const IMR_ROK: u16 = 1 << 0; // Receive OK Interrupt

#[derive(Debug, Clone)]
pub struct Ports {
    pub mac: [Port<u8>; 6],                      // ID Registers (IDR0 ... IDR5)
    pub tx_cmds: [Port<u32>; TX_BUFFERS_COUNT],  // Transmit Status of Descriptors (TSD0 .. TSD3)
    pub tx_addrs: [Port<u32>; TX_BUFFERS_COUNT], // Transmit Start Address of Descriptor0 (TSAD0 .. TSAD3)
    pub config1: Port<u8>,                       // Configuration Register 1 (CONFIG1)
    pub rx_addr: Port<u32>,                      // Receive (Rx) Buffer Start Address (RBSTART)
    pub capr: Port<u16>,                         // Current Address of Packet Read (CAPR)
    pub cbr: Port<u16>,                          // Current Buffer Address (CBR)
    pub cmd: Port<u8>,                           // Command Register (CR)
    pub imr: Port<u16>,                          // Interrupt Mask Register (IMR)
    pub isr: Port<u16>,                          // Interrupt Status Register (ISR)
    pub tx_config: Port<u32>,                    // Transmit (Tx) Configuration Register (TCR)
    pub rx_config: Port<u32>,                    // Receive (Rx) Configuration Register (RCR)
}

impl Ports {
    pub fn new(io_base: u16) -> Self {
        Self {
            mac: [
                Port::new(io_base + 0x00),
                Port::new(io_base + 0x01),
                Port::new(io_base + 0x02),
                Port::new(io_base + 0x03),
                Port::new(io_base + 0x04),
                Port::new(io_base + 0x05),
            ],
            tx_cmds: [
                Port::new(io_base + 0x10),
                Port::new(io_base + 0x14),
                Port::new(io_base + 0x18),
                Port::new(io_base + 0x1C),
            ],
            tx_addrs: [
                Port::new(io_base + 0x20),
                Port::new(io_base + 0x24),
                Port::new(io_base + 0x28),
                Port::new(io_base + 0x2C),
            ],
            config1: Port::new(io_base + 0x52),
            rx_addr: Port::new(io_base + 0x30),
            capr: Port::new(io_base + 0x38),
            cbr: Port::new(io_base + 0x3A),
            cmd: Port::new(io_base + 0x37),
            imr: Port::new(io_base + 0x3C),
            isr: Port::new(io_base + 0x3E),
            tx_config: Port::new(io_base + 0x40),
            rx_config: Port::new(io_base + 0x44),
        }
    }

    pub fn mac(&mut self) -> [u8; 6] {
        unsafe {
            [
            self.mac[0].read(),
            self.mac[1].read(),
            self.mac[2].read(),
            self.mac[3].read(),
            self.mac[4].read(),
            self.mac[5].read()
            ]
        }
    }
}

#[derive(Debug, Clone)]
pub struct RTL8139 {
    ports: Ports,
    rx_buffer: PhysBuf,
    rx_offset: usize,
    tx_buffers: [PhysBuf; TX_BUFFERS_COUNT],
    tx_id: usize,

    eth_addr: Option<EthernetAddress>,
}

impl RTL8139 {
    pub fn new(io_base: u16) -> Self {
        Self {
            ports: Ports::new(io_base),
            rx_buffer: PhysBuf::new(MTU + RX_BUFFER_LEN),
            tx_buffers: array![PhysBuf::new(TX_BUFFER_LEN); TX_BUFFERS_COUNT],
            rx_offset: 0,
            tx_id: TX_BUFFERS_COUNT - 1,
            eth_addr: None,
        }
    }

    pub unsafe fn init(&mut self) {
            // Power On The Card
            self.ports.config1.write(0);

            // Software Reset
            self.ports.cmd.write(CR_RST);
            while self.ports.cmd.read() & CR_RST != 0 {}

            // Get the Ethernet Address From the MAC Address
            self.eth_addr = Some(EthernetAddress::from_bytes(&self.ports.mac()));

            // Set The Receiving Buffer's Address
            let rx_addr = self.rx_buffer.addr();
            self.ports.rx_addr.write(rx_addr as u32);

            // Set the Transmitting Buffer's Addresses
            for i in 0..4 {
                let addr = self.tx_buffers[i].addr();

                self.ports.tx_addrs[i].write(addr as u32);
            }

            self.ports.imr.write(IMR_ROK | IMR_TOK);

            self.ports.rx_config.write(RCR_RBLEN | RCR_WRAP | RCR_AB | RCR_AM | RCR_APM | RCR_AAP);

            self.ports.tx_config.write(TCR_IFG | TCR_MXDMA0 | TCR_MXDMA1 | TCR_MXDMA2);

            // Enable Receive & Transmit
            self.ports.cmd.write(CR_RE | CR_TE);
    }

    pub fn send(&mut self, data: &[u8]) {
        let len = data.len();
        let tx_id = self.tx_id;
        let buf = &mut self.tx_buffers[tx_id][0..len];

        debug!("Copying Data Into Buffer.");
        for index in 0..len {
            buf[index] = data[index];
        }

        let mut cmd_port = self.ports.tx_cmds[tx_id].clone();
        unsafe {
            debug!("Writing Length {} Bytes.", len);
            cmd_port.write(0x1FFF & len as u32);

            debug!("Waiting For DMA");
            let old_cmd = cmd_port.read();
            cmd_port.write(old_cmd & !OWN);
            debug!("CMD: {:032b}", cmd_port.read());
            while cmd_port.read() & OWN != OWN {}
            // 5. When the whole packet is moved to line, the TOK bit is
            // set to 1.
            debug!("Waiting For TOK");
            while cmd_port.read() & TOK != TOK {}
        }
    }

    pub fn recv(&mut self, _data: &mut [u8]) -> usize {
        todo!()
    }
}


#[doc(hidden)]
pub struct RxToken {
    buffer: Vec<u8>,
}

#[doc(hidden)]
pub struct TxToken {
    device: RTL8139
}


impl<'a> Device<'a> for RTL8139 {
    type RxToken = RxToken;
    type TxToken = TxToken;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let cmd = unsafe {self.ports.cmd.read()};
        if cmd & CR_BUFE == CR_BUFE {return None};
        let capr = unsafe { self.ports.capr.read() };
        let cbr = unsafe { self.ports.cbr.read() };

        debug!("CAPR: {}", capr);
        debug!("CBR: {}", cbr);

        breakpoint!();

        let offset = ((capr as usize) + RX_BUFFER_PAD) % (1 << 16);
        let header = u16::from_le_bytes(self.rx_buffer[(offset + 0)..(offset + 2)].try_into().unwrap());
        debug!("RECV: Header: 0x{:04x}.", header);
        debug!("Offset: {}", offset);

        debug!("Buffer[offset+0..offset + 16]: {:?}", &self.rx_buffer[(offset + 0)..(offset + 16)]);

        if header & ROK != ROK {
            debug!("Header Type Is Not ROK, Aborting!");
            breakpoint!();
            unsafe { self.ports.capr.write(cbr); }
            return None;
        };

        let n = u16::from_le_bytes(self.rx_buffer[(offset + 2)..(offset + 4)].try_into().unwrap()) as usize;
        #[allow(unused)]
        let len = n - 4;
        debug!("Packet Size: {} Bytes.", len);

        self.rx_offset = (offset + n + 4 + 3) & !3;
        unsafe {
            self.ports.capr.write((self.rx_offset - RX_BUFFER_PAD) as u16);
        }

        let rx = RxToken {
            buffer: self.rx_buffer[(offset + 4)..(offset + n)].to_vec()
        };
        let tx = TxToken {
            device: self.clone()
        };

        Some((rx, tx))
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        //let isr = unsafe { self.ports.isr.read() };
        self.tx_id = (self.tx_id + 1) % TX_BUFFERS_COUNT;

            debug!("{}", "-".repeat(66));
            debug!("NET RTL8139 Transmitting:");
            debug!("TX Buffer:{}", self.tx_id);
            //printk!("Interrupt Status Register: {:#02X}\n", isr);

        let tx = TxToken {
            device: self.clone()
        };

        Some(tx)
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = Some(1);
        caps
    }
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(mut self, _timestamp: smoltcp::time::Instant, len: usize, f: F) -> smoltcp::Result<R>
        where F: FnOnce(&mut [u8]) -> smoltcp::Result<R> {
            let tx_id = self.device.tx_id;
            let mut buf = &mut self.device.tx_buffers[tx_id][0..len];

            debug!("Transmitting {} Bytes...", buf.len());
            let mut _packet= Ipv4Packet::new_unchecked(&buf);
            debug!("Packet Protocol: {:?}", packet.protocol());
    
            // 1. Copy the packet to a physically contiguous buffer in memory.
            let res = f(&mut buf);
    
            // 2. Fill in Start Address(physical address) of this buffer.
            // NOTE: This has was done during init
    
            if res.is_ok() {
                debug!("Transmitting Packet...");
                breakpoint!();
                let mut cmd_port = self.device.ports.tx_cmds[tx_id].clone();
                unsafe {
                    // 3. Fill in Transmit Status: the size of this packet, the
                    // early transmit threshold, and clear OWN bit in TSD (this
                    // starts the PCI operation).
                    // NOTE: The length of the packet use the first 13 bits (but
                    // should not exceed 1792 bytes), and a value of 0x000000
                    // for the early transmit threshold means 8 bytes. So we
                    // just write the size of the packet.
                    cmd_port.write(0x1FFF & len as u32);
    
                    // 4. When the whole packet is moved to FIFO, the OWN bit is
                    // set to 1.
                    while cmd_port.read() & OWN != OWN {}
                    // 5. When the whole packet is moved to line, the TOK bit is
                    // set to 1.
                    while cmd_port.read() & TOK != TOK {}


                }
            }
            res
    }
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, _timestamp: smoltcp::time::Instant, f: F) -> smoltcp::Result<R>
        where F: FnOnce(&mut [u8]) -> smoltcp::Result<R> {
            debug!("[{ }] Revcuieb", timestamp);
            f(&mut self.buffer)
    }
}

pub fn init() {
    if let Some(mut pci_device) = sys::pci::find_device(0x10EC, 0x8139) {
        pci_device.enable_bus_mastering();

        let io_base = (pci_device.base_addresses[0] as u16) & 0xFFF0;
        let mut net_device = RTL8139::new(io_base);

        unsafe {
            net_device.init();
        }

        if let Some(eth_addr) = net_device.eth_addr {
            log!("NET RTL8139 MAC {}\n", eth_addr);

            let neighbor_cache = NeighborCache::new(BTreeMap::new());
            let routes = Routes::new(BTreeMap::new());
            let ip_addrs = [
                IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0),
            ];
            let iface = EthernetInterfaceBuilder::new(net_device).
                ethernet_addr(eth_addr).
                neighbor_cache(neighbor_cache).
                ip_addrs(ip_addrs).
                routes(routes).
                finalize();


            //log!("Found {} IPs [{:?}]", iface.ip_addrs().len(), iface.ip_addrs());

            *sys::net::IFACE.lock() = Some(iface);
        }
    }
}



pub fn interrupt_handler() {
    debug!("RTL8139 interrupt!\n");
    if let Some(mut guard) = sys::net::IFACE.try_lock() {
        if let Some(ref mut iface) = *guard {
            unsafe { iface.device_mut().ports.isr.write(0xffff) } // Clear the interrupt
        }
    }
}