# cc-switch-tui 迁移逻辑

最后更新：2026-05-11

本文档记录会影响本地配置、SQLite 状态和 WebDAV 同步数据的迁移行为。它用于给后续修改提供实现上下文；如果行为发生变化，以源码为最终依据。

## 重要术语

- 应用配置目录：由 `src-tauri/src/config.rs` 中的 `get_app_config_dir()` 解析。
- 不要假定应用配置目录一定是 `~/.cc-switch-tui`。
- 设置了 `CC_SWITCH_TUI_CONFIG_DIR` 时，它拥有最高优先级。
- `CC_SWITCH_CONFIG_DIR` 是已废弃的旧覆盖变量，但仍然兼容。
- 默认应用配置目录是 `$HOME/.cc-switch-tui`。
- 旧应用配置目录是 `$HOME/.cc-switch`。
- WebDAV `V1` 和 `V2` 是同步协议/远端数据格式版本，不是 cc-switch-tui 的软件版本。

## 配置目录迁移

源码：`src-tauri/src/config.rs`

目录迁移会把旧默认目录 `$HOME/.cc-switch` 中的数据迁移到当前生效的应用配置目录。

触发条件：

- `get_app_config_dir()` 会在每个进程内调用一次 `migrate_legacy_config_dir_if_needed()`，前提是没有配置目录环境变量让旧路径含义发生变化。
- 也可以通过 `legacy_config_migration_paths()` 和 `check_legacy_config_dir_migration_needed()` 检查是否需要迁移。
- 用户可以通过 `skip_legacy_config_dir_migration()` 跳过迁移。

目标目录选择：

- 如果设置了 `CC_SWITCH_TUI_CONFIG_DIR`，它就是目标目录。
- 如果设置了已废弃的 `CC_SWITCH_CONFIG_DIR`，不会尝试自动旧目录迁移，因为这个旧环境变量本身已经指向一个显式配置目录。
- 否则目标目录是 `$HOME/.cc-switch-tui`。

迁移前置条件：

- 源目录必须是 `$HOME/.cc-switch`，必须存在，必须是目录，并且至少包含一个条目。
- 源目录和目标目录不能是同一路径。
- 目标目录要么不存在，要么是空目录。
- 目标目录不能包含 `.migrated-from-cc-switch`。

复制的数据：

- 非符号链接文件会被复制。
- 非符号链接目录会被递归复制。
- `backups/` 有特殊处理：只复制最近的三个非符号链接条目。
- 旧目录会被保留；这个迁移永远不会删除旧目录。

标志文件：

- 迁移成功后，目标目录会写入 `.migrated-from-cc-switch`。
- 标志文件内容会记录源路径和时间戳。
- 如果用户跳过迁移，也会在同一个标志文件路径写入 `User declined migration`。
- 标志文件位于目标目录，不在旧目录。

失败行为：

- 迁移失败只会写入 stderr 日志，不会阻塞启动。
- 因为旧目录会保留，失败不应破坏已有数据。

## AppState 启动迁移

源码：`src-tauri/src/store.rs`

大部分本地数据迁移都通过 `AppState::try_new()` 完成。

路径：

- `cc-switch.db`、`config.json` 和 `skills.json` 都位于 `get_app_config_dir()` 解析出来的当前应用配置目录下。

如果 `cc-switch.db` 已存在：

- 通过 `Database::init()` 打开数据库。
- 从 SQLite 导出运行时状态为 `MultiAppConfig`。
- 执行旧 Codex provider 配置规范化。
- 如有需要，执行 common config 上游语义迁移。
- 不会再次导入旧 `config.json`。

如果 `cc-switch.db` 不存在：

- 创建 DB 前会先校验已有的 `config.json`。
- 创建 DB 前会先校验已有的 `skills.json`。
- `Database::init()` 创建 SQLite 数据库。
- 如果存在旧 `config.json`，通过 `Database::migrate_from_json()` 导入。
- 导入后的 `config.json` 会归档为 `config.json.migrated`。
- 如果存在旧 `skills.json`，会导入它的同步方式、仓库、已安装 skill 记录和 SSOT pending 标志。
- 导入后的 `skills.json` 会归档为 `skills.json.migrated`。
- 缺失的默认 skill 仓库会被补齐。
- 从 DB 导出运行时状态为 `MultiAppConfig`。
- 执行旧 Codex provider 配置规范化。
- 如有需要，执行 common config 上游语义迁移。

## `config.json` 到 SQLite 的迁移

源码：`src-tauri/src/database/migration.rs`

`Database::migrate_from_json()` 会把旧 `MultiAppConfig` 数据导入 SQLite。

会导入的数据：

- Providers，包括当前 provider 标志。
- Provider metadata 中的 provider endpoints。
- MCP servers 及其各应用启用标志。
- Claude、Codex 和 Gemini 的 prompts。
- Skill repos。
- Common config snippets。

不会直接导入的数据：

- 旧 `config.json` 里的 `skills.skills` 安装状态不会直接导入，因为这些数据不足以保证 SSOT 一致性。Skill 恢复由 skill service 的扫描/导入路径以及 `skills_ssot_migration_pending` 流程处理。

## SQLite Schema 迁移

源码：`src-tauri/src/database/schema.rs`

`Database::init()` 会在打开或创建数据库时应用 DB schema migrations。

重要副作用：

- DB schema 从 v2 迁移到 v3 时，会设置 DB setting `skills_ssot_migration_pending=true`，让 skill service 后续执行 SSOT 迁移。

## Common Config 上游语义迁移

源码：`src-tauri/src/services/provider/common_config.rs`

这是 provider common-config 行为的一次性 DB-backed 迁移。

标志：

- DB setting key：`common_config_upstream_semantics_migrated_v1`
- 当该 setting 为 `true` 时，跳过迁移。

行为：

- 仅适用于 Claude、Codex 和 Gemini。
- 如果某个 app 有非空 common config snippet，而 provider 没有显式 `meta.apply_common_config` 值，则把它视为 `true`。
- 规范化 provider 存储配置，避免 provider-specific storage 重复嵌入 common config。
- 更新后的 providers 会保存回 SQLite。

失败行为：

- 错误会返回给启动调用方。这不是 best-effort 后台迁移。

## Skills SSOT 迁移

源码：`src-tauri/src/services/skill.rs`

Skill 系统使用当前应用配置目录下的单一来源目录，加上 DB metadata。

Pending 标志：

- DB setting key：`skills_ssot_migration_pending`

触发条件：

- `SkillService::load_index()` 读取 pending 标志。
- 当标志被设置时，`SkillService::migrate_ssot_if_pending()` 执行迁移。

行为：

- 如果 DB 中已经有 managed skill records，迁移只会尽力为这些已管理记录回填 SSOT 目录。
- 如果没有 managed skill records，迁移会扫描受支持 app 的 skill 目录，并把发现的 skills 复制到 SSOT。
- 迁移或 best-effort 回填后，pending 标志会被清除。

安全规则：

- 当已存在 managed records 时，迁移不会自动把每个 app-local skill 目录都认领为 managed。这可以避免意外接管原本不由 cc-switch-tui 管理的用户目录。

## Claude MCP Override 迁移

源码：`src-tauri/src/claude_mcp.rs`

这个迁移处理 Claude config 目录被覆盖时 Claude MCP JSON 路径的变化。

触发条件：

- `user_config_path()` 会调用 `ensure_mcp_override_migrated()`。

行为：

- 如果没有设置 Claude config override，不执行迁移。
- 如果新的派生 MCP 路径已经存在，不执行迁移。
- 如果默认 `$HOME/.claude.json` 存在，而派生 override 路径不存在，则把默认文件复制到派生路径。

失败行为：

- 创建目录或复制失败只会记录 warning，不会 panic。

## WebDAV 同步协议版本

源码：`src-tauri/src/services/webdav_sync/mod.rs`

WebDAV `V1` 和 `V2` 表示远端同步数据格式，不表示 app 发布版本。

### V1 远端布局

路径形态：

- `{remote_root}/v1/{profile}/`

Manifest：

- `manifest.json`
- `format = "cc-switch-webdav-sync"`
- `version = 1`
- artifacts:
  - `dbSql`
  - `skillsZip`
  - `settingsSync`

旧 settings artifact 的默认文件名是 `settings.sync.json`，但 V1 manifest 中记录的 artifact path 是权威来源。如果 manifest 写的是 `settings-sync.json`，就使用该路径。

### V2 远端布局

当前路径形态：

- `{remote_root}/v2/db-v6/{profile}/`

旧 V2 fallback 路径形态：

- `{remote_root}/v2/{profile}/`

Manifest：

- `manifest.json`
- `format = "cc-switch-webdav-sync"`
- `version = 2`
- 当前布局下 `dbCompatVersion = 6`
- artifacts:
  - `db.sql`
  - `skills.zip`
  - `settings.json`

Manifest 最后上传。Artifacts 先上传，确保可见 manifest 指向的数据已经存在。

## WebDAV Upload

源码：`src-tauri/src/services/webdav_sync/mod.rs`

触发条件：

- CLI/TUI upload action 调用 `WebDavSyncService::upload()`。
- V1 到 V2 迁移在本地应用 V1 数据后也会调用 upload。

行为：

- 通过 `get_webdav_sync_settings()` 从 `settings.json` 读取当前 WebDAV 设置。
- 确保当前 V2 远端目录存在。
- 构建本地快照：
  - 导出 SQLite sync SQL 为 `db.sql`
  - 把 skill SSOT 打包为 `skills.zip`
  - 把当前 app settings 序列化为 `settings.json`
  - 用 artifact hashes 和 sizes 构建 manifest
- 上传 artifacts：
  - `db.sql`
  - `skills.zip`
  - `settings.json`
  - 最后上传 `manifest.json`
- 读取远端 manifest 并校验字节完全一致。
- Best-effort 获取 manifest ETag。
- Best-effort 持久化同步成功状态。
- 上传成功后 best-effort 清理 V1 远端数据。

## WebDAV Download

源码：`src-tauri/src/services/webdav_sync/mod.rs`

触发条件：

- CLI/TUI download action 调用 `WebDavSyncService::download()`。

行为：

- 优先查找当前布局中的 V2 snapshot。
- 回退查找旧 V2 布局。
- 如果没有 V2 数据但检测到 V1 manifest，返回 `SyncDecision::V1MigrationNeeded`，让 UI 询问用户是否迁移。
- 校验 protocol format、protocol version 和 DB compatibility。
- 下载并校验必需 artifacts：
  - `db.sql`
  - `skills.zip`
- 当 manifest 包含 `settings.json` 时，下载并校验它。
- 获取 restore mutation guard。
- 当 proxy runtime 或 takeover 状态导致恢复不安全时，拒绝恢复。
- 以一个恢复单元应用 DB 和 skills：
  - 备份当前 skills
  - 恢复 skills zip
  - 将 SQL 导入 SQLite
  - 如果 DB 导入失败，回滚 skills
- 如果下载到了 settings，则应用它，但保留当前本地 WebDAV 连接设置。这可以避免 restore 覆盖正在使用的 WebDAV URL/credentials。
- Best-effort 持久化同步成功状态。
- Best-effort 清理 V1 远端数据。

## WebDAV V1 到 V2 迁移

源码：`src-tauri/src/services/webdav_sync/mod.rs`

触发条件：

- CLI：`config webdav migrate-v1-to-v2`
- TUI：用户确认 V1 migration prompt。
- 程序入口：`WebDavSyncService::migrate_v1_to_v2()`

行为：

1. 读取本地 WebDAV 连接设置。
2. 检测并下载 V1 manifest。
3. 如果 restore 不安全则拒绝迁移，例如本地 proxy takeover 处于 active 状态。
4. 下载并校验 V1 artifacts：
   - `dbSql`
   - `skillsZip`
   - `settingsSync`
5. 把 V1 `settingsSync` 解析为可同步 app settings：
   - language
   - skill sync method
   - security settings
   - Claude custom endpoints
   - Codex custom endpoints
6. 获取 restore mutation guard。
7. 在本地应用 DB 和 skills snapshot。
8. 在本地应用 V1 syncable settings，同时保留当前 WebDAV 连接设置。
9. 释放 restore guard。
10. 把本地状态上传为 V2：
    - `db.sql`
    - `skills.zip`
    - `settings.json`
    - `manifest.json`
11. Best-effort 清理旧 V1 远端目录。

重要细节：

- V1 `settingsSync` 本身不会作为 V2 artifact 上传。
- 它会先应用到本地 `settings.json`，然后当前 app settings 会被序列化并作为 V2 `settings.json` 上传。

## WebDAV Restore 安全检查

源码：`src-tauri/src/services/webdav_sync/mod.rs`

Download 和 V1 migration 在修改本地状态前都会执行 restore 安全检查。

以下情况会拒绝 restore：

- managed proxy runtime 正在运行。
- 任意 proxy takeover 状态处于 active。

原因：

- 在 proxy takeover active 时恢复 DB/live state，可能导致 live client config 和 DB state 不一致。

## 当前实现注意点

- `settings.json` 是 app settings，不是 Claude/Gemini live settings。
- `settings.json` 位置跟随 `get_app_config_dir()`。不要硬编码 `$HOME/.cc-switch-tui/settings.json`。
- WebDAV 远端 V1/V2 命名是协议层概念，不要解释成软件版本。
- 配置目录迁移标志文件是 `.migrated-from-cc-switch`，位于当前生效的目标目录。
- `config.json.migrated` 和 `skills.json.migrated` 是本地 DB 导入后的归档文件名，和配置目录 marker 是两回事。
- WebDAV V2 manifest compatibility 对当前布局严格校验；对旧 V2 布局会兼容缺失 DB compat 的情况，把它视为旧兼容代。

