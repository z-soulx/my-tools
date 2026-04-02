# Lexiangla Scraper — 腾讯乐享文档抓取工具

通过浏览器 Cookie 认证，抓取腾讯乐享（lexiangla.com）上的文档和文件，支持导出为 HTML、Markdown、原始文件或 **8K 超高清预览图 + PDF**。

---

## 快速开始（3 步搞定）

### 第 1 步：安装依赖

需要 **Python 3.10+** 和 **Google Chrome 浏览器**。

```bash
# 进入工具目录
cd lexiangla-scraper

# 安装 Python 依赖
pip install -r requirements.txt
```

> 没有 pip？先安装 Python：https://www.python.org/downloads/

### 第 2 步：获取 Cookie

工具需要你的乐享登录 Cookie 才能访问文档。获取方法：

1. 用 Chrome 打开乐享并登录
2. 按 **F12**（Mac 上是 `Cmd + Option + I`）打开开发者工具
3. 切换到 **Network**（网络）标签
4. 刷新页面，点击列表中任意一个请求
5. 在右侧 **Headers** 中找到 **Cookie** 一行
6. 右键 → 复制值

然后配置到 `.env` 文件：

```bash
# 复制模板
cp .env.example .env

# 用任意编辑器打开 .env，把 Cookie 粘贴进去
# 例如 macOS:
open -e .env
```

`.env` 文件内容示例：

```
LEXIANGLA_COOKIE=XSRF-TOKEN=xxx; token=eyJhbGci...; company_code=xxx
```

> Cookie 有效期通常几天到几周，过期后重新获取即可。

### 第 3 步：抓取文档

```bash
# 抓取一篇文档（把 URL 替换成你要抓的）
python cli.py fetch "https://lexiangla.com/teams/k100069/docs/c1bfb42222ba11f18e3b6a03128c6dab"
```

完成后文件保存在 `output/` 目录。

---

## 常用命令

### 抓取单篇

```bash
python cli.py fetch "文档URL"
```

### 抓取多篇

```bash
python cli.py fetch "URL1" "URL2" "URL3" -c
```

`-c` 表示某篇失败时跳过、继续处理下一篇。

### 指定输出目录

```bash
python cli.py fetch "URL" -o ./my-docs/
```

### 跳过预览截图（速度更快）

```bash
python cli.py fetch "URL" --no-preview
```

### 通过命令行传入 Cookie（不用 .env）

```bash
python cli.py --cookie "XSRF-TOKEN=xxx; token=xxx; ..." fetch "URL"
```

### 调试模式（查看 API 原始返回）

```bash
python cli.py debug "URL"
```

---

## 输出文件说明

抓取完成后，`output/` 目录结构如下：

```
output/
├── 文档标题.meta.json               # 文档元信息（标题、作者、阅读量等）
│
│  ── 富文本文档会生成 ──
├── 文档标题.html                     # HTML 格式
├── 文档标题.md                       # Markdown 格式
│
│  ── 文件类文档（PPT/Word/Excel）──
├── 文档标题.pptx                     # 原始文件（仅当文件允许下载时）
├── 文档标题.pdf                      # 预览图合成的 PDF（1920x1080/页）
└── 文档标题_pages/                   # 超高清预览截图
    ├── 文档标题_page_001.png         # 每页 7680x4320 (8K)
    ├── 文档标题_page_002.png
    ├── ...
    └── shapes/                       # 文档中嵌入的原始图片
        ├── shape_001_ea61d6ea.png
        └── ...
```

---

## 工作原理

```
输入文档 URL
    │
    ▼
通过 Cookie 调用乐享内部 API 获取文档信息
    │
    ├── 富文本文档 → 提取 HTML → 转换 Markdown → 保存
    │
    └── 文件类文档
         ├── 可下载 → 直接下载原始文件
         └── 不可下载 → 启动 Playwright (Chrome)
              │
              ├── 打开文档页面，提取 WebOffice 预览 iframe 地址
              ├── 用 3840x2160@2x 视口直接打开 iframe（幻灯片占满全屏）
              ├── 逐页截图，每页输出 7680x4320 PNG
              ├── 同时拦截网络请求，捕获嵌入的原始图片
              └── 合成 PDF（缩放到 1920x1080/页，约 7MB）
```

---

## 常见问题

### Cookie 过期了怎么办？

重新按「第 2 步」获取新的 Cookie，粘贴到 `.env` 即可。

### 抓取很慢？

预览截图需要启动浏览器逐页截取，51 页 PPT 约需 2-5 分钟。加 `--no-preview` 可跳过截图，只保存元信息。

### 报错 "未找到 WebOffice iframe"？

文档可能不支持在线预览，或页面结构变化。可以用 `debug` 命令查看原始 API 返回排查。

### 图片不够清晰？

当前输出为 8K（7680x4320），已是最高清晰度。如需调整，可修改 `preview_capture.py` 中的 `viewport` 和 `device_scale_factor` 参数。

### macOS 上 Chrome 在哪里？

Playwright 会自动使用系统安装的 Chrome（`channel="chrome"`）。确保已安装 Google Chrome 即可。

---

## 技术栈

- **Python 3.10+**
- **requests** — HTTP 请求
- **markdownify** — HTML 转 Markdown
- **python-dotenv** — 读取 .env 配置
- **playwright** — 浏览器自动化（截取预览图）
- **Pillow** — 图片处理 & PDF 合成
