# CLAUDE.md

This file provides guidance to AI coding assistants working on this project.

## Project Overview

SkyTrace is a Tauri 2.0 desktop app for OTA hotel business log troubleshooting. Users orchestrate multi-service log query chains against an internal Skynet API. Supports snapshot mode (read-only, encrypted) for non-developer users, and remote control via Feishu Bitable (kill switch, feature flags, version management).

## Tech Stack

| Layer | Tech |
|-------|------|
| Frontend | Vue 3 + TypeScript + Pinia + TailwindCSS 4 + Vue Router (hash mode) |
| Backend | Rust (Tauri 2.0), SQLite (rusqlite), reqwest |
| Build | `npm run tauri dev` / `npm run tauri build` |

## Architecture

```
Frontend (Vue)  ──IPC──>  Rust Commands  ──>  SQLite / HTTP API
src/                      src-tauri/src/
```

- `src/services/tauri.ts` — all IPC via `invoke()`
- `src-tauri/src/commands/mod.rs` — `#[tauri::command]` handlers
- `src-tauri/src/storage/db.rs` — SQLite CRUD
- `src-tauri/src/storage/models.rs` — Serde models, `#[serde(rename_all = "camelCase")]`
- `src-tauri/src/query_engine/client.rs` — Skynet API client
- `src-tauri/src/remote_config.rs` — Feishu Bitable remote control
- `src-tauri/src/snapshot.rs` — AES-256-GCM snapshot encrypt/decrypt

→ Deep architecture details: [docs/architecture.md](docs/architecture.md)

## Key Conventions

### Rust
- Models use `#[serde(rename_all = "camelCase")]` for frontend compat
- JSON columns in SQLite: serialize with `serde_json`
- Soft delete via `deleted_at TEXT` on `trace_flow`, `supplier`, `checklist_group`, `recovery_group`
- Commands return `Result<T, String>` with `.map_err(|e| e.to_string())`

### Frontend
- Path alias `@` → `src/`
- Types in `src/types/index.ts`; global state in `src/stores/app.ts`
- Router uses `createWebHashHistory()` — required for Tauri WebView
- `__APP_VERSION__` injected by vite.config.ts from package.json

## Common Tasks

### Add a Tauri command
1. Add method to `Database` in `db.rs`
2. Add `#[tauri::command]` fn in `commands/mod.rs`
3. Register in `lib.rs` `generate_handler![]`
4. Add TS wrapper in `services/tauri.ts`

### Add a page
1. Create `src/views/NewPage.vue`
2. Add route in `src/router.ts`
3. Add nav item in `Sidebar.vue` `allNavItems` (include `featureKey` if remotely controlled)
4. If data module: add to `stores/app.ts` loadAll, snapshot enterSnapshotMode, snapshot export

### Snapshot / Remote control
→ [docs/remote-control.md](docs/remote-control.md)
→ [docs/feishu-bitable-setup.md](docs/feishu-bitable-setup.md)

### Build & release
→ [docs/release.md](docs/release.md)

## Don't

- Don't use `createWebHistory()` — only hash mode works in Tauri
- Don't send `env` parameter in API queries (removed by design)
- Don't resolve relative time in Rust — frontend handles it via `resolveRelativeTime()`
- Don't hardcode time ranges in node configs
- Don't add HTML5 drag & drop (Tauri WebView compat issues) — use arrows or position input
