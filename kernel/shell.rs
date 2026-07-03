use crate::{print, println, vga::clear_screen};
use crate::setup::{SESSION, SetupState, hash_password, save_user_config};
use spin::Mutex;
use alloc::string::String;
use alloc::format;

const BUFFER_SIZE: usize = 64;
struct Shell {
    buffer: [u8; BUFFER_SIZE],
    cursor: usize,
    temp_username: String,
}

impl Shell {
    const fn new() -> Self {
        Shell {
            buffer: [0; BUFFER_SIZE],
            cursor: 0,
            temp_username: String::new(),
        }
    }

    

    
    fn add_char(&mut self, c: char) {
        // enter key executes command
        if c == '\n' {
            println!();
            self.handle_enter();
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

            // mask password at setup
            let current_state = SESSION.lock().state;
            if current_state == SetupState::SetupPassword || current_state == SetupState::AuthAdmin {
                print!("*");
            } else {
                print!("{}", c);
            }
        }
    }

    fn handle_enter(&mut self) {
        
        
        let (current_state, input_str) = {
            let session = SESSION.lock();
            let input = match core::str::from_utf8(&self.buffer[..self.cursor]) {
                Ok(s) => s.trim(),
                Err(_) => "",
            };
            (session.state, input)
        };

        match current_state {
            SetupState::SetupUsername => {
                if !input_str.is_empty() {
                    let mut session = SESSION.lock();
                    self.temp_username = format!("{}", input_str);
                    session.state = SetupState::SetupPassword;
                }
            }
            SetupState::SetupPassword => {
            if !input_str.is_empty() {
                let hash = hash_password(input_str.as_bytes());
                save_user_config(self.temp_username.as_str(), hash);

                let mut session = SESSION.lock();
                session.username = format!("{}", self.temp_username);
                session.password_hash = hash;
                session.state = SetupState::Complete;

                clear_screen();
                println!("========================================");
                println!("            Stiggi Setup Complete");
                println!("            Config saved to HDD.");
                println!("========================================\n");
            }
        }

            SetupState::AuthAdmin => {
            if !input_str.is_empty() {
                let input_hash = hash_password(input_str.as_bytes());
                let mut session = SESSION.lock();
                if input_hash == session.password_hash {
                    session.is_admin = true;
                    session.state = SetupState::Complete;
                    println!("\n[SUCCESS] Root privileges granted")
                } else {
                    session.state = SetupState::Complete;
                    println!("\n[FAILED] Incorrect Password");
                }
            }
        }
            
            
            SetupState::Complete => {
                self.interpret_command();
            }
        }
        self.clear_buffer();

        let session = SESSION.lock();
        self.print_prompt_internal(&session.state, &session.username);
    }


    fn clear_buffer(&mut self) {
        self.buffer = [0; BUFFER_SIZE];
        self.cursor = 0;
    }

    fn print_prompt_internal(&self, state: &SetupState, username: &str) {
        match state {
            SetupState::SetupUsername => print!("Enter your username: "),
            SetupState::SetupPassword => print!("Set your root password: "),
            SetupState::AuthAdmin => print!("Enter root password"),
            SetupState::Complete => print!("{}>", username),
        }
    }

    pub fn print_prompt(&self) {
        let session = SESSION.lock();
        self.print_prompt_internal(&session.state, &session.username);
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
       
        let mut session = SESSION.lock();
        match command {
            "help" => {
                println!("Availible Commands:");
                println!("  help      - Show all availible commands");
                println!("  clear     - Clear the screen");
                println!("  echo      - echoes what you said");
                println!("  panic     - Triggers a software crash");
                println!("  neofetch  - Shows you OS data");
                println!("  adm       - Gain root privileges");
                if session.is_admin {
                    println!("\nRoot Commands:");
                    println!("  exit_adm  - Drop root privileges");
                    println!("  reset     - reset Sector 0 (Deletes User config)");
                }
            }
            "adm" => {
                if session.is_admin {
                    println!("you are already root");
                } else {
                    session.state = SetupState::AuthAdmin;
                }
            }
            "exit.adm" => {
                if session.is_admin {
                    session.is_admin = false;
                    println!("Dropped Root Privileges");
                } else {
                    println!("you are not root");
                }
            }
            "reset" => {
                if session.is_admin {
                    let empty_sector = [0u8; 512];
                    crate::ata::write_sector(0, &empty_sector);
                    println!("Resettet succesfully Restart to trigger setup again")
                } else {
                    println!("You need to have root privileges to do this");
                }
            }
            "clear" => {
                crate::vga::clear_screen();
            }
            "neofetch" => {
                println!("  /\\   /\\   OS: Hay OS early phase");
                println!(" /  \\_/  \\  Kernel: Alpha0.1.2 'Age of filesystem'");
                println!(" |         |  Shell: Hay Shell 0.0.1-Root of the User");
                println!(" |   _ _   |  Arch: x86_64");
                println!(" \\_/   \\_/  Setup: Stiggi V0.1");
                println!("              Build: Root Commands - Stable");
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
    clear_screen();
    
    crate::setup::initalize_auth();
    
    println!("Weclome to hay os type help for a list of commands");

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