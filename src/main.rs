#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]
#![feature(asm)]

extern crate alloc;

mod serial;
mod interrupts;
mod macros;

pub mod sys;

use core::panic::PanicInfo;

use alloc::boxed::Box;
use sys::{console::Console, mem, timer};
use x86_64::{VirtAddr, structures::paging::Page};

use crate::sys::{ata, keyboard, pci};

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
	pci::init();
	ata::init();

	ata::init();

	print!("Press Any Key To Continue!");
	while keyboard::last_char().is_none() {sys::timer::pause(0.01)}
	clear!();
	//format_ata(0, 1);
	
	sys::shell::start();

	sys::acpi::shutdown();
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
	println!("PANIC: {}", info);
	serial_println!("PANIC: {}", info);
	sys::halt();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
	sys::halt();
}

