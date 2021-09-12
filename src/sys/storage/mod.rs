use core::result::Result;
mod ata;
pub type StorageResult = Result<(), &'static str>;

pub fn read(bus: usize, drive: usize, block_index: u32, buffer: &mut [u8]) -> StorageResult {
	if buffer.len() != 512 { return Err("Buffer Size Must Be 512 Bytes Exactly") }

	Err("Storage Not Implemented")
}

pub fn write(bus: usize, drive: usize, block_index: u32, buffer: &mut [u8]) -> StorageResult {
    Err("Storage Not Implemented")
}


