//! Output formatting and colored display for Synap CLI.
//!
//! This module provides functions for formatting and displaying notes,
//! tags, statistics, and other information with appropriate colors.

use chrono::{DateTime, Local};
use colored::*;
use std::time::SystemTime;
use synap_core::{Note, ServiceStats};

/// Format a SystemTime for display.
fn format_timestamp(timestamp: SystemTime) -> String {
    let duration_since_epoch = timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let dt: DateTime<Local> = DateTime::from_timestamp(duration_since_epoch.as_secs() as i64, 0)
        .unwrap()
        .into();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format tags for display.
///
/// Returns a string with tags colored in green and separated by commas.
pub fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        tags.iter()
            .map(|t| t.green().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Display a list of notes.
pub fn format_note_list(notes: Vec<Note>) {
    if notes.is_empty() {
        println!("\n📝 {}", "笔记列表 (0 篇)".dimmed());
        println!(
            "\n  {}",
            "还没有笔记。使用 'synap create <content>' 创建第一篇笔记。".dimmed()
        );
        return;
    }

    println!("\n📝 {} 笔记列表 ({} 篇)", "📝".cyan(), notes.len());

    // Calculate max ID width for alignment
    let max_id_len = notes
        .iter()
        .map(|n| n.id.to_string().len())
        .max()
        .unwrap_or(26);

    for note in &notes {
        let id = note.id.to_string().cyan();
        let content_preview: String = if note.content.len() > 30 {
            note.content.chars().take(27).collect::<String>() + "..."
        } else {
            note.content.clone()
        };

        // Get content padding to align tags
        let content_width = 30;
        let padding = if content_preview.chars().count() < content_width {
            " ".repeat(content_width - content_preview.chars().count())
        } else {
            String::new()
        };

        let tags = format_tags(&note.tags);
        println!(
            "  {:width$} {}{}",
            id,
            content_preview,
            padding,
            width = max_id_len
        );
        if !note.tags.is_empty() {
            println!("{:width$}     {}", "", tags, width = max_id_len + 2);
        }
    }

    println!();
}

/// Display detailed information about a single note.
pub fn format_note_detail(note: Note) {
    println!("\n📄 {}", "笔记详情".cyan().bold());

    println!("\n{}:", "ID".bright_black());
    println!("    {}", note.id.to_string().cyan());

    println!("{}:", "创建时间".bright_black());
    println!("    {}", format_timestamp(note.created_at).bright_black());

    println!("{}:", "更新时间".bright_black());
    println!("    {}", format_timestamp(note.updated_at).bright_black());

    if !note.tags.is_empty() {
        println!("{}:", "标签".bright_black());
        println!("    {}", format_tags(&note.tags));
    }

    println!();
    let separator = "─".repeat(45);
    println!("{}", separator.bright_black());
    println!("{}", note.content);
    println!("{}\n", separator.bright_black());
}

/// Display statistics about the database.
pub fn format_stats(stats: ServiceStats) {
    println!("\n📊 {}", "统计信息".cyan().bold());

    println!("总回复 {}", stats.total_replies);
    println!("总想法 {}", stats.total_thoughts);

    let unique_tags = stats.top_tags.len();
    println!("{}:", "标签总数".bright_black());
    println!("    {}", unique_tags.to_string().white().bold());

    if !stats.top_tags.is_empty() {
        println!("\n{}:", "热门标签".bright_black());
        for (i, (tag, count)) in stats.top_tags.iter().enumerate() {
            let rank = format!("#{}", i + 1).cyan();
            let tag_name = format!("{}", tag.green());
            let count_str = format!("({} 篇)", count).bright_black();
            println!("    {} {:20} {}", rank, tag_name, count_str);
        }
    }

    println!();
}

/// Display a knowledge graph starting from a note.
pub fn format_graph(graph: Vec<(Note, usize)>) {
    if graph.is_empty() {
        println!("\n🕸️  {}", "知识图谱 (空)".cyan());
        return;
    }

    let root_id = graph[0].0.id.to_string();
    let root_id_short = format!("{}...", &root_id[..10]);
    println!(
        "\n🕸️  {} ({})",
        "知识图谱".cyan().bold(),
        format!("根节点: {}", root_id_short).dimmed()
    );

    // Sort by depth
    let mut sorted = graph.clone();
    sorted.sort_by(|a, b| a.1.cmp(&b.1));

    // Build a tree structure for visualization
    for (note, depth) in &sorted {
        let indent = if *depth > 0 {
            // Check if this is the last child at this depth
            let parent_depth_idx = sorted
                .iter()
                .position(|(n, d)| n.id == note.id && d == depth)
                .unwrap();

            let is_last =
                parent_depth_idx == sorted.len() - 1 || sorted[parent_depth_idx + 1].1 < *depth;

            let connector = if is_last { "└─ " } else { "├─ " };
            "  ".repeat(*depth - 1) + connector
        } else {
            String::new()
        };

        let id_str = note.id.to_string();
        let id_short = format!("{}...", &id_str[..10]);
        let id = id_short.dimmed();
        let content: String = if note.content.len() > 40 {
            note.content.chars().take(37).collect::<String>() + "..."
        } else {
            note.content
                .lines()
                .next()
                .unwrap_or(&note.content)
                .to_string()
        };

        println!("{}{}  {}", indent, id, content);
    }

    println!();
}

/// Display an error message.
pub fn error(msg: &str) {
    eprintln!("{} {}", "错误:".red().bold(), msg);
}

/// Display a success message.
pub fn success(msg: &str) {
    println!("{}", msg.green().bold());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tags() {
        let tags = vec!["rust".to_string(), "programming".to_string()];
        let formatted = format_tags(&tags);
        assert!(formatted.contains("rust"));
        assert!(formatted.contains("programming"));
    }

    #[test]
    fn test_format_tags_empty() {
        let tags: Vec<String> = vec![];
        let formatted = format_tags(&tags);
        assert!(formatted.is_empty());
    }
}
