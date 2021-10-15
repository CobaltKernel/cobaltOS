use alloc::vec::Vec;

use crate::sys::shell::run;

pub fn main(_args: &Vec<&str>) -> usize {
    run!("dsk copy 0 0 0 1")
}