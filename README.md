# text-chisel

A macOS menu bar app that rewrites selected text using Claude AI.

## How it works

1. Select text in any app (Slack, Gmail, Notes, anywhere)
2. Press **Cmd+Option+R**
3. Pick a tone from the overlay: Professional · Casual · Concise · Friendly · Formal · Fix Grammar
4. The rewritten text is pasted back automatically

## Requirements

- macOS
- Accessibility permissions (for simulating Cmd+C and Cmd+V)
- Anthropic API key

## Setup

```bash
export ANTHROPIC_API_KEY=sk-ant-...
cargo run
```

On first run, macOS will prompt for Accessibility access. Approve it in **System Settings → Privacy & Security → Accessibility**.

## Build

```bash
cargo build --release
```

## Status

Work in progress — core flow not yet complete.
