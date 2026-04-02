# Lexiangla Scraper — 开发指引

## 工具概述

抓取腾讯乐享（lexiangla.com）上的文档和文件，支持导出 HTML、Markdown、原始文件下载、8K 高清预览截图和 PDF 合成。

## 文件结构

```
lexiangla-scraper/
├── CLAUDE.md              # 本文件：开发指引
├── README.md              # 用户使用文档
├── cli.py                 # CLI 入口（fetch / debug 子命令）
├── scraper.py             # 核心逻辑：API 调用、内容解析、文件下载
├── preview_capture.py     # Playwright 高清预览截图模块
├── requirements.txt       # Python 依赖
├── .env.example           # 环境变量模板
├── .env                   # 实际配置（git 忽略）
├── .gitignore
└── output/                # 输出目录（git 忽略）
```

## 核心模块

### scraper.py

- `LexianglaClient` — 主类，基于 `requests.Session` + Cookie 认证
- 认证方式：浏览器 Cookie（`LEXIANGLA_COOKIE`），自动提取 `XSRF-TOKEN` 用于请求头
- API 端点：`https://lexiangla.com/api/v1/docs/{doc_id}`（获取文档元数据）
- `fetch_doc(url)` → `DocResult`（包含标题、类型、内容、下载 URL 等）
- `save_doc(result, ...)` — 根据文档类型分发保存逻辑
- `_images_to_pdf(images, path)` — 将 PNG 列表合成 PDF（缩放至 1920x1080/页）
- 文档类型判断：`target.model == "file"` 或 `type in ("pptx", "docx", ...)` 为文件类型

### preview_capture.py

- 核心函数：`capture_preview_images(doc_url, cookie_str, output_dir, ...)`
- 策略：先加载父页面提取 WebOffice iframe URL，再用 **3840x2160@2x** 视口直接打开 iframe
- 幻灯片占满全视口，截图输出 **7680x4320**（8K）
- 同时通过 `page.on("response")` 拦截 `previewsh.myqcloud.com/shapes/` 的图片响应
- 逐页用 `ArrowRight` 键盘事件翻页
- 使用系统 Chrome（`channel="chrome"`），不单独下载 Chromium

### cli.py

- `fetch` 子命令：支持多 URL、`--output`、`--no-preview`、`--continue-on-error`
- `debug` 子命令：输出 API 原始 JSON
- Cookie 来源优先级：`--cookie` 参数 > `.env` 中 `LEXIANGLA_COOKIE`

## 关键技术细节

### 认证

```
Cookie: XSRF-TOKEN=xxx; token=eyJhbGci...; company_code=xxx
请求头: X-XSRF-TOKEN=<decoded XSRF-TOKEN>
```

XSRF-TOKEN 需要两次 URL decode。

### API 响应结构

```json
{
  "name": "文档标题",
  "type": "pptx",
  "target": {
    "model": "file",
    "downloadable": false,
    "preview_page": 51,
    "preview_status": "success",
    "signature": "..."
  },
  "owner": { "display_name": "..." },
  "team": { "name": "...", "code": "..." }
}
```

### 预览截图流程

```
1. GET lexiangla.com/teams/xxx/docs/xxx  →  找到 iframe[src*="prvsh.myqcloud.com"]
2. 提取 iframe URL（含 _w_provider + _w_sign 鉴权参数）
3. 新建 3840x2160@2x 浏览器上下文，直接打开 iframe URL
4. 等待 .play_root_container 渲染
5. 逐页 ArrowRight + element.screenshot() → 7680x4320 PNG
6. 拦截 previewsh.myqcloud.com/shapes/ 响应 → 嵌入原图
```

### 嵌入原图 API

```
POST prvsh.myqcloud.com/api/v3/office/file/{file_key}/shapes?_w_provider=...&_w_sign=...
Body: {"ids":["shape_hash"],"expire":28800,"support_webp":true}
→ 返回签名 URL

GET sid-preview-sh-xxx.previewsh.myqcloud.com/shapes/{file_key}/{shape_hash}?sign=...
→ 原始图片（1920x1080）
```

## 开发注意事项

- Cookie 有效期有限（通常几天到几周），测试时注意更新
- `output/` 和 `.env` 在 `.gitignore` 中，不要提交
- Playwright 依赖系统 Chrome，如果环境没有 Chrome 会报错
- 大文件 PPT（50+ 页）截图耗时 2-5 分钟，属正常
- PDF 合成使用 Pillow，将 8K 图缩放到 1920x1080 以控制文件大小

## 依赖

| 包 | 用途 |
|----|------|
| requests | HTTP 请求 |
| markdownify | HTML → Markdown |
| python-dotenv | .env 配置读取 |
| playwright | 浏览器自动化 |
| Pillow | 图片处理 & PDF 合成 |
