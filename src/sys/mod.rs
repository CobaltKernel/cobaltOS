pub mod timer;
pub mod pit;
pub mod process;
pub mod storage;
pub mod keyboard;
pub mod vga;
pub mod mem;
pub mod shell;
pub mod ata;
pub mod acpi;
pub mod pci;
pub mod pci_details;
pub mod net;
pub mod clock;
pub mod vfs;
pub mod ustar;
pub mod device_manager;

use x86_64::instructions::port::*;

use crate::serial_println;

pub fn shutdown() -> ! {
    serial_println!("[SYS]: Shutting System Down!");
    acpi::shutdown();
}

pub fn halt() -> ! {
	loop { timer::pause(0.1); }
}

pub fn spinlock(poll_time: f64, action: fn() -> bool) {
	loop { if action() {return}; timer::pause(poll_time); }
}

pub unsafe fn outportb(port: u16, data: u8) {
	let mut port: Port<u8> = Port::new(port);
	port.write(data);
}

pub unsafe fn outportw(port: u16, data: u16) {
    let mut port: Port<u16> = Port::new(port);
	port.write(data);
}

pub unsafe fn outportdw(port: u16, data: u32) {
    let mut port: Port<u32> = Port::new(port);
	port.write(data);
}

pub unsafe fn inportb(port: u16) -> u8 {
    let mut port: Port<u8> = Port::new(port);
    port.read()
}

pub unsafe fn inportw(port: u16) -> u16 {
    let mut port: Port<u16> = Port::new(port);
    port.read()
}

pub unsafe fn inportdw(port: u16) -> u32 {
    let mut port: Port<u32> = Port::new(port);
    port.read()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn qemu_exit(error_code: QemuExitCode) -> ! {
    unsafe {outportdw(0xf4, error_code as  u32)};
    loop {}
}





#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (
        #[cfg(feature = "log_debug")]
        $crate::serial_print!("[debug]: {}\n", format_args!($($arg)*))
    );
}