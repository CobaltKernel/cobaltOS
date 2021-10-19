
pub mod metadata;

use block_device::BlockDevice;
use metadata::Metadata;
use alloc::vec;
use vec::Vec;
use alloc::boxed::Box;

use core::{fmt::Debug, ops::{Index, IndexMut}};

use crate::{println, serial_println};

pub struct TarFileSystem<E> {
    disk: Box<dyn BlockDevice<Error = E>>,
    disk_size: usize,
}

impl<E: Debug> TarFileSystem<E> {
    pub fn new(size: usize, device: Box<dyn BlockDevice<Error = E>>) -> Self {
        Self {
            disk: device,
            disk_size: size
        }
    }

    pub fn metadata(&self, addr: u32) -> metadata::Metadata {
        let buffer = &mut [0; 512];
        self.disk.read(buffer, addr as usize, 1).expect("Failed To Read From Disk");
        Metadata::from(addr, buffer)
    }

    pub fn find(&self, name: &str) -> Option<Metadata> {
        let mut index = 0;
        for _ in 0..self.disk_size {
            let meta = self.metadata(index as u32);
            //serial_println!("Meta Name: '{}', File Name: '{}', Eq?: {}", meta.file_name(), name, meta.file_name().eq_ignore_ascii_case(name));
            if meta.file_name().eq_ignore_ascii_case(name) {
                //println!("Found Index!");
                return Some(meta);
            } else {
                index += meta.block_length();
            }

            if index >= self.disk_size {return None;}
        }
        return None;
    }

    pub fn load(&self, name: &str, buffer: &mut [u8]) -> Result<usize, &str> {
        if let Some(meta) = self.find(name) {

            let mut temp = vec![0; (meta.block_length() - 1) * 512];
            let temp: &mut [u8] = temp.as_mut_slice();
            self.disk.read(temp, (meta.addr() + 1) as usize, meta.block_length() - 1).expect("Failed To Read Device");
            let mut bytes_written = 0;
            for i in 0..buffer.len() {
                if i <= temp.len() {
                    bytes_written += 1;
                    buffer[i] = temp[i];
                }
            }

            Ok(bytes_written)
        } else {
            Err("Unable To Locate File!")
        }
    }

    pub fn size_of(&self, name: &str) -> Option<usize> {
        if let Some(meta) = self.find(name) {
            Some(meta.size())
        } else {
            None
        }

    }

    pub fn metadata_slice(&self, buffer: &mut Vec<Metadata>){
        let mut index = 0;
        for _ in 0..self.disk_size {
            let meta = self.metadata(index as u32);
            //serial_println!("Meta: {:?}", meta);
            index += meta.block_length();
            buffer.push(meta);
            if index >= self.disk_size {return;}
        }
    }
}

pub struct RamDisk {
    pub data: Vec<[u8; 512]>
}

impl BlockDevice for RamDisk {
    type Error = ();

    fn read(&self, buf: &mut [u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error> {
        if (buf.len() / 512) < number_of_blocks {return Err(());} else {
            for index in 0..buf.len() {
                buf[index] = self.data[address + index / 512][index % 512];
            }

            return Ok(());
        }
    }

    fn write(&self, buf: &[u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error> {
        todo!()
    }
}

impl RamDisk {
    pub fn from(data: &[u8]) -> Self {
        let mut blocks = data.len() / 512;
        if data.len() % 512 != 0 {blocks += 1;};

        let mut buffer: Vec<[u8; 512]> = vec![[0;512]; blocks];
        for index in 0..data.len() {
            buffer[index / 512][index % 512] = data[index];
        }

        Self {
            data: buffer
        }
    }
}

impl Index<usize> for RamDisk {
    type Output = [u8];
    fn index(&self, index: usize) -> &Self::Output {
        return &self.data[index]
    }
}

impl IndexMut<usize> for RamDisk {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return &mut self.data[index]
    }
}

