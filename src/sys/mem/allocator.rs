use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;



pub struct NullAllocator;

unsafe impl GlobalAlloc for NullAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unimplemented!("Should NEVER BE CALLED")
    }
}

