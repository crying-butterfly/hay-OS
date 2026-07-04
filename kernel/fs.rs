extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::println;

pub enum FsNode {
    File {
        name: String,
        content: Vec<u8>,
    },
    Directory {
        name: String,
        children: Vec<FsNode>,
    },
}

pub struct RamFs {
    root: FsNode,
}

impl RamFs {
    pub fn new() -> Self {
        RamFs {
            root: FsNode::Directory {
                name: format!("/"),
                children: alloc::vec![
                    FsNode::Directory {
                        name: format!("Documents"),
                        children: Vec::new(),
                    },
                    FsNode::Directory {
                        name: format!("Home"),
                        children: Vec::new(),
                    }
                ],
            },
        }
    }

    // create a new file in a specific folder
    pub fn create_file_in_dir(&mut self, dir_name: &str, file_name: &str, content: &[u8]) -> bool {
        if let FsNode::Directory { ref mut children, .. } = self.root {
            for child in children.iter_mut() {
                if let FsNode::Directory { name, ref mut children } = child {
                    if name == dir_name {
                        children.push(FsNode::File {
                            name: format!("{}", file_name),
                            content: content.to_vec(),
                        });
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn read_file(&self, dir_name: &str, file_name: &str) {
        if let FsNode::Directory { ref children, .. } = self.root {
            for child in children {
                if let FsNode::Directory { name, children: dir_children } = child {
                    if name == dir_name {
                        for c in dir_children {
                            if let FsNode::File { name: f_name, content } = c {
                                if f_name == file_name {
                                    // Wandelt die Bytes in einen String um, falls lesbar
                                    if let Ok(text) = core::str::from_utf8(content.as_slice()) {
                                        println!("{}", text);
                                    } else {
                                        println!("[Binary data: {} Bytes]", content.len());
                                    }
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("couldnt find file '{}' in {}", file_name, dir_name);
    }

    

    // lists all files in root directory
    pub fn list_dir(&self, dir_name: &str) {
        if let FsNode::Directory { ref children, .. } = self.root {
            for child in children {
                if let FsNode::Directory { name, children: dir_children } = child {
                    if name == dir_name {
                        for c in dir_children {
                            if let FsNode::File { name: f_name, .. } = c {
                                println!("File: {}", f_name);
                            }
                        }
                        return;
                    }
                }
            }
        }
        println!("couldnt find folder {}", dir_name);
    }
}

lazy_static! {
    pub static ref FILE_SYSTEM: Mutex<RamFs> = Mutex::new(RamFs::new());
}