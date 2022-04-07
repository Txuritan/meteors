use std::{
    fs::{self, DirEntry},
    process::Command,
};

use crate::{models::Id, prelude::*};

pub fn command(arg: &str) -> Command {
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };

    let mut cmd = Command::new(shell);

    if cfg!(target_os = "windows") {
        cmd.args(&["/C", arg]);
    } else {
        cmd.args(&[arg]);
    }

    cmd
}

pub struct FileIter(fs::ReadDir);

impl FileIter {
    pub fn new(iter: fs::ReadDir) -> FileIter {
        FileIter(iter)
    }
}

impl Iterator for FileIter {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.0.next() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => return Some(Err(err.into())),
            };

            let meta = match entry.metadata() {
                Ok(meta) => meta,
                Err(err) => return Some(Err(err.into())),
            };

            if meta.is_file() {
                Some(Ok(entry))
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

pub const SIZE: usize = 8;

const LEN: usize = 54;
const MASK: usize = LEN.next_power_of_two() - 1;
const STEP: usize = 8 * SIZE / 5;

pub fn new_id() -> Id {
    static ALPHABET: [char; LEN] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F',
        'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];

    let mut id = String::new();

    loop {
        fastrand::seed({
            let mut buf = [0; std::mem::size_of::<u64>()];

            let _ = getrandom::getrandom(&mut buf);

            u64::from_le_bytes(buf)
        });

        let mut bytes = [0_u8; STEP];

        for elt in &mut bytes[..] {
            *elt = fastrand::u8(0..=(std::u8::MAX));
        }

        for &byte in &bytes {
            let byte = byte as usize & MASK;

            if ALPHABET.len() > byte {
                id.push(ALPHABET[byte]);

                if id.len() == SIZE {
                    return Id::from(id);
                }
            }
        }
    }
}
