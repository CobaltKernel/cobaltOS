#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(cobalt_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(asm_sym)]
#![feature(llvm_asm)]
extern crate alloc;
use bootloader::{BootInfo, entry_point};
use cobalt_os::*;
use cobalt_os::sys::mem::HEAP_START;
use sys::*;
use arch::i386;
use i386::cmos;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;
entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static BootInfo) -> ! {

	clear!();
	log!("Initializing Interrupts...");
	interrupts::init();
	printk!("[OK]\n");
	log!("Initializing Timer...");
	timer::init();
	printk!("[OK]\n");
	log!("Enabling Interrupts...");
	interrupts::enable();
	printk!("[OK]\n");



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

	

	mem::alloc_page(VirtAddr::new(0x1_FFFF_FFFF), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

	//read_file("");

	

	print!("Press Any Key To Continue!");
	while keyboard::last_char().is_none() {sys::timer::pause(0.01)}
	clear!();
	run!("fs mount ata 0 1");

	//let mut files = Vec::new();
	//vfs::list(&mut files);



	//let mut buf = Vec::new();
	//vfs::load("root/boot/message.txt", &mut buf);

	//let msg =  String::from_utf8(buf).unwrap();


	//print_at!(40 - (msg.len() / 2), 12, &msg);
	//print!("\r");	


	cobalt_os::dump_mem_phys(HEAP_START as *const u8, 256);

	sys::shell::start();

	sys::acpi::shutdown();
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}




