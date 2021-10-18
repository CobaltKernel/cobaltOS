use alloc::vec::Vec;

use crate::{arch::i386::syscalls::syscall3, syscall};

pub fn main(args: &Vec<&str>) -> usize {
    let n: usize = usize::from_str_radix(args.iter().nth(1).unwrap_or(&""), 16).unwrap_or(0);
    let arg1: usize = usize::from_str_radix(args.iter().nth(2).unwrap_or(&""), 16).unwrap_or(0);
    let arg2: usize = usize::from_str_radix(args.iter().nth(3).unwrap_or(&""), 16).unwrap_or(0);
    let arg3: usize = usize::from_str_radix(args.iter().nth(4).unwrap_or(&""), 16).unwrap_or(0);

    unsafe {syscall3(n, arg1, arg2, arg3)}
}