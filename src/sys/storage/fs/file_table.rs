use alloc::{borrow::ToOwned, string::String, vec::Vec};
use bytes::{BufMut};

use crate::log;

use super::{BlockAddr, FILETABLE_ADDR, FILETABLE_SIZE, block::Block};
pub struct FileTable {}

pub const MAX_DIR: usize = 64;
pub const MAX_NAME: usize = 64;
pub const ENTRY_ALIGN: usize = 512;

pub type RecordIndex = u32;

// FILE ENTRY STRUCTURE: Aligned On 256 Byte Boundaries
// +===+======+===+====+=========+=====+=========+
// |LEN|PARENT|LEN|NAME|FILE_TYPE|FLAGS|FILE_HEAD|
// +===+======+===+====+=========+=====+=========+
// | u8| var  | u8| var| u8      | u8  | u32     |
// +===+======+===+====+=========+=====+=========+

impl FileTable {

    pub fn record_offset(record: RecordIndex) -> RecordIndex {
        record
    } 

    fn block_addr(record: RecordIndex) -> BlockAddr {
        let record = Self::record_offset(record);
        let addr = record - FILETABLE_ADDR;
        FILETABLE_ADDR + addr
    }

    fn offset_of(record: RecordIndex) -> usize {
        let record = Self::record_offset(record);
        ((record - FILETABLE_ADDR) % 1) as usize
    }

    fn byte_offset(record: RecordIndex) -> usize {
        Self::offset_of(record) * ENTRY_ALIGN
    }

    pub fn create_file(record: RecordIndex, parent: &String, file_type: u8, file_head: u32, flags: u8, name: &String) {
        let mut block = Block::read(Self::block_addr(record)).unwrap();

        log!("Creating File '{}'", name);

        

        let mut buffer = block.bytes_mut();
        buffer.put_u8(parent.len() as u8);
        buffer.put(parent.as_bytes());
        buffer.put_u8(name.len() as u8);
        buffer.put(name.as_bytes());
        log!("Writing File Type: '{}'",file_type);
        buffer.put_u8(file_type);
        log!("Writing File Flags: '0b{:08b}'",flags);
        buffer.put_u8(flags);
        log!("Writing File Head: '0x{:06x}'",file_head);
        buffer.put_u32(file_head);


        block.set_bytes_mut(buffer);
        block.write();
    }

    pub fn locate(name: &String) -> Option<RecordIndex> {
        for record in FILETABLE_ADDR..(FILETABLE_ADDR + FILETABLE_SIZE as u32) {
            let block = Block::read(Self::block_addr(record)).unwrap();
            let offset = block.read_str(&mut "".to_owned(), Self::byte_offset(record));
            let mut file_name = String::new();
            block.read_str(&mut file_name, offset);
            if *name == file_name {return Some(record);};
        }
        None
    }

    pub fn filename(record: RecordIndex, buffer: &mut String) {
        let block = Block::read(Self::block_addr(record)).unwrap();
        let offset = block.read_str(&mut "".to_owned(), Self::byte_offset(record));
        block.read_str(buffer, offset);    
    }

    pub fn list(buffer: &mut Vec<(String, u32)>) {
        for record in FILETABLE_ADDR..(FILETABLE_ADDR + FILETABLE_SIZE as u32) {
            let mut name = String::new();
            Self::filename(record, &mut name);
            if !name.is_empty() {
                buffer.push((name, record));
            }
        }
    }

}