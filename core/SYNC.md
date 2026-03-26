# P2P Synchronization Protocol

This document describes the P2P synchronization protocol implementation for Synap.

## Architecture

The sync protocol is built on top of the immutable ledger architecture:

- **Incremental sync**: Only transfer missing blocks and pointers
- **Turn-based protocol**: Eliminates deadlock with initiator/responder roles
- **Receiver deduplication**: Each peer rejects data they already have
- **Conflict-free**: No coordination needed; conflicts resolved at read-time
- **Idempotent**: Receiving the same data multiple times is safe

## Protocol Flow

The protocol uses a **turn-based (round-trip)** design to avoid deadlocks:

```
Initiator (Client)                  Responder (Server)
  |                                   |
  |--- 1. Hello + Summary ----------->|
  |    (All my IDs)                    |
  |                                   |
  |<-- 2. SummaryResponse ------------|
  |    (My IDs + What I need)          |
  |                                   |
  |--- 3. Blocks/Pointers ----------->|
  |    (Only what responder needs)    |
  |                                   |
  |<-- 4. Blocks/Pointers ------------|
  |    (Only what initiator needs)    |
  |                                   |
  |--- 5. Done ---------------------->|
  |                                   |
```

### Key Design Decisions

1. **Asymmetric roles**: Initiator leads, Responder follows - no deadlock
2. **Summary exchange**: Both sides send ID lists before data transfer
3. **Diff calculation**: Each side calculates what the other needs
4. **Incremental transfer**: Only missing data is sent
5. **One-way data flow per phase**: No simultaneous read/write

## Message Format

All messages use length-prefixed binary framing:

- **4 bytes**: Message length (big-endian u32)
- **N bytes**: Message payload (bincode-encoded)

### Message Types

```rust
pub enum Message {
    // Phase 1: Initiator sends local summary
    Hello {
        version: u8,
        ledger_id: String,
        block_ids: Vec<Ulid>,
        pointer_ids: Vec<Ulid>,
    },

    // Phase 2: Responder replies with summary + diff request
    SummaryResponse {
        block_ids: Vec<Ulid>,           // Responder's block IDs
        pointer_ids: Vec<Ulid>,         // Responder's pointer IDs
        needed_blocks: Vec<Ulid>,       // What responder needs from initiator
        needed_pointers: Vec<Ulid>,     // What responder needs from initiator
    },

    // Phase 3 & 4: Incremental data transfer
    Blocks { blocks: Vec<Block> },
    Pointers { pointers: Vec<Pointer> },

    // Phase 5: Completion
    Done,

    // Error handling
    Error { message: String },
}
```

## Usage Example

### Initiator (Client) - Active Sync

```rust
use synap_core::sync::{SyncProtocol, SyncConfig};
use synap_core::net::tcp::TcpConn;
use synap_core::db::SynapDb;
use std::net::SocketAddr;

// Connect to server
let addr: SocketAddr = "192.168.1.100:8080".parse().unwrap();
let conn = TcpConn::connect(addr)?;

// Create sync protocol
let db = SynapDb::open("my_notes.db")?;
let config = SyncConfig::default();
let mut protocol = SyncProtocol::with_db(conn, config, db);

// Sync as initiator (client role)
let stats = protocol.sync_as_initiator()?;

println!("Sync completed:");
println!("  New blocks: {}", stats.blocks_received);
println!("  New pointers: {}", stats.pointers_received);
println!("  Duration: {} ms", stats.duration_ms);
```

### Responder (Server) - Passive Sync

```rust
use synap_core::sync::{SyncProtocol, SyncConfig};
use synap_core::net::tcp::TcpListener;
use synap_core::db::SynapDb;

// Listen for incoming connections
let mut listener = TcpListener::bind("0.0.0.0:8080".parse()?)?;

loop {
    // Accept incoming connection
    let conn = listener.accept()?;

    // Create sync protocol
    let db = SynapDb::open("my_notes.db")?;
    let config = SyncConfig::default();
    let mut protocol = SyncProtocol::with_db(conn, config, db);

    // Sync as responder (server role)
    match protocol.sync_as_responder() {
        Ok(stats) => {
            println!("Sync with client completed:");
            println!("  Sent {} blocks", stats.blocks_received);
        }
        Err(e) => {
            eprintln!("Sync failed: {}", e);
        }
    }
}
```

### Custom Transport Implementation

The sync protocol works with any transport that implements the `Conn` trait:

```rust
use synap_core::net::Conn;
use std::io::{Read, Write};

struct MyWebSocket {
    // ... WebSocket implementation
}

impl Read for MyWebSocket {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Read from WebSocket
    }
}

impl Write for MyWebSocket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Write to WebSocket
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Flush WebSocket
    }
}

impl Conn for MyWebSocket {
    fn local_addr(&self) -> Addr {
        Addr::WebSocket("ws://localhost:8080".to_string())
    }

    fn remote_addr(&self) -> Addr {
        Addr::WebSocket("ws://peer:8080".to_string())
    }

    fn close(&mut self) -> std::io::Result<()> {
        // Close WebSocket
    }
}

// Use with sync protocol
let protocol = SyncProtocol::new(ws_conn, SyncConfig::default())?;
```

## Configuration

```rust
pub struct SyncConfig {
    /// Maximum blocks per batch (default: 1000)
    pub max_batch_size: usize,

    /// Enable compression (future feature)
    pub compression: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            compression: false,
        }
    }
}
```

## Database Integration

The sync protocol uses these database methods:

- `get_all_blocks()` - Get all blocks for broadcasting
- `get_all_pointers()` - Get all pointers for broadcasting
- `block_exists(id)` - Check if block already present (deduplication)
- `pointer_exists(id)` - Check if pointer already present (deduplication)
- `append_block(block)` - Append received block
- `append_pointer(pointer)` - Append received pointer

## Conflict Resolution

Conflicts are automatically resolved at read-time:

1. **Edit vs Delete**: "Life over death" - edits always win
2. **Concurrent Edits**: Both versions preserved; read-time resolution
3. **DAG Cycles**: Read-time cycle detection and pruning
4. **Tag Inheritance**: Tags follow logical IDs across edit chains

No special handling needed during sync - just append everything and let the read layer handle conflicts.

## Performance Characteristics

### Incremental Sync

- **First sync**: Full transfer (all blocks and pointers)
- **Subsequent syncs**: Only transfer differences
- **Bandwidth**: Proportional to changes, not total size
- **Memory**: Batching limits peak memory usage

### Example Scenarios

| Scenario | Data Size | Transfer Size |
|----------|-----------|---------------|
| First sync | 50 MB | 50 MB (full) |
| Add 1 note | 50 MB + 1 KB | ~2 KB (1 block + 1 pointer) |
| Edit note | 50 MB + 1 KB | ~2 KB (new block + edit pointer) |
| No changes | 50 MB each | ~200 bytes (ID lists only) |

### Scalability

- **Time complexity**: O(N) where N = number of changed items
- **Space complexity**: O(M) where M = max_batch_size
- **Network**: 3 round-trips per sync session

## Security Considerations

Current implementation:
- No authentication
- No encryption
- No authorization
- Trust-all-peers model

Recommended for production:
- Add TLS for transport encryption
- Implement peer authentication
- Add ledger signature verification
- Rate limiting and resource quotas
- Input validation and sanitization

## Testing

Run sync tests:

```bash
cargo test -p synap-core sync::
```

Tests cover:
- Message serialization/deserialization
- Block and pointer batching
- Deduplication logic
- Empty database handling
- Framing codec
