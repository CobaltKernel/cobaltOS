//! Handles Serial Communication Over A UART-16550 RS-232 COMM Port.

use uart_16550::SerialPort;
use spin::{Mutex, MutexGuard};
use lazy_static::lazy_static;

use crate::device::CharDevice;


lazy_static! {
    #[allow(missing_docs)]
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}


#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

/// Read From The Serial Port
pub fn read() -> u8 {
    SERIAL1.lock().receive()
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}


impl CharDevice for SerialPort {

    fn read_u8(&self, _: usize) -> Option<u8> {
        Some(SERIAL1.lock().receive())
    }

    fn read_u16(&self, addr: usize) ->  Option<u16> {
        unimplemented!();
    }

    fn read_u32(&self, addr: usize) ->  Option<u32> {
        unimplemented!();
    }

    fn read_u64(&self, addr: usize) ->  Option<u64> {
        unimplemented!();
    }

    fn read_u128(&self, addr: usize) -> Option<u128> {
        unimplemented!();
    }

    fn write_u8  (&mut self, addr: usize, value: u8)   -> crate::KResult<()> {
        SERIAL1.lock().send(value);
        Ok(())
    }

    fn write_u16 (&mut self, addr: usize, value: u16)  -> crate::KResult<()> {
        let data = value.to_be_bytes();
        for byte in data {
            self.write_u8(addr, byte)?;
        }
        Ok(())
    }

    fn write_u32 (&mut self, addr: usize, value: u32)  -> crate::KResult<()> {
        let data = value.to_be_bytes();
        for byte in data {
            self.write_u8(addr, byte)?;
        }
        Ok(())
    }

    fn write_u64 (&mut self, addr: usize, value: u64)  -> crate::KResult<()> {
        let data = value.to_be_bytes();
        for byte in data {
            self.write_u8(addr, byte)?;
        }
        Ok(())
    }

    fn write_u128(&mut self, addr: usize, value: u128) -> crate::KResult<()> {
        let data = value.to_be_bytes();
        for byte in data {
            self.write_u8(addr, byte)?;
        }
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        Some(1)
    }

    fn slice(&self) -> Option<&[u8]> {
        None
    }

    fn slice_mut(&mut self) -> Option<&mut [u8]> {
        None
    }
}