use core::fmt;
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

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte), // "ascii" character range

                _ => self.write_byte(0xfe), // square for unrecognised chars
            }
        }
    }

    fn new_line(&mut self) {
        return;
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

pub fn print_someting() {
    use core::fmt::Write;
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Brown, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, //cast direction as raw pointer then
                                                           //convert it to a mutable reference by dereferencing it (*) and then borrowing it again
                                                           //with &mut requieres unsafe since the raw pointer cannot be assured is valid
    };

    writer.write_byte(b'H');
    writer.write_string("eelo ");
    write!(writer, "The numbers are {} and {}", 42, 1.0 / 3.0).unwrap();
}
