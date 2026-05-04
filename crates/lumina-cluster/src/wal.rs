use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

pub struct WriteAheadLog {
    path: PathBuf,
    entries: Vec<WalEntry>,
    next_index: u64,
}

impl WriteAheadLog {
    /// Create or open a WAL at the given path. Replays existing entries on load.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut wal = Self {
            path: path.clone(),
            entries: Vec::new(),
            next_index: 1,
        };

        // Replay existing entries from disk
        if path.exists() {
            wal.replay()?;
        } else {
            // Create the file (and parent dirs if needed)
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            File::create(&path)?;
        }

        Ok(wal)
    }

    /// Append an entry to the WAL (both in-memory and on disk)
    pub fn append(&mut self, term: u64, data: Vec<u8>) -> std::io::Result<u64> {
        let index = self.next_index;
        let entry = WalEntry { term, index, data };

        // Write to disk as a JSON line
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let line = serde_json::to_string(&entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writeln!(file, "{}", line)?;
        file.flush()?;

        self.entries.push(entry);
        self.next_index += 1;

        Ok(index)
    }

    /// Replay all entries from the WAL file into memory
    fn replay(&mut self) -> std::io::Result<()> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        self.entries.clear();
        self.next_index = 1;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<WalEntry>(&line) {
                Ok(entry) => {
                    if entry.index >= self.next_index {
                        self.next_index = entry.index + 1;
                    }
                    self.entries.push(entry);
                }
                Err(_) => {
                    // Skip corrupted lines (best-effort recovery)
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Truncate all entries before (and including) the given index.
    /// Rewrites the WAL file with only the retained entries.
    pub fn truncate_before(&mut self, before_index: u64) -> std::io::Result<()> {
        self.entries.retain(|e| e.index > before_index);

        // Rewrite the file
        let mut file = File::create(&self.path)?;
        for entry in &self.entries {
            let line = serde_json::to_string(entry)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            writeln!(file, "{}", line)?;
        }
        file.flush()?;

        Ok(())
    }

    /// Get the last appended index
    pub fn last_index(&self) -> u64 {
        self.entries.last().map(|e| e.index).unwrap_or(0)
    }

    /// Get the last appended term
    pub fn last_term(&self) -> u64 {
        self.entries.last().map(|e| e.term).unwrap_or(0)
    }

    /// Get all entries (for state reconstruction)
    pub fn entries(&self) -> &[WalEntry] {
        &self.entries
    }

    /// Total number of entries in the log
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the WAL is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
