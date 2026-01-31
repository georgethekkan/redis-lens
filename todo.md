# Redis Lens - Feature Roadmap

## 🎯 Priority Features (High Impact)

### 1. Data Structure Support
- [x] List support: Browse list elements with pagination
- [x] Hash support: Display field-value pairs in table format
- [x] Set support: Show set members with count
- [x] Sorted Set support: Display members with scores
- [ ] Stream support: View stream entries with timestamps

### 2. Search & Filtering
- [ ] Search bar in TUI to filter keys by pattern
- [ ] Regex pattern matching support
- [ ] Real-time search results
- [ ] Persist search filters across navigation

### 3. Value Editor
- [ ] In-place editing for string values
- [ ] JSON pretty-printing for JSON values
- [ ] Save changes with confirmation
- [ ] Undo last edit

### 4. Database Management
- [ ] Database selector to switch between databases (0-15)
- [ ] Show total key count per database
- [ ] Server info display (memory, connections, version)
- [ ] Key statistics view (keys per type, memory distribution)

## 🚀 Medium Priority Features

### 5. Advanced Operations
- [ ] TTL management: Interactive TTL setter
- [ ] Set key expiration with dialog confirmation
- [ ] Rename keys in TUI
- [ ] Multi-select keys for bulk operations
- [ ] FLUSHDB with safety confirmation
- [ ] Copy key/value to clipboard

### 6. Navigation & UX Improvements
- [x] Keyboard shortcuts help modal (h or ?)
- [ ] Tab navigation between panes
- [x] Better pagination UI with current/total pages
- [ ] Sort keys by: name, TTL, type, memory usage
- [x] Light/dark theme support (Theme infrastructure implemented)
- [x] Color code keys by type

### 7. Performance Features
- [ ] Async loading for large key scans
- [ ] Connection status indicator
- [ ] Auto-refresh option for watching changes
- [ ] Show key size in bytes
- [ ] Display operation timings

## 💾 Lower Priority Features

### 8. Persistence & Export
- [ ] Export selected keys to JSON
- [ ] Export as Redis commands
- [ ] Import keys from file
- [ ] Bulk editor for raw data files

### 9. Monitoring & Debugging
- [ ] Real-time monitor for key changes
- [ ] Key access log/history
- [ ] Performance metrics dashboard
- [ ] Improved error messages with context

### 10. Configuration
- [ ] Config file support (~/.redis-lens.toml)
- [ ] Save recent connections
- [ ] User preferences (default DB, theme, page size)
- [ ] Customizable color schemes

## 🛠️ Code Quality

- [ ] Add unit tests for RedisClient
- [x] Refactor redis.rs into redis/mod.rs with subtypes
- [ ] Extend mock client with realistic test data
- [ ] Add debug logging mode
- [ ] Improve error context messages
- [ ] Update README with usage examples

## ✅ Quick Wins (Easy to Implement First)
- [x] Display key count
- [ ] Add Ctrl+f for search
- [x] Show key size in bytes
- [x] Color code keys by type
- [ ] Horizontal scrolling for long values
- [x] Display selected key name in details panel
