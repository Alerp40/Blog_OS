use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)] //allow unused code so compiler doesnt complain about each enum use
#[derive(Debug, Clone, Copy, PartialEq, Eq)] //make it printable and comparable
#[repr(u8)] //represents in the component under
//code for each color
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

lazy_static! {
    pub static ref  WRITER: Mutex<Writer> = Mutex::new(Writer { //mutex adds safe interiour mutability to
        //the writer static by using spin-lock based access
        //create a static WIRTER to make a global writer without
        //carrying a wirter instance around
        column_position: 0,
        color_code: ColorCode::new(Color::LightCyan, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)] // this makes sure the struct has the exact same data layout as a u8
struct ColorCode(u8); //contains the full color byte

//rust doesnt have u4 but the colour actually takes up 4 bits for foreground and 4 bits for
//background
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] //this makes sure the fields are represented exactly like C structs
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT], //Volatile makes it so that the
                                                                  //compiler doesnt overoptimize and delete the writes as we dont ever read from memory in vga
                                                                  //buffer only write, it also makes it so that we can only access it by a .write method
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, //static lifetime makes it stay alive for the whole program run
                                 //time so it isnt garbage collected
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),

            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    //this assures us the compiler wont ever optimize away the write
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..row {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte), // "ascii" character range

                _ => self.write_byte(0xfe), // square for unrecognised chars
            }
        }
    }

    fn new_line(&mut self) {
        //iterate and re write everything one line above to make space for the
        //new line, like a typewriter would it pushes everything up one and only writes in the same
        //line (we do not move the writer we move all the text)
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
}

//this enables support for rusts write macros on our custo writer
impl fmt::Write for Writer {
    //this is the only required method for the
    //Wirte implementation
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
