//! Configuration and path management for Synap CLI.
//!
//! This module handles XDG-compliant path resolution for database storage
//! across different platforms.

use std::path::PathBuf;
use std::{env, fs};

/// Get the default database path following XDG Base Directory specification.
///
/// # Platform-specific locations
/// - **Linux**: `~/.local/share/synap/synap.db`
/// - **Windows**: `%APPDATA%\synap\synap.db`
/// - **macOS**: `~/Library/Application Support/synap/synap.db`
pub fn get_db_path() -> PathBuf {
    // Check environment variable first
    if let Ok(path) = env::var("SYNAP_DB_PATH") {
        return PathBuf::from(path);
    }

    // Use XDG directories for cross-platform support
    let proj_dirs = directories::ProjectDirs::from("com", "synap", "synap")
        .expect("Failed to determine project directories");

    proj_dirs.data_dir().join("synap.db")
}

/// Ensure the parent directory exists for a specific database path.
///
/// # Errors
/// Returns an error if directory creation fails.
pub fn ensure_db_dir_exists_for(db_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// Resolve the database path, respecting command-line override.
///
/// If an explicit path is provided via command line, use that.
/// Otherwise, use the default XDG-compliant path (respecting SYNAP_DB_PATH env var).
///
/// # Arguments
/// * `override_path` - Optional path from command-line argument
pub fn resolve_db_path(override_path: Option<String>) -> PathBuf {
    if let Some(path) = override_path {
        PathBuf::from(path)
    } else {
        get_db_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_db_path_without_override() {
        let path = resolve_db_path(None);
        assert!(path.ends_with("synap.db"));
    }

    #[test]
    fn test_resolve_db_path_with_override() {
        let path = resolve_db_path(Some("/tmp/test.db".to_string()));
        assert_eq!(path, PathBuf::from("/tmp/test.db"));
    }
}
