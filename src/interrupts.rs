use crate::arch::i386::interrupts::{gdt, idt, pics};
pub fn init() {
	gdt::init();
	idt::init();
	unsafe {pics::init()};
	
}

pub fn enable() {
	x86_64::instructions::interrupts::enable();
}

// pub fn set_handler(irq: u8, handler: fn()) {

// }