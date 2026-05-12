---
name: text-chisel-review
description: >
  Code review skill for the text-chisel project. Reviews Rust source files
  against project-specific rules from CLAUDE.md and the rust-skills ruleset.
  Checks error handling, macOS gotchas, notification messages, config safety,
  unsafe usage, test coverage, and code quality. Invoke with /text-chisel-review.
metadata:
  author: text-chisel
  version: "1.0"
---

# text-chisel Code Review

Review the codebase at `src/` against the rules below. Read every source file
before commenting. Report findings grouped by severity: **CRITICAL**, **WARN**,
**SUGGESTION**. End with a summary scorecard.

---

## 1. Project Rules (from CLAUDE.md)

These are non-negotiable for this codebase.

### Unsafe usage

- Every `unsafe` block must have a comment explaining why it is safe.
- The only sanctioned unsafe patterns are:
  - `std::env::set_var` called before threads are spawned (`main.rs`)
  - `objc` macros (`msg_send!`, `class!`) inside `Event::NewEvents(StartCause::Init)`
- Any `unsafe` outside these two sites is a **CRITICAL** finding.

### macOS timing gotchas

- `clipboard::get_selected_text` must sleep `COPY_SETTLE_MS` after `Cmd+C` before reading.
- `clipboard::paste_text` must sleep `PASTE_SETTLE_MS` after setting clipboard before `Cmd+V`.
- Removing or bypassing these sleeps without justification is **CRITICAL**.

### Dock icon suppression

- `setActivationPolicy:1` must be called inside `Event::NewEvents(StartCause::Init)`, not before the event loop starts.
- Calling it anywhere else means the Dock icon will reappear — flag as **CRITICAL**.

### `Box::leak` in hotkey.rs

- `GlobalHotKeyManager` is intentionally leaked to keep the hotkey registered for process lifetime.
- This is correct; do not flag it as a bug.

### Error messages in notifications

- Every `notify_error(...)` call must pass a human-readable string — no raw error display, no `format!("... {e}")` for known error variants.
- The message must not contain em dashes (`—`).
- Review all call sites in `main.rs::handle_hotkey`. Flag any raw or technical message as **WARN**.

### Config safety

- `api_key` must never be logged at any level.
- `config::save` must not be called with an empty `api_key` unless explicitly clearing.
- Check `settings_window.rs` IPC save handler.

### Thread safety for env vars

- `std::env::set_var` must only be called before `hotkey::run()` spawns threads, or inside `settings_window.rs` (single threaded IPC handler).
- Any `set_var` call after thread spawn in a concurrent context is **CRITICAL**.

---

## 2. Rust Rules to Apply (from rust-skills)

Apply these in priority order. Only report violations that are actually present in the code.

### CRITICAL priority

- `err-no-unwrap-prod` — No `.unwrap()` in production paths. Allowed only in `#[cfg(test)]` blocks, `Lazy::new`, and infallible post-check positions.
- `err-context-chain` — Errors passed to the caller with `?` should have `.context()` if the site adds meaningful information.
- `own-arc-shared` — `Arc<Mutex<T>>` is correct for shared mutable state across threads. Verify `selected_tone` usage.
- `async-no-lock-await` — No `Mutex` held across an `.await` point.
- `anti-empty-catch` — No silent `let _ = ...` on `Result` without a comment explaining why the error is intentionally discarded.

### HIGH priority

- `err-anyhow-app` — This is an application, not a library. `anyhow::Result` throughout is correct; flag any `Box<dyn Error>` usage.
- `api-must-use` — Public functions returning `Result` should be `#[must_use]` or their return value must be used at every call site.
- `type-no-stringly` — Error matching in `handle_hotkey` uses string matching on error messages (`.contains("401")`, etc.). Note this as a **SUGGESTION** to use typed errors eventually, but it is acceptable given the current architecture.
- `anti-clone-excessive` — Flag unnecessary `.clone()` calls, especially on `String` and `Config`.

### MEDIUM priority

- `test-cfg-test-module` — All test modules must use `#[cfg(test)] mod tests { use super::*; }`.
- `test-descriptive-names` — Test function names should describe the scenario, not just the function name.
- `name-consts-screaming` — Module-level constants (`COPY_SETTLE_MS`, `MAX_INPUT_CHARS`, etc.) must be `SCREAMING_SNAKE_CASE`.
- `proj-pub-crate-internal` — Internal helpers should use `pub(crate)`, not `pub`. Verify `load_from`/`save_to` in `config.rs`.
- `anti-string-for-str` — Function parameters that only need to read a string should take `&str`, not `&String`.

### LOW / SUGGESTION

- `proj-flat-small` — Small binary crate; flat module structure is correct, do not suggest splitting into lib.rs.
- `lint-unsafe-doc` — Each `unsafe` block should be documented with `// SAFETY:` comment.
- `mem-avoid-format` — In `notify_error`, the intermediate `format!` calls in `handle_hotkey` before passing to `notify_error` are fine; flag only hot-path misuse.

---

## 3. Review Checklist

Work through each file in order. For each, check the applicable rules.

### `src/main.rs`
- [ ] `unsafe` blocks have `// SAFETY:` comments
- [ ] `set_var` calls are before thread spawn
- [ ] `setActivationPolicy` is inside `StartCause::Init`
- [ ] All `notify_error` messages are user-friendly, no em dashes
- [ ] No raw `format!("{e}")` passed to `notify_error` for known error variants
- [ ] `handle_hotkey` returns early on error (no silent fallthrough)
- [ ] `Box::leak` in hotkey is not flagged

### `src/rewrite.rs`
- [ ] No `.unwrap()` outside test module
- [ ] `validate`, `build_request`, `parse_response` are private (not `pub`)
- [ ] `call_api` error includes HTTP status code
- [ ] Test module uses `use super::*;` and covers all three pure functions

### `src/config.rs`
- [ ] `load_from` / `save_to` are `pub(crate)`, not `pub`
- [ ] `api_key` is never logged
- [ ] `load()` falls back to `Config::default()` on any parse error, not panic
- [ ] Test module covers round-trip, missing file, malformed TOML

### `src/clipboard.rs`
- [ ] `COPY_SETTLE_MS` sleep is present and positioned correctly
- [ ] `PASTE_SETTLE_MS` sleep is present and positioned correctly
- [ ] Original clipboard is restored after paste, even on early return paths
- [ ] Hardware-dependent tests are `#[ignore]`

### `src/hotkey.rs`
- [ ] `Box::leak` is intentional and documented
- [ ] Hotkey uses `Modifiers::SUPER | Modifiers::ALT` (not `META`)
- [ ] Test covers correct modifiers and key code

### `src/tray.rs`
- [ ] `set_tone` correctly updates checkmarks
- [ ] `_icon` is kept alive (held in struct, not dropped)

### `src/settings_window.rs`
- [ ] IPC handler validates input before saving (no empty key saved silently)
- [ ] `api_key` not logged at any level in save or test handlers
- [ ] `set_var` call in save handler is acceptable (main thread IPC callback)

### `src/prompts.rs`
- [ ] `user()` function includes tone and text in the expected format
- [ ] No hardcoded model or key references

---

## 4. Output Format

Report findings as:

```
[CRITICAL] src/file.rs:NN — description of issue (rule: rule-id)
[WARN]     src/file.rs:NN — description of issue (rule: rule-id)
[SUGGEST]  src/file.rs:NN — description of issue (rule: rule-id)
```

End with a scorecard:

```
## Scorecard
CRITICAL: N
WARN:     N
SUGGEST:  N
Overall:  PASS / NEEDS WORK / FAIL
```

PASS = 0 critical, ≤3 warn.
NEEDS WORK = 0 critical, >3 warn OR ≥1 suggestions worth acting on.
FAIL = any critical finding.
