//! Void command implementation - abandon thoughts.

use colored::Colorize;
use synap_core::SynapService;

/// Abandon: Add tombstone pointer, remove the block from view.
///
/// # Example
/// ```bash
/// synap void A3F
/// [-] Declare death: TOMBSTONE ──[abandon]─> (A3F)
/// ```
pub fn execute(short_id: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    let note = service.get_note_by_short_id(short_id)?;
    let short_id_display = &note.id.to_string()[..12];

    service.abandon_thought(note.id)?;

    println!(
        "{} 宣告死亡: TOMBSTONE ──[废弃]─> ({})",
        "[-]".red(),
        format!("{}...", short_id_display).red()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_success() {
        let service = SynapService::open_memory().unwrap();
        let note = service.add_thought("Test".to_string()).unwrap();
        let short_id = &note.id.to_string()[..12];

        let result = execute(short_id, &service);
        assert!(result.is_ok());

        // Verify note is abandoned
        let notes = service.list_thoughts().unwrap();
        assert_eq!(notes.len(), 0);
    }
}
