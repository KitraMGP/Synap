//! Stats command implementation.

use synap_core::SynapService;

/// Execute the stats command.
///
/// Displays statistics about the database.
pub fn execute(service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    let stats = service.get_stats()?;
    crate::output::format_stats(stats);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute() {
        let service = SynapService::open_memory().unwrap();
        let result = execute(&service);
        assert!(result.is_ok());
    }
}
