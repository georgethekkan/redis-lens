# 🦅 Redis Lens

**Redis Lens** is a high-performance, premium Terminal User Interface (TUI) for exploring and managing Redis and Valkey databases. Built with Rust and `ratatui`, it provides a sleek, interactive experience for navigating complex datasets.

![Redis Lens Dashboard](https://raw.githubusercontent.com/georgethekkan/redis-lens/main/assets/preview.png) *(Placeholder: Replace with actual screenshot when available)*

## ✨ Features

- **🌳 Interactive Key Tree**: Navigate your Redis database using a smart folder-based tree view. Automatically groups keys using common delimiters.
- **🔍 Powerful Search & Filter**: Instant key filtering with support for Redis glob patterns (`/`).
- **📝 In-place Value Editing**: Modify Strings, Hashes, Lists, Sets, and Sorted Sets directly within the TUI (`e`).
- **✨ Data Insertion**: Create new keys (`i`) or add items to existing collections (`a`) with intuitive multi-step dialogs.
- **📊 Live Server Metrics**: Real-time monitoring of **Memory Usage**, **CPU Load**, and **Key Counts** in a premium header dashboard.
- **🎯 Dual-Pane Navigation**: Seamlessly switch focus between your key hierarchy and data details (`Tab`) with intuitive visual cues.
- **🎨 Color-Coded Types**: Distinctive visual styles for every Redis data type (Strings, Hashes, Lists, etc.) for instant recognition.
- **⚡ Performance First**: Efficient type-aware pagination (50 items per page) and cursor-based scanning to handle large databases.

## ⌨️ Keyboard Shortcuts

| Key | Action |
| :--- | :--- |
| `↑` / `↓` | Navigate Tree or Collection Items |
| `Enter` / `Space` | Toggle Folder Expansion |
| `Tab` | Switch focus between Tree and Details |
| `←` / `→` | Collapse/Expand Folders or Page through Collections |
| `r` | **Refresh** stats, keys, and current data |
| `i` | **Insert** new key (Step-by-step) |
| `a` | **Add** item to current collection (at end/start) |
| `e` | **Edit** current value (In-place) |
| `d` | **Delete** selected key or collection item |
| `b` | **Database** selector (0-15) |
| `/` | Open **Search** pattern popup |
| `n` | Load next page of keys |
| `h` / `?` | Open keyboard shortcuts help modal |
| `Esc` | Cancel / Close popup |
| `q` | Quit application |

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- A running Redis or Valkey instance

### Installation
```bash
# Clone the repository
git clone https://github.com/georgethekkan/redis-lens.git
cd redis-lens

# Build and install
cargo install --path .
```

### Usage
```bash
# Connect to default (localhost:6379)
redis-lens

# Connect to a specific server
redis-lens --url redis://127.0.0.1:6379/0
```

## 🛠️ Configuration
You can pass the connection URL via command-line arguments:
- `--url <URL>`: Redis connection string (e.g., `redis://username:password@host:port/db`)

## 🤝 Credits & Inspiration

Inspired by **[RedisInsight](https://redis.io/insight/)**, aiming to bring a similar experience to the terminal.

Developed with the assistance of:
- **ChatGPT**: For architectural guidance and core logic.
- **Antigravity**: For advanced agentic coding and feature implementation.
- **[Ratatui](https://ratatui.rs/)**: The incredible library powering the TUI.

## 📄 License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
