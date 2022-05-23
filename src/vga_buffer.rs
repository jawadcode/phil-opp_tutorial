use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Blue, Color::Black),
        // Spooky 👻
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// The colour of a character
#[rustfmt::skip]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // Ensures the colour takes up only 1 byte, although the maximum used is 4 bits
pub enum Color {
    Black      = 0x0,
    Blue       = 0x1,
    Green      = 0x2,
    Cyan       = 0x3,
    Red        = 0x4,
    Magenta    = 0x5,
    Brown      = 0x6,
    LightGray  = 0x7,
    DarkGray   = 0x8,
    LightBlue  = 0x9,
    LightGreen = 0xA,
    LightCyan  = 0xB,
    LightRed   = 0xC,
    Pink       = 0xD,
    Yellow     = 0xE,
    White      = 0xF,
}

/// The colour code of a character, composed of the 4 bit background and foregound `Color`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // Ensures the newtype is laid out in memory equivalently to its inner type (`u8`)
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A character which can be written to video memory and printed, composed of the 8 bit char code and the 8 bit colour code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // Ensures the
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// The size of the VGA text buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// The VGA text buffer
#[repr(transparent)] // Ensures the struct is laid out equivalently to `chars`
struct Buffer {
    /// The characters currently being displayed (`volatile::Volatile` prevents reads/writes from being optimised away)
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A safe abstraction for writing to the buffer
pub struct Writer {
    /// The current position in the last row
    column_position: usize,
    /// The current background and foreground colour
    color_code: ColorCode,
    /// A static ref to the buffer itself
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Print any printable characters or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Invalid character
                _ => self.write_byte(0xfe),
            }
        }
    }
    /// Writes a byte to the buffer, substituting '\n' for a new line
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                // If the row is about to overflow, then move to a new one
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line()
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                // Write to the buffer
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    /// Advance to a new line
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

    /// Clears a row by overwiting all of its characters with a space character
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
