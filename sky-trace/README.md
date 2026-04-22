# SkyTrace - 天网日志链路追踪工具

OTA 酒店直连业务的日志排查工具。通过自定义编排查询链路，将分散在多个服务中的天网日志串联起来，实现一键并发查询。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 后端 | Rust (reqwest / rusqlite / aes-gcm) |
| 前端 | Vue 3 + TypeScript + Pinia |
| 样式 | TailwindCSS 4 |
| 数据库 | SQLite (本地) |
| 日志 API | 天网 Skynet (`skynetapi.dss.17usoft.com`) |

## 环境要求

- Node.js >= 18
- Rust >= 1.70 (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- macOS / Windows / Linux

## 快速启动

```bash
cd sky-trace
npm install
npm run tauri dev
```

生产构建:

```bash
npm run tauri build
```

## 项目结构

```
sky-trace/
├── src/                          # Vue 前端
│   ├── components/               # 通用组件
│   │   ├── Sidebar.vue           # 左侧导航
│   │   ├── FlowCard.vue          # 链路卡片
│   │   ├── FlowFormDialog.vue    # 新建/复制/编辑链路弹窗
│   │   ├── NodeEditor.vue        # 节点编辑器（含参考备注、字段提示）
│   │   ├── NodeResult.vue        # 节点查询结果展示
│   │   ├── DynamicParamEditor.vue# 动态参数编辑器（搜索、排序、选项）
│   │   ├── FieldBindingInput.vue # 字段绑定输入（固定值/动态参数/模板）
│   │   ├── TimeRangeSelector.vue # 时间范围下拉选择器
│   │   ├── HighlightText.vue     # 搜索高亮文本
│   │   └── SnapshotExportDialog.vue # 快照导出弹窗
│   ├── views/                    # 页面视图
│   │   ├── FlowList.vue          # 链路列表
│   │   ├── FlowDetail.vue        # 链路详情（执行/编排）
│   │   ├── QuickQuery.vue        # 快速查询
│   │   ├── ChecklistManager.vue  # 监控 Checklist 管理
│   │   ├── RecoveryManager.vue   # 快速恢复单元管理
│   │   ├── SupplierManager.vue   # 供应商管理
│   │   ├── Settings.vue          # 天网应用配置
│   │   └── TrashBin.vue          # 回收站
│   ├── services/tauri.ts         # Tauri IPC 调用封装
│   ├── stores/app.ts             # Pinia 全局状态
│   ├── types/index.ts            # TypeScript 类型定义
│   ├── router.ts                 # Vue Router (hash mode)
│   └── main.ts                   # 入口
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── lib.rs                # 应用入口 + 插件/状态注册
│   │   ├── commands/mod.rs       # Tauri IPC 命令
│   │   ├── storage/
│   │   │   ├── models.rs         # 数据模型 (Serde)
│   │   │   └── db.rs             # SQLite CRUD
│   │   ├── query_engine/
│   │   │   └── client.rs         # 天网 API 客户端 + UI 链接生成
│   │   └── snapshot.rs           # AES-256-GCM 快照加密/解密
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
└── docs/
      ├── architecture.md          ← 深度架构参考（启动流程、模块地图、设计模式）
      ├── remote-control.md        ← 远端控制 + feature flags + 快照模式
      ├── release.md               ← 版本号、构建、发布清单、文件托管选型
      ├── feishu-bitable-setup.md  ← 飞书多维表格配置操作教程（原
  feishu-setup.md）
      └── product-vision.md        ← 产品规划历史文档（原 PLAN.md）
```

## 核心功能

### 1. 排查链路编排

每条链路 (TraceFlow) 包含:

- **基础信息**: 名称、描述、关联供应商、标签（创建后可随时编辑）
- **动态参数**: 执行时由用户填写的变量 (如酒店 ID)，支持提示信息（多行）、预定义选项（`值|显示名` 格式）、允许自定义输入开关
- **字段绑定**: 节点的 Filter1/Filter2/模糊查询/TraceId 支持三种绑定模式：
  - **固定值**: 直接填写固定字符串
  - **绑定参数**: 绑定一个动态参数，执行时取用户填入的值
  - **模板**: 混合固定文本和动态参数，如 `inc_{{hotel}}`，使用 `{{参数名}}` 语法
- **节点列表**: 按顺序排列，支持拖拽排序，每个节点可附加参考备注
- **字段提示**: 天网查询节点的每个过滤字段可配置参考提示信息（如"三要素拼接 id_room_rpid"）

节点类型:

| 类型 | 说明 |
|------|------|
| `skynet_query` | 天网日志查询，配置服务、模块、分类、过滤条件 |
| `info` | 信息说明节点，可附加外部链接 |
| `checklist` | 引用 Checklist 分组，展示监控链接 |

### 2. 选定/并发执行

- **全部执行**: 所有 `skynet_query` 节点同时发起 API 请求
- **选定执行**: 通过 checkbox 勾选目标节点，仅执行选中部分
- 全局时间范围选择（预设/自定义）
- 动态参数填值 → 自动注入到绑定的节点字段（含模板插值）
- 参数输入支持：纯文本、下拉选择（`值|显示名` 格式）、带自定义的下拉
- 参数提示信息（多行）展示在输入框下方
- 参数引用追踪：显示每个参数被哪些节点的哪个字段引用
- 结果按健康度着色: 🟢 正常 / 🟡 有 ERROR / 🔴 无数据
- 每个节点提供「天网 UI ↗」跳转链接

### 3. 日志查看

- 日志按时间降序展示，长消息自动截断
- 日志等级过滤: FATAL / ERROR / WARN / INFO，显示过滤数/总数
- 单击展开/收起，双击弹出全屏详情
- `Ctrl+Shift+E` 切换精简/完整模式
- `Ctrl+F` 节点内搜索，高亮 + 上下导航
- 全局跨节点搜索，匹配数统计
- 一键复制日志内容

### 4. 监控 Checklist

三层结构管理: **分组 → 检查项 → 链接**

- 支持批量添加检查项（多行粘贴）
- 可在排查链路中引入 Checklist 节点
- 执行时直接展示可点击的监控链接

### 5. 快速恢复单元

三层结构管理: **分组 → 恢复步骤 → 工具链接**

- 管理常见故障的恢复流程和操作指引
- 支持批量添加步骤（多行粘贴）
- 复制、删除（软删除到回收站）
- 快照导出时可选择包含的恢复分组
- 快照模式下支持查看、隐藏编辑

### 6. AI 日志分析 (Beta)

- 节点级「AI 分析」按钮: 分析单个查询节点的日志结果
- 链路级「AI 全局分析」按钮: 跨节点综合分析
- 预设 prompt 快速选择: 异常分析、关键信息提取、排查建议等
- 支持自定义分析需求描述
- 当前为 UI 壳子，预留 AI 模型接入口

### 7. 快照导出与升级

导出加密的 `.skytrace` 文件给运营/产品使用:

- 选择要导出的链路、Checklist 分组、恢复分组
- 填写**数据版本号**（如 `1.0`、`1.2`）
- 配置权限限制（隐藏编辑、设置、供应商等）
- AES-256-GCM 加密
- 导入后进入只读模式

#### 数据版本号用途

导出时填写的版本号存储在快照文件内，与飞书多维表格中配置的 `latest_data_version` 做对比：

- **你导出新版快照后**，在飞书里把 `latest_data_version` 更新为新版本号，填上 `data_update_url`（文件下载链接）和更新说明
- **用户启动时**，应用检测到远端版本 > 本地快照版本，自动弹出数据更新提示
- 用户点击「立即更新」，应用直接下载并替换为新版快照，无需重新分发安装包

> 软件升级（新 .app/.exe）和数据升级（新 .skytrace）是两套独立机制，都通过飞书多维表格统一管控。

### 8. 其他功能

- **供应商管理**: 按供应商维度组织排查链路
- **链路信息编辑**: 创建后可随时修改名称、描述、供应商、标签
- **链路复制**: 复制时弹出编辑弹窗，可修改基础信息
- **动态参数管理**: 搜索、排序（▲▼箭头 + 手动输入位置）、预定义选项、提示信息
- **节点复制/粘贴**: JSON 格式，精确选择插入位置
- **链路导出/导入**: 完整链路 JSON 序列化
- **回收站**: 软删除，可恢复或永久删除
- **收藏**: 常用链路置顶

## 天网 API

### 查询接口

```
POST http://skynetapi.dss.17usoft.com/log/real/list
Header: token: {api-token}
Content-Type: application/json
```

关键参数:

| 字段 | 说明 |
|------|------|
| `appIds` | 应用 ID 数组 |
| `module` / `category` / `subCategory` | 日志分类 |
| `filter1` / `filter2` | 过滤条件 |
| `indexContext` | msg 模糊查询 |
| `contextId` | TraceId 链路追踪 |
| `beginTime` / `endTime` | 时间范围 `yyyy-MM-dd HH:mm:ss.SSS` |
| `pageSize` | 每页条数，最大 500 |

### UI 跳转链接

```
https://skyeye.17usoft.com/logs/realquery?app={appUk}&data={URL编码JSON}
```

`data` JSON 结构包含 `time` (含 `raw`)、`category` 数组、`filter1`、`filter2`、`message`、`flowID`。

## 快照版编译与分发

### 为什么选 Tauri 2.0

- Rust 编译后为原生二进制，源码不可逆向（满足不开源需求）
- 包体积小（~10-15MB vs Electron ~100MB+）
- Tauri 2.0 新特性：增强的插件系统、更好的安全模型、IPC 优化
- Rust 层处理加密，前端无法接触密钥（快照安全性）
- 跨平台：macOS / Windows / Linux

### 编译

```bash
npm run tauri build
```

编译完成后产物位置（全部在 `src-tauri/target/release/bundle/` 下）：

| 平台 | 产物路径 | 说明 |
|------|----------|------|
| macOS | `macos/SkyTrace.app` | 应用包，可直接运行 |
| macOS | `dmg/SkyTrace_0.1.0_aarch64.dmg` | 安装镜像 |
| Windows | `nsis/SkyTrace_0.1.0_x64-setup.exe` | 安装程序 |

> Windows 需在 Windows 机器上编译，或配置交叉编译工具链。

### 两个编译版本

| 版本 | 编译命令 | 用途 |
|------|---------|------|
| 完整版 | `npx tauri build` | 开发者自用，全功能 |
| 快照版 | `npx tauri build -- --features snapshot-only` | 给运营/产品，锁死只读 |

**快照版行为：**

| 场景 | 有 snapshot.skytrace | 无 snapshot.skytrace |
|------|---------------------|---------------------|
| 界面 | 只读快照模式 | 锁定界面，提示放置快照文件 |
| 编排 | 隐藏 | 不可用 |
| 天网配置 | 隐藏 | 不可用 |
| 退出按钮 | 无 | 无 |

即使删除快照文件，快照版也**无法**变成完整版。

### 面向运营/产品的分发流程（端到端）

**你只需要做一次编译，之后只更新快照文件即可。**

#### Step 1: 编译（仅首次）

```bash
cd sky-trace
npm install

# 编译完整版（你自己用）
unset CI && npx tauri build

# 编译快照版（给运营/产品）
unset CI && npx tauri build -- --features snapshot-only
```

> 注意: 如果环境有 `CI=1` 变量会干扰编译，需先 `unset CI`。

#### Step 2: 在开发版中编排链路

```bash
# 启动开发版（不放快照文件时就是完整版）
npm run tauri dev
```

在完整版中：
1. 配置天网应用（Settings 页）
2. 添加供应商
3. 创建排查链路，添加节点，绑定动态参数
4. 测试运行确认无误

#### Step 3: 导出快照

1. 侧边栏底部点击「导出快照」
2. 勾选要包含的排查链路
3. **填写数据版本号**（如 `1.0`，与飞书配置对应）
4. 配置权限限制（默认全部禁止，即纯只读执行）
5. 保存为 `.skytrace` 文件

#### Step 4: 打包分发

**macOS:**
```bash
# 复制编译产物
mkdir -p SkyTrace-运营版
cp -r src-tauri/target/release/bundle/macos/SkyTrace.app SkyTrace-运营版/

# 将导出的快照文件放到 .app 同级
cp /path/to/exported.skytrace SkyTrace-运营版/snapshot.skytrace

# 打包
zip -r SkyTrace-运营版-mac.zip SkyTrace-运营版/
```

**Windows:**
```
SkyTrace-运营版/
├── SkyTrace.exe          ← 从 bundle/nsis 安装后的 exe
└── snapshot.skytrace     ← 快照文件
```
打包为 zip 即可。

#### Step 5: 运营/产品使用

1. 解压 zip
2. 双击 `SkyTrace.app`（Mac）或 `SkyTrace.exe`（Windows）
3. 应用自动检测快照文件，进入只读模式
4. 看到侧边栏显示「快照版 (只读)」
5. 点击排查链路 → 填入动态参数（如酒店 ID）→ 选时间范围 → 点击执行
6. 查看日志结果，点击「天网 UI ↗」可跳转公司日志平台详细查看

**无需任何开发知识，无需管理员权限。**

#### 更新快照（数据升级）

**方式一：手动替换文件**
当你修改了链路编排后，重新导出（填写新版本号，如 `1.1`）→ 替换旧文件 → 用户重启应用即可，不需要重新编译。

**方式二：远程推送（推荐）**
1. 将新快照文件上传到可下载的地址（如共享网盘、内网文件服务器等）
2. 在飞书多维表格更新：
   - `latest_data_version` → 新版本号（如 `1.1`）
   - `data_update_url` → 文件下载链接
   - `data_update_notes` → 更新说明（展示给用户）
3. 用户下次启动时自动收到更新提示，点击「立即下载」在浏览器打开链接
4. 下载完成后，从侧边栏底部「导入快照」手动加载新文件

### 权限限制项（默认全部禁止）

| 限制项 | 说明 |
|--------|------|
| `hideEdit` | 隐藏编排功能（只能执行不能修改） |
| `hideSettings` | 隐藏天网配置页 |
| `hideSuppliers` | 隐藏供应商管理页 |
| `hideQuickQuery` | 隐藏快速查询页 |
| `hideChecklistEdit` | 隐藏 Checklist 编辑（只读展示） |
| `hideRecoveryEdit` | 隐藏恢复单元编辑（只读展示） |
| `hideTrash` | 隐藏回收站 |
| `hideDebug` | 隐藏调试信息 |
| `hideUiLink` | 隐藏天网 UI 跳转链接 |

### 快照自动检测

启动时 Rust 层自动扫描以下位置的 `snapshot.skytrace`:
1. 可执行文件同级目录
2. macOS `.app` bundle 外层目录

检测到后自动进入只读模式，侧边栏显示「快照版 (只读)」标识。

### 加密方案

- 算法: AES-256-GCM
- 文件格式: `SKYTRACE` magic (8B) + version (1B) + nonce (12B) + ciphertext
- 密钥硬编码于 Rust 二进制中，编译后不可读取

## 数据库

SQLite 文件存储在系统应用数据目录，**与应用二进制分离**，升级或重装不会丢失数据。

| 系统 | 数据库路径 |
|------|-----------|
| macOS | `~/Library/Application Support/com.soulx.sky-trace/skytrace.db` |
| Windows | `C:\Users\{用户}\AppData\Roaming\com.soulx.sky-trace\skytrace.db` |

> `npm run tauri dev` 和编译版使用**同一个数据库**（相同 app identifier）。

表结构: `sky_app`、`supplier`、`trace_flow`、`checklist_group`、`recovery_group`、`execution_history`

删除操作为软删除 (`deleted_at` 字段)，可通过回收站恢复。

### 数据备份与迁移

```bash
# 备份
cp ~/Library/Application\ Support/com.soulx.sky-trace/skytrace.db ~/Desktop/skytrace-backup.db

# 恢复或迁移到其他机器
cp ~/Desktop/skytrace-backup.db ~/Library/Application\ Support/com.soulx.sky-trace/skytrace.db
```

将 `skytrace.db` 复制到另一台机器的相同路径，启动应用即可看到所有数据。

## 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+E` | 切换日志精简/完整模式 |
| `Ctrl+F` | 节点内搜索 |
| `Enter` | Checklist 编辑器中快速新增检查项 |
