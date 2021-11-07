use alloc::vec::Vec;
use block_device::BlockDevice;
use fat32::volume::Volume;
use alloc::vec;

use crate::{println, serial_println, sys::{ata}};

pub fn read_file(_path: &str) -> Result<(), &str> {
    let disk = Disk(0,1);
    println!("Reading Volume From Disk {:?}...", disk);
    let volume = Volume::new(disk);
    println!("Volume: {}", volume.volume_label());
    let mut root = volume.root_dir();
    root.create_file("Yeet.txt").expect("Yeet");

    let mut file = root.open_file("Yeet.txt").expect("Error Opening File");
    file.write("Hello World".as_bytes(), fat32::file::WriteType::OverWritten).expect("");

    Ok(())
}
#[derive(Debug,Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Disk(u8, u8);

impl BlockDevice for Disk {
    type Error = &'static str;

    fn read(&self, buf: &mut [u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error> {
        let mut blocks: Vec<[u8; 512]> = vec![[0; 512]; number_of_blocks];

        

        for addr in address..address + number_of_blocks {
            let index = addr - address;
            ata::read(self.0, self.1, addr as u32, &mut blocks[index]);

            serial_println!("{:?}", blocks[index]);

            serial_println!("{:?}", &buf[0x0B..0x0D]);
        }

        for i in 0..buf.len() {
            blocks[i / 512][i % 512] = buf[i];
        }

        Ok(())
    }

    fn write(&self, buf: &[u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error> {
        let mut blocks: Vec<[u8; 512]> = vec![[0; 512]; number_of_blocks];

        for i in 0..buf.len() {
            blocks[i / 512][i % 512] = buf[i];
        }

        for addr in address..address + number_of_blocks {
            ata::write(self.0, self.1, addr as u32, &blocks[addr]);
        }

        
        Ok(())
    }
}