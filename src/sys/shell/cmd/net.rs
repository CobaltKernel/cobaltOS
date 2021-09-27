use alloc::vec::Vec;

use crate::{println, sys::net};

pub fn main(args: &Vec<&str>) -> usize {
    let mac = net::mac();
    if let Some(mac) = mac {
        println!("MAC: {}", mac.as_hex_str())
    } else {
        println!("MAC ??:??:??:??:??:??")
    }
    0
}