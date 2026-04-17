use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WalEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

pub struct WriteAheadLog {
    path: PathBuf,
    entries: Vec<WalEntry>,
}

impl WriteAheadLog {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut wal = Self {
            path: path.as_ref().to_path_buf(),
            entries: Vec::new(),
        };
        // For simplicity, we just keep it in memory in the simulation,
        // but this interface allows plugging in a real file-backed WAL.
        if !wal.path.exists() {
            File::create(&wal.path)?;
        }
        Ok(wal)
    }

    pub fn append(&mut self, entry: WalEntry) {
        self.entries.push(entry);
    }

    pub fn last_index(&self) -> u64 {
        self.entries.last().map(|e| e.index).unwrap_or(0)
    }

    pub fn last_term(&self) -> u64 {
        self.entries.last().map(|e| e.term).unwrap_or(0)
    }
}
