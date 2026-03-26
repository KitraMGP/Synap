//! Tag command implementation.

use synap_core::{SynapService, Ulid};

/// Execute the tag command.
pub fn execute(
    id_str: &str,
    tag: &str,
    remove: bool,
    service: &SynapService,
) -> Result<(), Box<dyn std::error::Error>> {
    if tag.is_empty() {
        return Err("标签名称不能为空".into());
    }

    let id = parse_ulid(id_str)?;

    // Verify the note exists
    let note = service.get_note(id)?;

    if note.deleted {
        return Err(format!("笔记 {} 已被删除", id_str).into());
    }

    if remove {
        service.remove_tag(id, tag)?;

        crate::output::success(&format!(
            "已删除标签: #{}",
            tag
        ));
    } else {
        service.add_tag(id, tag.to_string())?;

        crate::output::success(&format!(
            "已添加标签: #{}",
            tag
        ));
    }

    Ok(())
}

/// Execute tag add operation.
pub fn execute_add(id_str: &str, tag: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    execute(id_str, tag, false, service)
}

/// Execute tag remove operation.
pub fn execute_remove(id_str: &str, tag: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    execute(id_str, tag, true, service)
}

/// Parse a ULID from a string.
fn parse_ulid(s: &str) -> Result<Ulid, Box<dyn std::error::Error>> {
    Ulid::parse_str(s)
        .map_err(|e| format!("无效的 ULID: {}", e).into())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_parse_ulid_full() {
        let ulid = Uuid::new_v4();
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
