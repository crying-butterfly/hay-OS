use crate::{print, println};
use spin::Mutex;

const BUFFER_SIZE: usize = 64;

struct Shell {
    buffer: [u8; BUFFER_SIZE],
    cursor: usize,
}

impl Shell {
    const fn new() -> Self {
        Shell {
            buffer: [0; BUFFER_SIZE],
            cursor: 0,
        }
    }

    fn add_char(&mut self, c: char) {
        // enter key executes command
        if c == '\n' {
            println!();
            self.interpret_command();
            self.clear_buffer();
            self.print_prompt();
            return;
        }

        // backspace key: delete last letter
        if c == '\x08' {
            if self.cursor > 0 {
                self.cursor -= 1;
                self.buffer[self.cursor] = 0;
                // backspace only moves the cursor back so we must overwrite the letter with space
                print!("{}", c);
            }
            return;
        }

        // add normal letter to buffer if theres space left
        if self.cursor < BUFFER_SIZE - 1 && c.is_ascii() {
            self.buffer[self.cursor] = c as u8;
            self.cursor += 1;

            print!("{}", c);
        }
    }

    fn clear_buffer(&mut self) {
        self.buffer = [0; BUFFER_SIZE];
        self.cursor = 0;
    }

    fn print_prompt(&self) {
        print!("HayOS> ");
    }

    fn interpret_command(&mut self) {
        // transform the buffer into a readable String-Slice
        let command = match core::str::from_utf8(&self.buffer[..self.cursor]) {
            Ok(s) => s.trim(),
            Err(_) => return,
        };

        if command.is_empty() {
            return;
        }

       // we are poofing if the command starts with echo. we are writing it out of match command so we can use args
       if command.starts_with("echo ") {
        // seperates the string at the first space in "echo" "text"
        if let Some((_, args)) = command.split_once(' ') {
            println!("{}", args.trim());
        }
        return;
    }
       
        match command {
            "help" => {
                println!("Availible Commands:");
                println!("  help - Show all availible commands");
                println!("  clear - Clear the screen");
                println!("  echo - echoes what you said");
                println!("  panic - Triggers a software crash");
                println!("  neofetch - Shows you OS data");
            }
            "clear" => {
                crate::vga::clear_screen();
            }
            "neofetch" => {
                println!("  /\\   /\\   OS: Hay OS early phase");
                println!(" /  \\_/  \\  Kernel: Alpha0.1.2 '3 Rings of doom'");
                println!(" |         |  Shell: Hay Shell 0.0.1-Tired Shell");
                println!(" |   _ _   |  Arch: x86_64");
                println!(" \\_/   \\_/  Package Manager: Stiggi V0.0");
                println!("            Build: Ring 3 - Stable");
                println!();
}
            "panic" => {
                panic!("You executed panic in you shell shutdown the os and restart");
            }

            _ => {
                println!("Unknown command: '{}' type 'help' for list", command);
            }
        }
    }
}

struct KeyBuffer {
    data: [char; 64],
    head: usize,
    tail: usize,
}

static KEY_BUFFER: Mutex<KeyBuffer> = Mutex::new(KeyBuffer{
    data: ['\0'; 64],
    head: 0,
    tail: 0,
});

// global thread protected state of shell
static SHELL: Mutex<Shell> = Mutex::new(Shell::new());

// this function gets called by keyboard interrupt
pub fn handle_input(c: char) {
    let mut queue = KEY_BUFFER.lock();
    let head = queue.head;
    let next = (head + 1) % 64;
    if next != queue.tail {
        queue.data[head] = c;
        queue.head = next;
    }
}

// initalizes the shell and shows the first prompt
pub fn init() {
    SHELL.lock().print_prompt();
}

// help function that holds the mutex only for microsecounds
fn pop_key_buffer() -> Option<char> {
    let mut queue = KEY_BUFFER.lock();
    if queue.tail != queue.head {
        let c = queue.data[queue.tail];
        queue.tail = (queue.tail + 1) % 64;
        Some(c)
    } else {
        None
    }
}

// this function will be an task entry point
pub fn shell_task_main() -> ! {
    println!("Welcome To hay os press help for a list of commands");
    SHELL.lock().print_prompt();

    loop {
        // if theres a letter we process It In the Shell 
        if let Some(c) = pop_key_buffer() {
            SHELL.lock().add_char(c);
        } else {
            // if theres no letter we briefly yield the CPU
            core::hint::spin_loop();
        }
    }
}