//! Amend command implementation - immutable correction with editor integration.

use std::{
    env,
    io::{Read, Seek, SeekFrom, Write},
    process::Command,
};
use colored::Colorize;
use tempfile::NamedTempFile;
use synap_core::SynapService;

/// Evolution: Launch editor, generate new block and replace old block.
///
/// # Example
/// ```bash
/// synap amend B9X
/// # (Instantly launches clean nvim, save and exit after editing)
/// [~] Deflection occurred: New block (01HTX...C12) ──[replace]─> (B9X)
/// ```
pub fn execute(short_id: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    // Parse short ID
    let old_note = service.get_note_by_short_id(short_id)?;

    // Create temporary file
    let mut temp_file = NamedTempFile::new()?;
    write!(temp_file, "{}", old_note.content)?;

    // Get editor
    let editor = detect_editor();

    // Open editor
    println!("{} 正在打开 {}...", "[i]".blue(), editor);
    let status = Command::new(&editor)
        .arg(temp_file.path())
        .status()?;

    if !status.success() {
        return Err(format!("{} 编辑器退出异常", "[ERROR]".red()).into());
    }

    // Read edited content
    temp_file.seek(SeekFrom::Start(0))?;
    let mut new_content = String::new();
    temp_file.read_to_string(&mut new_content)?;

    if new_content.trim().is_empty() {
        return Err(format!("{} 内容不能为空", "[ERROR]".red()).into());
    }

    if new_content == old_note.content {
        println!("{} 内容未变更，跳过修正", "[~]".yellow());
        return Ok(());
    }

    // Execute immutable correction
    let new_note = service.edit_thought(old_note.id, new_content)?;

    let old_short = &old_note.id.to_string()[..12];
    let new_short = &new_note.id.to_string()[..12];

    println!(
        "{} 发生偏转: 新区块 ({}) ──[替代]─> ({})",
        "[~]".yellow(),
        format!("{}...", new_short).cyan(),
        format!("{}...", old_short).dimmed()
    );

    Ok(())
}

/// Detect available editor.
fn detect_editor() -> String {
    if let Ok(editor) = env::var("EDITOR") {
        return editor;
    }

    // Detect common editors
    let editors = ["nvim", "vim", "vi", "nano", "emacs"];
    for editor in &editors {
        if Command::new(editor).arg("--version").output().is_ok() {
            return editor.to_string();
        }
    }

    // Default to vi
    "vi".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_editor_with_env() {
        let original = env::var("EDITOR");
        env::set_var("EDITOR", "test-editor");
        let editor = detect_editor();
        assert_eq!(editor, "test-editor");

        // Restore original
        match original {
            Ok(v) => env::set_var("EDITOR", v),
            Err(_) => env::remove_var("EDITOR"),
        }
    }

    #[test]
    fn test_detect_editor_fallback() {
        // Ensure no EDITOR env var
        env::remove_var("EDITOR");
        let editor = detect_editor();
        // Should fallback to vi (or something else if available)
        // We just check it's not empty
        assert!(!editor.is_empty());
    }
}
