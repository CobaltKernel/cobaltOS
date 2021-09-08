
use x86_64::structures::idt::{InterruptStackFrame, InterruptDescriptorTable, PageFaultErrorCode};
use crate::{serial_println, serial_print};
use super::{gdt, pics::*};

use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(on_breakpoint);
		unsafe {
			idt.double_fault.set_handler_fn(on_double_fault)
			.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
		}
		idt.page_fault.set_handler_fn(on_page_fault);

		idt[InterruptIndex::Timer.as_usize()].set_handler_fn(on_timer_tick); 		

        idt
    };
}


pub fn init() {
	unsafe {
		IDT.load();
	}
}

extern "x86-interrupt" fn on_breakpoint(_: InterruptStackFrame) {
	serial_println!("Breakpoint Hit!");
} 

extern "x86-interrupt" fn on_double_fault(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}


extern "x86-interrupt" fn on_page_fault(stack_frame: InterruptStackFrame, ec: PageFaultErrorCode) {
	serial_println!("Page Fault: {:?},  Addr: {:?}",ec,x86_64::registers::control::Cr2::read());
}

extern "x86-interrupt" fn on_timer_tick(
    _stack_frame: InterruptStackFrame)
{
    crate::sys::timer::increment();
	send_eoi(InterruptIndex::Timer.as_u8());
}
