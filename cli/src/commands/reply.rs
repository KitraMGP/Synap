//! Reply command implementation - thought extension.

use colored::Colorize;
use synap_core::SynapService;

/// Thought extension: Derive new ideas based on a specified block.
///
/// # Example
/// ```bash
/// synap @A3F "But in this case, the traversal depth during garbage collection might become a performance bottleneck, need to limit with BFS"
/// ```
pub fn execute(
    target_short_id: &str,
    content: &str,
    service: &SynapService,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse short ID
    let parent = service.get_note_by_short_id(target_short_id)?;

    // Create reply
    let child = service.reply_thought(parent.id, content.to_string())?;

    // Auto-extract and add tags
    let tags = extract_tags(content);
    for tag in tags {
        let _ = service.add_tag(child.id, tag);
    }

    let parent_short = &parent.id.to_string()[..12];
    let child_short = &child.id.to_string()[..12];

    println!(
        "{} 思维延伸: ({}) ──[回复]─> ({})",
        "[+]".green(),
        format!("{}...", child_short).cyan(),
        format!("{}...", parent_short).dimmed()
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
}
