use core::fmt;
use spin::Mutex;
use x86_64::instructions::port::Port;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

// Global wirter protect through a mutex for thread security
pub static WRITER: Mutex<VGAWriter> = Mutex::new(VGAWriter {
    column_position: 0,
    row_position: 0,
    color_code: 0x0F // white on black
});


pub struct VGAWriter {
    pub column_position: usize,
    pub row_position: usize,
    pub color_code: u8,
}

// Everything that uses self needs to be in this block because rust compiler likes perfection (I dont)
impl VGAWriter {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => {
                if self.column_position > 0 {
                    self.column_position -= 1;
                    let row = self.row_position;
                    let col = self.column_position;
                    let offset = ((row * VGA_WIDTH +col) * 2) as isize;
                    unsafe {
                        *VGA_BUFFER.offset(offset) = b' ';
                        *VGA_BUFFER.offset(offset + 1) = self.color_code;
                    }
                }
            }
            byte => {
                if self.column_position >= VGA_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let offset = ((row * VGA_WIDTH + col) * 2) as isize;

                unsafe {
                    *VGA_BUFFER.offset(offset) = byte;
                    *VGA_BUFFER.offset(offset + 1) = self.color_code;
                }
                self.column_position += 1;
            }
        }

        self.update_cursor();
   
    }

    pub fn write_string(&mut self, s: &str) {  
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn new_line(&mut self) {
        self.column_position = 0; 
        if self.row_position < VGA_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            for row in 1..VGA_HEIGHT {
                for col in 0..VGA_WIDTH {
                    unsafe {
                        let current_offset = ((row * VGA_WIDTH + col) * 2) as isize;
                        let prev_offset = (((row - 1) * VGA_WIDTH + col) * 2) as isize;

                        let char_byte = *VGA_BUFFER.offset(current_offset);
                        let color_byte = *VGA_BUFFER.offset(current_offset + 1);

                        *VGA_BUFFER.offset(prev_offset) = char_byte;
                        *VGA_BUFFER.offset(prev_offset + 1) = color_byte;
                    }
                }
            }
            // the last row is getting cleared
            self.clear_row(VGA_HEIGHT - 1);
        }
    }

    // helpmethos to delete one single line
    fn clear_row(&mut self, row: usize) {
        for col in 0..VGA_WIDTH {
            let offset = ((row * VGA_WIDTH + col) * 2) as isize;
            unsafe {
                *VGA_BUFFER.offset(offset) = b' ';
                *VGA_BUFFER.offset(offset + 1) = self.color_code;
            }
        }
    }

    // this is for cleaning the screen very essential isnt like it took me 15minutes to work
    pub fn clear_screen(&mut self) {
        unsafe {
            for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
                *VGA_BUFFER.offset((i * 2) as isize) = b' ';
                *VGA_BUFFER.offset((i * 2 + 1) as isize) = self.color_code;
            }
        }
        
        // resets the coursour
        self.column_position = 0;
        self.row_position = 0;

        self.update_cursor();
    }

    fn update_cursor(&self) {
        let pos = self.row_position * VGA_WIDTH + self.column_position;
        
        let mut index_port = Port::new(0x3D4);
        let mut data_port = Port::new(0x3D5);

        unsafe {
            // send low byte of the position
            index_port.write(0x0F_u8);
            data_port.write((pos & 0xFF) as u8);

            // send high byte of the position
            index_port.write(0x0E_u8);
            data_port.write(((pos >> 8) & 0xFF) as u8);
        }
    }
}



impl fmt::Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// the macros that can be used anywhere
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)))
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

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}