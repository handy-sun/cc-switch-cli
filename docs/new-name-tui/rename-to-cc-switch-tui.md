# Rename cc-switch-cli to cc-switch-tui

## Motivation

"cli" is misleading — this is a TUI application. "tui" better describes what it is.
Also avoids name collision with the upstream `cc-switch-cli` repo.

## Scope: Two Commits

Commit 1 (core rename) + Commit 2 (docs).

---

## Commit 1: Core Rename

### src-tauri/Cargo.toml

- [x] `name = "cc-switch"` → `name = "cc-switch-tui"`
- [x] All `[[bin]]` entries: `name = "cc-switch"` → `name = "cc-switch-tui"`
- [x] `repository` URL: `SaladDay/cc-switch-cli` → `handy-sun/cc-switch-tui`
- [x] (Keep `lib.name = "cc_switch_lib"` unchanged — avoids mass import churn)

### src-tauri/src/config.rs — default config directory

- [x] `get_app_config_dir()`: `.cc-switch` → `.cc-switch-tui`
  - Isolates from upstream GUI `cc-switch` project (same DB file conflict)
  - `CC_SWITCH_CONFIG_DIR` env var override still works as before

### .github/workflows/release.yml

- [x] Artifact directory names: `cc-switch-cli-*` → `cc-switch-tui-*`
- [x] Release asset filenames: `cc-switch-cli-*` → `cc-switch-tui-*`
- [x] Binary reference: `cc-switch.exe` → `cc-switch-tui.exe`
- [x] Release title: `cc-switch-cli` → `cc-switch-tui`
- [x] Repo URL: `SaladDay/cc-switch-cli` → `handy-sun/cc-switch-tui`

### .github/workflows/rust-ci.yml

- [x] Artifact name: `cc-switch-${{ matrix.target }}` → `cc-switch-tui-${{ matrix.target }}`

### install.sh

- [x] `REPO="SaladDay/cc-switch-cli"` → `REPO="handy-sun/cc-switch-tui"`
- [x] All asset name patterns: `cc-switch-cli-*` → `cc-switch-tui-*`
- [x] Binary path references

### scripts/generate_latest_json.py

- [x] Asset filename patterns: `cc-switch-cli-*` → `cc-switch-tui-*`

### flake.nix (both)

- [x] `flake.nix`: package name, description references
- [x] `src-tauri/flake.nix`: same

### src-tauri/src/cli/commands/update.rs

- [x] All asset name strings: `cc-switch-cli-*` → `cc-switch-tui-*`
- [x] `tagged_asset_name()`: strip_prefix + format strings

### src-tauri/src/cli/commands/update/tests.rs

- [x] All asset name strings: `cc-switch-cli-*` → `cc-switch-tui-*`
- [x] Repo URLs: `saladday/cc-switch-cli` → `handy-sun/cc-switch-tui`

### src-tauri/tests/install_script.rs

- [x] All asset name strings: `cc-switch-cli-*` → `cc-switch-tui-*`

### Tests (rebuild verification)

- [ ] Fix `CARGO_BIN_EXE_cc-switch` → `CARGO_BIN_EXE_cc-switch-tui` (separate task)

---

## Commit 2: Documentation

### README.md + README_ZH.md

- [x] All `cc-switch-cli` references → `cc-switch-tui`
- [x] All `cc-switch` binary command examples → `cc-switch-tui`
- [x] Install URLs: `SaladDay/cc-switch-cli` → `handy-sun/cc-switch-tui`
- [x] Asset filename patterns

### CHANGELOG.md

- [x] Headline `cc-switch-cli` → `cc-switch-tui`
- [ ] Add rename entry for next version

### AGENTS.md / CLAUDE.md (if they exist)

- [x] Project references (check what's current)

---

## NOT Changed

| What | Why |
|------|-----|
| `cc_switch_lib` crate name | Would touch every `use cc_switch_lib::...` line — massive diff, zero user-facing impact |
| `com.ccswitch.desktop` in tauri.conf.json | GUI-only; this project dropped Tauri GUI |
| Rust source code `"cc-switch"` string refs | Only if they're binary-name paths in docs/help text |
| `docs/plans/`, `docs/design/`, `docs/superpowers/` | Historical documents — keep as-is for traceability |
| `provider_templates.rs` PackyAPI promo code | Third-party registration code — must not change |

---

## GitHub Repo

After commits are pushed: **rename the repo on GitHub** from `handy-sun/cc-switch-cli` to `handy-sun/cc-switch-tui`. GitHub auto-redirects old URLs.

---

## Breaking Change

Users who run `cc-switch` from PATH will need to use `cc-switch-tui` instead.
Users who have data in `~/.cc-switch/` will start with a fresh `~/.cc-switch-tui/`.

## Auto-Migration

On first run, if `~/.cc-switch/` exists but `~/.cc-switch-tui/` doesn't:
- Copy `cc-switch.db`, `settings.json`, `skills/`, `backups/` to new directory
- Print a message telling the user what was migrated
- Keep the old directory untouched (no delete)

Implementation point: `src-tauri/src/config.rs` `get_app_config_dir()` — after determining the config dir path, check for legacy dir and do a one-shot copy.
