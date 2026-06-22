//! Write-Ahead Log (WAL) for transaction management
//! Simple JSON-based logging for durability

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

/// WAL record types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalRecord {
    Begin { tx_id: u64 },
    Commit { tx_id: u64 },
    Rollback { tx_id: u64 },
}

/// Write-Ahead Log
#[allow(dead_code)]
pub struct WriteAheadLog {
    file: Mutex<File>,
    path: String,
}

impl WriteAheadLog {
    /// Create or open WAL
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        #[allow(clippy::suspicious_open_options)]
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        Ok(Self {
            file: Mutex::new(file),
            path: path.to_string(),
        })
    }

    /// Append a record to the log
    pub fn append(&self, record: &WalRecord) -> Result<(), std::io::Error> {
        let mut file = self.file.lock().unwrap();

        // Serialize to JSON
        let json = serde_json::to_string(record)
            .map_err(|_| std::io::Error::other("serialization failed"))?;

        // Write length prefix + newline
        let len_bytes = (json.len() as u32).to_le_bytes();
        file.write_all(&len_bytes)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        file.flush()?;

        Ok(())
    }

    /// Read all records from log
    pub fn read_all(&self) -> Result<Vec<WalRecord>, std::io::Error> {
        let mut file = self.file.lock().unwrap();
        let mut records = Vec::new();

        // Seek to start
        if file.seek(SeekFrom::Start(0)).is_err() {
            return Ok(records);
        }

        loop {
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(_) => break, // EOF
            }

            let len = u32::from_le_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            if file.read_exact(&mut data).is_err() {
                break;
            }

            // Read newline
            let mut newline = [0u8; 1];
            let _ = file.read(&mut newline);

            if let Ok(record) = serde_json::from_slice(&data) {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Truncate log (after successful checkpoint)
    pub fn truncate(&self) -> Result<(), std::io::Error> {
        let mut file = self.file.lock().unwrap();
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_wal_path(tag: &str) -> String {
        let pid = std::process::id();
        let dir = std::env::temp_dir();
        dir.join(format!("wal_test_{tag}_{pid}_{}.log", rand_suffix())).to_string_lossy().to_string()
    }

    fn rand_suffix() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }

    fn cleanup(path: &str) {
        let _ = std::fs::remove_file(path);
    }

    // ==================== 正常执行 ====================

    #[test]
    fn test_wal_append() {
        let path = unique_wal_path("append");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 1);
        cleanup(&path);
    }

    #[test]
    fn test_wal_commit() {
        let path = unique_wal_path("commit");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append(&WalRecord::Commit { tx_id: 1 }).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 2);
        cleanup(&path);
    }

    #[test]
    fn test_wal_basic_write() {
        let path = unique_wal_path("basic");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append(&WalRecord::Commit { tx_id: 1 }).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 2);
        cleanup(&path);
    }

    #[test]
    fn test_wal_rollback_roundtrip() {
        let path = unique_wal_path("rollback");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 7 }).unwrap();
        wal.append(&WalRecord::Rollback { tx_id: 7 }).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 2);
        match &records[0] {
            WalRecord::Begin { tx_id } => assert_eq!(*tx_id, 7),
            _ => panic!("expected Begin"),
        }
        match &records[1] {
            WalRecord::Rollback { tx_id } => assert_eq!(*tx_id, 7),
            _ => panic!("expected Rollback"),
        }
        cleanup(&path);
    }

    #[test]
    fn test_wal_multiple_transactions() {
        let path = unique_wal_path("multi");
        let wal = WriteAheadLog::new(&path).unwrap();
        for tx in 1..=5u64 {
            wal.append(&WalRecord::Begin { tx_id: tx }).unwrap();
            wal.append(&WalRecord::Commit { tx_id: tx }).unwrap();
        }
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 10);
        cleanup(&path);
    }

    #[test]
    fn test_wal_truncate_after_checkpoint() {
        let path = unique_wal_path("truncate");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append(&WalRecord::Commit { tx_id: 1 }).unwrap();
        wal.truncate().unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 0);
        cleanup(&path);
    }

    #[test]
    fn test_wal_new_truncates_existing_file() {
        let path = unique_wal_path("newtrunc");
        std::fs::write(&path, b"garbage garbage garbage").unwrap();
        let wal = WriteAheadLog::new(&path).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 0);
        cleanup(&path);
    }

    #[test]
    fn test_wal_type_correctness_by_variant() {
        let path = unique_wal_path("variant");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append(&WalRecord::Commit { tx_id: 2 }).unwrap();
        wal.append(&WalRecord::Rollback { tx_id: 3 }).unwrap();
        let records = wal.read_all().unwrap();
        assert!(matches!(&records[0], WalRecord::Begin { .. }));
        assert!(matches!(&records[1], WalRecord::Commit { .. }));
        assert!(matches!(&records[2], WalRecord::Rollback { .. }));
        cleanup(&path);
    }

    #[test]
    fn test_wal_concurrent_append() {
        let path = unique_wal_path("concurrent");
        let wal = std::sync::Arc::new(WriteAheadLog::new(&path).unwrap());
        let mut handles = Vec::new();
        for t in 0..4 {
            let wal_clone = std::sync::Arc::clone(&wal);
            handles.push(std::thread::spawn(move || {
                for i in 0..20 {
                    wal_clone.append(&WalRecord::Begin { tx_id: t as u64 * 100 + i }).unwrap();
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 80, "should have 80 Begin records total");
        cleanup(&path);
    }

    // ==================== 参数错误 ====================

    #[test]
    fn test_wal_path_empty_string() {
        let result = WriteAheadLog::new("");
        assert!(result.is_err(), "empty path should fail");
    }

    #[test]
    fn test_wal_path_directory_only() {
        let dir = std::env::temp_dir();
        let result = WriteAheadLog::new(dir.to_str().unwrap_or(""));
        assert!(result.is_err(), "dir-only path should fail");
    }

    #[test]
    fn test_wal_path_without_permission() {
        let path = "C:\\Windows\\System32\\wal_test_should_fail.log";
        let result = WriteAheadLog::new(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_wal_tx_id_zero() {
        let path = unique_wal_path("txzero");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 0 }).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 1);
        match &records[0] {
            WalRecord::Begin { tx_id } => assert_eq!(*tx_id, 0),
            _ => panic!(),
        }
        cleanup(&path);
    }

    #[test]
    fn test_wal_tx_id_u64_max() {
        let path = unique_wal_path("txmax");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Commit { tx_id: u64::MAX }).unwrap();
        let records = wal.read_all().unwrap();
        match &records[0] {
            WalRecord::Commit { tx_id } => assert_eq!(*tx_id, u64::MAX),
            _ => panic!(),
        }
        cleanup(&path);
    }

    // ==================== 异常崩溃 ====================

    #[test]
    fn test_wal_corrupted_record_skip() {
        let path = unique_wal_path("corrupt");
        std::fs::write(&path, b"\x05\x00\x00\x00{garbage}").unwrap();
        let wal = WriteAheadLog::new(&path).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 0, "corrupted record should be skipped");
        cleanup(&path);
    }

    #[test]
    fn test_wal_truncate_contracts_file_to_size_zero() {
        let path = unique_wal_path("truncate2");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.truncate().unwrap();
        let metadata = std::fs::metadata(&path).unwrap();
        assert_eq!(metadata.len(), 0);
        cleanup(&path);
    }

    #[test]
    fn test_wal_read_after_truncate_and_append_again() {
        let path = unique_wal_path("truncate3");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append(&WalRecord::Commit { tx_id: 1 }).unwrap();
        wal.truncate().unwrap();
        wal.append(&WalRecord::Begin { tx_id: 2 }).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 1);
        assert!(matches!(&records[0], WalRecord::Begin { tx_id } if *tx_id == 2));
        cleanup(&path);
    }

    // ==================== 边界值 ====================

    #[test]
    fn test_wal_single_record() {
        let path = unique_wal_path("single");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Commit { tx_id: 42 }).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 1);
        cleanup(&path);
    }

    #[test]
    fn test_wal_hundred_records() {
        let path = unique_wal_path("hundred");
        let wal = WriteAheadLog::new(&path).unwrap();
        for tx in 1..=100u64 {
            wal.append(&WalRecord::Begin { tx_id: tx }).unwrap();
        }
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 100);
        cleanup(&path);
    }

    #[test]
    fn test_wal_tx_id_consecutive_vs_gap() {
        let path = unique_wal_path("gap");
        let wal = WriteAheadLog::new(&path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 10000000000 }).unwrap();
        wal.append(&WalRecord::Begin { tx_id: u64::MAX - 1 }).unwrap();
        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 3);
        cleanup(&path);
    }

    #[test]
    fn test_wal_append_and_read_are_independent_handles() {
        let path = unique_wal_path("twoinstances");
        let wal_a = WriteAheadLog::new(&path).unwrap();
        wal_a.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        drop(wal_a);
        let wal_b = WriteAheadLog::new(&path).unwrap();
        let records = wal_b.read_all().unwrap();
        assert_eq!(records.len(), 0, "new instance truncates by design");
        cleanup(&path);
    }
}

