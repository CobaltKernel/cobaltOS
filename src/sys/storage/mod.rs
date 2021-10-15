
pub mod fs;

use core::result::Result;

use alloc::vec::Vec;

use super::ata;
pub type StorageResult<T> = Result<T, &'static str>;
pub const BLOCK_SIZE: usize = 512;
#[derive(Debug, Clone, Copy)]
pub struct Block {
	data: [u8; BLOCK_SIZE],
	index: u32,
}

impl Block {
	pub fn new(index: u32, data: &[u8]) -> Self {
		let mut buffer = [0; BLOCK_SIZE];
		assert!(data.len() <= buffer.len(), "Data Size Is Too Large!, Data Size: {}, Max: {}", data.len(), BLOCK_SIZE);
		for index in 0..data.len() {
			buffer[index] = data[index];
		}
		Self {
			data: buffer,
			index
		}
	}

	pub fn get_next_block(&self) -> u32 {
		get_next_block(&self.data[0..3])
	}


	/// Copies The Data Held in this block, starting from the given offset
	pub fn copy_data_to_vec(&self, buffer: &mut Vec<u8>, offset: usize) {
		for index in offset..self.data.len() {
			buffer.push(self.data[index]);
		}
	}
}

#[must_use]
pub fn read(drive: usize, block_index: u32) -> StorageResult<Block> {
	let buffer = &mut [0; 512];

	ata::read(get_bus(drive), get_drive(drive), block_index, buffer);
	Ok(Block::new(block_index,buffer))
}

#[must_use]
pub fn read_multi(drive: usize, block_index: u32, length: u32, blocks: &mut Vec<Block>) -> StorageResult<()> {
	
	for index in block_index..=block_index+length {
		blocks.push(read(drive, index)?);
	}

	Ok(())
}

#[must_use]
pub fn read_linked(drive: usize, block_index: u32, blocks: &mut Vec<Block>) -> StorageResult<()> {
	let mut next_block = block_index;
	loop {
		let block = read(drive, next_block)?;
		next_block = block.get_next_block();
		blocks.push(block);
		if next_block == 0 {break;}
	}
	Ok(())
}

#[must_use]
pub fn read_linked_raw(drive: usize, block_index: u32, data: &mut Vec<u8>) -> StorageResult<()> {
	let mut blocks: Vec<Block> = Vec::new();
	read_linked(drive, block_index, &mut blocks)?;
	for block in blocks {
		let mut temp: Vec<u8> = Vec::new();
		// Skip the first four Bytes, they are the pointer to the next block
		block.copy_data_to_vec(&mut temp, 4);
		data.append(&mut temp)
	}

	Ok(())
}

pub fn read_multi_raw(drive: usize, block_index: u32, length: u32, data: &mut Vec<u8>) -> StorageResult<()> {
	let mut blocks: Vec<Block> = Vec::new();
	read_multi(drive, block_index, length, &mut blocks)?;
	for block in blocks {
		let mut temp: Vec<u8> = Vec::new();
		// Skip the first four Bytes, they are the pointer to the next block
		block.copy_data_to_vec(&mut temp, 4);
		data.append(&mut temp)
	}

	Ok(())
}

#[must_use]
pub fn write_raw(drive: usize, block_index: u32, buffer: &[u8]) -> StorageResult<()> {
    if buffer.len() != BLOCK_SIZE { return Err("Buffer Size Must Be The EXACT Same Size As A Single Block") };

	ata::write(get_bus(drive), get_drive(drive), block_index, buffer);
	Ok(())
}

#[must_use]
pub fn write_block(drive: usize, block: Block) -> StorageResult<()> {
	write_raw(drive, block.index, &block.data)
}

fn get_bus(drive: usize) -> u8 {
	(drive / 2) as u8
}

fn get_drive(drive: usize) -> u8 {
	(drive % 2) as u8
}

fn get_next_block(data: &[u8]) -> u32 {
	let mut next_block: u32 = 0;
	for i in 0..4 {
		next_block |= data[i] as u32;
		next_block = next_block << 8;
	}
	next_block
}


