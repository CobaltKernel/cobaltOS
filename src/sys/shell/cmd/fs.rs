use crate::{println, sys::storage::fs::{*, dev_handle::{AtaDevice, DeviceHandle, MemDevice}}};

use alloc::vec::Vec;

pub fn main(args: &Vec<&str>) -> usize {
    match args[1] {
        "mount" => {mount(args)},
        _ => {},
    }
    0
}

fn mount(args: &Vec<&str>) {
    match args[2] {
        "ramdisk" => {mount_ramdisk(args)},
        "ata" => {mount_ata(args)},
        _ => {println!("Unknown Device '{}'.", args[2])}
    }
}

fn mount_ramdisk(args: &Vec<&str>) {
    mount_device(DeviceHandle::MemBlockDevice(MemDevice::new()));
}

fn mount_ata(args: &Vec<&str>) {
    mount_device(DeviceHandle::AtaBlockDevice(AtaDevice::new(args[3].parse().unwrap(),args[4].parse().unwrap())));
}