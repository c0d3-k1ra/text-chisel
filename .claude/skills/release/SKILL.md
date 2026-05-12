---
name: release
description: >
  Automates the text-chisel release process. Reads all commits since the last
  git tag, writes a proper changelog entry, bumps the version in Cargo.toml,
  creates a signed commit (no co-author), tags, and pushes. Invoke with
  /release [patch|minor|major].
metadata:
  author: text-chisel
  version: "1.0"
argument-hint: "[patch|minor|major]"
---

# text-chisel Release

## Current state

Last release tag: `!`git describe --tags --abbrev=0 2>/dev/null || echo "none"``

Current version (Cargo.toml): `!`grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'``

Commits since last release:
`!`{ LAST=$(git describe --tags --abbrev=0 2>/dev/null); [ -n "$LAST" ] && git log "${LAST}..HEAD" --format="  %s (%h)" --reverse || git log --format="  %s (%h)" --reverse; }``

---

## Steps

### 1. Determine the version bump

If `$ARGUMENTS` is `patch`, `minor`, or `major`, use that. Otherwise, infer from
the commits above:
- Any breaking change or removal → **major**
- Any new feature or user-visible addition → **minor**
- Bug fixes, refactors, docs, chores only → **patch**

Show the user the inferred bump and the new version before proceeding. Wait for
confirmation if the bump was inferred (not passed explicitly).

### 2. Compute the new version

Split the current version on `.` into MAJOR, MINOR, PATCH. Apply the bump:
- `patch` → PATCH + 1, MINOR and MAJOR unchanged
- `minor` → MINOR + 1, PATCH resets to 0, MAJOR unchanged
- `major` → MAJOR + 1, MINOR and PATCH reset to 0

### 3. Write the changelog entry

Group the commits into sections (only include non-empty sections):

```
## vX.Y.Z — YYYY-MM-DD

### Added
- …

### Fixed
- …

### Changed
- …

### Internal
- …
```

Rewrite raw commit subjects into clear, user-facing sentences. Drop merge
commits and chore commits that have no user impact (e.g. `cargo fmt`,
`clippy fix`). Keep internal/refactor items under **Internal**.

If a `CHANGELOG.md` exists, prepend the new entry at the top (after any title
line). If it does not exist, create it with just the new entry.

### 4. Bump the version in Cargo.toml

Edit `Cargo.toml` — change the `version = "…"` line in `[package]` to the new
version. Do not touch any other line.

### 5. Verify

Run:
```bash
cargo check --quiet 2>&1 | head -20
```

If it fails, stop and report the error. Do not proceed.

### 6. Stage and commit (signed, no co-author)

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
```

Commit with a signed commit. **Do not add a Co-Authored-By line.**

```bash
git commit -S -m "release version X.Y.Z"
```

If signing fails (e.g. GPG/SSH key not configured), stop and tell the user what
to fix — do not fall back to an unsigned commit.

### 7. Tag

Create an annotated tag:

```bash
git tag -a vX.Y.Z -m "release vX.Y.Z"
```

### 8. Push

```bash
git push origin main vX.Y.Z
```

Confirm success by showing the pushed commit and tag.

---

## Rules

- Never add `Co-Authored-By` to the commit message.
- Always use `git commit -S` — abort if signing fails.
- Never push without the tag.
- Do not amend existing commits.
- The version in `Cargo.toml` and the tag must match exactly (e.g. `1.3.0` and `v1.3.0`).
