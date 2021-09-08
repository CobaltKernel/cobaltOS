#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod systems;
mod serial;
mod interrupts;

pub mod sys;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
	interrupts::init();	
	serial_println!("Hello, World!");
	x86_64::instructions::interrupts::int3();
	serial_println!("Resuming From BP!");
	sys::timer::init();
	interrupts::enable();
	
	let mut pb = sys::ansi_widgets::ProgressBar::new(0, 10, "Test PB");
	for i in 0..10 {
		pb.set_value(i);
		pb.draw();
		sys::timer::pause(1.0);
	}

	serial_println!("Stopping!");
	sys::halt();
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
	serial_println!("PANIC: {}", info);
	loop {}
}


