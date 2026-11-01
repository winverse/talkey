//! API keys stored in a file only the user can read.
//!
//! macOS ties keychain access to the app's code signature. A build signed
//! without an Apple Team ID (self-signed or ad hoc) is identified by its code
//! hash, which changes on every rebuild, so the keychain asked for permission
//! again after each build. On macOS Talkey therefore keeps API keys in
//! `credentials.json` in the app data folder, with permission 600.
//!
//! The file maps a credential account (`llm.claude.api_key`) to the stored
//! payload. `null` marks an account as deleted or known to be absent, so a
//! stale keychain copy is never imported again.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = "credentials.json";
/// Bundle identifier from tauri.conf.json; names the app data folder.
pub const APP_IDENTIFIER: &str = "com.winverse.talkey";
const FILE_VERSION: u8 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
struct CredentialFile {
    version: u8,
    accounts: BTreeMap<String, Option<String>>,
}

/// What the file says about one account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// The file has never seen this account.
    Unknown,
    /// The account was deleted or is known to have no secret.
    Absent,
    Present(String),
}

fn load(path: &Path) -> Result<CredentialFile> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .with_context(|| format!("parse credential file {}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(CredentialFile {
            version: FILE_VERSION,
            accounts: BTreeMap::new(),
        }),
        Err(error) => {
            Err(error).with_context(|| format!("read credential file {}", path.display()))
        }
    }
}

/// Writes the file atomically: a temporary file with permission 600 in the
/// same folder, then a rename over the old file.
fn save(path: &Path, file: &CredentialFile) -> Result<()> {
    use std::io::Write;

    let dir = path
        .parent()
        .context("credential file path has no parent folder")?;
    std::fs::create_dir_all(dir)
        .with_context(|| format!("create credential folder {}", dir.display()))?;
    let text = serde_json::to_string_pretty(file).context("serialize credential file")?;
    let temp = dir.join(format!(".{FILE_NAME}.tmp"));

    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut handle = options
        .open(&temp)
        .with_context(|| format!("open {}", temp.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // `mode` only applies when the file is created; fix a leftover file too.
        handle.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    handle.write_all(text.as_bytes())?;
    handle.sync_all()?;
    drop(handle);
    std::fs::rename(&temp, path).with_context(|| format!("replace {}", path.display()))
}

/// Serializes read-modify-write cycles within the process.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|error| error.into_inner())
}

pub fn read(path: &Path, account: &str) -> Result<Entry> {
    let _guard = lock();
    Ok(match load(path)?.accounts.get(account) {
        None => Entry::Unknown,
        Some(None) => Entry::Absent,
        Some(Some(secret)) => Entry::Present(secret.clone()),
    })
}

/// Stores `secret` for `account`, or marks it absent when `secret` is `None`.
pub fn write(path: &Path, account: &str, secret: Option<&str>) -> Result<()> {
    let _guard = lock();
    let mut file = load(path)?;
    file.version = FILE_VERSION;
    file.accounts
        .insert(account.to_string(), secret.map(str::to_string));
    save(path, &file)
}

static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Sets the app data folder that holds `credentials.json`. Called once at
/// startup, before anything reads a credential. Until it is called (in tests,
/// and on Windows and Linux) credentials stay in the system vault.
pub fn set_dir(dir: PathBuf) {
    let _ = DIR.set(dir);
}

/// Path of `credentials.json`, or `None` before `set_dir` (e.g. in tests).
pub fn path() -> Option<PathBuf> {
    DIR.get().map(|dir| dir.join(FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file() -> PathBuf {
        std::env::temp_dir()
            .join(format!(
                "talkey-credentials-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
            .join(FILE_NAME)
    }

    #[test]
    fn app_identifier_matches_tauri_config() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], APP_IDENTIFIER);
    }

    #[test]
    fn missing_file_reads_as_unknown() {
        let path = temp_file();
        assert_eq!(read(&path, "llm.claude.api_key").unwrap(), Entry::Unknown);
    }

    #[test]
    fn stores_marks_absent_and_overwrites() {
        let path = temp_file();
        write(&path, "llm.claude.api_key", Some("secret-1")).unwrap();
        write(&path, "session.cloud.api_key", None).unwrap();
        assert_eq!(
            read(&path, "llm.claude.api_key").unwrap(),
            Entry::Present("secret-1".to_string())
        );
        assert_eq!(read(&path, "session.cloud.api_key").unwrap(), Entry::Absent);

        write(&path, "llm.claude.api_key", None).unwrap();
        assert_eq!(read(&path, "llm.claude.api_key").unwrap(), Entry::Absent);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn file_is_readable_only_by_the_owner() {
        use std::os::unix::fs::PermissionsExt;
        let path = temp_file();
        write(&path, "llm.claude.api_key", Some("secret")).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
