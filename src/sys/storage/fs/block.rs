use core::{fmt::Display, ops::{Index, IndexMut, Range}};
use bytes::{Bytes, BytesMut};

use alloc::string::String;
use core::str;

use crate::{debug, warn};

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
        let size = text.as_bytes().len();
        let bytes = text.as_bytes();
        self[offset] = size as u8;
        self.set_slice_range(1+offset..(1 + offset + size), bytes);
        self.write();
        offset + size
    }

    pub fn read_str(&self, text: &mut String, offset: usize) -> usize {
        let size = self[offset];
        let offset = offset + 1;
        let end = offset + size as usize;
        let data = self.slice_range(offset..end);
        text.push_str(unsafe {str::from_utf8_unchecked(data)});
        end
    }

    pub fn bytes(&self) -> Bytes {
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
        let bytes_per_row = 16;
        for row in (0..BLOCK_SIZE).step_by(bytes_per_row) {
            let mut char_buffer = String::new();
            for col in 0..bytes_per_row {
                write!(f,"{:02x} ", self[row * bytes_per_row + col])?;
                if (0x20..0x7F).contains::<u8>(&self[row * bytes_per_row + col]) {
                    char_buffer.push(self[row * bytes_per_row + col] as char);
                } else {
                    char_buffer.push('.');
                }
            }
            write!(f, "| {}\n", char_buffer)?;
        }
        Ok(())
    }
}