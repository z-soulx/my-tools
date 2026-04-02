# CLAUDE.md

This file provides guidance to AI coding assistants working on this project.

## Project Overview

SkyTrace is a Tauri 2.0 desktop app for OTA hotel business log troubleshooting. Users orchestrate multi-service log query chains against an internal Skynet API.

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Pinia + TailwindCSS 4 + Vue Router (hash mode)
- **Backend**: Rust (Tauri 2.0) with SQLite (rusqlite), reqwest for HTTP
- **Build**: `npm run tauri dev` / `npm run tauri build`

## Architecture

```
Frontend (Vue)  ──IPC──>  Rust Commands  ──>  SQLite / HTTP API
src/                      src-tauri/src/
```

- `src/services/tauri.ts` — all IPC calls via `@tauri-apps/api/core` `invoke()`
- `src-tauri/src/commands/mod.rs` — `#[tauri::command]` handlers
- `src-tauri/src/storage/db.rs` — SQLite CRUD, `Database` struct with `Mutex<Connection>`
- `src-tauri/src/storage/models.rs` — Serde models, `#[serde(rename_all = "camelCase")]`
- `src-tauri/src/query_engine/client.rs` — Skynet API client + UI link generation

## Key Conventions

### Rust
- All models use `#[serde(rename_all = "camelCase")]` for frontend compat
- JSON columns in SQLite (tags, nodes, dynamic_params, items) — serialize with serde_json
- Soft delete via `deleted_at TEXT` column on `trace_flow`, `supplier`, `checklist_group`, `recovery_group`
- `Database` methods take `&self`, lock the `Mutex<Connection>` internally
- Commands return `Result<T, String>` with `.map_err(|e| e.to_string())`

### Frontend
- Path alias `@` → `src/`
- Types in `src/types/index.ts`
- Global state in `src/stores/app.ts` (Pinia)
- Router uses `createWebHashHistory()` (required for Tauri WebView)
- `FieldBinding` pattern: fields like filter1/filter2/indexContext/contextId can be `{ mode: "fixed", fixedValue }` or `{ mode: "dynamic", paramKey }` bound to flow-level DynamicParam

### API Integration
- Skynet API: `POST http://skynetapi.dss.17usoft.com/log/real/list`
- Times must be absolute `yyyy-MM-dd HH:mm:ss.SSS` — frontend resolves relative expressions like `now-30m` before sending
- UI link: `https://skyeye.17usoft.com/logs/realquery?app={appUk}&data={urlencoded JSON}`

## Common Tasks

### Add a new Tauri command
1. Add method to `Database` in `db.rs`
2. Add `#[tauri::command]` fn in `commands/mod.rs`
3. Register in `lib.rs` `generate_handler![]`
4. Add TS wrapper in `services/tauri.ts`

### Add a new page
1. Create `src/views/NewPage.vue`
2. Add route in `src/router.ts`
3. Add nav item in `src/components/Sidebar.vue` `allNavItems`
4. If data module: add to `stores/app.ts` loadAll, snapshot enterSnapshotMode, snapshot export

### Execution modes
- `executeNodes(false)` — execute all nodes
- `executeNodes(true)` — execute only checkbox-selected nodes (`selectedNodeIds`)

### Recovery Unit
- Same CRUD pattern as ChecklistGroup: `recovery_group` table, `RecoveryGroup` / `RecoveryGroupInput` models
- Soft delete, trash support, snapshot export with selectable groups
- Restriction: `hideRecoveryEdit` — hides edit buttons in snapshot mode

### AI Analysis (UI Shell)
- `NodeResult.vue`: per-node AI analysis panel with `showAiPanel`, `aiPrompt`, `aiResponse`
- `FlowDetail.vue`: global AI modal with `showGlobalAi`, `globalAiPrompt`, `globalAiResponse`
- Currently placeholder — replace `'功能开发中...'` with actual AI API call when ready

### Add a new node type
1. Add config interface in `src/types/index.ts`
2. Update `TraceNode.type` union and `config` union
3. Add form section in `NodeEditor.vue`
4. Add render section in `FlowDetail.vue` (execute mode template)
5. Handle in `executeAll()` if it produces results

## Database Tables

`sky_app`, `supplier`, `trace_flow`, `checklist_group`, `recovery_group`, `execution_history`

DB file: `{app_data_dir}/skytrace.db`

## Snapshot Mode

- Cargo feature `snapshot-only` → compiled binary is permanently locked to snapshot mode
- On startup, Rust checks for `snapshot.skytrace` next to the executable (`snapshot::detect_snapshot()`)
- `get_app_mode` command returns `{ snapshotOnly: bool, hasSnapshot: bool }`
- Frontend boot: if `snapshotOnly && !hasSnapshot` → shows lock screen; if `hasSnapshot` → `store.enterSnapshotMode(data, true)`
- `store.isAutoSnapshot` = true prevents exiting snapshot mode
- `store.snapshotOnlyBuild` / `store.missingSnapshot` control lock screen
- Build: `npx tauri build` (full) / `npx tauri build -- --features snapshot-only` (locked)
- Snapshot file format: AES-256-GCM encrypted in `snapshot.rs`
- `SnapshotData` includes `recovery_groups` with `#[serde(default)]` for backward compat with old snapshots
- `SnapshotRestrictions` fields: `hideEdit`, `hideSettings`, `hideSuppliers`, `hideQuickQuery`, `hideChecklistEdit`, `hideRecoveryEdit`, `hideTrash`, `hideDebug`, `hideUiLink`

## Don't

- Don't use `createWebHistory()` — only hash mode works in Tauri
- Don't send `env` parameter in API queries (removed by design)
- Don't resolve relative time in Rust — frontend handles it via `resolveRelativeTime()`
- Don't hardcode time ranges in node configs — time is a global execution parameter
