
pub mod filesystem;

use ustar::*;
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
