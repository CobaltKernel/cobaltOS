use self::dev_handle::DeviceHandle;

pub mod block_device;
pub mod block;
pub mod bitmap;
pub mod dev_handle;

use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};

pub(self) type BlockAddr = u32;

pub(self) const BLOCK_SIZE: usize = 512; 
pub(self) const DISK_SIZE: usize = 262144;
pub(self) const KERNEL_SIZE: usize = (32 << 20) / BLOCK_SIZE; // 32MB For Boot Code
pub(self) const SUPER_BLOCK_ADDR: BlockAddr = KERNEL_SIZE as BlockAddr;
pub(self) const SUPER_BLOCK_SIZE: usize = 2;
pub(self) const BITMAP_ADDR: BlockAddr = SUPER_BLOCK_ADDR + SUPER_BLOCK_SIZE as u32; 
pub(self) const BITMAP_SIZE: usize = 1024;
pub(self) const DATA_ADDR: BlockAddr = BITMAP_ADDR + BITMAP_SIZE as u32;
pub(self) const DATA_SIZE: usize = DISK_SIZE - DATA_ADDR as usize;

lazy_static! {
    pub static ref HANDLE: Mutex<Option<DeviceHandle>> = Mutex::new(None); 
}

pub fn mount_device(handle: DeviceHandle) {
    *HANDLE.lock() = Some(handle);
}

pub fn is_mounted() -> bool {
    return HANDLE.lock().is_some()
}

pub fn device<'a>() -> MutexGuard<'a, Option<DeviceHandle>> {
    return HANDLE.lock()
}



