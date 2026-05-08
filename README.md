# Text Chisel

[![Rust](https://github.com/c0d3-k1ra/text-chisel/actions/workflows/rust.yml/badge.svg)](https://github.com/c0d3-k1ra/text-chisel/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A macOS menu bar app that rewrites selected text using Claude AI. Lives in your menu bar, stays out of your way — select text anywhere, press a hotkey, get it back polished.

---

## How it works

1. Select text in any app — Slack, Gmail, Notes, Terminal, anywhere
2. Press **⌘⌥R** (Cmd+Option+R)
3. Text Chisel rewrites it in your chosen tone and pastes it back automatically

No copy-paste. No switching windows. No context lost.

---

## Tones

Pick a tone from **menu bar icon → Tone**:

| Tone | What it does |
| ---- | ------------ |
| **Professional** | Neutral and polished — clear without being stiff |
| **Polite** | Soft and respectful — takes the edge off |
| **Assertive** | Direct and firm — makes the point land |
| **Concise** | Strips it down — no filler, no fluff |
| **Gen Z** | Casual and internet-native — lowercase, emojis, the whole bit |

The active tone is shown in the submenu title (e.g. "Tone: Concise ▶") so you can always see your selection without opening it.

---

## Requirements

- macOS 12 or later
- An [Anthropic API key](https://console.anthropic.com/)
- Accessibility permission (so the app can simulate Cmd+C and Cmd+V)

---

## Setup

```bash
git clone https://github.com/c0d3-k1ra/text-chisel
cd text-chisel
git config core.hooksPath .githooks
cargo run
```

`git config core.hooksPath .githooks` installs the project's git hooks — `fmt` + `clippy` on commit, tests on push. Run it once after cloning.

On first launch, the Settings window opens automatically. Paste your Anthropic API key and click **Save**.

macOS will prompt for Accessibility access the first time — approve it in **System Settings → Privacy & Security → Accessibility**.

### Build a standalone .app bundle

```bash
cargo install cargo-bundle
cargo bundle --release
```

The app is output to `target/release/bundle/osx/Text Chisel.app`. Drag it to `/Applications` to install.

### Gatekeeper warning

The app is not signed with an Apple Developer certificate, so macOS will block it on first launch with a "cannot be opened because the developer cannot be verified" message. To bypass it, run this once after installing:

```bash
xattr -cr "/Applications/Text Chisel.app"
```

Then launch normally. You only need to do this once.

---

## Settings

Click the menu bar icon → **Settings** to configure:

- **API Key** — your `sk-ant-...` key from [console.anthropic.com](https://console.anthropic.com/)
- **Model** — `claude-haiku-4-5` for speed, `claude-sonnet-4-6` for higher quality
- **Test Connection** — verifies the key is valid before saving

Settings are saved to `~/.config/text-chisel/config.toml`.

---

## Menu bar

The menu bar icon shows:

- **Connection status** — 🟢 Connected, 🔴 Not connected, or ⏳ Checking... (checked on startup and after every settings save)
- **Tone submenu** — active tone shown in the title, all five tones selectable
- **Launch at Login** — toggle to start Text Chisel automatically on login
- **Settings** — open the settings window
- **Quit**

---

## Error notifications

When something goes wrong, Text Chisel shows a macOS notification with a Basso sound so you always know what happened:

| Situation | Message |
| --------- | ------- |
| Nothing selected | Select some text first, then press ⌘⌥R |
| No API key | Add your Anthropic API key in Settings to get started |
| Invalid API key | API key not accepted. Open Settings to update it |
| Selection too long | Try again with under 8,000 characters |
| Rate limited | Too many requests. Wait a moment and try again |
| Claude busy | Claude is busy right now. Give it a moment and try again |
| Timeout | Claude took too long to respond. Try again in a moment |
| Accessibility denied | Enable Accessibility access in System Settings |

**Notification tip:** For notifications to appear as banners on screen (rather than arriving silently), go to **System Settings → Notifications → Script Editor** and set the alert style to **Banners** or **Alerts**.

---

## Logging

```bash
RUST_LOG=debug cargo run    # verbose — clipboard contents, API calls, timing
cargo run                   # default info level
```

---

## Project structure

```
src/
├── main.rs            # entry point, event loop, hotkey dispatch
├── clipboard.rs       # Cmd+C to read selection, Cmd+V to paste back
├── hotkey.rs          # global Cmd+Option+R registration
├── rewrite.rs         # Claude API call and response parsing
├── prompts.rs         # system prompt and per-tone instructions with few-shot examples
├── tray.rs            # menu bar icon, tone submenu, status item, launch at login
├── settings_window.rs # settings UI (wry webview)
├── login_item.rs      # LaunchAgent plist — enable/disable launch at login
└── config.rs          # load/save ~/.config/text-chisel/config.toml
assets/
├── icon.svg           # menu bar icon source
├── icon.icns          # compiled macOS icon for .app bundle
└── settings.html      # settings window UI
```

---

## Releasing

Use the `/release` skill — it handles the changelog, version bump, signed commit, tag, and push automatically. Or manually:

```bash
# edit Cargo.toml and CHANGELOG.md, then:
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -S -m "release version x.y.z"
git tag -a vx.y.z -m "release vx.y.z"
git push origin main vx.y.z
```

GitHub Actions builds the `.app`, pulls release notes from `CHANGELOG.md`, and attaches the zip to the release automatically.

---

## Limitations

- **macOS only** — relies on AppKit, global hotkeys, and osascript
- **Accessibility required** — the app cannot simulate Cmd+C/Cmd+V without it
- **8,000 character limit** — selections longer than this are rejected to keep API costs low
- **No offline mode** — requires a live Anthropic API connection
