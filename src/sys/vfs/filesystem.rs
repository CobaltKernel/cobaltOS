use core::{fmt::Display, ops::{Index, IndexMut}};

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::{breakpoint, debug, log, print, serial, serial_print, serial_println, sys::{storage::fs::{block::Block, device}, vfs::filesystem::filesystem_values::INODE_BITMAP_BASE}, warn};

use self::{filesystem_values::{BLOCKS_PER_BITMAP, BLOCK_SIZE, DATA_BASE, DATA_BITMAP_SIZE, INODE_BASE, INODE_BITMAP_SIZE, INODE_SIZE, PHYSICAL_OFFSET}, inode_flags::*};

use bit_field::BitField;
use bytes::{Buf, BufMut, Bytes};


pub mod inode_meta {
    use core::mem::size_of;

    use super::filesystem_values::BLOCK_SIZE;

    pub const FILENAME_SIZE: usize = 64;
    pub const FLAGS_OFFSET: usize = FILENAME_SIZE;
    pub const PARENT_OFFSET: usize = FLAGS_OFFSET + size_of::<u16>();
    pub const SIZE_OFFSET: usize = PARENT_OFFSET + size_of::<u32>();
    pub const CHILDREN_OFFSET: usize = SIZE_OFFSET + size_of::<u32>();
    pub const CHILDREN_LEN: usize = (BLOCK_SIZE - CHILDREN_OFFSET) / size_of::<u32>();
}

pub mod inode_flags {
    pub const ROOT_READ: u16 = 1 << 0;
    pub const ROOT_WRITE: u16 = 1 << 1;
    pub const ROOT_EXEC: u16  = 1 << 2;

    pub const DIR: u16  = 1 << 3;
    pub const FILE: u16 = 1 << 4;
    pub const DEV: u16  = 1 << 5;

    pub const HIDDEN: u16 = 1 << 6;
}

pub mod filesystem_values {
    pub const PHYSICAL_OFFSET:      usize = (20 << 20) / BLOCK_SIZE;
    pub const BLOCK_SIZE:           usize = 512;
    pub const PARTITION_SIZE:       usize = (20 << 20) / BLOCK_SIZE;
    pub const SUPERBLOCK_SIZE:      usize = 1;
    pub const INODE_SIZE:           usize = 4096;
    pub const DATA_SIZE:            usize = 4096;
    pub const BLOCKS_PER_BITMAP:    usize = 8 * 512; 
    pub const INODE_BITMAP_SIZE:    usize = INODE_SIZE / BLOCKS_PER_BITMAP;
    pub const DATA_BITMAP_SIZE:     usize = DATA_SIZE / BLOCKS_PER_BITMAP;

    pub const METADATA_SIZE:        usize = SUPERBLOCK_SIZE + INODE_BITMAP_SIZE + DATA_BITMAP_SIZE;
    pub const USABLE_SIZE:          usize = PARTITION_SIZE - METADATA_SIZE;

    pub const INODE_BITMAP_BASE:    u32 = PHYSICAL_OFFSET as u32 + SUPERBLOCK_SIZE as u32;
    pub const DATA_BITMAP_BASE:     u32 = INODE_BITMAP_BASE + INODE_BITMAP_SIZE as u32;
    pub const INODE_BASE:           u32 = DATA_BITMAP_BASE + DATA_BITMAP_SIZE as u32;
    pub const DATA_BASE:            u32 = INODE_BASE + INODE_SIZE as u32;
}


pub trait FileSystem {
    fn current_dir(&self) -> Directory;
    fn change_dir(&mut self, dir: Directory);
    fn create_file(&mut self, file_name: String) -> File;
    fn delete_file(&mut self, file_name: String);
    fn file_exists(&mut self, file_name: String) -> File;
    fn file_update(&mut self, file: &File); 
}

fn logical_to_physical(base: u32, logical: u32) -> u32 {
    return base + logical + PHYSICAL_OFFSET as u32;
}

pub struct DataBitmap;

impl DataBitmap {
    pub fn data_index_to_logical(index: u32) -> u32 {
        index / BLOCKS_PER_BITMAP as u32 / 8
    }

    pub fn data_index_to_buffer_offset(index: u32) -> u32 {
        index % BLOCKS_PER_BITMAP as u32
    }

    pub fn is_allocated(index: u32) -> bool {
        let logical_index = Self::data_index_to_logical(index);
        let physical_index = logical_to_physical(INODE_BASE, logical_index);
        let block = Block::read(physical_index).unwrap();

        let buffer = block.data();
        let offset = Self::data_index_to_buffer_offset(index);
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        return buffer[byte_idx as usize].get_bit(bit_idx as usize);
    }

    pub fn allocate(index: u32) {
        let logical_index = Self::data_index_to_logical(index);
        let physical_index = logical_to_physical(INODE_BASE, logical_index);
        let mut block = Block::read(physical_index).unwrap();

        let offset = Self::data_index_to_buffer_offset(index);
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        block[byte_idx as usize].set_bit(bit_idx as usize, true);

        block.write();
    }

    pub fn free(index: u32) {
        let logical_index = Self::data_index_to_logical(index);
        let physical_index = logical_to_physical(INODE_BASE, logical_index);
        let mut block = Block::read(physical_index).unwrap();

        let offset = Self::data_index_to_logical(index);
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        block[byte_idx as usize].set_bit(bit_idx as usize, false);

        block.write();
    }

    pub fn next_free() -> Option<u32> {
        for index in 0..INODE_SIZE as u32 {

            if !Self::is_allocated(index) {
                return Some(index);
            };
        }
        None
    }

    pub fn allocate_next() -> Option<u32> {
        if let Some(next) = Self::next_free() {
            Self::allocate(next);
            Some(next)
        } else {
            None
        }
    }


}

pub struct InodeBitmap;

impl InodeBitmap {
    pub fn inode_index_to_logical(index: u32) -> u32 {
        index / 4096 / 8
     }

    pub fn inode_index_to_buffer_offset(index: u32) -> u32 {
        index % 4096 as u32
    }

    pub fn is_allocated(index: u32) -> bool {
        let logical_index = Self::inode_index_to_logical(index);
        let physical_index = logical_to_physical(INODE_BASE, logical_index);
        let block = Block::read(physical_index).unwrap();

        let buffer = block.data();
        let offset = Self::inode_index_to_buffer_offset(index);
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        return buffer[byte_idx as usize].get_bit(bit_idx as usize);
    }

    pub fn allocate(index: u32) {
        let logical_index = Self::inode_index_to_logical(index);
        let physical_index = logical_to_physical(INODE_BASE, logical_index);
        let mut block = Block::read(physical_index).unwrap();

        let offset = Self::inode_index_to_buffer_offset(index);
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        block[byte_idx as usize].set_bit(bit_idx as usize, true);

        block.write();
    }

    pub fn free(index: u32) {
        let logical_index = Self::inode_index_to_logical(index);
        let physical_index = logical_to_physical(INODE_BASE, logical_index);
        let mut block = Block::read(physical_index).unwrap();

        let offset = Self::inode_index_to_buffer_offset(index);
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        block[byte_idx as usize].set_bit(bit_idx as usize, false);

        block.write();
    }

    pub fn next_free() -> Option<u32> {
        for index in 0..INODE_SIZE as u32 {
            //breakpoint!("Checking Status Of Inode {}", index);
            if !Self::is_allocated(index) {
                return Some(index);
            };
        }
        None
    }

    pub fn allocate_next() -> Option<u32> {
        if let Some(next) = Self::next_free() {
            Self::allocate(next);
            Some(next)
        } else {
            None
        }
    }

    pub fn get_allocated(buffer: &mut Vec<Inode>) {
    
        serial_print!("Searching Inodes");
        let pct: u32 = (INODE_SIZE / 100) as u32;
        for index in 0..INODE_SIZE as u32 {
            if index % pct == 0 { serial_print!("."); }
            if Self::is_allocated(index) {buffer.push(Inode::read(index))}
        }
    }

    pub unsafe fn erase_all() {
        for idx in 0..INODE_BITMAP_SIZE as u32 {
            Block::read(idx).unwrap().erase();
        }
    }




}

#[derive(Debug)]
pub struct Inode {
    addr: u32,
    name: String, 
    flags: InodeFlags,
    parent: Option<u32>,
    children: Vec<u32>,
    size: u32
}


impl Inode {

    pub fn read(addr: u32) -> Self {
        let block = Block::read(Self::physical_addr(addr)).unwrap();
        let mut name: Vec<u8> = Vec::new();
        let mut offset = 0;
        let mut index = 0;
        while block[offset] != 0 {
            name.push(block[offset]);
            offset += 1;
        }

        offset += index;
        let name = (*String::from_utf8_lossy(&name)).to_owned();

        let (flags, new_offset) = block.read_u16(offset);
        offset = new_offset;
        let flags = InodeFlags::new(flags);

        let mut parent: Option<u32> = None;
        let (pid, new_offset) = block.read_u32(offset);
        offset = new_offset;
        if pid < 0xFFFF_FFFF {parent = Some(pid)};
        let (child_count, new_offset) = block.read_u32(offset);
        offset = new_offset;
        let mut children = Vec::new();
        for i in 0..child_count {
            let (child, new_offset) = block.read_u32(offset);
            offset = new_offset;
            children.push(child);
        }

        //Read in the Size
        let (size, _) = block.read_u32(offset);

        serial_println!("name: {}",name);
        serial_println!("size: {} Bytes", size);
        serial_println!("Child Count: {}", child_count);
        serial_println!("Children: {:?}", children);

        Self {
            addr,
            children,
            flags,
            name,
            parent,
            size,
        }

        
    }

    pub fn write(&self) {
        let mut block = Block::read(Self::physical_addr(self.addr)).unwrap();
        let bytes = &mut block;
        let mut offset = 0;

        offset = bytes.write_str(self.name(), offset);


        offset += bytes.write_u16(offset, self.flags.value());
        if let Some(parent) = self.parent() {
            serial_println!("Parent: {}", parent);
            offset = bytes.write_u32(offset, parent);
        } else {
            offset = bytes.write_u32(offset,0xFFFF_FFFF);
        }
        
        offset = bytes.write_u8(offset, self.children.len() as u8);
        for child in &self.children {
            offset = bytes.write_u32(offset, *child);
        }

        // Write The Size Out
        offset = bytes.write_u32(offset, self.size);

        block.write();


        let written = Block::read(Self::physical_addr(self.addr)).unwrap();
        
        log!("block: {}", block);
        log!("written: {}", written);
        
        assert_eq!(block, written);
    }

    pub fn create(name: &str, flags: InodeFlags, parent: Option<u32>) -> Option<Self> {
        if let Some(inode) = InodeBitmap::allocate_next() {
            Some(
                Self {
                    addr: inode,
                    children: Vec::new(),
                    parent,
                    flags,
                    name: name.to_owned(),
                    size: 0,
                }
            )
        } else {
            None
        }
    }

    pub fn new(addr: u32, name: String, flags: InodeFlags, children: Vec<u32>, parent: Option<u32>) -> Self {
            Self {
                addr, 
                children,
                flags,
                name,
                parent,
                size: 0
            }
    }

    pub fn flags(&self) -> InodeFlags {
        self.flags
    }

    pub fn parent(&self) -> Option<u32> {
        self.parent
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn children(&self) -> &Vec<u32> {
        &self.children
    }

    pub fn has_child(&self, index: u32) -> bool {
        self.children.contains(&index)
    }

    pub fn add_child(&mut self, index: u32) {
        self.children.push(index);
    }

    pub fn clear_children(&mut self) {
        for child in &self.children {
            if self.flags.is_dir() { InodeBitmap::free(*child) };
            if self.flags.is_file() { DataBitmap::free(*child) };
        }
        self.children.clear();
    }

    fn physical_addr(index: u32) -> u32 {
        logical_to_physical(INODE_BASE, index)
    }

    
}

pub struct InodeBlocks;

impl InodeBlocks {
    pub fn create_file(name: &str, parent_dir: Option<u32>) -> Option<Inode> {
        Inode::create(name,  InodeFlags::file(), parent_dir)
    }

    pub fn create_dir(name: &str, parent_dir: Option<u32>) -> Option<Inode> {
        Inode::create(name,  InodeFlags::dir(), parent_dir)
    }

    pub fn open_file(name: &str) -> Option<File> {
        print!("Locating File");
        for index in 0..INODE_SIZE {
            if index % 64 == 0 {print!(".")};
            let inode = Inode::read(index as u32);
            if inode.name() == name {return Some(File::from_inode(inode))};
        }
        return None;
    }

    pub fn inodes(buffer: &mut Vec<Inode>) {
        InodeBitmap::get_allocated(buffer);
    }

    pub fn debug() {
        log!("==== Inodes ====");
        let mut inodes = Vec::new();
        Self::inodes(&mut inodes);
        for inode in &inodes {
            log!("inode: {}", inode);
        }
    }
}

pub struct DataBlocks;

impl DataBlocks {
    pub fn read(index: u32) -> DataNode {
        let block = Block::read(logical_to_physical(DATA_BASE, index)).unwrap();
        DataNode::new(index, block.data())
    }

    pub fn allocate(count: usize, buffer: &mut Vec<DataNode>) -> usize {
        print!("Allocating {} Blocks", count);
        let mut pct = count / 100;
        if pct == 0 {pct = 1};

        for i in 0..count {
            if i % pct == 0 { print!("{:02.3}%\r", (i as f32 / count as f32) * 100.0); }
            if let Some(block) = DataNode::alloc() {
                buffer.push(block);
            } else {
                return i;
            }
        }

        return count;
    }
}


pub struct SuperBlock {
    partition_num: u32,
    physical_offset: u32,
}

pub struct DataNode {
    logical_address: u32,
    data: [u8; 512],
}

impl DataNode {
    pub fn new(index: u32, data: &[u8]) -> DataNode {
        let mut buffer = [0; 512];
        for i in 0..data.len() {
            buffer[i] = data[i];
        }
        DataNode { logical_address: index, data: buffer }
    }

    pub fn alloc() -> Option<DataNode> {
        if let Some(index) = DataBitmap::allocate_next() {
            let block = Block::read(logical_to_physical(DATA_BASE, index)).unwrap();
            return Some(DataNode::new(index, block.data()));
        } else {
            None
        }
    } 

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn logical_addr(&self) -> u32 {
        self.logical_address
    }

    pub fn physical_addr(&self) -> u32 {
        logical_to_physical(DATA_BASE, self.logical_address)
    }

    pub fn sync(&self) {
        let mut block = Block::read(self.physical_addr()).unwrap();
        for i in 0..self.data.len() {
            block[i] = self[i];
        }
        block.write();
    }


}

impl Index<usize> for DataNode {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data()[index]
    }
}

impl IndexMut<usize> for DataNode {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data_mut()[index]
    }
}

pub struct Directory {
    inode: Inode,
    children: Vec<Inode>,
    parent: Option<Inode>
}

impl Directory {
    pub fn new(name: &str, parent: Option<Inode>) -> Self {
        Self {
            inode: Inode::new(0,String::from(name), InodeFlags::dir(), Vec::new(), None),
            children: Vec::new(),
            parent
        }
    }
}

#[derive(Debug)]
pub struct File {
    pos: u32,
    inode: Inode,
    data: Vec<u8>,
}

impl File {

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn inode(&self) -> &Inode {
        &self.inode
    } 

    pub fn log_addr(&self) -> u32 {
        return self.inode.addr
    }

    pub fn open(name: &str) -> Option<File> {
        InodeBlocks::open_file(name)
    }

    pub fn open_or_create(name: &str) -> Option<File> {
        if let Some(file) = Self::open(name) {
            Some(file)
        } else {
            Self::new(name)
        }
    }

    pub fn new(name: &str) -> Option<File> {
        let inode = InodeBlocks::create_file(name, None);
        if let Some(inode) = inode {
        Some(File::from_inode(inode))
        } else { None }
    }

    
    pub fn from_inode(node: Inode) -> Self {
        let mut data = Vec::new();
        for index in node.children() {
            let block = DataBlocks::read(*index);
            for i in 0..block.data().len() {
                if data.len() > node.size as usize {break;}
                data.push(block[i]);
            }
        }
        Self {
            data,
            inode: node,
            pos: 0,
        }
    }

    pub fn write(&mut self, value: u8) {
        if !self.inode.flags().can_write() {panic!("File Cannot Be Written To!")}
        assert!(self.pos < self.size() as u32, "Position Out Of Bounds, pos: {}, size: {}", self.pos, self.size());
        self.data[self.pos as usize] = value;
        self.pos += 1;
    }

    pub fn read(&mut self) -> u8 {
        if !self.inode.flags().can_read() {panic!("File Cannot Be Read")}
        assert!(self.pos < self.size() as u32);
        let value = self.data[self.pos as usize];
        self.pos += 1;
        value
    }

    pub fn append(&mut self, data: u8) {
        self.data.push(data);
    }

    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len)
    }

    pub fn copy_into(&mut self, buffer: &mut [u8]) -> usize {
        self.to_start();
        if self.can_operate_on(buffer.len()) {
            for index in 0..buffer.len() {
                buffer[index] = self.read();
            }
            buffer.len()
        } else {
            warn!("Buffer Size Is Too Large!");
            0
        }

    }

    pub fn seek(&mut self, pos: u32) {
        assert!(pos < self.size() as u32);
        self.pos = pos;
    }

    pub fn to_start(&mut self) {
        self.seek(0);
    }

    pub fn to_end(&mut self) {
        self.seek((self.size() - 1) as u32);
    }

    pub fn can_operate_on(&mut self, buffer_size: usize) -> bool {
        return (buffer_size + self.pos as usize) < self.size(); 
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn close(&mut self) {
        self.inode.clear_children();
        let chunks = self.data.chunks(BLOCK_SIZE);
        let mut blocks = Vec::new();
        breakpoint!("Allocating {} Data Blocks",chunks.len());
        DataBlocks::allocate(chunks.len(), &mut blocks);
        serial_print!("Writing To Disk");
        for (idx,chunk) in chunks.enumerate() {
            let mut block = &mut blocks[idx];
            for idx in 0..chunk.len() {
                block[idx] = chunk[idx];
            }
            serial_print!(".");
            block.sync();
        }
        serial_println!();
        debug!("Adding {} Blocks To Children", blocks.len());

        for block in blocks {
            self.inode.add_child(block.logical_addr());
        }

        self.inode.size = self.data.len() as u32;

        debug!("Writing Out Inode");
        self.inode.write();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InodeFlags(u16);

impl InodeFlags {
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    pub fn file() -> Self {
        use inode_flags::*;
        Self(FILE | ROOT_READ | ROOT_WRITE)
    }

    pub fn dir() -> Self {
        use inode_flags::*;
        Self(DIR | ROOT_READ | ROOT_WRITE)
    }

    pub fn dev() -> Self {
        use inode_flags::*;
        Self(DIR | ROOT_READ | ROOT_WRITE)
    }

    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn is_file(&self) -> bool {
        return self.0 & FILE > 0;
    }

    pub fn is_device(&self) -> bool {
        return self.0 & DEV > 0;
    }

    pub fn is_dir(&self) -> bool {
        return self.0 & DIR > 0;
    }

    pub fn can_read(&self) -> bool {
        return self.0 & ROOT_READ > 0;
    }

    pub fn can_write(&self) -> bool {
        return self.0 & ROOT_WRITE > 0;
    }

    pub fn can_exec(&self) -> bool {
        return self.0 & ROOT_EXEC > 0;
    }

    pub fn is_hidden(&self) -> bool {
        return self.0 & HIDDEN > 0;
    }

    pub fn set_flag(&mut self, flag_bit: u16) -> &mut Self {
        self.0 |= flag_bit;
        self
    }

    pub fn clear_flag(&mut self, flag_bit: u16) -> &mut Self {
        self.0 &= !flag_bit;
        self
    }
}

impl Display for InodeFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0 & inode_flags::DEV >  0{write!(f, "Dv-")?;};
        if self.0 & inode_flags::DIR >  0{write!(f, "Dr-")?;};
        if self.0 & inode_flags::FILE > 0{write!(f, "Fi-")?;};

        if self.0 & inode_flags::HIDDEN > 0{write!(f, "H")?;} else {write!(f, "-")?;};
        if self.0 & inode_flags::ROOT_READ > 0{write!(f, "R")?;} else {write!(f, "-")?;};
        if self.0 & inode_flags::ROOT_WRITE > 0{write!(f, "W")?;} else {write!(f, "-")?;};
        if self.0 & inode_flags::ROOT_EXEC > 0{write!(f, "X")?;} else {write!(f, "-")?;};

        Ok(())
    }
}

impl Display for Inode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Inode: [name: {}, Flags: {}, children: {:?}, parent: {:?}, Size: {}]",
            self.name,
            self.flags,
            self.children,
            self.parent,
            self.size,
        )
    }
}