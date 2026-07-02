extern crate alloc;
use alloc::string::String;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SetupState {
    SetupUsername,
    SetupPassword,
    Complete,
}

pub struct UserSession {
    pub state: SetupState,
    pub username: String,
    pub password_hash: u64,
}

impl UserSession {
    fn new() -> Self {
        UserSession {
            state: SetupState::SetupUsername,
            username: format!(""),
            password_hash: 0,
        }
    }
    
}

lazy_static! {
    pub static ref SESSION: Mutex<UserSession> = Mutex::new(UserSession::new());
}

pub fn hash_password(password: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for &byte in password {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

// Checks if sector 0 exists on the HDD.
    pub fn initalize_auth() {
        let mut sector = [0u8; 512];
        crate::ata::read_sector(0, &mut sector);

        let mut session = SESSION.lock();

        // proof if HAY1 exists
        if sector[0] == b'H' && sector[1] == b'A' && sector[2] == b'Y' && sector[3] == b'1' {
            let uname_len = sector[4] as usize;
            if let Ok(uname_str) = core::str::from_utf8(&sector[5..5 + uname_len]) {
                session.username = format!("{}", uname_str);
            }

            let mut hash_bytes = [0u8; 8];
            hash_bytes.copy_from_slice(&sector[40..48]);
            session.password_hash = u64::from_le_bytes(hash_bytes);

            session.state = SetupState::Complete;
        } else {
            // no signature so start setup
            session.state = SetupState::SetupUsername;
        }
    }

    // saving the data on Sektor 0 of the HDD
    pub fn save_user_config(username: &str, password_hash: u64) {
        let mut sector = [0u8; 512];

        sector[0] = b'H'; sector[1] = b'A'; sector[2] = b'Y'; sector[3] = b'1';

        let uname_bytes = username.as_bytes();
        sector[4] = uname_bytes.len() as u8;
        for i in 0..uname_bytes.len() {
            sector[5 + i] = uname_bytes[i];
        }

        let hash_bytes = password_hash.to_le_bytes();
        for i in 0..8 {
            sector[40 + i] = hash_bytes[i];
        }

        crate::ata::write_sector(0, &sector);
    }
    