#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(CobaltOS::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use bootloader::{BootInfo, entry_point};
use CobaltOS::*;
use sys::*;
use arch::i386::cmos;
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

	//breakpoint!();

	




	print!("Press Any Key To Continue!");
	while keyboard::last_char().is_none() {sys::timer::pause(0.01)}
	clear!();
	run!("fs mount ata 0 1");
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
	
	sys::shell::start();

	sys::acpi::shutdown();
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}



