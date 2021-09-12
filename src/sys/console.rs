use super::vga::{self, Color, ColorCode, ScreenBuffer, ScreenChar};
use crate::sys;

struct RingBuffer<T: Sized + Default> {
    write_ptr: usize,
    read_ptr: usize,
    data: [T; 32],
}

impl<T: Sized + Default + Copy> RingBuffer<T> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); 32],
            read_ptr: 0,
            write_ptr: 0,
        }
    }

    pub fn read(&mut self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let value = &self.data[self.read_ptr];
            self.read_ptr += 1;
            self.read_ptr %= 32;
            Some(value)
        }
    }


    pub fn write(&mut self, item: T) {
        if self.is_full() {
            
        } else {
            self.data[self.write_ptr] = item;
            self.write_ptr += 1;
            self.write_ptr %= 32;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.read_ptr == self.write_ptr
    }

    pub fn is_full(&self) -> bool {
        self.write_ptr + 1 == self.read_ptr
    }
}

pub struct Style(Color, Color);

impl Default for Style {
    fn default() -> Self {
        Self(Color::White, Color::Blue)
    }
}

impl Style {
    pub fn bg(&self) -> Color {
        self.1
    }

    pub fn fg(&self) -> Color {
        self.0
    }

    pub fn colorcode(&self) -> ColorCode {
        ColorCode::new(self.fg(), self.bg())
    }
}

pub struct Console {
    buffer: &'static mut ScreenBuffer,
    command_buffer: RingBuffer<char>,
    x: usize, y: usize,
    style: Style,
}

impl Console {
    pub fn new() -> Self {
        Self {
            command_buffer: RingBuffer::new(),
            buffer: ScreenBuffer::new(0x1000),
            x: 0, y: 0,
            style: Style::default()
        }
    }

    fn swap(&self) {
        vga::set_text_buffer(&*self.buffer)
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }
}

