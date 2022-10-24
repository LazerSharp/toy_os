
#![allow(dead_code)]

use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;

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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);


impl ColorCode {
    fn new(fg_color: Color, bg_color: Color) -> Self {
        ColorCode(((bg_color as u8) << 4) | (fg_color as u8)) 
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_code: u8,
    color_code: ColorCode
}


const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25; 

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH] ;BUFFER_HEIGHT]
}

struct Writer {
    buffer: & 'static mut Buffer,
    color_code: ColorCode,
    line_position: usize
}

impl Writer {
    fn new() -> Self {
        Writer { 
            color_code: ColorCode::new(Color::LightGreen, Color::Black), 
            buffer: unsafe {&mut  *(0xb8000 as *mut Buffer)}, 
            line_position: 0 
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row - 1][col].write(self.buffer.chars[row][col].read());
            }
           
        }

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT -1][col].write(ScreenChar {
                ascii_code: b' ',
                color_code: ColorCode::new(Color::Black, Color::Black)
            })
        }
        self.line_position = 0;

    }

    fn write_byte(&mut self, byte: u8) {

        match byte {
            b'\n' => {self.new_line()}
            byte => {

                if  self.line_position >= BUFFER_WIDTH {
                    self.new_line();
                    //self.write_byte(byte);
                }

                let row = BUFFER_HEIGHT - 1;
                let column = self.line_position; 
                self.buffer.chars[row][column].write(ScreenChar {
                    ascii_code: byte,
                    color_code: self.color_code
                });
                self.line_position += 1;
            }
        }

    }

    fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe)
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.write_string(text);
        Ok(())
    }
}


lazy_static! {
    static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}


pub fn _print_something(text: &str) {
    WRITER.lock().write_string(text);
}


pub fn _print(args : fmt::Arguments) {
    use fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => {$crate::print!("\n")};
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! bg {
    () => {$crate::vga_buffer::_set_bg_color($crate::vga_buffer::Color::Black)}; // reset
    ($color:tt) => {
        $crate::vga_buffer::_set_bg_color($crate::vga_buffer::Color::$color);
    };
}

pub fn _set_bg_color(bg_color: Color) {
    let mut w = WRITER.lock();
    w.color_code.0 = ((w.color_code.0) & 0x0F) | ((bg_color as u8) << 4) ;
}

#[macro_export]
macro_rules! fg {
    () => {$crate::vga_buffer::_set_fg_color($crate::vga_buffer::Color::LightGreen)}; // reset
    ($color:tt) => {
        $crate::vga_buffer::_set_fg_color($crate::vga_buffer::Color::$color);
    };
}

pub fn _set_fg_color(bg_color: Color) {
    let mut w = WRITER.lock();
    w.color_code.0 = ((w.color_code.0) & 0xF0) | ((bg_color as u8)) ;
}