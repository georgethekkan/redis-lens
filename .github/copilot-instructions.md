# Redis Lens - AI Coding Assistant Instructions

## Project Overview
Redis Lens is a terminal user interface (TUI) application for browsing Redis and Valkey databases. It provides an interactive way to explore keys, view their values, types, and TTL information.

## Architecture
- **Main Entry Point**: `src/main.rs` - Handles CLI parsing, client initialization, and mode selection (TUI vs CLI operations)
- **TUI App**: `src/app.rs` - Core application logic with two-panel layout (key list + details)
- **Redis Abstraction**: `src/redis/` - Trait-based client interface with real (`real.rs`) and mock (`mock.rs`) implementations
- **CLI Args**: `src/args.rs` - Command-line argument parsing using clap

## Key Components
- **RedisClient Trait**: Defines interface for Redis operations (get, scan, ttl, key_type, del)
- **Connection Pooling**: Uses r2d2 for Redis connection management in production
- **Mock Client**: Provides fake data for testing without Redis server
- **TUI Framework**: Ratatui for terminal UI, Crossterm for event handling

## Development Workflow
- **Build**: `cargo build` (debug) or `cargo build --release`
- **Run**: `cargo run` (connects to localhost:6379 by default)
- **Dry Run**: `cargo run -- --dry-run` (uses mock client)
- **Specific Key**: `cargo run -- --key "mykey"` (prints key value to stdout)
- **Delete Pattern**: `cargo run -- --delete-all "pattern*"` (deletes matching keys)

## Code Patterns
### Client Abstraction
Use trait objects for Redis client polymorphism:
```rust
let redis_client: Box<dyn RedisClient> = if args.dry_run {
    Box::new(RedisClientMock::new("mock".to_string()))
} else {
    Box::new(RedisClientImpl::new(args.url, args.db)?)
};
```

### TUI Layout
Two-panel horizontal layout with list selection:
```rust
let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]);
let [left, right] = layout.areas(frame.area());
```

### Error Handling
Use color-eyre for consistent error reporting:
```rust
use color_eyre::Result;
fn main() -> Result<()> {
    color_eyre::install()?;
    // ... operations that return Result
}
```

### Connection Management
Pool connections with timeout for reliability:
```rust
let pool = r2d2::Pool::builder().build(manager)?;
let conn = pool.get_timeout(Duration::from_secs(5))?;
```

## Dependencies
- `ratatui` + `crossterm`: TUI framework
- `redis` + `r2d2`: Redis client with connection pooling
- `clap`: CLI argument parsing
- `color-eyre`: Error handling
- `itertools`: Utility functions

## Testing Approach
- Mock client provides deterministic test data
- Examples in `examples/` directory for UI component testing
- No unit tests currently implemented

## File Organization
- `src/main.rs`: Application entry and mode dispatch
- `src/lib.rs`: Module declarations
- `src/app.rs`: TUI application logic
- `src/args.rs`: CLI argument definitions
- `src/redis/mod.rs`: Client trait and re-exports
- `src/redis/real.rs`: Production Redis client implementation
- `src/redis/mock.rs`: Test/mock client implementation
- `examples/`: Standalone UI component demos</content>
<parameter name="filePath">c:\Users\georg\workspace\git\gth\redis-lens\.github\copilot-instructions.md