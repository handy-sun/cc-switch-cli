# cc-switch-tui Migration Logic

Last updated: 2026-05-11

This document records the migration behavior that affects local configuration,
SQLite state, and WebDAV sync data. It is intended as implementation context for
future changes; use source code as the final authority when behavior changes.

## Important Terms

- App config directory: resolved by `get_app_config_dir()` in
  `src-tauri/src/config.rs`.
- Do not assume the app config directory is `~/.cc-switch-tui`.
- `CC_SWITCH_TUI_CONFIG_DIR` takes priority when set.
- `CC_SWITCH_CONFIG_DIR` is the deprecated legacy override and still works.
- Default app config directory is `$HOME/.cc-switch-tui`.
- Legacy app config directory is `$HOME/.cc-switch`.
- WebDAV `V1` and `V2` are sync protocol/data-format versions, not
  cc-switch-tui software versions.

## Config Directory Migration

Source: `src-tauri/src/config.rs`

The directory migration moves data from the legacy default directory
`$HOME/.cc-switch` to the active app config directory.

Trigger:

- `get_app_config_dir()` runs `migrate_legacy_config_dir_if_needed()` once per
  process when no config-dir env override forces a different legacy path.
- The migration can also be checked via `legacy_config_migration_paths()` and
  `check_legacy_config_dir_migration_needed()`.
- Users can skip it with `skip_legacy_config_dir_migration()`.

Target selection:

- If `CC_SWITCH_TUI_CONFIG_DIR` is set, that is the target.
- If deprecated `CC_SWITCH_CONFIG_DIR` is set, automatic legacy migration is not
  attempted because the old env var already points at an explicit config dir.
- Otherwise target is `$HOME/.cc-switch-tui`.

Migration guard:

- Source must be `$HOME/.cc-switch`, must exist, must be a directory, and must
  contain at least one entry.
- Source and target must not be the same path.
- Target must either not exist or exist as an empty directory; if it only
  contains an early-created `cc-switch.db`, migration is still allowed so legacy
  JSON config can be copied.
- Target must not contain `.migrated-from-cc-switch`. If a program-written
  success marker already exists but the target is missing legacy `settings.json`
  or `config.json`, startup may silently repair the missing JSON copy.

Copied data:

- Non-symlink files are copied without overwriting existing target files.
- Non-symlink directories are copied recursively.
- `backups/` is special-cased: only the three most recent non-symlink entries
  are copied.
- The old directory is preserved; it is never deleted by this migration.

Marker:

- On successful migration, the target directory receives
  `.migrated-from-cc-switch`.
- The marker text records the source path and timestamp.
- If the user skips migration, the same marker path is written with
  `User declined migration`.
- The marker lives in the target directory, not the old directory.

Failure behavior:

- Migration failures are logged to stderr and do not block startup.
- Because the old directory is preserved, failure should not destroy existing
  data.

## App State Startup Migrations

Source: `src-tauri/src/store.rs`

Most local data migrations happen through `AppState::try_new()`.

Paths:

- `cc-switch.db`, `config.json`, and `skills.json` are all resolved under the
  active app config directory from `get_app_config_dir()`.

If `cc-switch.db` already exists:

- The DB is opened through `Database::init()`.
- Runtime state is exported from SQLite into `MultiAppConfig`.
- Legacy Codex provider config normalization runs.
- Common-config upstream semantics migration runs if needed.
- Legacy `config.json` is not imported again.

If `cc-switch.db` does not exist:

- Existing `config.json` is validated before creating the DB.
- Existing `skills.json` is validated before creating the DB.
- `Database::init()` creates the SQLite database.
- If legacy `config.json` exists, `Database::migrate_from_json()` imports it.
- Imported `config.json` is archived as `config.json.migrated`.
- If legacy `skills.json` exists, its sync method, repos, installed-skill rows,
  and SSOT pending flag are imported.
- Imported `skills.json` is archived as `skills.json.migrated`.
- Default skill repos are inserted if missing.
- Runtime state is exported from DB into `MultiAppConfig`.
- Legacy Codex provider config normalization runs.
- Common-config upstream semantics migration runs if needed.

## `config.json` to SQLite Migration

Source: `src-tauri/src/database/migration.rs`

`Database::migrate_from_json()` imports old `MultiAppConfig` data into SQLite.

Imported data:

- Providers, including current-provider flags.
- Provider endpoints from provider metadata.
- MCP servers and their per-app enabled flags.
- Prompts for Claude, Codex, and Gemini.
- Skill repos.
- Common config snippets.

Not directly imported:

- Legacy `skills.skills` install state is not imported from `config.json`
  because it is not complete enough to guarantee SSOT consistency. Skill
  recovery is handled by the skill service scan/import paths and by the
  `skills_ssot_migration_pending` flow.

## SQLite Schema Migrations

Source: `src-tauri/src/database/schema.rs`

`Database::init()` applies DB schema migrations on open/create.

Important side effect:

- The DB schema migration from v2 to v3 sets the DB setting
  `skills_ssot_migration_pending=true`, so the skill service can perform the
  SSOT migration later.

## Common Config Upstream Semantics Migration

Source: `src-tauri/src/services/provider/common_config.rs`

This is a one-time DB-backed migration for provider common-config behavior.

Marker:

- DB setting key: `common_config_upstream_semantics_migrated_v1`
- When the setting is `true`, the migration is skipped.

Behavior:

- Applies only to Claude, Codex, and Gemini.
- If an app has a non-empty common config snippet, providers without an explicit
  `meta.apply_common_config` value are treated as `true`.
- Provider stored settings are normalized so provider-specific storage does not
  redundantly embed common config.
- Updated providers are saved back to SQLite.

Failure behavior:

- Errors are returned to startup callers. This is not a best-effort background
  migration.

## Skills SSOT Migration

Source: `src-tauri/src/services/skill.rs`

The skill system uses a single source-of-truth directory under the active app
config directory plus DB metadata.

Pending flag:

- DB setting key: `skills_ssot_migration_pending`

Trigger:

- `SkillService::load_index()` reads the pending flag.
- `SkillService::migrate_ssot_if_pending()` performs the migration when the flag
  is set.

Behavior:

- If the DB already has managed skill records, migration only backfills SSOT
  directories for those managed records where possible.
- If there are no managed skill records, migration scans supported app skill
  directories and copies discovered skills into SSOT.
- After migration or best-effort backfill, the pending flag is cleared.

Safety rule:

- When managed records already exist, the migration does not automatically claim
  every app-local skill directory as managed. This avoids surprising users by
  taking ownership of directories that were not previously managed by
  cc-switch-tui.

## Claude MCP Override Migration

Source: `src-tauri/src/claude_mcp.rs`

This migration handles Claude MCP JSON path changes when the Claude config
directory is overridden.

Trigger:

- `user_config_path()` calls `ensure_mcp_override_migrated()`.

Behavior:

- If no Claude config override is set, no migration runs.
- If the new derived MCP path already exists, no migration runs.
- If default `$HOME/.claude.json` exists and the derived override path is
  missing, the default file is copied to the derived path.

Failure behavior:

- Directory creation or copy errors are logged as warnings and do not panic.

## WebDAV Sync Protocol Versions

Source: `src-tauri/src/services/webdav_sync/mod.rs`

WebDAV `V1` and `V2` describe remote sync data formats, not app release
versions.

### V1 Remote Layout

Path shape:

- `{remote_root}/v1/{profile}/`

Manifest:

- `manifest.json`
- `format = "cc-switch-webdav-sync"`
- `version = 1`
- artifacts:
  - `dbSql`
  - `skillsZip`
  - `settingsSync`

The default file name for the legacy settings artifact is
`settings.sync.json`, but the V1 manifest artifact path is authoritative. If the
manifest says `settings-sync.json`, that path is used.

### V2 Remote Layout

Current path shape:

- `{remote_root}/v2/db-v6/{profile}/`

Legacy V2 fallback path shape:

- `{remote_root}/v2/{profile}/`

Manifest:

- `manifest.json`
- `format = "cc-switch-webdav-sync"`
- `version = 2`
- `dbCompatVersion = 6` for current layout
- artifacts:
  - `db.sql`
  - `skills.zip`
  - `settings.json`

The manifest is uploaded last. Artifacts are uploaded first so a visible
manifest only points at data that should already be present.

## WebDAV Upload

Source: `src-tauri/src/services/webdav_sync/mod.rs`

Trigger:

- CLI/TUI upload action calls `WebDavSyncService::upload()`.
- V1 to V2 migration also calls upload after applying V1 data locally.

Behavior:

- Reads current WebDAV settings from `settings.json` through
  `get_webdav_sync_settings()`.
- Ensures the current V2 remote directory exists.
- Builds a local snapshot:
  - exports SQLite sync SQL as `db.sql`
  - zips skill SSOT as `skills.zip`
  - serializes current app settings as `settings.json`
  - builds a manifest with artifact hashes and sizes
- Uploads artifacts:
  - `db.sql`
  - `skills.zip`
  - `settings.json`
  - `manifest.json` last
- Reads back the manifest and verifies bytes match.
- Fetches manifest ETag best-effort.
- Persists sync success status best-effort.
- Cleans up V1 remote data best-effort after successful upload.

## WebDAV Download

Source: `src-tauri/src/services/webdav_sync/mod.rs`

Trigger:

- CLI/TUI download action calls `WebDavSyncService::download()`.

Behavior:

- Looks for a V2 snapshot in current layout first.
- Falls back to legacy V2 layout.
- If no V2 data exists but a V1 manifest is detected, returns
  `SyncDecision::V1MigrationNeeded` so UI can ask for confirmation.
- Validates protocol format, protocol version, and DB compatibility.
- Downloads and verifies required artifacts:
  - `db.sql`
  - `skills.zip`
- Downloads and verifies `settings.json` when the manifest contains it.
- Acquires the restore mutation guard.
- Refuses restore when proxy runtime or takeover state makes restore unsafe.
- Applies DB and skills as one restore unit:
  - backs up current skills
  - restores skills zip
  - imports SQL into SQLite
  - rolls back skills if DB import fails
- Applies downloaded settings if present, but preserves the current local
  WebDAV connection settings. This prevents a restore from replacing the
  WebDAV URL/credentials used to perform the restore.
- Persists sync success status best-effort.
- Cleans up V1 remote data best-effort.

## WebDAV V1 to V2 Migration

Source: `src-tauri/src/services/webdav_sync/mod.rs`

Trigger:

- CLI: `config webdav migrate-v1-to-v2`
- TUI: user confirms the V1 migration prompt.
- Programmatic entry: `WebDavSyncService::migrate_v1_to_v2()`

Behavior:

1. Load local WebDAV connection settings.
2. Detect and download the V1 manifest.
3. Refuse migration if restore is unsafe, for example local proxy takeover is
   active.
4. Download and verify V1 artifacts:
   - `dbSql`
   - `skillsZip`
   - `settingsSync`
5. Parse V1 `settingsSync` into syncable app settings:
   - language
   - skill sync method
   - security settings
   - Claude custom endpoints
   - Codex custom endpoints
6. Acquire the restore mutation guard.
7. Apply DB and skills snapshot locally.
8. Apply V1 syncable settings locally while preserving the current WebDAV
   connection settings.
9. Drop the restore guard.
10. Upload the local state as V2:
    - `db.sql`
    - `skills.zip`
    - `settings.json`
    - `manifest.json`
11. Cleanup of the old V1 remote directory is best-effort.

Important nuance:

- V1 `settingsSync` is not itself uploaded as a V2 artifact.
- Instead, it is applied to local `settings.json`, then current app settings are
  serialized and uploaded as V2 `settings.json`.

## WebDAV Restore Safety

Source: `src-tauri/src/services/webdav_sync/mod.rs`

Download and V1 migration both call restore safety checks before mutating local
state.

Restore is refused when:

- The managed proxy runtime is running.
- Any proxy takeover state is active.

Reason:

- Restoring DB/live state while proxy takeover is active can leave live client
  config and DB state out of sync.

## Current Implementation Gotchas

- `settings.json` is app settings, not Claude/Gemini live settings.
- `settings.json` location follows `get_app_config_dir()`. Do not hard-code
  `$HOME/.cc-switch-tui/settings.json`.
- WebDAV remote V1/V2 naming is protocol-level; do not explain it as a software
  version.
- The config directory migration marker is `.migrated-from-cc-switch` and lives
  in the active target directory.
- `config.json.migrated` and `skills.json.migrated` are archive names for local
  DB import, separate from the config-directory marker.
- WebDAV V2 manifest compatibility checks are strict for current layout and
  tolerate legacy V2 layout by treating missing DB compat as the old compatible
  generation.
