//! Graph command implementation.

use synap_core::{SynapService, Ulid};

/// Execute the graph command.
///
/// Displays the knowledge graph starting from a note.
pub fn execute(
    id_str: &str,
    depth: usize,
    service: &SynapService,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = parse_ulid(id_str)?;

    // Verify the note exists
    let note = service.get_note(id)?;

    if note.deleted {
        return Err(format!("笔记 {} 已被删除", id_str).into());
    }

    let graph = service.get_graph(id, depth)?;

    if graph.is_empty() {
        println!("\n🕸️  {}", "知识图谱 (空)");
    } else {
        crate::output::format_graph(graph);
    }

    Ok(())
}

/// Parse a ULID from a string.
fn parse_ulid(s: &str) -> Result<Ulid, Box<dyn std::error::Error>> {
    Ulid::parse_str(s).map_err(|e| format!("无效的 ULID: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ulid_full() {
        let ulid = Ulid::new_v4();
        let ulid_str = ulid.to_string();
        let result = parse_ulid(&ulid_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ulid);
    }

    #[test]
    fn test_parse_ulid_invalid() {
        let result = parse_ulid("invalid_ulid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ulid_short() {
        let result = parse_ulid("01KJYFFB4Z");
        assert!(result.is_err());
    }
}
