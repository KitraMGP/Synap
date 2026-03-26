//! Search command implementation with #tag support.

use colored::Colorize;
use synap_core::SynapService;

/// Execute the search command.
///
/// Searches for notes containing the query string or by #tag.
pub fn execute(query: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    if query.is_empty() {
        return Err("搜索内容不能为空".into());
    }

    // Detect if it's a tag search (#tag)
    if query.starts_with('#') {
        return execute_tag(&query[1..], service);
    }

    // Original content search logic
    let results = service.search_notes(query)?;

    if results.is_empty() {
        println!("\n{} 未找到匹配的笔记", "🔍".cyan());
    } else {
        println!("\n{} 找到 {} 条匹配的笔记:", "🔍".cyan(), results.len());

        for note in results {
            let short_id = &note.id.to_string()[..12];
            println!("  [{}] {}", short_id.cyan(), note.content);
            if !note.tags.is_empty() {
                let tags: Vec<String> = note.tags.iter()
                    .map(|t| format!("#{}", t))
                    .collect();
                let tags_formatted = tags.join(" ");
                let tags_colored = tags_formatted.cyan();
                println!("      {}", tags_colored);
            }
        }
    }

    println!();
    Ok(())
}

/// Tag search.
pub fn execute_tag(tag: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    let notes = service.get_notes_by_tag(tag)?;

    if notes.is_empty() {
        println!("\n{} 未找到标签 #{} 的笔记", "🏷️".cyan(), tag.cyan());
    } else {
        println!("\n{} 找到 {} 条标签为 #{} 的笔记:", "🏷️".cyan(), notes.len(), tag.cyan());

        for note in notes {
            let short_id = &note.id.to_string()[..12];
            println!("  [{}] {}", short_id.cyan(), note.content);
        }
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_with_results() {
        let service = SynapService::open_memory().unwrap();
        service.create_note("Test note about rust".to_string()).unwrap();
        let result = execute("rust", &service);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_empty_query() {
        let service = SynapService::open_memory().unwrap();
        let result = execute("", &service);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_tag_search() {
        let service = SynapService::open_memory().unwrap();
        let note = service.create_note("Test note".to_string()).unwrap();
        service.add_tag(note.id, "rust".to_string()).unwrap();

        let result = execute("#rust", &service);
        assert!(result.is_ok());
    }
}
