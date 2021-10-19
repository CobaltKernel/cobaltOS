use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;
use crate::arch::i386::interrupts::gdt::GDT;
use crate::sys;
use object::Object;
use object::ObjectSegment;
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::Mapper;
use x86_64::structures::paging::Page;
use x86_64::{VirtAddr, structures::paging::PageTableFlags};

use super::mem::frame_alloc;

static STACK_ADDR: AtomicU64 = AtomicU64::new(0x200_0000);
static CODE_ADDR: AtomicU64 = AtomicU64::new(0x100_0000);
const PAGE_SIZE: u64 = 4 * 1024;

pub fn exec(bin: &[u8]) {
    let mut mapper = unsafe {
        sys::mem::paging::init_mapper(sys::mem::PHYS_MEM_OFFSET)
    };

    let mut frame_allocator = unsafe {
        frame_alloc::BootFrameAllocator::new(sys::mem::MEMORY_MAP.unwrap())
    };

    let flags = PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE | PageTableFlags::PRESENT;

    let stack_size = 256 * PAGE_SIZE;
    let stack_addr = STACK_ADDR.fetch_add(stack_size, Ordering::SeqCst);

    let pages = {
        let stack_start_page = Page::containing_address(VirtAddr::new(stack_addr));
        let stack_end_page = Page::containing_address(VirtAddr::new(stack_addr + stack_size));
        Page::range_inclusive(stack_start_page, stack_end_page)
    };

    for page in pages {
        let frame = frame_allocator.allocate_frame().unwrap();
        unsafe {
            mapper.map_to(page, frame, flags, &mut frame_allocator).unwrap().flush();
        }
    }

    let code_size = 1024 * PAGE_SIZE;
    let code_addr = CODE_ADDR.fetch_add(code_size, Ordering::SeqCst);
    let pages = {
        let code_start_page = Page::containing_address(VirtAddr::new(code_addr));
        let code_end_page = Page::containing_address(VirtAddr::new(code_addr + code_size));
        Page::range_inclusive(code_start_page, code_end_page)
    };

    for page in pages {
        let frame = frame_allocator.allocate_frame().unwrap();
        unsafe {
            mapper.map_to(page, frame, flags, &mut frame_allocator).unwrap().flush();
        }
    }

    let code_ptr = code_addr as *mut u8;
    let mut entry = 0;
    if &bin[1..4] == b"ELF" {
        if let Ok(obj) = object::File::parse(bin) {
            entry = obj.entry();
            for segment in obj.segments() {
                let addr = segment.address() as usize;
                if let Ok(data) = segment.data() {
                        unsafe {
                        for (i, op) in data.iter().enumerate() {
                            let ptr = code_ptr.add(addr + i);
                            core::ptr::write(ptr, *op)
                        }
                    }
                }
            }
        }
    } else {
        unsafe {
            for (i, op) in bin.iter().enumerate() {
                let ptr = code_ptr.add(i);
                core::ptr::write(ptr, *op);
            }
        }
    }


     //x86_64::instructions::tlb::flush_all();
     let data = GDT.1.user_data.0;
     let code = GDT.1.user_code.0;
     unsafe {
         asm!(
             "cli",        // Disable interrupts
             "push rax",   // Stack segment (SS)
             "push rsi",   // Stack pointer (RSP)
             "push 0x200", // RFLAGS with interrupts enabled
             "push rdx",   // Code segment (CS)
             "push rdi",   // Instruction pointer (RIP)
             "iretq",
             in("rax") data,
             in("rsi") stack_addr + stack_size,
             in("rdx") code,
             in("rdi") code_addr + entry,
         );
     }

}

