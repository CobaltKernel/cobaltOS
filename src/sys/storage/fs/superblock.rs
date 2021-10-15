use alloc::string::String;

use crate::log;

use super::block::Block;
use super::SUPER_BLOCK_ADDR;
use core::str;
pub struct SuperBlock {}

const MAGIC: &str = "COBALTFS";

impl SuperBlock {
    pub fn is_valid() -> bool {
        let block = Block::read(SUPER_BLOCK_ADDR).expect("Failed To Locate Superblock");
        let mut magic = String::new();
        block.read_str(&mut magic, 0);
        return magic == MAGIC;
    }

    pub fn format() {
        let mut block = Block::read(SUPER_BLOCK_ADDR).expect("Failed To Create Superblock");
        block.erase();
        block.write_str(MAGIC, 0);
        block.write();
    }

    pub fn mount() {
        if !SuperBlock::is_valid() {
            SuperBlock::format();
            log!("Formatted Device!");
        } else {
            log!("Device Is Valid, Found Superblock!");
        }
    }
}
