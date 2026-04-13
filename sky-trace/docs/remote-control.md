# Remote Control & Snapshot Guide

SkyTrace uses Feishu Bitable as a serverless control plane — no backend server needed.

## How It Works

At every app launch:

1. Rust fetches `tenant_access_token` from Feishu Open API
2. Rust reads the single config row from Bitable
3. If any step fails → app locks (fail-closed)
4. Frontend checks `enabled`, `min_version`, `features`, then proceeds

## Feishu Bitable Fields

| Field | Type | Purpose |
|-------|------|---------|
| `enabled` | Checkbox | Kill switch. Uncheck → all clients lock immediately |
| `min_version` | Text | Minimum allowed version. Below this → force update screen |
| `latest_version` | Text | Latest version. Above client version → update dialog |
| `message` | Text | Text shown on lock screen |
| `update_url_mac` | URL | macOS installer download link |
| `update_url_win` | URL | Windows installer download link |
| `update_notes` | Text | Update changelog shown in dialog |
| `features` | Text | JSON feature flags (see below) |
| `announcement_text` | Text | Banner text. Empty = no banner |
| `announcement_type` | Single-select | `info` / `warning` / `error` |

## Feature Flags JSON

Store as a text field in Bitable:

```json
{"skynetQuery":true,"snapshotExport":true,"snapshotImport":true,"checklistEdit":true,"recoveryEdit":true,"trashAccess":true}
```

Set any value to `false` to hide the corresponding UI. Missing keys default to `true` (enabled).

## Kill Switch Scenarios

| Action in Bitable | Client effect |
|-------------------|--------------|
| Uncheck `enabled` | Lock screen with `message` text |
| Delete the app in Feishu Open Platform | API returns error → lock screen |
| Delete the Bitable table | API returns error → lock screen |
| Set `min_version` above client version | Force update screen |

## Credentials Location

`src-tauri/src/remote_config.rs` — `const CRED: &str`

Stored as base64(`app_id:app_secret:app_token:table_id`). Anyone extracting this can only **read** the Bitable (readonly API permission). Revoke the Feishu app at any time to invalidate all credentials.

To update credentials:
```bash
echo -n "app_id:app_secret:app_token:table_id" | base64
# paste result into CRED const
```

→ Setup tutorial: [feishu-bitable-setup.md](feishu-bitable-setup.md)

## Snapshot Mode

Two ways to enter snapshot mode:

1. **Auto-snapshot**: place `snapshot.skytrace` next to the executable. App detects it at startup and enters snapshot mode automatically (cannot exit).
2. **Manual import**: user clicks "导入快照" in sidebar, selects a `.skytrace` file.

### Snapshot-only build

```bash
npx tauri build -- --features snapshot-only
```

This compiled binary permanently shows a lock screen unless `snapshot.skytrace` is present.

### SnapshotRestrictions

Set during export — controls what's hidden in snapshot mode:

| Flag | Hides |
|------|-------|
| `hideEdit` | Edit/orchestrate tab in FlowDetail |
| `hideSettings` | "天网配置" nav item |
| `hideSuppliers` | "供应商管理" nav item |
| `hideQuickQuery` | "快速查询" nav item |
| `hideChecklistEdit` | Checklist create/edit/delete buttons |
| `hideRecoveryEdit` | Recovery create/edit/delete buttons |
| `hideTrash` | "回收站" nav item |
| `hideDebug` | Debug info toggle on node results |
| `hideUiLink` | "天网UI ↗" external links |

Note: `SnapshotRestrictions` only applies in snapshot mode. `features` (Feishu) applies in all modes.
