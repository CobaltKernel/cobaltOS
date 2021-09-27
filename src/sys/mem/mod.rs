pub mod paging;
pub mod frame_alloc;
pub mod allocator;
pub mod heap;
pub mod allocators;


use core::{convert::TryInto};
use bootloader::BootInfo;

use linked_list_allocator::LockedHeap;
use x86_64::{PhysAddr, VirtAddr, structures::paging::{Translate}};

use crate::println;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut PHYS_MEM_OFFSET: u64 = 0;


pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
pub const TB: usize = 1024 * GB;

pub const HEAP_SIZE: usize = 16 * MB;
pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_END: u64 = HEAP_START + HEAP_SIZE as u64 + 1u64;

pub fn init(info: &'static BootInfo) {
    let phys_offset = info.physical_memory_offset;
    let mut mapper = unsafe { paging::init_mapper(phys_offset) };
    let mut frame_allocator = frame_alloc::BootFrameAllocator::new(&info.memory_map);
    println!("{} MB of Memory Detected...", frame_allocator.get_mem_size() / MB as u64);

    unsafe {PHYS_MEM_OFFSET = phys_offset};

    heap::init(&mut mapper, &mut frame_allocator).expect("Failed To Initialize Heap Space");
    unsafe {
        ALLOCATOR.lock().init(HEAP_START.try_into().unwrap(), HEAP_SIZE);
    }
}

#[inline]
pub fn size() -> usize {
    HEAP_SIZE
} 

#[inline]
pub fn used() -> usize {
    ALLOCATOR.lock().used()
} 

#[inline]
pub fn free() -> usize {
    ALLOCATOR.lock().free()
} 

#[inline]
pub fn available() -> usize {
    size() - used()
} 

pub fn phys_to_virt(address: PhysAddr) -> VirtAddr {
    VirtAddr::new(address.as_u64() + unsafe { PHYS_MEM_OFFSET })
}

pub fn virt_to_phys(address: VirtAddr) -> Option<PhysAddr> {
    let mapper = unsafe { paging::init_mapper(PHYS_MEM_OFFSET)};
    mapper.translate_addr(address)
}
