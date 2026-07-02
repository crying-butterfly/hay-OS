extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

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
                children: Vec::new(),
            },
        }
    }

    // create new file in root directory
    pub fn create_file(&mut self, name: &str, content: &[u8]) {
        if let FsNode::Directory { ref mut children, .. } = self.root {
            children.push(FsNode::File {
                name: format!("{}", name),
                content: content.to_vec(),
            });
        }
    }

    // lists all files in root directory
    pub fn list_root(&self) {
        if let FsNode::Directory { ref children, .. } = self.root {
            for child in children {
                match child {
                    FsNode::File { name, .. } => crate::println!("FILE: {}", name),
                    FsNode::Directory { name, .. } => crate::println!("DIR: {}", name),
                }
            }
        }
    }
}

lazy_static! {
    pub static ref FILE_SYSTEM: Mutex<RamFs> = Mutex::new(RamFs::new());
}