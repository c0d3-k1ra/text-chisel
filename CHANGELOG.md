# Changelog

All notable changes to this project will be documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [1.0.0] - 2026-05-07

### Added

- Global hotkey **⌘⌥R** rewrites selected text in any app and pastes it back automatically
- Five tones selectable from the menu bar: Professional, Polite, Assertive, Concise, Gen Z
- Settings window (API key, model selector, test connection button)
- Config persisted to `~/.config/text-chisel/config.toml`
- macOS notifications with sound (Basso) on every error — user-friendly messages for all known failure cases (missing key, 401, rate limit, timeout, accessibility denied, and more)
- No Dock icon — runs silently as a background process
- `.app` bundle via `cargo bundle --release`
