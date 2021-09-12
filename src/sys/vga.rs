use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts::without_interrupts;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Blue),
        buffer: unsafe { &mut *(0xb8000 as *mut ScreenBuffer) },
    });
}

/// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    pub fn new(chr: u8, color: ColorCode) -> ScreenChar {
        ScreenChar {
            ascii_character: chr,
            color_code: color
        }
    }
}

/// The height of the text buffer (normally 25 lines).
const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer (normally 80 columns).
const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
pub struct ScreenBuffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl ScreenBuffer {
    pub fn set_contents(&mut self, other: &ScreenBuffer) {
        for (y, row) in self.chars.iter_mut().enumerate() {
            for (x, chr) in row.iter_mut().enumerate() {
              chr.write(other.chars[y][x].read());  
            }
        }
    }

    pub fn new(address: usize) -> &'static mut Self {
         unsafe { &mut *(address as *mut ScreenBuffer)}
    }

    pub fn read(&self, x: usize, y: usize) -> ScreenChar {
        self.chars[y][x].read()
    }

    pub fn write(&mut self, x: usize, y: usize, chr: ScreenChar) {
        self.chars[y][x].write(chr)
    }
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `ScreenBuffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut ScreenBuffer,
}

impl Writer {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.column_position = 0,
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_byte_at(&mut self, x: usize, y: usize, byte: u8) {
        match byte {
            b'\n' => {},
            b'\r' => {},
            byte => {

                let row = y;
                let col = x;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' | b'\r' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn write_string_at(&mut self, x:usize, y:usize, s: &str) {
        let mut offset = 0;
        for (index, byte) in s.bytes().enumerate() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' | b'\r' => self.write_byte_at(x + offset, y, byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
            offset += 1;
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear(&mut self) {
        for y in 0..BUFFER_HEIGHT {
            self.clear_row(y)
        }
        self.column_position = 0;
    }

    pub fn set_buffer_contents(&mut self, other: &ScreenBuffer) {
        self.buffer.set_contents(other);
    }

    pub fn set_style(&mut self, color: ColorCode) {
        self.color_code = color;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sys::vga::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    without_interrupts(|| {
        use core::fmt::Write;
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! clear {
    () => ($crate::sys::vga::_clear());
}

#[macro_export]
macro_rules! set_style {
    ($fg:expr, $bg:expr) => ($crate::sys::vga::_set_style($crate::sys::vga::ColorCode::new($fg, $bg)));
}


pub fn set_text_buffer(other: &ScreenBuffer) {
    without_interrupts(|| {
        WRITER.lock().set_buffer_contents(other);       
    }); 
}


pub fn _clear() {
    without_interrupts(|| {
        WRITER.lock().clear();     
    }); 
}

pub fn _set_style(color: ColorCode) {
    without_interrupts(|| {
        WRITER.lock().set_style(color);     
    }); 
}

#[macro_export]
macro_rules! print_at {
    ($x:expr, $y:expr, $text:expr) => (
        $crate::sys::vga::_print_str_at($x, $y, $text);
    );
}

#[macro_export]
macro_rules! clear_line {
    ($y:expr) => (
        $crate::sys::vga::_clear_line($y);
    );
}

pub fn _print_str_at(x: usize, y: usize, text: &str) {
    without_interrupts(|| {
        WRITER.lock().write_string_at(x, y, text);    
    }); 
}

pub fn _clear_line(y: usize) {
    without_interrupts(|| {
        WRITER.lock().clear_row(y);    
    }); 
}

