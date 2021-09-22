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