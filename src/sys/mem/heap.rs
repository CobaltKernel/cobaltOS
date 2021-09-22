

use x86_64::{VirtAddr, structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError, page}};
use crate::sys::mem;

use lazy_static::lazy_static;
use spin::Mutex;



use super::{HEAP_SIZE, HEAP_START, paging};
pub fn init(mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut mem::frame_alloc::BootFrameAllocator,
) -> Result<(), MapToError<Size4KiB>> {


    let size = HEAP_SIZE as u64;
    let page_range = {
        
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + size - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let page_count = page_range.count();
    let mut index: usize = 0;
    let ten_percent: usize = page_count / 4;
    let mut progress: usize = 0;
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        crate::print!("Initializing Heap ({} MB) - {:02.3}%...                      \r", size as usize / mem::MB, (index as f64 / page_count as f64) * 100.0);


        index += 1;

        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };

    }
    crate::println!("Initializing Heap ({} MB) - {:02.3}% - [OK]                   \r", size / mem::MB as u64, (index as f64 / page_count as f64) * 100.0);
    Ok(())
}
