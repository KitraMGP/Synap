//! Trace command implementation - graph topology rendering.

use colored::Colorize;
use synap_core::SynapService;

/// Execute trace command - display recent thoughts or graph topology.
pub fn execute(
    short_id_or_flag: Option<&str>,
    service: &SynapService,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check for --stream flag
    if short_id_or_flag == Some("--stream") {
        return execute_stream(service);
    }

    // Original trace behavior
    if let Some(id) = short_id_or_flag {
        // Show graph for specific note
        let note = service.get_note_by_short_id(id)?;
        let graph = service.get_graph(note.id, 0)?;

        if graph.is_empty() {
            println!("\n{} 该思想孤立无援", "[i]".blue());
            return Ok(());
        }

        // Render ASCII art
        render_graph_ascii(graph)?;
    } else {
        // No ID: display recent thoughts
        execute_recent(service)?;
    }

    Ok(())
}

/// Execute recent mode - display recent active thoughts.
pub fn execute_recent(service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    let recent = service.iter_thoughts()?;
    let mut count = 0;

    println!("\n{} 最近的活跃思想", "[*]".blue());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    for thought_result in recent.take(20) {
        let thought = thought_result?;
        let short_id = &thought.id.to_string()[..12];

        let content_preview: String = if thought.content.len() > 60 {
            format!("{}...", thought.content.chars().take(57).collect::<String>())
        } else {
            thought.content.clone()
        };

        let timestamp = format_timestamp(thought.updated_at);

        println!("{} | {} | {}",
            format!("[{}...]", short_id).cyan(),
            timestamp.dimmed(),
            content_preview
        );

        if !thought.tags.is_empty() {
            let tags: Vec<String> = thought.tags.iter()
                .map(|t| format!("#{}", t))
                .collect();
            println!("    {}", tags.join(" ").cyan());
        }

        println!();
        count += 1;
    }

    if count == 0 {
        println!("\n{} 账本尚无思想", "[i]".blue());
    } else {
        println!("{} 共 {} 条思想 (最近)", "[i]".blue(), count.to_string().cyan());
        println!("{} 使用 'synap trace --stream' 查看所有思想", "[i]".dimmed());
    }
    println!();

    Ok(())
}

/// Execute stream mode - iterate over thoughts in reverse chronological order.
pub fn execute_stream(service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{} 按时间倒序流式显示思想 (Ctrl+C 退出)", "[*]".blue());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    let mut count = 0;
    let iter = service.iter_thoughts()?;

    for thought_result in iter {
        let thought = thought_result?;

        // Format short ID
        let short_id = &thought.id.to_string()[..12];

        // Generate content preview
        let content_preview: String = if thought.content.len() > 60 {
            format!("{}...", thought.content.chars().take(57).collect::<String>())
        } else {
            thought.content.clone()
        };

        // Format timestamp
        let timestamp = format_timestamp(thought.updated_at);

        // Display thought
        println!("{} | {} | {}",
            format!("[{}...]", short_id).cyan(),
            timestamp.dimmed(),
            content_preview
        );

        // Display tags
        if !thought.tags.is_empty() {
            let tags: Vec<String> = thought.tags
                .iter()
                .map(|t| format!("#{}", t))
                .collect();
            let tags_formatted = tags.join(" ");
            println!("    {}", tags_formatted.cyan());
        }

        println!();
        count += 1;
    }

    if count == 0 {
        println!("\n{} 账本尚无思想", "[i]".blue());
    } else {
        println!("{} 共 {} 条思想", "[i]".blue(), count.to_string().cyan());
    }
    println!();

    Ok(())
}

/// Format a timestamp for display.
fn format_timestamp(timestamp: std::time::SystemTime) -> String {
    let duration_since_epoch = timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let dt: chrono::DateTime<chrono::Local> = chrono::DateTime::from_timestamp(duration_since_epoch.as_secs() as i64, 0)
        .unwrap()
        .into();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Render graph as ASCII tree structure.
pub fn render_graph_ascii(graph: Vec<(synap_core::NoteView, usize)>) -> Result<(), Box<dyn std::error::Error>> {
    println!();

    for (note, depth) in &graph {
        let indent = "│  ".repeat(*depth);

        // Generate content preview
        let content_preview: String = if note.content.len() > 60 {
            format!("{}...", note.content.chars().take(57).collect::<String>())
        } else {
            note.content.clone()
        };

        // Format short ID
        let short_id = &note.id.to_string()[..8];

        // Render node
        println!(
            "{}[{}] {}",
            indent.dimmed(),
            format!("{}...", short_id).cyan(),
            content_preview
        );

        // Display tags
        if !note.tags.is_empty() {
            let tags: Vec<String> = note.tags
                .iter()
                .map(|t| format!("#{}", t))
                .collect();
            let tags_formatted = tags.join(" ");
            let tags_colored = tags_formatted.cyan();
            println!("{}    {}", indent.dimmed(), tags_colored);
        }
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_with_no_notes() {
        let service = SynapService::open_memory().unwrap();
        let result = execute(None, &service);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_notes() {
        let service = SynapService::open_memory().unwrap();
        let note = service.add_thought("Test".to_string()).unwrap();
        let short_id = &note.id.to_string()[..8];

        let result = execute(Some(short_id), &service);
        assert!(result.is_ok());
    }
}
