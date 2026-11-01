//! One-time import of data written by OpenTypeless, the app Talkey is based on.
//!
//! Talkey uses its own bundle identifier, so Tauri resolves a new data
//! directory. On first launch the files from the OpenTypeless directory are
//! copied over (never moved), so the original app keeps working.

use std::fs;
use std::io;
use std::path::Path;

pub const DB_FILE_NAME: &str = "talkey.db";
const LEGACY_IDENTIFIER: &str = "com.opentypeless.app";
const LEGACY_DB_FILE_NAME: &str = "opentypeless.db";
const MARKER_FILE_NAME: &str = ".imported-from-opentypeless";

/// Imports the OpenTypeless data directory that sits next to `data_dir`.
/// Returns the names of the copied files, or `None` when nothing was done.
pub fn import_legacy_data_dir(data_dir: &Path) -> io::Result<Option<Vec<String>>> {
    let Some(parent) = data_dir.parent() else {
        return Ok(None);
    };
    import_from(&parent.join(LEGACY_IDENTIFIER), data_dir)
}

fn import_from(legacy_dir: &Path, data_dir: &Path) -> io::Result<Option<Vec<String>>> {
    if legacy_dir == data_dir || !legacy_dir.is_dir() {
        return Ok(None);
    }
    let already_set_up = ["settings.json", DB_FILE_NAME, MARKER_FILE_NAME]
        .iter()
        .any(|name| data_dir.join(name).exists());
    if already_set_up {
        return Ok(None);
    }

    fs::create_dir_all(data_dir)?;
    let mut copied = Vec::new();
    for entry in fs::read_dir(legacy_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains(".bak") {
            continue;
        }
        // opentypeless.db, opentypeless.db-wal, opentypeless.db-shm -> talkey.db*
        let target_name = match name.strip_prefix(LEGACY_DB_FILE_NAME) {
            Some(suffix) => format!("{DB_FILE_NAME}{suffix}"),
            None => name,
        };
        let target = data_dir.join(&target_name);
        if target.exists() {
            continue;
        }
        fs::copy(entry.path(), &target)?;
        copied.push(target_name);
    }
    copied.sort();
    fs::write(
        data_dir.join(MARKER_FILE_NAME),
        legacy_dir.display().to_string(),
    )?;
    Ok(Some(copied))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "talkey-legacy-{name}-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn copies_legacy_files_and_renames_the_database() {
        let root = temp_root("copy");
        let legacy = root.join(LEGACY_IDENTIFIER);
        let data = root.join("com.winverse.talkey");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("settings.json"), "{}").unwrap();
        fs::write(legacy.join("opentypeless.db"), "db").unwrap();
        fs::write(legacy.join("opentypeless.db-wal"), "wal").unwrap();
        fs::write(legacy.join("settings.json.bak-before-x"), "old").unwrap();
        fs::create_dir_all(legacy.join("nested")).unwrap();

        let copied = import_legacy_data_dir(&data).unwrap().unwrap();

        assert_eq!(copied, vec!["settings.json", "talkey.db", "talkey.db-wal"]);
        assert_eq!(fs::read_to_string(data.join("talkey.db")).unwrap(), "db");
        assert!(
            legacy.join("opentypeless.db").exists(),
            "legacy data is kept"
        );
        assert!(!data.join("settings.json.bak-before-x").exists());
        // Second launch: nothing to do.
        assert_eq!(import_legacy_data_dir(&data).unwrap(), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn skips_when_talkey_already_has_data() {
        let root = temp_root("skip");
        let legacy = root.join(LEGACY_IDENTIFIER);
        let data = root.join("com.winverse.talkey");
        fs::create_dir_all(&legacy).unwrap();
        fs::create_dir_all(&data).unwrap();
        fs::write(legacy.join("settings.json"), "legacy").unwrap();
        fs::write(data.join("settings.json"), "talkey").unwrap();

        assert_eq!(import_legacy_data_dir(&data).unwrap(), None);
        assert_eq!(
            fs::read_to_string(data.join("settings.json")).unwrap(),
            "talkey"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn does_nothing_without_legacy_dir() {
        let root = temp_root("none");
        let data = root.join("com.winverse.talkey");
        assert_eq!(import_legacy_data_dir(&data).unwrap(), None);
        assert!(!data.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
