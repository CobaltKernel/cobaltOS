use x86_64::instructions::port::Port;




#[macro_export]
macro_rules! inb {
    ($port:expr) => {
        $crate::macros::_inb($port)
    };
}

#[macro_export]
macro_rules! inw {
    ($port:expr) => {
        $crate::macros::_inbw($port)
    };
}

#[macro_export]
macro_rules! indw {
    ($port:expr) => {
        $crate::macros::_indw($port)
    };
}

pub fn _inb(port: u16) -> u8 {
    let mut port = Port::new(port);
    unsafe { port.read() }
}

pub fn _inw(port: u16) -> u16 {
    let mut port = Port::new(port);
    unsafe { port.read() }

}

pub fn _indw(port: u16) -> u32 {
    let mut port = Port::new(port);
    
    unsafe { port.read() }
}

#[cfg(feature = "breakpoints")]
#[macro_export]
macro_rules! breakpoint {
    ($fmt:expr, $($arg:tt)*) => {
        
        $crate::println!(concat!("Breakpoint @ {}:{}:{}: ", $fmt), file!(), line!(), column!(), $($arg)*);
        $crate::serial_println!(concat!("Breakpoint @ {}:{}:{}: ", $fmt), file!(), line!(), column!(), $($arg)*);
        #[cfg(feature = "breakpoint")]
        x86_64::instructions::interrupts::int3();
        
    };

    () => {
        $crate::println!("Breakpoint @ {}:{}:{}", file!(), line!(), column!());
        x86_64::instructions::interrupts::int3();
        
    };
}

#[cfg(not(feature = "breakpoints"))]
#[macro_export]
macro_rules! breakpoint {
    ($fmt:expr, $($arg:tt)*) => {
    };

    () => {
    };
}


#[macro_export]
macro_rules! kmalloc { 
    ($t:ty) => {
        
    };
}


#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => (
        $crate::serial_print!("\n\x1b[32m[LOG]:\x1b[0m {}", format_args!($($arg)*));
        $crate::print!("\n[LOG]: {}", format_args!($($arg)*));
    );
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (
        $crate::serial_print!("\n\x1b[33m[WARN]:\x1b[0m {}", format_args!($($arg)*));
        $crate::print!("\n[WARN]: {}", format_args!($($arg)*));
    );
}

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => (
        $crate::serial_print!("\n\x1b[31m[ERROR]:\x1b[0m {}", format_args!($($arg)*))
        $crate::print!("\n[ERROR]: {}", format_args!($($arg)*));
    );
}