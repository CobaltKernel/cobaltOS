#![no_std]
#![no_main]

mod systems;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
	systems::output::Output::write("Hello, Bastards!");
	loop {}
}

#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
	loop {}
}
