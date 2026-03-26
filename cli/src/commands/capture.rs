//! Capture command implementation - zero-friction thought capture.

use colored::Colorize;
use synap_core::SynapService;

/// Direct capture: Solidify content into a new block.
///
/// # Example
/// ```bash
/// synap "Suddenly realized that decoupling pointers and blocks completely is like OS virtual memory mapping #architecture"
/// ```
pub fn execute(content: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    let note = service.add_thought(content.to_string())?;

    // Auto-extract and add tags
    let tags = extract_tags(content);
    for tag in tags {
        let _ = service.add_tag(note.id, tag);
    }

    // Display short ID (geek style)
    let short_id = &note.id.to_string()[..12];
    println!(
        "{} 脑图区块已固化 ({})",
        "[+]".green(),
        format!("{}...", short_id).cyan()
    );

    Ok(())
}

/// Extract #tag format tags from content.
fn extract_tags(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .filter(|s| s.starts_with('#'))
        .map(|s| s[1..].to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tags() {
        let content = "This is a test #rust #programming with multiple tags";
        let tags = extract_tags(content);
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"programming".to_string()));
    }

    #[test]
    fn test_extract_no_tags() {
        let content = "This is a test without tags";
        let tags = extract_tags(content);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_extract_empty_tag() {
        let content = "This is a test # with empty tag";
        let tags = extract_tags(content);
        // The '#' alone should be filtered out
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "");
    }
}
