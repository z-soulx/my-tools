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

Key columns on `trace_flow`: `dynamic_params TEXT`, `nodes TEXT`, `node_groups TEXT` (all JSON).

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

Template transform syntax: `{{key:split(delimiter,index)}}`.
Example: value `"9_41_42177771"`, `{{key:split(_,2)}}` → `"42177771"`.

`resolveBinding()`, `applyTransform()`, and `extractTemplateParams()` in `src/types/index.ts`.

## DynamicParam Enhancements

- `hint` — multi-line tips shown at execution
- `options` — predefined choices (`value|label` or plain text)
- `allowCustom` — allow free input when options exist
- `snippets` — quick-fill chips (`value|label` format)
- `hidden` — param not shown in execute panel (used for internal derived values)
- `paramType` — auto format conversion: `text | datetime | date | timestamp_ms | timestamp_s | day_timestamp_s`
- Reordering via ▲▼ arrows or manual position input ("移到第 X 位")
- No HTML5 drag & drop (Tauri WebView compat)

## Node Types

| Type | Config struct | Display | Component |
|------|--------------|---------|-----------|
| `skynet_query` | `SkynetQueryConfig` | 天网查询 | `NodeResult.vue` |
| `info` | `InfoNodeConfig` | 信息节点 | inline in FlowDetail |
| `link` | `LinkNodeConfig` | 链接 | inline in FlowDetail |
| `checklist` | `ChecklistNodeConfig` | 监控Checklist | inline in FlowDetail |
| `jcp_order` | `JcpOrderConfig` | 产品组成单分析 | `JcpOrderResult.vue` |

Each has a form section in `NodeEditor.vue` and a render section in `FlowDetail.vue`.

## JCP Order Node (产品组成单分析)

Two-phase execution inside `executeNodes()`:

1. **JCP API** — `POST http://jcp.mis.elong.com/orderparse/getBookingDetailAjax`
   - Query by `orderId` or `traceId`
   - Extracts: roomTypeId, shotelId, ratePlanId, checkInDate, checkOutDate, requestTime
2. **Supplier Mapping** (optional, when `supplierMappingEnabled`) — `POST http://hotedcapi.vip.elong.com:8104/.../GetMapping4ProductReq`
   - Input: elongHotelId (shotelId), elongRoomId (roomTypeId), elongRateplanId (ratePlanId)
   - Extracts: supplierHotelId, supplierRatePlanId, supplierRoomTypeId

Time fields auto-derive suffixed params: `_ymd`, `_full`, `_ts`, `_tsSec`, `_dayTs`.
requestTime auto-focuses TimeRangeSelector (before/after window configurable separately via `requestTimeWindowBefore`/`requestTimeWindowAfter`).

Recursive deep search (`findDeep`) traverses both objects and arrays to find fields at any depth.

Rust commands: `query_jcp_order`, `query_supplier_mapping` in `commands/mod.rs`.

## Node Groups

Stored on `TraceFlow.nodeGroups: NodeGroup[]` (DB column: `node_groups TEXT DEFAULT '[]'`).

- **Edit mode**: collapsible panel with checkboxes per queryable node + inline name input to save groups
- **Execute mode**: read-only chips that set `selectedNodeIds` on click

## Execution Modes

Two-phase execution in `executeNodes()`:
- **Phase A**: `jcp_order` nodes run sequentially (extract params for downstream use)
- **Phase B**: `skynet_query` nodes run in parallel (using extracted params + user input)

Incremental: only targeted node results are cleared; other results preserved.

Toolbar actions in execute panel:
- **Refresh** (`refreshFlow`) — reloads flow config from DB, preserves user-entered `dynamicValues`
- **Clear params** (`clearParams`) — resets all `dynamicValues` to empty
- **Node group chips** — click to apply saved selection

## AI Analysis (Shell)

`NodeResult.vue`: per-node panel (`showAiPanel`, `aiPrompt`, `aiResponse`).  
`FlowDetail.vue`: global modal (`showGlobalAi`, `globalAiPrompt`, `globalAiResponse`).  
Currently placeholder — replace `'功能开发中...'` with actual API call when ready.
