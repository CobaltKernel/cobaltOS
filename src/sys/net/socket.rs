

use core::str::FromStr;

use alloc::vec::{self, Vec};
use smoltcp::socket::{TcpSocket, TcpSocketBuffer};
use smoltcp::wire::{IpAddress, Ipv4Address};
pub struct Socket<'a> {
    _socket: TcpSocket<'a>,
    addr: IpAddress,
    port: u16,
}

impl<'a> Socket<'a> {
    // pub fn new_v4(a0: u8, a1: u8, a2: u8, a3: u8, port: u16, tx_buffer: Vec<u8>, rx_buffer: Vec<u8>) -> Option<Self> {
    //     let sock = TcpSocket::new(TcpSocketBuffer::new(rx_buffer), TcpSocketBuffer::new(tx_buffer));
    //     match sock.connect(IpAddress::v4(a0, a1, a2, a3), local_endpoint)

    // }
}