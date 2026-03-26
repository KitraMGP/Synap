//! Show command implementation with short ID support.

use colored::Colorize;
use synap_core::{CoreError, SynapService};

/// Execute the show command.
///
/// Displays detailed information about a specific note using short ID prefix.
/// If the prefix is ambiguous, displays all matching notes for selection.
pub fn execute(short_id: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    // Use short ID prefix to find note
    match service.get_note_by_short_id(short_id) {
        Ok(note) => {
            display_note_detail(&note, service)?;
            Ok(())
        }
        Err(CoreError::AmbiguousPrefix(prefix, count)) => {
            display_ambiguous_matches(&prefix, count, service)?;
            Err(format!("前缀 '{}' 匹配 {} 个笔记，请提供更多字符", prefix, count).into())
        }
        Err(e) => Err(e.into()),
    }
}

/// Display detailed information about a note.
fn display_note_detail(
    note: &synap_core::NoteView,
    service: &SynapService,
) -> Result<(), Box<dyn std::error::Error>> {
    let full_id = note.id.to_string();

    println!();
    println!("  ID:      {}", full_id.cyan());
    println!("  创建于:  {}", format_timestamp(note.created_at).dimmed());

    println!();
    println!("  内容:");
    println!("    {}", note.content);

    if !note.tags.is_empty() {
        println!();
        println!("  标签:");
        let tags: Vec<String> = note.tags.iter()
            .map(|t| format!("#{}", t))
            .collect();
        let tags_formatted = tags.join(" ");
        let tags_colored = tags_formatted.cyan();
        println!("    {}", tags_colored);
    }

    // Display replies (if any)
    let replies = service.get_replies(note.id)?;
    if !replies.is_empty() {
        println!();
        println!("  回复 ({}):", replies.len());
        for reply in replies {
            let reply_short = &reply.id.to_string()[..12];
            println!("    [{}] {}", reply_short.cyan(), reply.content);
        }
    }

    // Display parents (if any)
    let parents = service.get_parents(note.id)?;
    if !parents.is_empty() {
        println!();
        println!("  父节点 ({}):", parents.len());
        for parent in parents {
            let parent_short = &parent.id.to_string()[..12];
            println!("    [{}] {}", parent_short.cyan(), parent.content);
        }
    }

    println!();
    Ok(())
}

/// Display all matching notes when prefix is ambiguous.
fn display_ambiguous_matches(
    prefix: &str,
    count: usize,
    service: &SynapService,
) -> Result<(), Box<dyn std::error::Error>> {
    let matches = service.find_notes_by_prefix(prefix)?;

    println!();
    println!("{} 前缀 '{}' 匹配 {} 个笔记:", "[!]".yellow(), prefix.cyan(), count);
    println!("{}", "─".repeat(60).dimmed());

    for (i, note) in matches.iter().enumerate() {
        let short_id = &note.id.to_string()[..12];
        let content_preview: String = if note.content.len() > 50 {
            format!("{}...", note.content.chars().take(47).collect::<String>())
        } else {
            note.content.clone()
        };

        println!("  {} {} | {}",
            format!("[{}]", i + 1).blue(),
            format!("{}...", short_id).cyan(),
            content_preview
        );
    }

    println!();
    println!("{} 请使用更长的前缀来唯一标识笔记", "[i]".dimmed());
    println!();

    Ok(())
}

/// Format a SystemTime for display.
fn format_timestamp(timestamp: std::time::SystemTime) -> String {
    let duration_since_epoch = timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let dt: chrono::DateTime<chrono::Local> = chrono::DateTime::from_timestamp(duration_since_epoch.as_secs() as i64, 0)
        .unwrap()
        .into();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_with_short_id() {
        let service = SynapService::open_memory().unwrap();
        let note = service.add_thought("Test".to_string()).unwrap();
        let short_id = &note.id.to_string()[..12];

        let result = execute(short_id, &service);
        assert!(result.is_ok());
    }
}
