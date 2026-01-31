# 🦅 Redis Lens

**Redis Lens** is a high-performance, premium Terminal User Interface (TUI) for exploring and managing Redis and Valkey databases. Built with Rust and `ratatui`, it provides a sleek, interactive experience for navigating complex datasets.

![Redis Lens Dashboard](https://raw.githubusercontent.com/georgethekkan/redis-lens/main/assets/preview.png) *(Placeholder: Replace with actual screenshot when available)*

## ✨ Features

- **🌳 Interactive Key Tree**: Navigate your Redis database using a smart folder-based tree view. Automatically groups keys using common delimiters.
- **🔍 Powerful Search & Filter**: Instant key filtering with support for Redis glob patterns.
- **📝 In-place Value Editing**: Modify Strings, Hashes, Lists, Sets, and Sorted Sets directly within the TUI.
- **📊 Live Server Metrics**: Real-time monitoring of **Memory Usage**, **CPU Load**, and **Key Counts** in a premium header dashboard.
- **🎯 Dual-Pane Navigation**: Seamlessly switch focus between your key hierarchy and data details with intuitive visual cues.
- **🎨 Color-Coded Types**: Distinctive visual styles for every Redis data type (Strings, Hashes, Lists, etc.) for instant recognition.
- **⚡ Performance First**: Efficient type caching and cursor-based scanning to handle large databases without freezing the UI.

## ⌨️ Keyboard Shortcuts

| Key | Action |
| :--- | :--- |
| `↑` / `↓` | Navigate Tree or Collection Items |
| `Enter` / `Space` | Toggle Folder Expansion |
| `Tab` | Switch focus between Tree and Details |
| `←` / `→` | Collapse/Expand Folders or Page through Collections |
| `r` | **Refresh** stats, keys, and current data |
| `e` | **Edit** current value (Strings/Hashes/Lists/Sets) |
| `d` | **Delete** selected key or collection item |
| `/` | Open **Search** pattern popup |
| `n` | Load next page of keys |
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
- `--url <URL>`: Redis connection string (e.g., `redis://user:password@host:port/db`)

## 📄 License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
