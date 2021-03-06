//! The Main CobaltOS Library

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]
#![feature(asm)]
#![feature(asm_sym)]
#![feature(naked_functions)]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

//#![warn(missing_docs)]

extern crate alloc;
use alloc::string::String;
use iced_x86::{DecoderOptions, Instruction, NasmFormatter};
use x86_64::VirtAddr;

pub mod serial;
pub mod interrupts;
pub mod macros;
pub mod arch;
pub mod sys;
pub mod device;

pub mod api;


use core::panic::PanicInfo;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static bootloader::BootInfo) -> ! {

    use crate::{arch::i386::cmos, sys::{clock, net, pci, timer}};

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
	sys::mem::init(boot_info);
	pci::init();
	net::init();
	sys::ata::init();
    test_main();
    loop {}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    serial_println!("{}", info);
    sys::halt();
}

const HEXBYTES_COLUMN_BYTE_LENGTH: usize = 10;

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    sys::halt();
}

pub fn dump_instructions(ptr: VirtAddr, len: usize) {
    let bin: &[u8] = unsafe { 
        core::slice::from_raw_parts(ptr.as_ptr(), len)
    };

    let mut decoder = iced_x86::Decoder::new(64, bin, DecoderOptions::NONE);
    decoder.set_ip(ptr.as_u64());

    let _output = String::new();

    let instruction = Instruction::default();

    for ins in decoder.iter() {
        print!("{:016X} | ", ins.ip());

        let start_index = (ins.ip() - ptr.as_u64()) as usize;
        let instr_bytes = &bin[start_index..start_index + ins.len()];
        for b in instr_bytes.iter() {
            print!("{:02X} ", b);
        }
        if instr_bytes.len() < HEXBYTES_COLUMN_BYTE_LENGTH {
            for _ in 0..HEXBYTES_COLUMN_BYTE_LENGTH - instr_bytes.len() {
                print!("   ");
            }
        }

        println!(" {}", instruction);
    }


}


pub fn dump_instructions_phys(ptr: *const u8, len: usize) {
    let bin: &[u8] = unsafe { 
        core::slice::from_raw_parts(ptr, len)
    };

    let mut decoder = iced_x86::Decoder::new(64, bin, DecoderOptions::NONE);
    decoder.set_ip(ptr as u64);

    for ins in decoder.iter() {
        serial_print!("${:016X} | ", ins.ip());

        let start_index = (ins.ip() - ptr as u64) as usize;
        let instr_bytes = &bin[start_index..start_index + ins.len()];
        for b in instr_bytes.iter() {
            serial_print!("{:02X} ", b);
        }
        if instr_bytes.len() < HEXBYTES_COLUMN_BYTE_LENGTH {
            for _ in 0..HEXBYTES_COLUMN_BYTE_LENGTH - instr_bytes.len() {
                serial_print!("   ");
            }
        }

        serial_println!("| {}", ins);
    }

    
    
    


}

pub fn dump_mem_phys(ptr: *const u8, len: usize) {

    for row in (0..len).step_by(16) {
        serial_print!("${:016X}: ", row + ptr as usize);
        for col in 0..16 {
            let ptr_offset = (row + col + ptr as usize) as *const u8;
            serial_print!("{:02X}, ", unsafe {*(ptr_offset)});
        }
        serial_println!();
    }
}


#[macro_export]
macro_rules! sprintk {
    ($($args:tt)*) => {
        $crate::serial::_print(format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! vprintk {
    ($($args:tt)*) => {
        $crate::sys::vga::_print(format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! printk {
    ($($args:tt)*) => {
        $crate::sys::vga::_print(format_args!($($args)*));
        $crate::serial::_print(format_args!($($args)*));
    };
}


pub type KResult<T> = core::result::Result<T, &'static str>;
