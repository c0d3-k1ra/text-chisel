# text-chisel

A macOS menu bar app written in Rust. Select any text, press `Cmd+Option+R`, and it rewrites it using the Claude API and pastes the result back in place. Runs as a background process with no Dock icon and no window.

## Collaboration style

- User writes the code, Claude is a guide and pair programmer
- Don't rewrite large chunks unprompted — answer questions, review code, point out issues
- Always read the relevant file before commenting on it
- Always ask before committing

## What works today

- `Cmd+Option+R` → copies selected text → calls Claude API → pastes rewritten text back
- Five tones in a submenu ("Tone: Professional ▶") — Professional, Polite, Assertive, Concise, Gen Z
- Connection status indicator in the menu (🟢/🔴/⏳) — checked on startup and after every save
- Launch at Login — menu item writes a LaunchAgent plist, toggles on/off
- Settings window (wry webview) — API key, model selector, test connection button; form resets to saved values on every open
- Config persisted to `~/.config/text-chisel/config.toml`
- `.app` bundle built with `cargo bundle --release`
- No Dock icon via `LSUIElement` + `NSApplicationActivationPolicyAccessory`
- Structured logging via `log`/`env_logger` — `RUST_LOG=debug` for verbose output
- macOS notifications with sound (Basso) on every error, with user-friendly messages per error type
- Per-tone prompts with few-shot examples baked in for all five tones
- Unit and manual tests across all testable modules (46 passing, 6 ignored)
- Git hooks in `.githooks/` — fmt + clippy on commit, tests on push
- GitHub Actions CI — build → format + test → release on tag

## Project structure

```text
src/
├── main.rs            # entry point, tao event loop, wires all modules together
├── clipboard.rs       # Cmd+C to copy selection, Cmd+V to paste, restores clipboard after
├── hotkey.rs          # registers Cmd+Option+R globally, sends events via mpsc channel
├── rewrite.rs         # Claude API call — validate, build_request, call_api, parse_response
├── prompts.rs         # system prompt, per-tone instructions, and few-shot examples
├── tray.rs            # menu bar icon, tone submenu, status item, launch-at-login toggle
├── settings_window.rs # Settings UI via wry webview, IPC for save/test/cancel actions
├── login_item.rs      # LaunchAgent plist — write_plist, remove_plist, enable, disable
└── config.rs          # load/save ~/.config/text-chisel/config.toml
assets/
├── icon.svg           # source icon (rendered at runtime via resvg)
├── icon.icns          # compiled macOS icon for the .app bundle
├── icon.iconset/      # PNG sizes used to build icon.icns
├── Info.plist.ext     # extra plist keys: LSUIElement, NSAccessibilityUsageDescription
└── settings.html      # Settings window HTML/CSS/JS
```

## Key crates

- `tao` — event loop, keeps CFRunLoop alive for global hotkey and tray to work
- `tray-icon` — menu bar icon and menu
- `wry` — webview for Settings window
- `global-hotkey` — system-wide hotkey registration
- `enigo` — keyboard simulation for Cmd+C and Cmd+V
- `arboard` — clipboard read and write
- `reqwest` — HTTP client for Claude API calls
- `tokio` — async runtime (used for API calls only)
- `resvg` — renders SVG icon to RGBA pixels for the tray
- `objc` — calls `setActivationPolicy:` to hide from Dock; sets up NSMenu for WKWebView keyboard shortcuts
- `serde` + `toml` — config serialization
- `anyhow` — error propagation
- `log` + `env_logger` — structured logging
- `dotenvy` — loads `.env` as a fallback source for `ANTHROPIC_API_KEY`
- `once_cell` — lazy static HTTP client

## macOS gotchas worth knowing

1. **Clipboard settle time** — sleep 100ms after Cmd+C before reading (`COPY_SETTLE_MS`), and after writing clipboard before Cmd+V (`PASTE_SETTLE_MS`). Without this the clipboard isn't populated yet.
2. **Clipboard restore** — `paste_text()` saves the original clipboard contents and restores them after pasting. Uses `simulate_paste()` so restore always runs even if paste simulation fails mid-way.
3. **Accessibility permission** — `enigo` needs Accessibility access to simulate Cmd+C and Cmd+V. Without it both clipboard operations fail silently.
4. **`Modifiers::SUPER` not `META`** — SUPER is the cross-platform name for the Cmd key in `global-hotkey`.
5. **Dock icon reappears** — `tao` resets the activation policy during event loop init. The `setActivationPolicy:1` call must happen inside `Event::NewEvents(StartCause::Init)`, not before the loop starts.
6. **`set_var` is unsafe** — `std::env::set_var` is called before `hotkey::run()` spawns threads. This is intentional and safe only because it happens before any threads are spawned that read those vars.
7. **`Box::leak` in hotkey.rs** — `GlobalHotKeyManager` must stay alive for the hotkey to remain registered. Leaking it is intentional; it lives for the whole process lifetime.
8. **WKWebView keyboard shortcuts** — apps without a standard menu bar need an NSMenu with Edit items (`cut:`, `copy:`, `paste:`, `selectAll:`) for Cmd+C/V/X/A to work in webviews. Set up in `setup_edit_menu()` called from `StartCause::Init`.
9. **Launch at Login** — intentionally does NOT call `launchctl bootstrap` after writing the plist. Doing so with `RunAtLoad=true` spawns a second instance immediately. The plist in `~/Library/LaunchAgents/` is sufficient for macOS to pick it up at next login.

## Hotkey

`Cmd+Option+R` — registered as `Modifiers::SUPER | Modifiers::ALT` + `Code::KeyR`

## Config file

```toml
# ~/.config/text-chisel/config.toml
api_key = "sk-ant-..."
model   = "claude-haiku-4-5-20251001"
```

## Claude API

- Default model: `claude-haiku-4-5-20251001` (fast, cheap). Override with `claude-sonnet-4-6` for better quality.
- API key loaded from config, falls back to `ANTHROPIC_API_KEY` env var or `.env` file.
- Endpoint: `https://api.anthropic.com/v1/messages`
- `rewrite_with_key()` exists separately for the Settings test connection button and the startup connection check — it bypasses the env var so it uses a key directly.

## Error notifications

On any failure, a macOS notification fires with sound (`Basso`) via `osascript display notification`. Messages are human-readable, not raw error strings. Notification style must be set to Banners or Alerts for Script Editor in System Settings > Notifications — otherwise they arrive silently.

Mapped messages:

- Nothing selected → "Select some text first, then press ⌘⌥R."
- Accessibility denied → guide to System Settings > Privacy > Accessibility
- No API key → "Add your Anthropic API key in Settings to get started."
- 401 → "API key not accepted. Open Settings to update it."
- Text too long (>8,000 chars) → "Selection is too long. Try again with under 8,000 characters."
- 429 rate limit → "Too many requests. Wait a moment and try again."
- 529 overloaded → "Claude is busy right now. Give it a moment and try again."
- Timeout → "Claude took too long to respond. Try again in a moment."
- Paste failed → guide to check Accessibility access

## Testing

- Pure logic tests are inline `#[cfg(test)]` blocks in each module file. No separate test files.
- Tests that need hardware (clipboard, keyboard, real API) are marked `#[ignore = "reason"]` and run manually with `cargo test -- --ignored`.
- Covered: `rewrite`, `config`, `prompts`, `login_item` (xml_escape, write_plist, remove_plist), `settings_window` (build_html, handle_ipc, config_from_save_payload), `tray` (TONES constant), `hotkey` (modifiers).

```bash
cargo test                   # run all non-ignored tests
cargo test -- --ignored      # run hardware-dependent tests manually
```

## Building

```bash
cargo run                    # dev run
RUST_LOG=debug cargo run     # verbose logging
cargo bundle --release       # build .app bundle
```

## Releasing

Use the `/release` skill — it handles changelog, version bump, signed commit, tag, and push automatically. Or do it manually:

```bash
git add Cargo.toml CHANGELOG.md
git commit -S -m "release version x.y.z"   # signed, no co-author
git tag -a vx.y.z -m "release vx.y.z"
git push origin main vx.y.z
```

CI verifies the tag matches the version, builds the `.app`, and creates a GitHub Release with auto-generated notes.

## Agent skills

Skills live in `.agents/skills/` and work with Claude Code (via `SKILL.md`) and Codex-style agents (via `AGENTS.md`).

| Skill | Invoke | Purpose |
| --- | --- | --- |
| `text-chisel-review` | `/text-chisel-review` | Full code review against project and Rust rules |
| `rust-skills` | `/rust-skills` | 179 Rust best-practice rules across 14 categories |
| `release` | `/release [patch\|minor\|major]` | Changelog, version bump, signed commit, tag, push |
