use x86_64::instructions::port::Port;

use crate::sys::mem::allocator::PhysBuf;

// 00 = 8K + 16 bytes
// 01 = 16K + 16 bytes
// 10 = 32K + 16 bytes
// 11 = 64K + 16 bytes
const RX_BUFFER_IDX: usize = 0;

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
}
