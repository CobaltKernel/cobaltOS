mod idt;
mod gdt;
mod pics;
pub fn init() {
	gdt::init();
	idt::init();
	unsafe {pics::init()};
	
}

pub fn enable() {
	x86_64::instructions::interrupts::enable();
}

