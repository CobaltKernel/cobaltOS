//! Contains System Call Numbers
pub const SLEEP: usize =        0b0000000;
pub const PRINT_BYTE: usize =   0b0000001;
pub const PRINT_STR: usize =    0b0000010;
pub const OPEN_FILE: usize =    0b0001000;
pub const CLOSE_FILE: usize =   0b0001001;
pub const READ_FILE:  usize =   0b0001010;




pub fn sleep(millis: usize) {
    unsafe { crate::syscall!(SLEEP, millis); }
}
