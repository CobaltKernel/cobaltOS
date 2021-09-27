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

mod arch;

pub mod sys;

use core::panic::PanicInfo;

use alloc::{borrow::ToOwned};
use sys::{mem, timer};


use crate::{arch::i386::cmos::{self}, sys::{ata, clock, keyboard, net, pci, storage::fs::{FILETABLE_ADDR, bitmap, file_table}}};

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


	let rtc = cmos::CMOS::new().rtc();
	println!("Current Time: {}/{}/20{} {}:{}:{}", rtc.day, rtc.month, rtc.year, rtc.hour, rtc.minute, rtc.second);
	println!("Unix TimeStamp: {}", clock::realtime());
	mem::init(boot_info);
	pci::init();
	net::init();
	ata::init();

	breakpoint!();

	print!("Press Any Key To Continue!");
	while keyboard::last_char().is_none() {sys::timer::pause(0.01)}
	clear!();
	run!("fs mount ata 0 1");
	file_table::FileTable::create_file(FILETABLE_ADDR, &"/".to_owned(), 0, bitmap::Bitmap::next_free().unwrap(), 0, &"Test.txt".to_owned());
	
	//format_ata(0, 1);
	
	sys::shell::start();

	sys::acpi::shutdown();
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
	println!("PANIC: {}", info);
	err!("PANIC: {}", info);
	sys::halt();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

