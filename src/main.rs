#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(cobalt_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use bootloader::{BootInfo, entry_point};
use cobalt_os::*;
use cobalt_os::arch::i386::syscalls::calls;
use sys::*;
use arch::i386;
use i386::interrupts::*;
use i386::cmos;
use i386::syscalls;
use storage::fs::{dev_handle::*};
use device_manager::Device;
use alloc::vec::Vec;

use ustar::TarFileSystem;
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

	
	

	#[cfg(test)]
	test_main();

	//println!("{}", config::Interface::get());


	//breakpoint!();

	




	print!("Press Any Key To Continue!");
	while keyboard::last_char().is_none() {sys::timer::pause(0.01)}
	clear!();
	run!("fs mount ata 0 1");

	let dev = device_manager::get_device("ATA/0/1").unwrap();
	let dev = dev.block_dev().unwrap().clone();

	let mut files = Vec::new();
	vfs::list(&mut files);

	for meta in files.iter() {
		println!("File: {:?}", meta);
	}


	

	//unsafe {InodeBitmap::erase_all()};
	// let file = File::open_or_create("Test.txt");
	// let mut file = file.unwrap();
	// println!("{:?}", file);
	// println!("File Data: {:?}",file.data());

	// file.append(0xAA);

	// file.close();


	

	//log!("{:?}", file);


	//InodeBlocks::debug();


	
	
	//format_ata(0, 1);
	
	unsafe {
		syscall!(calls::SLEEP, 5000);
	}

	sys::shell::start();

	sys::acpi::shutdown();
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}



