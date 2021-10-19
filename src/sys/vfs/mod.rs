
pub mod filesystem;

use crate::{println, sys::ustar::*};
use metadata::*;
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::boxed::Box;
use alloc::vec::Vec;
use x86_64::instructions::interrupts::without_interrupts;
use crate::sys::storage::fs::{device, dev_handle::BlockDeviceIO};



pub fn list(buf: &mut Vec<Metadata>) {
    let dev = device().lock();
    let dev = dev.as_ref();
    let dev = dev.unwrap().clone();

    let fs = TarFileSystem::new(dev.sector_count() as usize, Box::new(dev));
    fs.metadata_slice(buf);
}

pub fn read(meta: &Metadata, buf: &mut Vec<u8>) {

}

pub fn load(path: &str, buf: &mut Vec<u8>) -> Result<(), &'static str> {
    let dev = device().lock();
    let dev = dev.as_ref();
    let dev = dev.unwrap().clone();

    let fs = TarFileSystem::new(dev.sector_count() as usize, Box::new(dev));
    if let Some(meta) = fs.find(path) {
        let mut buffer: Vec<u8> = alloc::vec![0; meta.size()];
        //println!("Buffer Size: {}", buffer.len());
        fs.load(path, buffer.as_mut_slice()).expect("Failed To Read File");
        buf.append(&mut buffer);
        Ok(())
    } else {
        return Err("Failed To Read File");
    }

}
