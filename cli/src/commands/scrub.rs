//! Scrub command implementation - ZFS-style garbage collection.

use colored::Colorize;
use synap_core::SynapService;

/// Pruning: Execute local GC, physically erase unreferenced tombstone blocks.
///
/// # Example
/// ```bash
/// synap scrub
/// [i] Starting read-only ledger reachability scan (Mark and Sweep)...
/// [i] Scan completed: 1524 blocks
/// [i] Found 12 orphaned blocks covered by tombstones without child nodes
/// [!] Physical erasure in progress... Released 4KB. Ledger has been reset to absolute purity.
/// ```
pub fn execute(service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} 启动只读账本可达性扫描 (Mark and Sweep)...", "[i]".blue());

    let (blocks_scanned, bytes_freed) = service.scrub_garbage()?;

    println!("{} 扫描完成: {} 个区块", "[i]".blue(), blocks_scanned);

    if bytes_freed > 0 {
        println!(
            "{} 发现 {} 个被墓碑遮蔽且无子节点的孤块",
            "[i]".blue(),
            bytes_freed
        );
        println!(
            "{} 物理抹除执行中... 释放 {}。账本已重置为绝对纯净状态。",
            "[!]".red(),
            format!("{} KB", bytes_freed / 1024).cyan()
        );
    } else {
        println!("{} 账本纯净，无需清理", "[✓]".green());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_empty_db() {
        let service = SynapService::open_memory().unwrap();
        let result = execute(&service);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_abandoned_note() {
        let service = SynapService::open_memory().unwrap();
        let note = service.add_thought("Test".to_string()).unwrap();
        service.abandon_thought(note.id).unwrap();

        let result = execute(&service);
        assert!(result.is_ok());
    }
}
