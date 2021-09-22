use alloc::{string::String, vec, vec::Vec};

use crate::sys::storage::fs::DISK_SIZE;

use super::{BLOCK_SIZE, BlockAddr};



pub trait BlockDeviceIO {
    fn read(&mut self, addr: BlockAddr, buf: &mut [u8]);
    fn write(&mut self, addr: BlockAddr, buf: &[u8]);
}

#[derive(Debug)]
pub enum DeviceHandle {
    MemBlockDevice(MemDevice),
    AtaBlockDevice(AtaDevice),
    ResBlockDevice(ResDevice)   
}

impl BlockDeviceIO for DeviceHandle {
    fn read(&mut self, addr: BlockAddr, buf: &mut [u8]) {
        match self {
            Self::AtaBlockDevice(dev) => {dev.read(addr, buf)},
            Self::MemBlockDevice(dev) => {dev.read(addr, buf)},
            Self::ResBlockDevice(dev) => {dev.read(addr, buf)},
        }
    }

    fn write(&mut self, addr: BlockAddr, buf: &[u8]) {
        match self {
            Self::AtaBlockDevice(dev) => {dev.write(addr, buf)},
            Self::MemBlockDevice(dev) => {dev.write(addr, buf)},
            Self::ResBlockDevice(dev) => {dev.write(addr, buf)},
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemDevice {disk: Vec<[u8; BLOCK_SIZE]>}
#[derive(Debug, Clone)]
pub struct AtaDevice {bus: u8, disk: u8}
#[derive(Debug, Clone, Copy)]
pub struct ResDevice {/* TODO: Design ResourceDevice Implementation. */}

impl BlockDeviceIO for AtaDevice {
    fn read(&mut self, addr: BlockAddr, buf: &mut [u8]) {
        todo!()
    }

    fn write(&mut self, addr: BlockAddr, buf: &[u8]) {
        todo!()
    }

}

impl BlockDeviceIO for MemDevice {
    fn read(&mut self, addr: BlockAddr, buf: &mut [u8]) {
        todo!()
    }

    fn write(&mut self, addr: BlockAddr, buf: &[u8]) {
        todo!()
    }

}

impl BlockDeviceIO for ResDevice {
    fn read(&mut self, addr: BlockAddr, buf: &mut [u8]) {
        todo!()
    }

    fn write(&mut self, addr: BlockAddr, buf: &[u8]) {
        todo!()
    }

}

impl MemDevice {
    pub fn new() -> Self {
        let disk = vec![[0; BLOCK_SIZE]; DISK_SIZE];
        Self {
            disk,
        }
    }
}

impl AtaDevice {
    pub fn new(bus: u8, disk: u8) -> Self {
        Self {
            bus,
            disk,
        }
    }
}