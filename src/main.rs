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

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use sys::{mem, timer};


use crate::{arch::i386::cmos::{self}, sys::{ata, clock, keyboard, net, pci, storage::fs::{FILETABLE_ADDR, bitmap, file_table}, vfs::filesystem::{DataNode, File, Inode, InodeBitmap, InodeBlocks, InodeFlags, filesystem_values::{self, BLOCK_SIZE}, inode_flags::HIDDEN}}};

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

	log!("METADATA Block Size: {}", filesystem_values::METADATA_SIZE);
	log!("USABLE Block Size: {}", filesystem_values::USABLE_SIZE);
	log!("Inode Byte Address: 0x{:08x}", filesystem_values::INODE_BASE * BLOCK_SIZE as u32);


	let rtc = cmos::CMOS::new().rtc();
	println!("Current Time: {}/{}/20{} {}:{}:{}", rtc.day, rtc.month, rtc.year, rtc.hour, rtc.minute, rtc.second);
	println!("Unix TimeStamp: {}", clock::realtime());
	mem::init(boot_info);
	pci::init();
	//net::init();
	ata::init();

	//breakpoint!();

	




	print!("Press Any Key To Continue!");
	while keyboard::last_char().is_none() {sys::timer::pause(0.01)}
	clear!();
	run!("fs mount ata 0 1");
	//unsafe {InodeBitmap::erase_all()};
	let file = File::open_or_create("Test.txt");
	let mut file = file.unwrap();
	println!("{:?}", file);
	println!("File Data: {:?}",file.data());

	file.append(0xAA);

	file.close();

	

	log!("{:?}", file);


	//InodeBlocks::debug();


	
	
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

