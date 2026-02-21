# Redis Lens - Feature Roadmap

## 🎯 High Priority (Post 0.1.0)

### 1. Data Structure Support
- [ ] Stream support: View stream entries with timestamps
- [ ] JSON pretty-printing for JSON values

### 2. Search & Filtering
- [ ] Regex pattern matching support
- [ ] Real-time search results (search as you type)
- [ ] Persist search filters across navigation

### 3. Database Management & Monitoring
- [ ] Key statistics view (keys per type, memory distribution)
- [ ] Connection status indicator
- [ ] Real-time monitor for key changes
- [ ] Performance metrics dashboard

## 🚀 Medium Priority

### 4. Advanced Operations
- [ ] TTL management: Interactive TTL setter
- [ ] Rename keys in TUI
- [ ] Multi-select keys for bulk operations
- [ ] FLUSHDB with safety confirmation
- [ ] Copy key/value to clipboard

### 5. Navigation & UX Improvements
- [ ] Sort keys by: name, TTL, type, memory usage
- [ ] Horizontal scrolling for long values
- [ ] Custom themes support

### 6. Performance
- [ ] Async loading for large key scans
- [ ] Display operation timings

## 💾 Lower Priority

### 7. Persistence & Configuration
- [ ] Export selected keys to JSON/Redis commands
- [ ] Config file support (~/.redis-lens.toml)
- [ ] Save recent connections

## 🛠️ Code Quality
- [ ] Add unit tests for RedisClient
- [ ] Extend mock client with realistic test data
- [ ] Add debug logging mode
- [ ] Improve error context messages
