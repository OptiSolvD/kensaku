//! Data directory maintenance: what is there, what can go, and what a reset
//! should keep.
//!
//! Two problems this solves.
//!
//! **Reset was all-or-nothing.** Wiping the data directory also deleted the
//! downloaded OCR and model packs (about 55 MB), so "reset to fix a bad index"
//! cost a re-download before search could work again. A reset now keeps those
//! unless the user explicitly asks otherwise.
//!
//! **Nothing ever cleaned up after an upgrade.** Each content-schema rebuild
//! renames the old index to `index.legacy-<timestamp>` and leaves it there; the
//! updater leaves installers in the temp directory; a download that failed
//! leaves a `.part` file. Over a few releases that is hundreds of megabytes of
//! dead weight in a user's profile, which is exactly what makes an upgrade feel
//! like it left a mess.

use crate::config::KensakuConfig;
use std::path::{Path, PathBuf};

/// Directories that hold downloaded artifacts, not user state.
///
/// A reset keeps these by default: they are reproducible only by downloading
/// them again, and they say nothing about the user's own data.
const PACK_DIRS: &[&str] = &["models", "ocr", "ocr-indra", "tessdata"];

/// What the data directory currently holds, in bytes.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct DataInventory {
    pub config_dir: String,
    pub total_bytes: u64,
    pub index_bytes: u64,
    pub metadata_bytes: u64,
    pub thumbnail_bytes: u64,
    pub pack_bytes: u64,
    /// Content-schema backups left behind by past upgrades.
    pub legacy_backups: usize,
    pub legacy_backup_bytes: u64,
    /// Installers and partial downloads left in the temp directory.
    pub update_temp_bytes: u64,
}

fn dir_size(path: &Path) -> u64 {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return 0,
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            total += dir_size(&entry.path());
        } else if let Ok(meta) = entry.metadata() {
            total += meta.len();
        }
    }
    total
}

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

/// Where the updater stages downloads.
pub fn update_temp_dir() -> PathBuf {
    std::env::temp_dir().join("kensaku-updates")
}

/// Names of the content-index backup directories under the config directory.
fn legacy_backup_dirs(config_dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(config_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("index.legacy-")
                })
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default();
    // Oldest first: the timestamp is part of the name.
    dirs.sort();
    dirs
}

/// Measures everything Kensaku owns on disk.
pub fn inventory() -> DataInventory {
    let config_dir = KensakuConfig::config_dir();
    let legacy = legacy_backup_dirs(&config_dir);
    let legacy_bytes: u64 = legacy.iter().map(|dir| dir_size(dir)).sum();

    let pack_bytes = PACK_DIRS
        .iter()
        .map(|name| dir_size(&config_dir.join(name)))
        .sum();

    DataInventory {
        config_dir: config_dir.display().to_string(),
        total_bytes: dir_size(&config_dir),
        index_bytes: dir_size(&KensakuConfig::index_dir()),
        metadata_bytes: ["metadata.db", "metadata.db-wal", "metadata.db-shm"]
            .iter()
            .map(|name| file_size(&config_dir.join(name)))
            .sum(),
        thumbnail_bytes: dir_size(&config_dir.join("thumbs")),
        pack_bytes,
        legacy_backups: legacy.len(),
        legacy_backup_bytes: legacy_bytes,
        update_temp_bytes: dir_size(&update_temp_dir()),
    }
}

/// Outcome of a cleanup pass, in bytes reclaimed.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct CleanupReport {
    pub legacy_backups_removed: usize,
    pub bytes_reclaimed: u64,
    pub update_temp_cleared: bool,
    pub errors: Vec<String>,
}

/// Removes artifacts that no run needs: superseded index backups and staged
/// installers.
///
/// Safe to call on every start. The newest backup is kept, because it is the
/// one a rollback would want; older ones are pure accumulation.
pub fn cleanup_stale_artifacts() -> CleanupReport {
    let mut report = CleanupReport::default();
    let config_dir = KensakuConfig::config_dir();

    let mut backups = legacy_backup_dirs(&config_dir);
    if backups.len() > 1 {
        // Drop all but the newest.
        backups.pop();
        for dir in backups {
            let bytes = dir_size(&dir);
            match std::fs::remove_dir_all(&dir) {
                Ok(()) => {
                    report.legacy_backups_removed += 1;
                    report.bytes_reclaimed += bytes;
                }
                Err(error) => report.errors.push(format!(
                    "could not remove {}: {}",
                    dir.display(),
                    error
                )),
            }
        }
    }

    let temp = update_temp_dir();
    if temp.is_dir() {
        let before = dir_size(&temp);
        match std::fs::remove_dir_all(&temp) {
            Ok(()) => {
                report.update_temp_cleared = true;
                report.bytes_reclaimed += before;
            }
            Err(error) => report.errors.push(format!(
                "could not clear {}: {}",
                temp.display(),
                error
            )),
        }
    }

    report
}

/// Everything a full uninstall deletes, and how much of it there is.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RemovalReport {
    pub path: String,
    pub bytes: u64,
}

impl RemovalReport {
    /// Human-readable size, e.g. "1.4 GB".
    pub fn human_bytes(&self) -> String {
        crate::doctor::format_bytes(self.bytes)
    }
}

/// Removes everything Kensaku owns, for an uninstall.
///
/// The data directory holds the config, index, database, thumbnails, packs,
/// tokens and logs, so removing it covers every platform. The updater's temp
/// directory sits outside it and is cleared too.
pub fn remove_all_data() -> Result<RemovalReport, String> {
    let config_dir = KensakuConfig::config_dir();
    let bytes = dir_size(&config_dir) + dir_size(&update_temp_dir());

    if config_dir.exists() {
        std::fs::remove_dir_all(&config_dir).map_err(|error| {
            format!(
                "could not remove {}: {}. Quit Kensaku, then run this again.",
                config_dir.display(),
                error
            )
        })?;
    }
    let _ = std::fs::remove_dir_all(update_temp_dir());

    Ok(RemovalReport {
        path: config_dir.display().to_string(),
        bytes,
    })
}

/// What a reset removes.
///
/// `Default` is `remove_packs: false`: a reset clears user state and keeps the
/// downloaded packs, because they are reproducible only by downloading them
/// again and they are not the user's data.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResetScope {
    /// Also delete downloaded OCR and model packs. Off by default: they are
    /// reproducible only by downloading them again, and they are not the
    /// user's data.
    pub remove_packs: bool,
}

/// Clears user state, keeping the data directory itself.
///
/// Runs in the fresh process before any store is opened (see
/// `apply_pending_reset_in`), so no file is held open while it deletes.
pub fn apply_reset(config_dir: &Path, scope: ResetScope) -> Result<(), String> {
    let entries = std::fs::read_dir(config_dir)
        .map_err(|error| format!("cannot read {}: {}", config_dir.display(), error))?;

    let mut failures: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // The marker that brought us here is consumed by the caller.
        if name_str.starts_with(".reset-requested") || name_str.starts_with(".rebuild-requested") {
            continue;
        }
        if !scope.remove_packs && PACK_DIRS.contains(&name_str.as_ref()) {
            continue;
        }

        let path = entry.path();
        let result = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        if let Err(error) = result {
            failures.push(format!("{}: {}", path.display(), error));
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "kensaku-maint-{}-{}-{}",
            tag,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn reset_keeps_downloaded_packs_by_default() {
        let dir = temp_dir("reset-keep");
        std::fs::create_dir_all(dir.join("index")).unwrap();
        std::fs::write(dir.join("index/segment"), b"x").unwrap();
        std::fs::write(dir.join("config.json"), b"{}").unwrap();
        std::fs::create_dir_all(dir.join("models")).unwrap();
        std::fs::write(dir.join("models/embed.onnx"), b"y").unwrap();
        std::fs::create_dir_all(dir.join("ocr-indra")).unwrap();
        std::fs::write(dir.join("ocr-indra/pack.onnx"), b"z").unwrap();

        apply_reset(&dir, ResetScope::default()).unwrap();

        assert!(!dir.join("index").exists(), "index should be cleared");
        assert!(!dir.join("config.json").exists(), "config should be cleared");
        assert!(dir.join("models").exists(), "model packs are not user data");
        assert!(dir.join("ocr-indra").exists(), "OCR packs are not user data");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reset_can_remove_packs_when_asked() {
        let dir = temp_dir("reset-packs");
        std::fs::create_dir_all(dir.join("models")).unwrap();
        std::fs::write(dir.join("models/embed.onnx"), b"y").unwrap();

        apply_reset(
            &dir,
            ResetScope {
                remove_packs: true,
            },
        )
        .unwrap();

        assert!(!dir.join("models").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reset_keeps_the_marker_files_it_is_consuming() {
        let dir = temp_dir("reset-markers");
        std::fs::write(dir.join(".reset-requested"), b"1").unwrap();
        std::fs::write(dir.join("metadata.db"), b"db").unwrap();

        apply_reset(&dir, ResetScope::default()).unwrap();

        // The caller removes the marker after a successful reset; deleting it
        // here would make a failed reset unrecoverable.
        assert!(dir.join(".reset-requested").exists());
        assert!(!dir.join("metadata.db").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_keeps_only_the_newest_legacy_backup() {
        let dir = temp_dir("cleanup-legacy");
        for stamp in ["100", "200", "300"] {
            let backup = dir.join(format!("index.legacy-{stamp}"));
            std::fs::create_dir_all(&backup).unwrap();
            std::fs::write(backup.join("segment"), vec![0u8; 32]).unwrap();
        }

        let backups = legacy_backup_dirs(&dir);
        assert_eq!(backups.len(), 3);

        // cleanup_stale_artifacts works on the real config dir, so exercise the
        // rule directly against this fixture.
        let mut to_remove = legacy_backup_dirs(&dir);
        to_remove.pop(); // keep newest
        assert_eq!(to_remove.len(), 2);
        assert!(to_remove.iter().all(|p| !p.ends_with("index.legacy-300")));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
