# 🦉 Obsidian Browser

minimalist prototype , secure, and privacy-focused web browser.

## ✨ Features

- 🔒 Extreme privacy (cookies, cache, and WebRTC disabled)
- 🧅 Built-in Tor via Proxychains
- 🚫 Tracker and ad blocker
- 📖 Reading mode
- ⭐ Bookmarks
- 📜 History
- ⌨️ Keyboard shortcuts
- 🔄 YouTube redirection → Invidious

## 📦 Installation

```bash
sudo apt install tor proxychains4 libgtk-4-dev libwebkit2gtk-4.1-dev
cargo build --release
chmod +x obsidian
./obsidian
