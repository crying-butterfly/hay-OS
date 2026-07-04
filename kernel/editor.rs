use alloc::string::String;
use spin::Mutex;
use crate::{print, println, clear_screen};
use crate::fs::FILE_SYSTEM;
use crate::ata::write_sector;

#[derive(PartialEq)]
pub enum EditorState {
    Inactive,
    PromptFolder,
    PromptFilename,
    Editing,
}

pub struct Editor {
    pub state: EditorState,
    folder: String,
    filename: String,
    buffer: [u8; 512],
    cursor: usize,
    lba_sector: u32, // where it gets safed at the HDD
}

pub static EDITOR: Mutex<Editor> = Mutex::new(Editor {
    state: EditorState::Inactive,
    folder: String::new(),
    filename: String::new(),
    buffer: [0; 512],
    cursor: 0,
    lba_sector: 10,
});

impl Editor {
    pub fn start(&mut self) {
        self.state = EditorState::PromptFolder;
        self.folder.clear();
        self.filename.clear();
        self.buffer = [0; 512];
        self.cursor = 0;
        clear_screen();
        print!("Documents or Home choose saving location: ");
    }

    pub fn handle_input(&mut self, c: char) {
        match self.state {
            EditorState::PromptFolder => self.handle_folder_prompt(c),
            EditorState::PromptFilename => self.handle_filename_prompt(c),
            EditorState::Editing => self.handle_editing(c),
            EditorState::Inactive => {}
        }
    }

    fn handle_folder_prompt(&mut self, c: char) {
        if c == '\n' {
            if self.folder == "Documents" || self.folder == "Home" {
                self.state = EditorState::PromptFilename;
                println!();
                print!("enter filename: ");
            } else {
                println!("\nUnknown Folder Only Documents or Home are folders");
                self.folder.clear();
                print!("Documents or Home choose Saving location");

            }
        } else if c == '\x08' && !self.folder.is_empty() {
            self.folder.pop();
            print!("\x08 \x08");
        } else if c.is_ascii_alphanumeric() {
            self.folder.push(c);
            print!("{}", c);
        }

    }
    

    fn handle_filename_prompt(&mut self, c: char) {
        if c == '\n' {
            self.state = EditorState::Editing;
            self.draw_ui();
        } else if c == '\x08' && !self.filename.is_empty() {
            self.filename.pop();
            print!("\x08 \x08");
        } else if c.is_ascii_alphanumeric() || c == '.' {
            self.filename.push(c);
            print!("{}", c);
        }
    }

    fn handle_editing(&mut self, c: char) {
        if c == '\x11' {
            self.state = EditorState::Inactive;
            clear_screen();
            println!("Closed Editor!");
            crate::shell::init();
            return;
        }

        if c == '\x13' {
            self.save_file();
            return;
        }

        if c == '\x08' {
            if self.cursor > 0 {
                self.cursor -= 1;
                self.buffer[self.cursor] = 0;
                print!("\x08 \x08");
            }
        } else if self.cursor < 512 {
            self.buffer[self.cursor] = c as u8;
            self.cursor += 1;
            print!("{}", c);
        }
     }

     fn draw_ui(&self) {
        clear_screen();
        println!("================================================================================");
        println!("  Hay Text editor | File: {}/{} | STRG+S: Save | STRG+Q: exit", self.folder, self.filename);
        println!("================================================================================");
     }
     
     fn save_file(&mut self) {
        // write to the hard drive
        write_sector(self.lba_sector, &self.buffer);

        let mut fs = FILE_SYSTEM.lock();
        let content = &self.buffer[..self.cursor];
        fs.create_file_in_dir(&self.folder, &self.filename, content);

        self.lba_sector += 1;
        println!("\n[Saved on HDD and RamFs]")
     }

}



