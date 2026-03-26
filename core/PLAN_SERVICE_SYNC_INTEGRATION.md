# Service-Sync Integration Plan

## Current Architecture Analysis

### Completed Modules

| Module | Status | Description |
|--------|--------|-------------|
| `lib.rs` | ✅ Complete | Public API exposure, correctly exports essential types only |
| `service.rs` | ✅ Complete | Full service layer with all CRUD operations, DAG traversal, GC |
| `db/` | ✅ Complete | Database layer (SynapDb) - properly private (`pub(crate)`) |
| `models/` | ✅ Complete | Block, Pointer, NoteView - correctly exports public types |
| `error.rs` | ✅ Complete | CoreError enum with all error variants |
| `net.rs` | ✅ Complete | Conn trait abstraction for transport-agnostic networking |
| `net/tcp.rs` | ✅ Complete | Example TCP implementation of Conn trait |
| `sync/protocol.rs` | ✅ Complete | Full P2P sync protocol implementation |
| `sync/mod.rs` | ✅ Complete | Module re-exports |

### Key Problem: Integration Gap

The `SyncProtocol` creates its own isolated database:
```rust
// Current implementation in sync/protocol.rs:219
pub fn new(conn: C, config: SyncConfig) -> Result<Self> {
    let db = SynapDb::open_memory()?;  // ❌ Isolated from service!
    Ok(Self { conn, config, db })
}
```

This prevents `SyncProtocol` from syncing with the user's actual data.

## Implementation Plan

### Phase 1: Fix SyncProtocol Architecture (High Priority)

**Problem**: `SyncProtocol` creates isolated database instead of using service's database.

**Solution**:
1. Modify `SyncProtocol::new()` to require a `SynapDb` reference
2. Add `SyncProtocol::with_db()` for explicit database injection
3. Update constructor to handle owned vs borrowed database

**Files to modify**:
- `core/src/sync/protocol.rs`

**Changes**:
```rust
// Remove the default constructor that creates isolated db
// pub fn new(conn: C, config: SyncConfig) -> Result<Self> { ... }

// Keep and improve the with_db constructor
pub fn with_db(conn: C, config: SyncConfig, db: SynapDb) -> Self {
    Self { conn, config, db }
}
```

### Phase 2: Add Sync Methods to SynapService (High Priority)

**Problem**: Service layer has no sync integration.

**Solution**: Add sync methods to `SynapService` that internally use `SyncProtocol`.

**Files to modify**:
- `core/src/service.rs`

**New Methods**:
```rust
impl SynapService {
    /// Synchronize with a remote peer (initiator mode)
    pub fn sync_with<C: Conn>(&self, conn: C) -> Result<SyncStats>

    /// Synchronize with a remote peer (responder mode)
    pub fn sync_respond<C: Conn>(&self, conn: C) -> Result<SyncStats>

    /// Synchronize with configuration
    pub fn sync_with_config<C: Conn>(&self, conn: C, config: SyncConfig) -> Result<SyncStats>
}
```

### Phase 3: Update Public API Exports (Medium Priority)

**Problem**: Sync-related types need proper re-export in lib.rs

**Current state**:
```rust
pub mod sync;  // Module is public
pub use sync::{Message, SyncConfig, SyncProtocol, SyncStats};
```

**Assessment**: This is actually correct! Users can access sync types via:
- `synap_core::sync::SyncProtocol`
- `synap_core::SyncConfig`
- `synap_core::SyncStats`

**Action**: No changes needed, but verify re-exports are complete.

### Phase 4: Verify Encapsulation (Low Priority)

**Goal**: Ensure implementation details are properly hidden.

**Checklist**:
- ✅ `db::` module is `pub(crate)` - hidden from public API
- ✅ `db::SynapDb` is `pub(crate)` - hidden from public API
- ✅ `models::note.rs` is private - implementation detail
- ✅ `models::pointer.rs` internal structure is public (needed for sync)
- ⚠️ `models::Block` and `models::Pointer` are public - needed for sync protocol

**Assessment**: Current encapsulation is correct. Block and Pointer must be public for the sync protocol to work.

### Phase 5: Integration Testing (Medium Priority)

**Tests needed**:
1. Service integration test for sync_with()
2. Service integration test for sync_respond()
3. End-to-end sync test with actual data
4. Verify data consistency after sync

**File to create/modify**:
- `core/src/service.rs` - Add integration tests
- `core/src/sync/tests.rs` - Extend existing tests

## Execution Order

1. **First**: Fix `SyncProtocol` constructor (Phase 1)
2. **Second**: Add sync methods to `SynapService` (Phase 2)
3. **Third**: Add integration tests (Phase 5)
4. **Fourth**: Verify public API (Phase 3 - validation only)
5. **Fifth**: Final encapsulation review (Phase 4 - validation only)

## Design Decisions

### Why Remove `SyncProtocol::new()`?

The current `new()` creates an isolated memory database, which is never what users want:
- ❌ Can't sync user's actual data
- ❌ Creates data isolation
- ✅ `with_db()` makes database injection explicit

### Why Add Sync Methods to Service?

- **Abstraction**: Users interact with `SynapService`, not `SyncProtocol` directly
- **Simplicity**: One less type to manage
- **Database sharing**: Service shares its `SynapDb` with protocol
- **Error handling**: Service's `Result` type is already integrated

### Why Keep `SyncProtocol` Public?

- **Flexibility**: Advanced users can implement custom sync logic
- **Testing**: Direct protocol testing is easier
- **Transparency**: No magic - users can see how sync works

## API Usage Examples

### After Integration

```rust
// Simple sync (service manages everything)
let service = SynapService::open("notes.db")?;
let conn = TcpConn::connect("192.168.1.100:8080".parse()?)?;
let stats = service.sync_with(conn)?;
println!("Synced {} blocks", stats.blocks_received);

// Advanced sync (direct protocol access)
let protocol = SyncProtocol::with_db(conn, config, service.db.clone());
let stats = protocol.sync_as_initiator()?;
```

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking change to `SyncProtocol::new()` | Low - sync likely unused | Keep `with_db()` unchanged |
| Service bloat | Low - only 3 new methods | Methods are thin wrappers |
| Cloning database | Medium - performance | Use `Arc` internally if needed |
| Test complexity | Medium - need E2E tests | Add separate sync test module |

## Success Criteria

1. ✅ `SyncProtocol` requires database injection
2. ✅ `SynapService` has sync methods
3. ✅ All existing tests pass
4. ✅ New integration tests pass
5. ✅ Public API remains clean and minimal
6. ✅ Documentation examples work
