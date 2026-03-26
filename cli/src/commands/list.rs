//! List command implementation.

use synap_core::SynapService;

/// Execute the list command.
///
/// Lists all notes, optionally filtered by tag.
pub fn execute(service: &SynapService, tag_filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let notes = if let Some(tag) = tag_filter {
        service.get_notes_by_tag(&tag)?
    } else {
        service.list_notes()?
    };

    crate::output::format_note_list(notes);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_list_all() {
        let service = SynapService::open_memory().unwrap();
        let result = execute(&service, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_list_by_tag() {
        let service = SynapService::open_memory().unwrap();
        let result = execute(&service, Some("rust".to_string()));
        assert!(result.is_ok());
    }
}
