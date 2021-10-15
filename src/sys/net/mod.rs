pub mod dhcp;

use core::fmt::Write;

use alloc::string::String;
use lazy_static::lazy_static;
use smoltcp::{socket::TcpSocket, wire::{EthernetAddress, IpAddress, Ipv4Address}};
use spin::Mutex;

pub type EthernetInterface<T> = smoltcp::iface::EthernetInterface<'static, T>;

pub mod rtl8139;
pub mod socket;

// pub fn local_endpoint() -> IpAddress {
//     (*IFACE.lock()).unwrap().ip_addrs()[0].address()
// }

lazy_static! {
    pub static ref IFACE: Mutex<Option<EthernetInterface<rtl8139::RTL8139>>> = Mutex::new(None);
}

pub fn init() {
    rtl8139::init();
    dhcp::init();
}

pub struct NetworkDevice<'a> {
    socket: TcpSocket<'a>,
}

pub fn mac() -> Option<MacAddress> {
    let guard = IFACE.lock();
    if let Some(iface) = &*guard {
        return Some(MacAddress::new(iface.ethernet_addr().as_bytes()));
    } else {
        None
    }
}


pub struct MacAddress {
    data: [u8; 6]
}

impl MacAddress {
    pub fn new(addr_bytes: &[u8]) -> Self {
        let mut data = [0;6];
        for i in 0..6 {
            data[i] = addr_bytes[i];
        }
        Self {
            data
        }
    }

    pub fn as_hex_str(&self) -> String {
        let mut buffer = String::new();
        write!(buffer, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", self.data[0], self.data[1], self.data[2], self.data[3], self.data[4], self.data[5]).expect("Failed To Write");
        buffer
    }
}
 

