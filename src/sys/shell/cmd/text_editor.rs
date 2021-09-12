use core::ops::Add;

use alloc::string::String;
use alloc::vec::Vec;
use crate::sys::keyboard;
use crate::sys::shell::pause;
use crate::{clear, clear_line, print, print_at, run, set_style};
use crate::sys::vga::Color;
pub fn main(args: &Vec<&str>) -> usize {
    run!("clear");
    
    let mut text_buffer = String::new();
    
    
    loop {
        set_style!(Color::Black, Color::White);
        clear_line!(0);
        print_at!(0,0,"Text Editor - v0.1.0");
        set_style!(Color::White, Color::Blue);
        if let Some(chr) = keyboard::consume_char() {
            if chr == '\x08' {
                text_buffer.pop(); 
            } else if chr == '\x1b' { 
                break;
            } else if chr != '\x08' {
                text_buffer.push(chr);
            } 
        }

        for (index, line) in text_buffer.lines().enumerate() {
            clear_line!(1 + index);
            print_at!(0, 1 + index, line);
            clear_line!(1 + index + 1);
        }
        run!("pause 0.01");
    }
    run!("clear");
    return 0;
}