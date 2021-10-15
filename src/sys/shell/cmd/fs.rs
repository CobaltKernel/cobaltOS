use crate::{log, print, println, sys::{storage::fs::{*, dev_handle::{AtaDevice, DeviceHandle, MemDevice}, file_table::RecordIndex}, timer}};
use alloc::{string::String, vec::Vec};

pub fn main(args: &Vec<&str>) -> usize {
    match args[1] {
        "mount" => {mount(args)},
        "visualize" => {visualize(args)},
        "count_free" => {blocks_free()},
        "alloc" => {alloc(args)},
        "free" => {free(args)},
        "ls" => {list_all(args)},
        _ => {println!("Unknown command '{}'.", args[1])},
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

fn mount_ramdisk(_args: &Vec<&str>) {
    mount_device(DeviceHandle::MemBlockDevice(MemDevice::new()));
}

fn mount_ata(args: &Vec<&str>) {
    mount_device(DeviceHandle::AtaBlockDevice(AtaDevice::new(args[3].parse().unwrap(),args[4].parse().unwrap())));
    log!("Mounted ATA {}:{}", args[3], args[4]);
}

fn visualize(args: &Vec<&str>) {
    if args.len() < 4 {println!("Usage: fs visualize <clr|bin> <addr start>"); return;}

    let start: u32 = args[3].parse().expect("Expected A u32...");
    match args[2] {
        "clr" => bitmap::Bitmap::visualize(start),
        "bin" => bitmap::Bitmap::visualize_bin(start),
        _ => {println!("Usage: fs visualize <clr|bin> <addr start>")},
    }
}

fn blocks_free() {
    println!("");
    let mut sum = 0;
    for addr in DATA_ADDR..(DATA_SIZE as u32 + DATA_ADDR) {
        
        let start = timer::uptime_millis();
        if !bitmap::Bitmap::is_alloc(addr) {sum += 1};
        let end = timer::uptime_millis();
        print!("Checking Block 0x{:06x}, {}ms/Block\r", addr, (end - start));
    }
    println!("Blocks Free: {} ({} Bytes)", sum, sum * BLOCK_SIZE);
}

fn alloc(args: &Vec<&str>) {
    let addr = DATA_ADDR + args[2].parse::<u32>().expect("Expected A U32");
    bitmap::Bitmap::alloc(addr);
    println!("Allocated Block 0x{:06x}", addr);
}

fn free(args: &Vec<&str>) {
    let addr = DATA_ADDR + args[2].parse::<u32>().expect("Expected A U32");
    bitmap::Bitmap::free(addr);
    println!("Allocated Block 0x{:06x}", addr);
}

fn list_all(_args: &Vec<&str>) {
    let records: Vec<(String, RecordIndex)> = Vec::new();
    for (name, index) in records {
        println!("{}: {}", index, name);
    }
}


