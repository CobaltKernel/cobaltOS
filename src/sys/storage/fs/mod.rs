use crate::debug;

use self::dev_handle::{BlockDeviceIO, DeviceHandle};

pub mod block_device;
pub mod block;
pub mod bitmap;
pub mod dev_handle;
pub mod superblock;
pub mod file_table;

use lazy_static::lazy_static;
use spin::Mutex;

pub type BlockAddr = u32;


// == Partition Structure ==
// Bootcode: 32MB / 65536 Blocks
// Superblock: 2 Blocks, 65536..65538
// Bitmap: 1024 Blocks, 65538..66562
// Filetable: 1024 Blocks, 
// Data: 

pub const BLOCK_SIZE: usize = 512; 
pub const DISK_SIZE: usize = 128 * 1024 * 1024;
pub const KERNEL_SIZE: usize = (32 << 20) / BLOCK_SIZE; // 32MB For Boot Code
pub const SUPER_BLOCK_ADDR: BlockAddr = KERNEL_SIZE as BlockAddr;
pub const SUPER_BLOCK_SIZE: usize = 2;
pub const BITMAP_ADDR: BlockAddr = SUPER_BLOCK_ADDR + SUPER_BLOCK_SIZE as u32; 
pub const BITMAP_SIZE: usize = 1024;
pub const FILETABLE_ADDR: BlockAddr = BITMAP_ADDR + BITMAP_SIZE as u32;
pub const FILETABLE_SIZE: usize = 1024;
pub const DATA_ADDR: BlockAddr = FILETABLE_ADDR + FILETABLE_SIZE as u32;
pub const DATA_SIZE: usize = (DISK_SIZE - DATA_ADDR as usize) - FILETABLE_SIZE;

lazy_static! {
    pub static ref HANDLE: Mutex<Option<DeviceHandle>> = Mutex::new(None); 
}

pub fn mount_device(handle: DeviceHandle) {
    let mut handle = handle;
    debug!("Mounted Handle With Size Of {} Blocks", handle.sector_count());
    assert!(handle.sector_count() <= DISK_SIZE as u32);
    *HANDLE.lock() = Some(handle);
    //superblock::SuperBlock::mount();
}

pub fn is_mounted() -> bool {
    return HANDLE.lock().is_some()
}

pub fn device<'a>() -> &'a Mutex<Option<DeviceHandle>> {
    return &HANDLE
}



