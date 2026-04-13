# Architecture Reference

Deep-dive into SkyTrace's internal structure. Read this when adding new modules or understanding existing ones.

## Directory Map

```
src-tauri/src/
├── main.rs                  # binary entry
├── lib.rs                   # plugin registration + command handler list
├── commands/mod.rs          # all #[tauri::command] handlers
├── query_engine/
│   ├── mod.rs
│   └── client.rs            # Skynet API HTTP client + UI link generation
├── storage/
│   ├── mod.rs               # re-exports
│   ├── db.rs                # Database struct (Mutex<Connection>), all CRUD
│   └── models.rs            # Serde structs for every table
├── snapshot.rs              # AES-256-GCM encrypt/decrypt + detect_snapshot()
└── remote_config.rs         # Feishu token fetch + Bitable read + fail-closed

src/
├── App.vue                  # boot sequence: remote check → snapshot → loadAll
├── router.ts
├── types/index.ts           # all TypeScript interfaces + utility functions
├── stores/app.ts            # Pinia global store
├── services/tauri.ts        # every invoke() call
├── views/                   # page components
└── components/              # shared UI components
```

## Boot Sequence (App.vue onMounted)

```
1. checkRemoteConfig()  ──fail──> RemoteLockScreen (network error)
        │
        ├─ enabled=false  ──────> RemoteLockScreen (kill switch)
        ├─ version < min  ──────> RemoteLockScreen (force update)
        │
        v
2. getAppMode()
        ├─ hasSnapshot=true ───> enterSnapshotMode(auto=true) → done
        ├─ snapshotOnly=true ──> missingSnapshot=true → done
        └─ normal ─────────────> loadAll()
        │
        v
3. Optional: latestVersion > appVersion ──> UpdateDialog
4. Optional: announcement ──────────────> AnnouncementBanner
```

## Feature Flag System

`remoteConfig.features` (from Feishu Bitable) controls UI visibility in **all** modes.

`store.featureEnabled(key)` returns:
- `false` if `remoteConfig` not loaded yet (fail-closed)
- `false` if `features[key] === false`
- `true` otherwise (missing key = enabled)

### Wired-up keys

| Key | Controls |
|-----|----------|
| `skynetQuery` | Sidebar "快速查询" nav item |
| `snapshotExport` | Sidebar export button |
| `snapshotImport` | Sidebar import button |
| `checklistEdit` | ChecklistManager create/edit/delete |
| `recoveryEdit` | RecoveryManager create/edit/delete |
| `trashAccess` | Sidebar "回收站" nav item |

Snapshot-mode-only restrictions (`store.snapshotRestrictions`) still apply on top of feature flags.

## Database Tables

`sky_app`, `supplier`, `trace_flow`, `checklist_group`, `recovery_group`, `execution_history`

DB file: `{app_data_dir}/skytrace.db`

Soft-delete pattern: `deleted_at TEXT` column, filtered out in normal queries.

## Snapshot Format

AES-256-GCM encrypted binary file (`.skytrace`). Key derived in Rust from hardcoded seed + per-snapshot salt. Front-end never sees the raw key.

`detect_snapshot()` checks for `snapshot.skytrace` next to the executable at startup.

`SnapshotData` includes `recovery_groups` with `#[serde(default)]` for backward compat.

## FieldBinding Pattern

Three modes for any query field (filter1, filter2, indexContext, contextId):

| Mode | Value source |
|------|-------------|
| `fixed` | `fixedValue` literal |
| `dynamic` | `paramKey` → flow-level `DynamicParam` at runtime |
| `template` | `templateValue` with `{{paramKey}}` interpolation |

`resolveBinding()` and `extractTemplateParams()` in `src/types/index.ts`.

## DynamicParam Enhancements

- `hint` — multi-line tips shown at execution
- `options` — predefined choices (`value|label` or plain text)
- `allowCustom` — allow free input when options exist
- Reordering via ▲▼ arrows or manual position input ("移到第 X 位")
- No HTML5 drag & drop (Tauri WebView compat)

## Node Types

| Type | Config struct |
|------|--------------|
| `skynet_query` | `SkynetQueryConfig` |
| `info` | `InfoNodeConfig` |
| `link` | `LinkNodeConfig` |
| `checklist` | `ChecklistNodeConfig` |

Each has a form section in `NodeEditor.vue` and a render section in `FlowDetail.vue`.

## Execution Modes

- `executeNodes(false)` — execute all nodes
- `executeNodes(true)` — execute only checkbox-selected nodes (`selectedNodeIds`)

## AI Analysis (Shell)

`NodeResult.vue`: per-node panel (`showAiPanel`, `aiPrompt`, `aiResponse`).  
`FlowDetail.vue`: global modal (`showGlobalAi`, `globalAiPrompt`, `globalAiResponse`).  
Currently placeholder — replace `'功能开发中...'` with actual API call when ready.
