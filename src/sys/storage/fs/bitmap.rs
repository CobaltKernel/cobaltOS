use core::ops::Range;

use bit_field::BitField;

use crate::{clear, log, print_at, set_style, sys::{timer::{self, uptime_millis}, vga::Color}};

use super::{BITMAP_ADDR, BITMAP_SIZE, BlockAddr, DATA_ADDR, DATA_SIZE, block::Block};

pub struct Bitmap {
    
}

impl Bitmap {
    pub fn block_addr(addr: BlockAddr) -> BlockAddr {
        let size = BITMAP_SIZE as BlockAddr ;
        let i = addr - DATA_ADDR;
        return BITMAP_ADDR + (i / size );
    }

    /// What Is The Buffer Offset? offset = (i - DATA_ADDR) % BITMAP_SIZE
    pub fn buffer_offset(addr: BlockAddr) -> usize {
        let i = (addr - DATA_ADDR) as usize;
        return i % BITMAP_SIZE;
    }

    /// Get Block State from a given &[u8]
    fn get_block_state(buffer: &[u8], addr: BlockAddr) -> bool {
        let i = Bitmap::buffer_offset(addr);
        return buffer[i / 8].get_bit(i % 8)
    }

    /// Get Block State from a given &[u8]
    fn set_block_state(buffer: &mut [u8], addr: BlockAddr, state: bool) {
        let i = Bitmap::buffer_offset(addr);
        buffer[i / 8].set_bit(i % 8, state);
    }

    pub fn is_alloc(addr: BlockAddr) -> bool {
        let block = Block::read(Self::block_addr(addr)).expect("Failed To Load Bitmap");
        Self::get_block_state(block.data(), addr)
    }

    pub fn alloc(addr: BlockAddr) {
        let mut block = Block::read(Self::block_addr(addr)).expect("Failed To Load Bitmap");
        Self::set_block_state(block.data_mut(), addr, true);
        log!("Allocating Block 0x{:06}", addr);
        block.write();
    }

    pub fn free(addr: BlockAddr) {
        let mut block = Block::read(Self::block_addr(addr)).expect("Failed To Load Bitmap");
        Self::set_block_state(block.data_mut(), addr, false);
        log!("freeing Block 0x{:06}", addr);
        block.write();
    }

    /// Attempts to Find The Next Free Block. Returns [None] if Failed.
    pub fn next_free() -> Option<BlockAddr> {
        let tp1 = timer::uptime_millis();
        for addr in DATA_ADDR..(DATA_ADDR + DATA_SIZE as u32) {
            
            if !Bitmap::is_alloc(addr) {
                let tp2 = uptime_millis();
                log!("Found Block In {} ms", (tp2 - tp1));
                return Some(addr);
            }
        }
        None
    }

    pub fn alloc_next_free<'a>() -> Option<Block<'a>> {
        if let Some(addr) = Self::next_free() {
            return Block::read(addr);
        } else {
            return None;
        };
    }

    pub fn bulk_alloc(range: Range<BlockAddr>) {
        for addr in range {
            Self::alloc(addr);
        }
    }

    pub fn visualize(start: u32) {
        clear!();
        let mut _offset = 0;
        let start = start + DATA_ADDR;
        let width = 80 as usize;
        let height = 20 as usize;
        let _length = width * height;
        for y in 0..height {
            for x in 0..width {
                let addr = ((x as usize) + (y as usize) * width) + start as usize;
                let color = if Self::is_alloc(addr as u32) {
                    Color::Red
                } else {
                    Color::Green
                };

                set_style!(Color::Black, color);
                print_at!(x, y + 1, " ");
            }
        }

        set_style!(Color::White, Color::Blue);
    }

    pub fn visualize_bin(start: u32) {
        clear!();
        let mut _offset = 0;
        let start = start + DATA_ADDR;
        let width = 80 as usize;
        let height = 20 as usize;
        let _length = width * height;
        for y in 0..height {
            for x in 0..width {
                let addr = ((x as usize) + (y as usize) * width) + start as usize;
                if Self::is_alloc(addr as u32) {
                    print_at!(x, y + 1, "1");
                } else {
                    print_at!(x, y + 1, "0");
                };
            }
        }

        set_style!(Color::White, Color::Blue);
    }

}