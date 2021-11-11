pub mod paging;
pub mod page_tables;
pub mod frame_alloc;
pub mod allocator;
pub mod heap;
pub mod allocators;


use core::{convert::TryInto};
use bootloader::{BootInfo, bootinfo::MemoryMap};

use linked_list_allocator::LockedHeap;
use x86_64::{PhysAddr, VirtAddr, structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Translate}};

use crate::println;

use self::frame_alloc::BootFrameAllocator;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub(crate) static mut PHYS_MEM_OFFSET: u64 = 0;
pub(crate) static mut MEMORY_MAP: Option<&MemoryMap> = None;
pub(crate) static mut FRAME_ALLOCATOR: Option<BootFrameAllocator> = None;
pub(crate) static mut MAPPER: Option<OffsetPageTable> = None;


pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
pub const TB: usize = 1024 * GB;

pub const HEAP_SIZE: usize = 1 * MB;
pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_END: u64 = HEAP_START + HEAP_SIZE as u64 + 1u64;

pub fn init(info: &'static BootInfo) {
    let phys_offset = info.physical_memory_offset;
    let mut mapper = unsafe { paging::init_mapper(phys_offset) };
    let mut frame_allocator = frame_alloc::BootFrameAllocator::new(&info.memory_map);
    println!("{} MB of Memory Detected...", frame_allocator.get_mem_size() / MB as u64);

    


    unsafe {
        PHYS_MEM_OFFSET = phys_offset;
        MEMORY_MAP = Some(&info.memory_map);
        FRAME_ALLOCATOR = Some(frame_allocator);
    };

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

pub fn alloc_frame() -> Option<PhysFrame> {
    unsafe {
        if let Some(mut allocator) = FRAME_ALLOCATOR {
            return allocator.allocate_frame();
        };
        return None;
    }
}

pub fn alloc_page(addr: VirtAddr, flags: PageTableFlags) -> Option<Page> {
    unsafe {
        if let Some(frame) = alloc_frame() {
            if let Some(mapper) = &mut MAPPER {
                mapper.map_to(Page::containing_address(addr), frame, flags, &mut FRAME_ALLOCATOR.unwrap()).expect("Failed To Allocate Page").flush();
                return Some(Page::containing_address(addr));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

fn map_page(page: Page, flags: PageTableFlags) {
    unsafe {
        if let Some(frame) = alloc_frame() {
            if let Some(mapper) = &mut MAPPER {
                mapper.map_to(page, frame, flags, &mut FRAME_ALLOCATOR.unwrap()).expect("Failed To Allocate Page").flush();
            } 
        }
    }
} 

pub unsafe fn grow_heap(amount: usize) {
    let allocator = &mut *ALLOCATOR.lock();
    let new_end = allocator.top() + amount;
    let new_end = VirtAddr::new(new_end as u64);
    let end = allocator.top();
    let end = VirtAddr::new(end as u64);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let pages = Page::range_inclusive(Page::containing_address(end), Page::containing_address(new_end));

    for page in pages {
        map_page(page, flags);
    }


    allocator.extend(amount);

}
