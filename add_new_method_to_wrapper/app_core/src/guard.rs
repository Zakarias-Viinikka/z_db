use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub struct Guard {
    pub target: PathBuf,
    pub original_bytes: Vec<u8>,
    pub original_hash: String,
    pub written_hash: Option<String>,
    pub backup_path: PathBuf,
    pub marker_path: PathBuf,
    pub restored: bool,
    pub restore_problem: Option<String>,
}

#[derive(Debug)]
pub enum GuardError {
    Read(String),
    CacheDir(String),
    WriteBackup(String),
    WriteMarker(String),
    Verify(String),
}

fn backup_path_in(cache_root: &Path) -> PathBuf {
    cache_root.join("macro_core_backup.rs")
}

fn marker_path_in(cache_root: &Path) -> PathBuf {
    cache_root.join("macro_core_backup.marker")
}

fn short_hash(h: &str) -> String {
    h.chars().take(12).collect()
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

impl Guard {
    pub fn arm(
        target: &Path,
        original_bytes: Vec<u8>,
        cache_root: &Path,
    ) -> Result<Self, GuardError> {
        std::fs::create_dir_all(cache_root).map_err(|e| {
            GuardError::CacheDir(format!("couldn't create {}: {}", cache_root.display(), e))
        })?;

        let original_hash = hash_bytes(&original_bytes);
        let backup = backup_path_in(cache_root);
        std::fs::write(&backup, &original_bytes).map_err(|e| {
            GuardError::WriteBackup(format!("couldn't write backup {}: {}", backup.display(), e))
        })?;

        let marker = marker_path_in(cache_root);
        let marker_text = format!(
            "path={}\nhash={}\nbackup={}\n",
            target.display(),
            original_hash,
            backup.display()
        );
        std::fs::write(&marker, marker_text).map_err(|e| {
            GuardError::WriteMarker(format!("couldn't write marker {}: {}", marker.display(), e))
        })?;

        Ok(Guard {
            target: target.to_path_buf(),
            original_bytes,
            original_hash,
            written_hash: None,
            backup_path: backup,
            marker_path: marker,
            restored: false,
            restore_problem: None,
        })
    }

    /// Call after the check process has exited and been joined.
    pub fn finish(&mut self) -> Result<(), GuardError> {
        if self.restored {
            return Ok(());
        }
        self.restore_impl(true)?;
        self.restored = true;
        Ok(())
    }

    fn restore_impl(&mut self, strict: bool) -> Result<(), GuardError> {
        let current = std::fs::read(&self.target).map_err(|e| {
            GuardError::Read(format!("couldn't read {}: {}", self.target.display(), e))
        })?;
        let current_hash = hash_bytes(&current);

        if let Some(written) = &self.written_hash {
            if &current_hash != written {
                let msg = format!(
                    "someone else edited {} while the check ran; leaving it alone. Backup at {}",
                    self.target.display(),
                    self.backup_path.display()
                );
                self.restore_problem = Some(msg.clone());
                if strict {
                    return Err(GuardError::Verify(msg));
                }
                return Ok(());
            }
        }

        std::fs::write(&self.target, &self.original_bytes).map_err(|e| {
            GuardError::WriteBackup(format!("couldn't restore {}: {}", self.target.display(), e))
        })?;

        let after = std::fs::read(&self.target).map_err(|e| {
            GuardError::Read(format!("couldn't re-read {}: {}", self.target.display(), e))
        })?;
        let after_hash = hash_bytes(&after);
        if after_hash != self.original_hash {
            let msg = format!(
                "restore hash mismatch: original {} vs after {} (backup at {})",
                short_hash(&self.original_hash),
                short_hash(&after_hash),
                self.backup_path.display()
            );
            self.restore_problem = Some(msg.clone());
            if strict {
                return Err(GuardError::Verify(msg));
            }
            return Ok(());
        }

        let _ = std::fs::remove_file(&self.marker_path);
        Ok(())
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if !self.restored {
            let _ = self.restore_impl(false);
        }
    }
}

pub struct StartupRecovery {
    pub marker_path: PathBuf,
    pub target: PathBuf,
    pub backup_path: PathBuf,
    pub needs_restore: bool,
}

pub fn check_startup_marker(cache_root: &Path) -> Option<StartupRecovery> {
    let marker = marker_path_in(cache_root);
    let text = std::fs::read_to_string(&marker).ok()?;
    let mut path = None;
    let mut hash = None;
    let mut backup = None;
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("path=") {
            path = Some(PathBuf::from(v));
        } else if let Some(v) = line.strip_prefix("hash=") {
            hash = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("backup=") {
            backup = Some(PathBuf::from(v));
        }
    }
    let (path, hash, backup) = (path?, hash?, backup?);

    let target_bytes = std::fs::read(&path).ok()?;
    let current_hash = hash_bytes(&target_bytes);
    if current_hash == hash {
        let _ = std::fs::remove_file(&marker);
        return None;
    }

    Some(StartupRecovery {
        marker_path: marker,
        target: path,
        backup_path: backup,
        needs_restore: true,
    })
}

pub fn restore_from_backup(recovery: &StartupRecovery) -> Result<(), String> {
    let bytes = std::fs::read(&recovery.backup_path).map_err(|e| {
        format!(
            "couldn't read backup {}: {}",
            recovery.backup_path.display(),
            e
        )
    })?;
    std::fs::write(&recovery.target, &bytes)
        .map_err(|e| format!("couldn't write {}: {}", recovery.target.display(), e))?;
    let _ = std::fs::remove_file(&recovery.marker_path);
    Ok(())
}

pub fn ignore_recovery(recovery: &StartupRecovery) {
    let _ = std::fs::remove_file(&recovery.marker_path);
}