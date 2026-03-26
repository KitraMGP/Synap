//! Simple demo showing how to use SynapService

use synap_core::{SynapService, Result};

fn main() -> Result<()> {
    println!("🚀 Synap Core Demo\n");

    // Open an in-memory database
    let service = SynapService::open_memory()?;
    println!("✅ Opened in-memory database\n");

    // Create some notes
    println!("📝 Creating notes...");
    let rust = service.create_note("Rust is a systems programming language".to_string())?;
    let ownership = service.create_note("Ownership ensures memory safety".to_string())?;
    let borrowing = service.create_note("Borrowing allows temporary access".to_string())?;
    println!("✅ Created 3 notes\n");

    // Link notes to build knowledge graph
    println!("🔗 Building knowledge graph...");
    service.link_notes(rust.id, ownership.id)?;
    service.link_notes(rust.id, borrowing.id)?;
    println!("✅ Created relationships\n");

    // Add tags
    println!("🏷️  Adding tags...");
    service.add_tag(rust.id, "rust".to_string())?;
    service.add_tag(rust.id, "systems".to_string())?;
    service.add_tag(ownership.id, "concept".to_string())?;
    service.add_tag(borrowing.id, "concept".to_string())?;
    println!("✅ Added tags\n");

    // Display knowledge graph
    println!("📊 Knowledge Graph:");
    let graph = service.get_graph(rust.id, 0)?;
    for (note, depth) in graph {
        let indent = "  ".repeat(depth);
        println!("{}└─ {}", indent, note.content);
    }
    println!();

    // Display all tags
    println!("🏷️  All Tags:");
    let tags = service.get_all_tags()?;
    for tag in tags {
        println!("  • {}", tag);
    }
    println!();

    // Display statistics
    println!("📈 Statistics:");
    let stats = service.get_stats()?;
    println!("  Total Notes: {}", stats.total_notes);
    println!("  Total Edges: {}", stats.total_edges);
    println!("  Top Tags: {:?}", stats.top_tags);
    println!();

    // Search functionality
    println!("🔍 Search Results for 'ownership':");
    let results = service.search_notes("ownership")?;
    for note in results {
        println!("  • {}", note.content);
    }
    println!();

    println!("✨ Demo completed successfully!");

    Ok(())
}
