use alloc::{vec, vec::Vec};

use crate::sys::{ata, storage::fs::DISK_SIZE};

use super::{BLOCK_SIZE, BlockAddr};

use block_device::BlockDevice;



pub trait BlockDeviceIO {
    fn read(&self, addr: BlockAddr, buf: &mut [u8]);
    fn write(&mut self, addr: BlockAddr, buf: &[u8]);
    fn sector_count(&self) -> u32;
}

#[derive(Debug, Clone)]
pub enum DeviceHandle {
    MemBlockDevice(MemDevice),
    AtaBlockDevice(AtaDevice),
    ResBlockDevice(ResDevice)   
}

impl BlockDeviceIO for DeviceHandle {
    fn read(&self, addr: BlockAddr, buf: &mut [u8]) {
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

    fn sector_count(&self) -> u32 {
        match self {
            Self::AtaBlockDevice(dev) => {dev.sector_count()},
            Self::MemBlockDevice(dev) => {dev.sector_count()},
            Self::ResBlockDevice(dev) => {dev.sector_count()},
        }
    }
}

#[derive(Debug,  Clone)]
pub struct MemDevice {disk: Vec<[u8; BLOCK_SIZE]>}
#[derive(Debug, Copy, Clone)]
pub struct AtaDevice {bus: u8, disk: u8}
#[derive(Debug, Clone, Copy)]
pub struct ResDevice {/* TODO: Design ResourceDevice Implementation. */}

impl BlockDeviceIO for AtaDevice {
    fn read(&self, addr: BlockAddr, buf: &mut [u8]) {
        assert!(addr < self.sector_count());
        ata::read(self.bus, self.disk, addr, buf);
    }

    fn write(&mut self, addr: BlockAddr, buf: &[u8]) {
        assert!(addr < self.sector_count());
        ata::write(self.bus, self.disk, addr, buf);
    }

    fn sector_count(&self) -> u32 {
        ata::sector_count(self.bus, self.disk)
    }

}

impl BlockDeviceIO for MemDevice {
    fn read(&self, _addr: BlockAddr, _buf: &mut [u8]) {
        todo!()
    }

    fn write(&mut self, _addr: BlockAddr, _buf: &[u8]) {
        todo!()
    }

    
    fn sector_count(&self) -> u32 {
        todo!()
    }
}

impl BlockDeviceIO for ResDevice {
    fn read(&self, _addr: BlockAddr, _buf: &mut [u8]) {
        todo!()
    }

    fn write(&mut self, _addr: BlockAddr, _buf: &[u8]) {
        todo!()
    }

    fn sector_count(&self) -> u32 {
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

impl BlockDevice for DeviceHandle {
    type Error = &'static str;

    
    fn read(&self, buffer: &mut [u8], addr: usize, block_count: usize) -> Result<(), Self::Error> {
        let mut blocks: Vec<[u8; 512]> = vec![[0; 512]; block_count];
        for i in addr..addr+block_count {
            match self {
                Self::AtaBlockDevice(dev) => dev.read(i as u32, &mut blocks[i - addr]),
                _ => unimplemented!()
            }
        }

        for index in 0..buffer.len() {
            buffer[index] = blocks[index / 512][index % 512]
        }

        Ok(())
    }

    fn write(&self, buffer: &[u8], addr: usize, block_count: usize) -> Result<(), Self::Error> {
        todo!();
    }
}