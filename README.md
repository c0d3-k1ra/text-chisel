# Text Chisel

[![Rust](https://github.com/c0d3-k1ra/text-chisel/actions/workflows/rust.yml/badge.svg)](https://github.com/c0d3-k1ra/text-chisel/actions/workflows/rust.yml)

A macOS menu bar app that rewrites selected text using Claude AI. Runs silently in the background — select text, press a hotkey, get it back rewritten.

## How it works

1. Select text in any app — Slack, Gmail, Notes, Terminal, anywhere
2. Press **Cmd+Option+R**
3. The selected text is rewritten in your chosen tone and pasted back automatically

## Tones

Switch tone anytime from the menu bar icon:

| Tone | Style |
|------|-------|
| Professional | Neutral and clear, polished without being authoritative |
| Polite | Soft, respectful, non-confrontational |
| Assertive | Firm and direct, highlights impact |
| Concise | Minimal words, straight to the point |
| Gen Z | Casual, internet-native, contemporary phrasing |

## Requirements

- macOS
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Anthropic API key ([get one here](https://console.anthropic.com/))
- Accessibility permissions (for Cmd+C / Cmd+V simulation)

## Setup

```bash
git clone https://github.com/c0d3-k1ra/text-chisel
cd text-chisel
cargo run
```

On first launch, the Settings window opens automatically. Enter your Anthropic API key and click **Save**.

macOS will also prompt for Accessibility access — approve it in **System Settings → Privacy & Security → Accessibility**.

## Settings

Click the menu bar icon → **Settings** to configure:

- **Anthropic API Key** — your `sk-ant-...` key
- **Model** — Haiku 4.5 (fast) or Sonnet 4.6 (quality)

Settings are saved to `~/.config/text-chisel/config.toml`.

## Build

```bash
cargo build --release
```

The binary is at `target/release/text-chisel`. Copy it anywhere and run it — no installation needed.

## Logging

Run with `RUST_LOG=debug` to see detailed logs including clipboard contents and API responses:

```bash
RUST_LOG=debug cargo run
```

Default log level is `info`.

## Project structure

```
src/
├── main.rs            # entry point, event loop
├── clipboard.rs       # copy selection, paste text
├── hotkey.rs          # global hotkey registration
├── rewrite.rs         # Claude API call
├── prompts.rs         # system and user prompt templates
├── tray.rs            # menu bar icon and menu
├── settings_window.rs # settings UI (wry webview)
└── config.rs          # config file load/save
assets/
├── icon.svg           # menu bar icon
└── settings.html      # settings window UI
```
