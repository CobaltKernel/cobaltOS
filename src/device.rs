//! Handles Device IO, Supports:
//! - Block Devices - 'dev/null' ([NullDevice])
//! - Character Devices - 'dev/null' ([NullDevice]), 'dev/tty' | 'dev/comm' ([SerialPort])

use alloc::vec::Vec;
use uart_16550::SerialPort;

use crate::{KResult, sys::ata};

pub struct Device;
pub struct Disk(u8, u8);

pub enum DeviceHandle {
    Serial(SerialPort),
    Null(NullDevice)
}

impl CharDevice for DeviceHandle {
    fn read_u8(&self, addr: usize) ->   Option<u8> {
        match self {
            Self::Null(dev) => dev.read_u8(addr),
            Self::Serial(dev) => dev.read_u8(addr),
        }
    }

    fn read_u16(&self, addr: usize) ->  Option<u16> {
        match self {
            Self::Null(dev) => dev.read_u16(addr),
            Self::Serial(dev) => dev.read_u16(addr),
        }
    }

    fn read_u32(&self, addr: usize) ->  Option<u32> {
        match self {
            Self::Null(dev) => dev.read_u32(addr),
            Self::Serial(dev) => dev.read_u32(addr),
        }
    }

    fn read_u64(&self, addr: usize) ->  Option<u64> {
        match self {
            Self::Null(dev) => dev.read_u64(addr),
            Self::Serial(dev) => dev.read_u64(addr),
        }
    }

    fn read_u128(&self, addr: usize) -> Option<u128> {
        match self {
            Self::Null(dev) => dev.read_u128(addr),
            Self::Serial(dev) => dev.read_u128(addr),
        }
    }

    fn write_u8  (&mut self, addr: usize, value: u8)   -> KResult<()> {
        match self {
            Self::Null(dev) => dev.write_u8(addr, value),
            Self::Serial(dev) => dev.write_u8(addr, value),
        }
    }

    fn size(&self) -> Option<usize> {
        match self {
            Self::Null(dev) => dev.size(),
            Self::Serial(dev) => dev.size(),
        }
    }

    fn slice(&self) -> Option<&[u8]> {
        match self {
            Self::Null(dev) => dev.slice(),
            Self::Serial(dev) => dev.slice(),
        }
    }

    fn slice_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            Self::Null(dev) => dev.slice_mut(),
            Self::Serial(dev) => dev.slice_mut(),
        }
    }
}

impl Device {
    pub fn open_char_dev(path: &str) -> KResult<DeviceHandle> {
        let sections = path.split("/").collect::<Vec<&str>>();
        if sections[0] != "dev"  { return Err("Not A Device File") }
        if sections.len() == 1   { return Err("Must Point To A Device Descriptor (ie dev/ata/0/0, dev/mem)") }
        match sections[1] {
            "null" => Ok(DeviceHandle::Null(NullDevice)),
            "tty" | "comm" => {
                let mut tty = unsafe {SerialPort::new(0x3F8)};
                tty.init();
                Ok(DeviceHandle::Serial(tty))},
            _ => Err("Not A Valid Char Device Type"),
        }
    }
    pub fn open_block_dev(path: &str) -> KResult<impl BlockDevice> {
        let sections = path.split("/").collect::<Vec<&str>>();
        if sections[0] != "dev"  { return Err("Not A Device File") }
        if sections.len() == 1   { return Err("Must Point To A Device Descriptor (ie dev/ata/0/0, dev/mem)") }
        match sections[1] {
            "null" => Ok(NullDevice),
            _ => Err("Not A Valid Block Device Type"),
        }
    }
}

pub trait CharDevice {
    fn read_u8(&self, addr: usize) ->   Option<u8>;
    fn read_u16(&self, addr: usize) ->  Option<u16>;
    fn read_u32(&self, addr: usize) ->  Option<u32>;
    fn read_u64(&self, addr: usize) ->  Option<u64>;
    fn read_u128(&self, addr: usize) -> Option<u128>;

    fn write_u8  (&mut self, addr: usize, value: u8)   -> KResult<()>;

    fn write_u16 (&mut self, addr: usize, value: u16)  -> KResult<()> {
        let data = value.to_be_bytes();
        for (index, byte) in data.iter().enumerate() {
            self.write_u8(addr + index, *byte)?;
        }
        Ok(())
    }

    fn write_u32 (&mut self, addr: usize, value: u32)  -> KResult<()> {
        let data = value.to_be_bytes();
        for (index, byte) in data.iter().enumerate() {
            self.write_u8(addr + index, *byte)?;
        }
        Ok(())
    }

    fn write_u64 (&mut self, addr: usize, value: u64)  -> KResult<()> {
        let data = value.to_be_bytes();
        for (index, byte) in data.iter().enumerate() {
            self.write_u8(addr + index, *byte)?;
        }
        Ok(())
    }

    fn write_u128(&mut self, addr: usize, value: u128) -> KResult<()> {
        let data = value.to_be_bytes();
        for (index, byte) in data.iter().enumerate() {
            self.write_u8(addr + index, *byte)?;
        }
        Ok(())
    }
    
    fn size(&self) -> Option<usize>;
    fn slice(&self) -> Option<&[u8]>;
    fn slice_mut(&mut self) -> Option<&mut [u8]>;
}

pub trait BlockDevice {
    fn read(&self, addr: usize,buf: &mut [u8]) -> KResult<()>;
    fn write(&mut self, addr: usize, buf: &[u8]) -> KResult<()>;

    fn write_multi(&mut self, base_addr: usize, buf: &[u8]) -> KResult<()> {
        if let Some(block_size) = self.block_size() {
            if buf.len() % block_size > 0 {return Err("Buffer Size Must Be An Integer Multiple Of Block Size")};
            for (addr, block) in buf.chunks(block_size).enumerate() {
                
                let index = addr + base_addr;
                self.write(index, block)?;
            }
            Ok(())
        } else {
            self.write(base_addr, buf)
        }
    }

    fn block_count(&self) -> Option<usize>;
    fn block_size(&self) -> Option<usize> {Some(512)}
}


pub struct NullDevice;

impl CharDevice for NullDevice {
    fn read_u8(&self, _: usize) ->   Option<u8> {
        None
    }

    fn read_u16(&self, _: usize) ->  Option<u16> {
        None
    }

    fn read_u32(&self, _: usize) ->  Option<u32> {
        None
    }

    fn read_u64(&self, _: usize) ->  Option<u64> {
        None
    }

    fn read_u128(&self, _: usize) -> Option<u128> {
        None
    }

    fn write_u8  (&mut self, _: usize, _: u8)   -> KResult<()> {
        Ok(())
    }

    fn write_u16 (&mut self, _: usize, _: u16)  -> KResult<()> {
        Ok(())
    }

    fn write_u32 (&mut self, _: usize, _: u32)  -> KResult<()> {
        Ok(())
    }

    fn write_u64 (&mut self, _: usize, _: u64)  -> KResult<()> {
        Ok(())
    }

    fn write_u128(&mut self, _: usize, _: u128) -> KResult<()> {
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        None
    }

    fn slice(&self) -> Option<&[u8]> {
        None
    }

    fn slice_mut(&mut self) -> Option<&mut [u8]> {
        None
    }
}

impl BlockDevice for NullDevice {
    fn read(&self, _: usize, _: &mut [u8]) -> KResult<()> {
        Ok(())
    }

    fn write(&mut self, _: usize, _: &[u8]) -> KResult<()> {
        Ok(())
    }

    fn block_count(&self) -> Option<usize> {
        None
    }

    fn block_size(&self) -> Option<usize> {
        Some(512)
    }
}

impl BlockDevice for Disk {
    fn read(&self, addr: usize,buf: &mut [u8]) -> KResult<()> {
        ata::read(self.0, self.1, addr as u32, buf);
        Ok(())
    }

    fn write(&mut self, addr: usize, buf: &[u8]) -> KResult<()> {
        ata::write(self.0, self.1, addr as u32, buf);
        Ok(())
    }

    fn block_count(&self) -> Option<usize> {
        Some(ata::sector_count(self.0, self.1) as usize)
    }
}



#[test_case]
fn null_block_device() {
    let mut dev = Device::open_block_dev("dev/null").expect("");
    dev.write(0, &[0]).expect("");
    assert!(dev.write(0, &[0,1,2,3,4,5,6,7,8,9,10]).is_ok());
}

/// Should Pass, Writes ASCII A (65) to TTY
#[test_case]
fn tty_device() {
    let mut dev = Device::open_char_dev("dev/tty").expect("");
    dev.write_u8(0, b'A').expect("");
}

/// Should Pass, Writes ASCII A (65) to COMM
#[test_case]
fn tty_device() {
    let mut dev = Device::open_char_dev("dev/comm").expect("");
    dev.write_u8(0, b'A').expect("");
}

/// Should Fail, 'dev/invalid' Doesn't Exist
#[test_case]
fn bad_device() {
    assert!(Device::open_char_dev("dev/invalid").is_err());
    
}

/// Should Fail, TTY Isn't A Block Device
#[test_case]
fn tty_as_block_dev() {
    assert!(Device::open_block_dev("dev/tty").is_err());
}
