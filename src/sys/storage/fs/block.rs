use core::{fmt::Display, mem::size_of, ops::{Index, IndexMut, Range}};
use bytes::{BufMut, Bytes, BytesMut};

use alloc::string::String;
use core::str;

use crate::{debug, log, serial_print, serial_println, warn};

use super::{BLOCK_SIZE, BlockAddr, dev_handle::{BlockDeviceIO}, device, is_mounted};

#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
pub struct Block<'a> {
    addr: BlockAddr,
    data: [u8; BLOCK_SIZE],
    next: Option<&'a Block<'a>>
}

impl<'a> Block<'a> {
    pub fn read(addr: BlockAddr) -> Option<Self> {
        debug!("Reading Block 0x{:06x}", addr);
        if !is_mounted() {warn!("No Device Is Mounted."); return None;}
        let mut device = device().lock();
        let device = device.as_mut();
        
        let mut data: [u8; 512] = [0; 512];
        device.unwrap().read(addr, &mut data);
        let block = Self {
            addr,
            data,
            next: None
        };
        Some(block)
    }

    pub fn write(&self) {
        debug!("Writing Block 0x{:06x}", self.addr);
        if !is_mounted() {warn!("No Device Is Mounted."); }
        let mut device_lock = device().lock();
        let device = device_lock.as_mut().expect("No Device Mounted");
        device.write(self.addr, &self.data);
        drop(device_lock);
        assert_eq!(Block::read(self.addr).unwrap(), *self);
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn addr(&self) -> BlockAddr {
        self.addr
    }

    pub fn erase(&mut self) {
        self.data = [0; BLOCK_SIZE];
        self.write();
    }

    pub fn slice_range(&self, range: Range<usize>) -> &[u8] {
        &self.data()[range]
    }

    pub fn slice_range_mut(&mut self, range: Range<usize>) -> &mut [u8] {
        &mut self.data_mut()[range]
    }

    pub fn set_slice_range(&mut self, range: Range<usize>, slice: &[u8]) {
        debug!("Setting Range {}..{} to {:?}", range.start, range.end, slice);
        let mut src_offset = 0;
        for index in range {
            self.data[index] = slice[src_offset];
            src_offset += 1;
        }
    }

    pub fn write_str(&mut self, text: &str, offset: usize) -> usize {
        let mut size = 0;
        for byte in text.as_bytes() {
            self[offset + size] = *byte;
            size += 1;

            debug!("Wrote '{}'", *byte as char);
        }
        self[offset + size] = 0;
        offset + size
    }

    pub fn read_str(&self, text: &mut String, offset: usize) -> usize {
        let mut index = 0;
        while self[index + offset] != 0 {
            text.push(self[index + offset] as char);
            index += 1;
        }
        offset + index
    }

    pub fn read_u16(&self, offset: usize) -> (u16, usize) {
        const LENGTH: usize = size_of::<u16>();
        let mut buffer = [0;LENGTH];
        let slice = self.slice_range(offset..offset+LENGTH);
        for index in 0..LENGTH {
            buffer[index] = slice[index];
        }
        let value = u16::from_be_bytes(buffer);
        (value, offset + LENGTH)
    }

    pub fn read_u8(&self, offset: usize) -> (u8, usize) {
        const LENGTH: usize = size_of::<u8>();
        let mut buffer = [0;LENGTH];
        let slice = self.slice_range(offset..offset+LENGTH);
        for index in 0..LENGTH {
            buffer[index] = slice[index];
        }
        let value = u8::from_be_bytes(buffer);
        (value, offset + LENGTH)
    }

    pub fn read_u32(&self, offset: usize) -> (u32, usize) {
        const LENGTH: usize = size_of::<u32>();
        let mut buffer = [0;LENGTH];
        let slice = self.slice_range(offset..offset+LENGTH);
        for index in 0..LENGTH {
            buffer[index] = slice[index];
        }
        let value = u32::from_be_bytes(buffer);

        debug!("Old Offset: {}, New Offset: {}", offset, offset + LENGTH);

        (value, offset + LENGTH)
    }

    pub fn bytes(&self) -> Bytes {
        log!("Data: {:?}", self.data);
        Bytes::copy_from_slice(&self.data)
    }

    pub fn bytes_mut(&mut self) -> BytesMut {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&self.data);
        buffer
    }

    pub fn set_bytes_mut(&mut self, buffer: BytesMut) {
        for index in 0..buffer.len() {
            if index < BLOCK_SIZE {
            self[index] = buffer[index];
            log!("buffer[{}] = {:02x}", index, buffer[index]);
            };
        }
        self.write();
    }

    pub fn set_bytes(&mut self, buffer: Bytes) {
        for index in  0..buffer.len() {
            self[index] = buffer[index];
        }
        self.write();
    }

    pub fn write_u8(&mut self, offset: usize, value: u8) -> usize {
        self.set_slice_range(offset..offset+1, &value.to_be_bytes());
        offset + (u8::BITS / 8) as usize
    }

    pub fn write_u16(&mut self, offset: usize, value: u16) -> usize {
        self.set_slice_range(offset..offset+2, &value.to_be_bytes());
        offset + (u16::BITS / 8) as usize
    }

    pub fn write_u32(&mut self, offset: usize, value: u32) -> usize {
        self.set_slice_range(offset..offset+4, &value.to_be_bytes());
        offset + (u32::BITS / 8) as usize
    }

    pub fn write_u64(&mut self, offset: usize, value: u64) -> usize {
        self.set_slice_range(offset..offset+8, &value.to_be_bytes());
        offset + (u64::BITS / 8) as usize
    }
    
}

impl<'a> Index<usize> for Block<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data()[index]
    }
} 

impl<'a> IndexMut<usize> for Block<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data_mut()[index]
    }
}


// impl<'a> Drop for Block<'a> {
//     fn drop(&mut self) {
//         self.write();
//     }
// }

impl<'a> Display for Block<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bytes_per_row = 8;
        for row in (0..BLOCK_SIZE).step_by(bytes_per_row) {
            let mut char_buffer = String::new();
            for col in 0..bytes_per_row {
                write!(f,"{:02x} ", self[row  + col])?;
                if (0x20..0x7F).contains::<u8>(&self[row + col]) {
                    char_buffer.push(self[row  + col] as char);
                } else {
                    char_buffer.push('.');
                }
            }
            write!(f, "| {}\n", char_buffer)?;
        }
        Ok(())
    }
}

pub struct BlockWriter<'a> {
    block: Block<'a>,
    ptr: usize,
}