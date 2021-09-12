#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]

extern crate alloc;

mod serial;
mod interrupts;

pub mod sys;

use core::panic::PanicInfo;

use alloc::boxed::Box;
use sys::{console::Console, mem, timer};
use x86_64::{VirtAddr, structures::paging::Page};

use crate::sys::{ata, keyboard, timer::clear};

use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
	clear!();
	print!("Initializing Interrupts...");
	interrupts::init();
	println!("[OK]");
	print!("Initializing Timer...");
	timer::init();
	println!("[OK]");
	print!("Enabling Interrupts...");
	interrupts::enable();
	println!("[OK]");

	mem::init(boot_info);

	ata::init();

	print!("Press Any Key To Continue!");
	while keyboard::last_char().is_none() {sys::timer::pause(0.01)}
	clear!();

	sys::shell::start();

	sys::halt();
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
	println!("PANIC: {}", info);
	serial_println!("PANIC: {}", info);
	loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
	serial_println!("allocation error: {:?}", layout);
}

