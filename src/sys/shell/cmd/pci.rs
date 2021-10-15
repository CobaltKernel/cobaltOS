
use alloc::vec::Vec;

use crate::{print, println, sys::{pci, pci_details}};

pub fn main(args: &Vec<&str>) -> usize {
    if args.len() < 2 {println!("pci <command>");  return 1;}
    match args[1] {
        "ls" => { ls(args); },
        _ => {
            println!("Unknown Command '{}'", args[1]);
            println!("Available Commands: ");
            println!("ls - List Device IDs");
        }
    }

    0
}

fn ls(_args: &Vec<&str>) {
    let devices = &*pci::PCI_DEVICES.lock();
    for dev in devices {
        println!("{} {}: ", pci_details::vendor(dev.vendor_id), pci_details::device(dev.device_id)); 
        println!("+=========+============+==============+============+==================+");
        println!("| Bus: {:02x} | Device: {:02x} | Function: {:02x} | Vendor: {:04x} | DeviceID: {:04x} |", dev.bus, dev.device, dev.function, dev.vendor_id, dev.device_id);
        for addr in dev.base_addresses {
            if addr > 0 {print!("| BAR: {:08x} ", addr);}
        }
        println!("|");
    }
}