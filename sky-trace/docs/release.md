# Build & Release Guide

## Dev

```bash
npm run tauri dev
```

## Build

### Full build (all features)
```bash
npm run tauri build
```

### Snapshot-only build (locked to snapshot file)
```bash
npx tauri build -- --features snapshot-only
```

Output: `src-tauri/target/release/bundle/`
- macOS: `.dmg` + `.app`
- Windows: `.msi` + `.exe`

## Version Bump

Update version in **three places** (must stay in sync):
1. `package.json` → `"version"`
2. `src-tauri/Cargo.toml` → `version`
3. `src-tauri/tauri.conf.json` → `"version"`

`__APP_VERSION__` in the frontend is injected from `package.json` at build time (see `vite.config.ts`).

## Release Checklist

1. Bump version in all three files
2. Build: `npm run tauri build`
3. Upload `.dmg` / `.exe` to file hosting (Gitee Release, OSS, etc.)
4. Update Feishu Bitable:
   - `latest_version` → new version number
   - `update_url_mac` / `update_url_win` → new download URLs
   - `update_notes` → changelog
5. Clients will see update dialog on next launch

## Force-upgrade a Version

1. Upload new installer to hosting
2. Set `min_version` in Bitable to the new version
3. Clients below this version will see force-update screen (cannot use app)

## File Hosting Options

The download URLs in Bitable must be **direct download links** (no login required, no redirect to preview page). Feishu Drive / Doubao Drive links **do not work** — they require authentication.

Recommended:
- **Gitee Releases** (free, 100 MB per file limit, public repos)
- **Aliyun OSS / Tencent COS** (~¥0.12/GB/month)
- **Cloudflare R2** (free tier 10 GB)
