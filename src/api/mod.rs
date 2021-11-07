//! The Cobalt Standard Library

use crate::arch::i386::syscalls::calls::*;
use crate::syscall;

pub fn sleep(milliseconds: usize) {
    unsafe {syscall!(SLEEP, milliseconds);}
}

pub fn print(text: &str) {
    unsafe {syscall!(PRINT_STR, text.as_ptr() as usize, text.bytes().len());}
}
