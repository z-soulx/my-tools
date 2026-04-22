# SkyTrace - 天网日志链路追踪工具

## 一、产品定位

**一句话描述：** 面向 OTA 酒店直连开发者的日志链路编排与追踪工具，将碎片化的天网日志查询串联为可复用的排查流程。

**目标用户：**
- 主要用户：开发者（编排模式，完整功能）
- 次要用户：运营/产品（快照模式，只读执行）

---

## 二、核心问题

| 痛点 | 现状 | 目标 |
|------|------|------|
| 日志查询不连贯 | 查入口 → 手动换参数 → 查另一个服务 → 忘记关键字 → 翻代码 | 一条链路串联所有节点，一键执行 |
| 参数记不住 | 每次都要回忆 category、filter、module 等 | 模板化，填入动态参数即可 |
| 排查流程无法复用 | 同样的排查路径每次重复操作 | 链路模板一键复制，改参数即可 |
| 多供应商管理混乱 | 不同供应商的 appId、token 散落各处 | 供应商维度统一管理 |
| 无法分享给非技术人员 | 运营/产品无法自行排查 | 快照模式，只读可执行 |

---

## 三、核心功能模块

### 3.1 天网查询引擎（Query Engine）

封装两种查询方式：

**API 查询（程序调用）：**
- 接口：`POST https://skyeye.inf.17usoft.com/skynet/api/logQuery`
- 支持所有查询参数：appIds、module、category、subCategory、filter1/2、contextId、priority、env、ips、indexContext、时间范围、分页
- 自动处理分页（lastRowTime + lastRowData）
- Token 管理（Header 或 Body 两种方式）

**UI 跳转（浏览器查看）：**
- 生成 `https://skyeye.17usoft.com/logs/realquery?app=xxx&data=xxx` 格式链接
- 一键打开浏览器查看详情
- 支持从 API 查询结果中直接跳转到 UI 查看单条日志

**天网应用配置管理：**
```
示例配置：
┌──────────┬───────────────────────┬──────────────────────────────────────┐
│ appId    │ appUk                 │ token                                │
├──────────┼───────────────────────┼──────────────────────────────────────┤
│ 106676   │ mvt.dc.product.mapping│ a013521a-f6db-48fd-a0fc-26266f67242d │
│ 106677   │ mvt.dc.supplier.agent │ 8804a1c8-0295-447b-b9e9-4c1f13c80163 │
└──────────┴───────────────────────┴──────────────────────────────────────┘
```

### 3.2 链路编排器（Flow Editor）

**核心概念：**
- **链路（Flow）：** 一个完整的排查流程，由多个节点按顺序/分支连接
- **节点（Node）：** 链路中的单个步骤

**节点类型：**

| 类型 | 技术类型 | 说明 | 示例 |
|------|----------|------|------|
| 天网查询节点 | `skynet_query` | 调用 API 查询日志 | 查 agent 侧 RMQ consume 日志 |
| 信息节点 | `info` | 展示固定文本、说明、外部链接 | "确认 Redis 品牌数据"，附链接 |
| 链接节点 | `link` | 生成天网 UI 查询链接 | 跳转天网查看详细日志 |
| 监控 Checklist | `checklist` | 引用预定义检查项组 | 检查各监控面板是否异常 |
| 产品组成单分析 | `jcp_order` | JCP API 查询 + 参数提取 + 供应商映射 | 分析成单错误码、提取房型/价格计划 |

**参数绑定机制（FieldBinding）：**

每个查询字段（filter1、filter2、indexContext、contextId）支持三种模式：

| 模式 | 说明 | 示例 |
|------|------|------|
| 固定值 (`fixed`) | 直接填写字面值 | `"RMQConsume"` |
| 动态绑定 (`dynamic`) | 绑定流程级动态参数，执行时填值 | 绑定 `hotelId` 参数 |
| 模板 (`template`) | 变量插值，支持多参数拼接 | `"inc_{{hotel}}_{{rpid}}"` |

模板变换语法：
```
{{参数key}}                — 原始值
{{参数key:split(分隔符,索引)}}  — 分割取值，如 {{id:split(_,0)}}
```

**时间衍生参数：**

提取 checkInDate / checkOutDate / requestTime 后自动生成以下后缀参数：
```
{{key}}_ymd    — yyyy-MM-dd 日期
{{key}}_full   — yyyy-MM-dd HH:mm:ss 完整时间
{{key}}_ts     — 毫秒时间戳
{{key}}_tsSec  — 秒级时间戳
{{key}}_dayTs  — 当日零点秒级时间戳
```

**场景示例 - 铂涛酒店 Mapping 未生效排查：**
```
[输入参数] hotelId, supplierName=铂涛
    │
    ▼
[节点1: 信息] 确认导入表中是否有该酒店 status=10 记录
    │         附带：数据库查询SQL模板、相关系统链接
    ▼
[节点2: 天网查询] 查 RMQ consume 是否被执行
    │  app: mvt.dc.product.mapping
    │  category: RMQConsume
    │  filter1: {{input.hotelId}}
    ▼
[节点3: 天网查询] 查 agent 侧是否收到请求
    │  app: mvt.dc.supplier.agent
    │  category: AgentRequest
    │  filter1: {{input.hotelId}}
    ▼
[节点4: 信息] 确认 Redis botao_brand{innId} 品牌数据
    │         附带：Redis 查询命令模板
    ▼
[节点5: 天网查询] 查活动列表是否为空
    │  app: mvt.dc.supplier.agent
    │  category: ActivityList
    │  filter1: {{input.hotelId}}
    ▼
[节点6: 条件] 根据各步骤结果判断问题定位
    ├─ 节点2无日志 → 消息未投递，检查 RMQ
    ├─ 节点3无日志 → agent 未收到，检查网络/路由
    └─ 节点5为空   → 品牌数据异常，检查 Redis
```

### 3.3 供应商管理（Supplier Manager）

- 自定义添加供应商（铂涛、华住、锦江等）
- 每个供应商关联可用的服务（agent、mapping 等）
- 供应商维度查看所有链路
- 供应商级别的通用参数预设

### 3.4 产品组成单分析节点（JCP Order）

**两阶段执行：**

1. 调用 JCP API (`POST /orderparse/getBookingDetailAjax`) 获取订单解析数据
   - 展示成单错误码（responseCode、errorDesc、handlerDepartment）
   - 展示产品链接（价格、规则、库存、rateplan）
   - 从响应中递归提取字段：roomTypeId、shotelId、ratePlanId、checkInDate、checkOutDate、requestTime

2. 可选：链式调用供应商映射 API (`POST /GetMapping4ProductReq`)
   - 输入：shotelId → elongHotelId、roomTypeId → elongRoomId、ratePlanId → elongRateplanId
   - 提取：supplierHotelId、supplierRatePlanId、supplierRoomTypeId

**关键特性：**
- 提取的参数自动填充到流程动态参数，供下游天网查询节点使用
- requestTime 自动聚焦：将查询时间范围设为 requestTime ± N 分钟（前/后可分别配置）
- 时间字段自动派生多格式后缀参数（_ymd、_full、_ts 等）
- 执行顺序保证：jcp_order 节点先于 skynet_query 执行

### 3.5 快照系统（Snapshot System）

**设计原则：** 同一个应用，两种模式

| 特性 | 编排模式（开发者） | 快照模式（运营/产品） |
|------|-------|-------|
| 链路编辑 | ✅ | ❌ |
| 链路执行 | ✅ | ✅（仅快照中的链路） |
| 参数修改 | ✅ | ⚠️ 仅允许填写输入节点的动态参数 |
| 天网配置 | ✅ | ❌ 不可见 |
| 源码/配置 | 可见 | 不可见（加密存储） |
| 快照管理 | ✅ 生成/更新 | ❌ |

**快照生成流程：**
1. 开发者选择链路 → 导出快照
2. Rust 层对链路配置进行 AES-256 加密
3. Token 等敏感信息内嵌在加密数据中
4. 生成 `.skytrace` 快照文件
5. 运营/产品使用同一应用打开快照文件 → 自动进入快照模式
6. 快照模式下隐藏所有编排功能，仅展示执行面板

### 3.6 配置中心（Config Center）

- 天网应用管理（appId、token、appUk）
- 环境切换（UAT/生产）
- 服务列表管理（agent、mapping 等可选服务）
- 全局参数预设

### 3.7 快速查询入口（Quick Query）

不是每次排查都需要走完整链路，提供独立的快速查询模式：
- 选服务 → 填参数 → 查，三步完成
- 查询参数可保存为"查询片段"，下次直接调用
- 相当于一个更好用的天网查询客户端
- 查询结果可一键"转为链路节点"，方便从临时查询演变为正式链路

### 3.8 排查知识库（Troubleshoot Knowledge Base）

每次链路执行完毕后，可记录排查结论：
- 问题定位到哪个节点
- 根因描述
- 解决方案
- 状态（已解决/待跟进）

结论关联到链路+供应商维度，下次同类问题可先查历史结论，避免重复排查。

---

## 四、扩展功能（未提到但很有价值的）

| 功能 | 说明 | 状态 |
|------|------|------|
| 执行历史 | 记录每次链路执行的时间、参数、结果摘要 | ⬜ P1 |
| 日志结果高亮 | 对 msg 中的关键字高亮、ERROR 标红 | ✅ 已实现 |
| 常用链路收藏/置顶 | 高频使用的链路快速访问 | ✅ 已实现 |
| 快捷键支持 | Ctrl+Enter 执行、Tab 切换节点等 | ⬜ P1 |
| 环境切换 | UAT/生产一键切换，自动调整 API 地址 | ⬜ 已移除（设计决策） |
| 链路执行报告导出 | 将排查过程导出为 HTML/PDF 报告 | ⬜ P2 |
| 定时巡检 | 周期性执行链路，异常时桌面通知 | ⬜ P2 |
| 日志结果对比 | 两次执行结果的 diff 对比 | ⬜ P2 |
| 节点分组 | 命名节点选择集合，一键勾选 | ✅ 已实现（nodeGroups） |
| 全文搜索 | 对历史日志结果全文检索 | ⬜ P3 |
| 智能参数提取 | 从上一步结果中提取字段传递给下一节点 | ✅ 已实现（jcp_order extractMappings） |
| 节点健康度指标 | 每个节点执行后自动判定红绿灯状态 | ✅ 已实现 |
| 并行执行模式 | 无依赖节点同时查询，加速排查 | ✅ 已实现（Phase B 天网并行） |
| 链路模板分享 | 导出链路配置（自动剥离Token），开发者间复用 | ✅ 已实现（导入/导出/快照） |
| 排查知识库 | 记录排查结论，沉淀排查经验 | ⬜ P2 |
| 快速查询入口 | 不编排也好用，三步查日志 | ✅ 已实现 |
| 产品组成单分析 | JCP订单解析 + 供应商映射 | ✅ 已实现（jcp_order 节点） |
| 监控 Checklist | 预定义检查项引用 | ✅ 已实现（checklist 节点） |
| 远端控制 | 飞书多维表格控制 kill switch / feature flags | ✅ 已实现 |

---

## 五、技术架构

### 5.1 技术选型

```
┌─────────────────────────────────────────────────┐
│                   SkyTrace App                   │
├────────────────────┬────────────────────────────┤
│    Frontend        │        Backend (Rust)       │
│                    │                             │
│  Vue 3 + TypeScript│  Tauri 2.0 Runtime          │
│  Pinia (状态管理)   │                             │
│  Vue Flow (流程图)  │  模块：                     │
│  TailwindCSS (样式) │  ├─ query_engine  (天网API)  │
│  Shiki (日志高亮)   │  ├─ flow_executor (链路执行)  │
│                    │  ├─ snapshot_mgr  (快照加密)  │
│                    │  ├─ storage       (SQLite)   │
│                    │  └─ config        (配置管理)  │
├────────────────────┴────────────────────────────┤
│              SQLite (本地数据存储)                  │
└─────────────────────────────────────────────────┘
```

**为什么选 Tauri 2.0：**
- Rust 编译后为原生二进制，源码不可逆向（满足不开源需求）
- 包体积小（~10-15MB vs Electron ~100MB+）
- Tauri 2.0 新特性：增强的插件系统、更好的安全模型、IPC 优化
- Rust 层处理加密，前端无法接触密钥（快照安全性）
- 跨平台：macOS / Windows / Linux

**前端选型理由：**
- Vue 3：组合式 API 适合工具类应用，生态成熟
- Vue Flow：基于 Vue 3 的流程图组件，适合链路可视化编排
- TailwindCSS：快速构建一致性 UI
- Pinia：轻量状态管理，TypeScript 友好

### 5.2 数据模型

```sql
-- 天网应用配置
CREATE TABLE sky_app (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id      TEXT NOT NULL UNIQUE,      -- 如 "106676"
    app_uk      TEXT NOT NULL,             -- 如 "mvt.dc.product.mapping"
    token       TEXT NOT NULL,             -- API token
    name        TEXT,                      -- 自定义别名
    env         TEXT DEFAULT 'prod',       -- 环境
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 供应商
CREATE TABLE supplier (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,             -- 如 "铂涛"
    code        TEXT NOT NULL UNIQUE,      -- 如 "botao"
    description TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 链路流程
CREATE TABLE trace_flow (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT NOT NULL,
    description    TEXT NOT NULL DEFAULT '',
    supplier_id    INTEGER,
    tags           TEXT NOT NULL DEFAULT '[]',     -- JSON array of strings
    is_favorite    INTEGER NOT NULL DEFAULT 0,
    sort_order     INTEGER NOT NULL DEFAULT 0,
    dynamic_params TEXT NOT NULL DEFAULT '[]',     -- JSON: DynamicParam[]
    nodes          TEXT NOT NULL DEFAULT '[]',     -- JSON: TraceNode[]
    node_groups    TEXT NOT NULL DEFAULT '[]',     -- JSON: NodeGroup[]
    created_at     TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    deleted_at     TEXT                            -- soft delete
);

-- 执行历史
CREATE TABLE execution_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    flow_id     INTEGER REFERENCES trace_flow(id),
    input_params TEXT NOT NULL,            -- JSON: 用户输入参数
    results     TEXT NOT NULL,             -- JSON: 各节点执行结果
    status      TEXT DEFAULT 'success',    -- success/failed/partial
    duration_ms INTEGER,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**TraceFlow 数据结构（实际实现）：**

`dynamic_params` 定义流程级动态参数：
```json
[
  { "key": "hotelId", "label": "酒店ID", "required": true, "defaultValue": "" },
  { "key": "rpid", "label": "RatePlanId", "required": false, "defaultValue": "", "paramType": "text" },
  { "key": "cin", "label": "入住日期", "required": false, "defaultValue": "", "paramType": "date", "hidden": true }
]
```

`nodes` 使用 FieldBinding 模式绑定参数：
```json
[
  {
    "id": "node_1",
    "type": "skynet_query",
    "label": "查 RMQ consume",
    "sortOrder": 0,
    "config": {
      "skyAppId": 1,
      "module": "",
      "category": "RMQConsume",
      "subCategory": "",
      "filter1": { "mode": "dynamic", "fixedValue": "", "paramKey": "hotelId" },
      "filter2": { "mode": "template", "fixedValue": "", "paramKey": "", "templateValue": "inc_{{hotel}}_{{rpid}}" },
      "indexContext": { "mode": "fixed", "fixedValue": "", "paramKey": "" },
      "contextId": { "mode": "fixed", "fixedValue": "", "paramKey": "" },
      "pageSize": 100
    }
  },
  {
    "id": "node_2",
    "type": "jcp_order",
    "label": "成单分析",
    "sortOrder": 1,
    "config": {
      "queryField": "traceId",
      "queryValue": { "mode": "dynamic", "fixedValue": "", "paramKey": "traceId" },
      "extractMappings": [
        { "sourceField": "roomTypeId", "targetParamKey": "roomType" },
        { "sourceField": "requestTime", "targetParamKey": "reqTime" }
      ],
      "requestTimeWindowBefore": 5,
      "requestTimeWindowAfter": 10,
      "supplierMappingEnabled": true,
      "supplierExtractMappings": [
        { "sourceField": "supplierHotelId", "targetParamKey": "sHotelId" }
      ]
    }
  }
]
```

`node_groups` 定义节点分组快选：
```json
[
  { "id": "lg1abc", "name": "Booking 全链路", "nodeIds": ["node_1", "node_3", "node_5"] }
]
```

---

## 六、UI 设计要点

### 主界面布局
```
┌──────────────────────────────────────────────────────┐
│  SkyTrace                    [环境:PROD ▼] [⚙ 设置]  │
├──────────┬───────────────────────────────────────────┤
│          │                                           │
│ 供应商    │           链路编排 / 执行区域               │
│ ├ 铂涛   │                                           │
│ │ ├ Map  │   [节点1] ──→ [节点2] ──→ [节点3]          │
│ │ └ Agt  │      │                       │            │
│ ├ 华住   │      ▼                       ▼            │
│ │ └ ...  │   [结果面板]             [结果面板]         │
│ └ 锦江   │                                           │
│          │                                           │
│ ──────── │                                           │
│ 收藏链路  │                                           │
│ ├ 链路A  │                                           │
│ └ 链路B  │                                           │
│          │                                           │
│ ──────── │                                           │
│ 最近执行  │                                           │
│          │                                           │
├──────────┴───────────────────────────────────────────┤
│ 日志详情面板（可展开/收起）                              │
└──────────────────────────────────────────────────────┘
```

### 快照模式界面
```
┌──────────────────────────────────────────────────────┐
│  SkyTrace [快照模式]              铂涛Mapping排查       │
├──────────────────────────────────────────────────────┤
│                                                      │
│  请输入参数：                                          │
│  酒店ID: [____________]                               │
│                                                      │
│  [▶ 开始排查]                                         │
│                                                      │
│  ┌─────────────────────────────────────────────┐     │
│  │ ✅ 步骤1: 查 RMQ consume         [查看日志]  │     │
│  │ ✅ 步骤2: 查 agent 请求           [查看日志]  │     │
│  │ ⏳ 步骤3: 检查 Redis 品牌数据      [执行中...]  │     │
│  │ ○ 步骤4: 查活动列表               [等待]     │     │
│  │ ○ 步骤5: 结果判定                 [等待]     │     │
│  └─────────────────────────────────────────────┘     │
│                                                      │
│  日志详情：                                           │
│  ┌─────────────────────────────────────────────┐     │
│  │ 2024-03-15 10:23:45.123 [INFO] ...          │     │
│  │ 2024-03-15 10:23:45.456 [ERROR] ...         │     │
│  └─────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────┘
```

---

## 七、开发分期计划

### P0 - 核心 MVP ✅
- [x] Tauri 2.0 + Vue 3 项目脚手架搭建
- [x] SQLite 数据层（sky_app、trace_flow 表）
- [x] 天网 API 查询封装（Rust 侧 HTTP 客户端）
- [x] 基础链路编排（列表式，顺序节点）
- [x] 参数模板变量解析引擎（FieldBinding 三模式 + split 语法）
- [x] 链路执行器（两阶段：jcp_order sequential → skynet_query parallel）
- [x] 日志结果展示面板
- [x] 天网 UI 链接生成与跳转

### P1 - 增强体验 ✅
- [x] 供应商维度管理
- [ ] ~~可视化拖拽编排~~（已改为列表式 + 拖拽排序，满足需求）
- [x] 链路复制/导入/导出
- [x] 自定义信息节点（文本+链接）
- [x] 快照生成与加载（AES-256 加密）
- [ ] 执行历史记录（DB 表已建，UI 待实现）
- [x] 日志关键字高亮
- [ ] ~~环境切换~~（设计决策：不发送 env 参数）
- [x] 远端控制（飞书多维表格 kill switch + feature flags）
- [x] 产品组成单分析节点（JCP API + 供应商映射）
- [x] 节点分组快选
- [x] 增量执行（只刷新选中节点）
- [x] 动态参数增强（paramType / hidden / snippets / options）

### P2 - 进阶功能（部分待实现）
- [ ] 条件分支节点
- [ ] 链路执行报告导出
- [x] 常用链路收藏/置顶
- [ ] 快捷键系统
- [ ] 定时巡检 + 桌面通知
- [ ] 执行结果 diff 对比
- [x] 监控 Checklist 节点
- [x] 恢复操作指引（Recovery Groups）

---

## 八、项目结构（预览）

```
sky-trace/
├── src-tauri/                   # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── query_engine/        # 天网 API 封装
│   │   │   ├── mod.rs
│   │   │   ├── skynet_client.rs # HTTP 请求
│   │   │   └── models.rs        # 请求/响应结构体
│   │   ├── flow_executor/       # 链路执行引擎
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs
│   │   │   └── template.rs      # 模板变量解析
│   │   ├── snapshot/            # 快照加密/解密
│   │   │   ├── mod.rs
│   │   │   └── crypto.rs
│   │   ├── storage/             # SQLite 数据访问
│   │   │   ├── mod.rs
│   │   │   └── migrations/
│   │   └── commands/            # Tauri IPC 命令
│   │       └── mod.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                         # Vue 3 前端
│   ├── App.vue
│   ├── main.ts
│   ├── views/
│   │   ├── FlowEditor.vue       # 链路编排页面
│   │   ├── FlowExecutor.vue     # 链路执行页面
│   │   ├── SupplierManager.vue  # 供应商管理
│   │   ├── SnapshotViewer.vue   # 快照模式页面
│   │   └── Settings.vue         # 设置页面
│   ├── components/
│   │   ├── nodes/               # 各类型节点组件
│   │   ├── LogPanel.vue         # 日志展示面板
│   │   └── ParamInput.vue       # 参数输入组件
│   ├── stores/                  # Pinia stores
│   ├── services/                # Tauri IPC 调用封装
│   └── types/                   # TypeScript 类型定义
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
└── PLAN.md
```

---

## 九、关键技术决策记录

| 决策 | 选项 | 结论 | 理由 |
|------|------|------|------|
| 桌面框架 | Tauri 2.0 / Electron / 纯Web | Tauri 2.0 | 包体积小、Rust安全性保证源码不泄露、性能好 |
| 前端框架 | Vue 3 / React / Solid | Vue 3 | 工具类应用适合、Vue Flow 生态、上手快 |
| 数据存储 | SQLite / JSON文件 / IndexedDB | SQLite | 结构化查询、Rust生态支持好(rusqlite)、可靠 |
| 快照机制 | 加密文件+同App / 独立编译 | 加密文件+同App | 灵活、不需要每次重编译、一个App两种模式 |
| 流程编排 | 拖拽 / 列表 / JSON | P0列表 → P1拖拽 | 渐进式，先快速可用再优化体验 |

---

## 十、链路编排引擎详细设计

### 10.1 核心抽象

```
TraceFlow（链路）
├─ DynamicParam[]     流程级动态参数（执行时用户填值）
│   ├─ paramType      值类型（text/datetime/date/timestamp_ms/timestamp_s/day_timestamp_s）
│   └─ hidden         内部参数，不在执行面板显示
├─ TraceNode[]        节点列表（按 sortOrder 排序）
│   ├─ SkynetQueryNode    天网 API 查询
│   ├─ InfoNode           固定信息展示（文本+链接）
│   ├─ LinkNode           天网 UI 跳转链接
│   ├─ ChecklistNode      引用监控检查项组
│   └─ JcpOrderNode       产品组成单分析 + 供应商映射
├─ NodeGroup[]        节点分组快选
└─ HealthRules        节点健康度评估（基于日志条数和优先级）
```

### 10.2 执行引擎流程

```
用户填写 DynamicParam 参数 + 选择时间范围
          │
          ▼
  校验必填参数
          │
          ▼
  ┌──────────────────────────────────────────┐
  │  Phase A: jcp_order 节点（顺序执行）       │
  │  1. 调用 JCP API                         │
  │  2. 递归提取字段值（findDeep，穿透数组）    │
  │  3. 时间字段自动派生后缀参数                │
  │  4. requestTime 自动聚焦查询时间范围        │
  │  5. (可选) 链式调用供应商映射 API           │
  │  6. 提取值回填到 dynamicValues             │
  └──────────────────────────────────────────┘
          │
          ▼
  ┌──────────────────────────────────────────┐
  │  Phase B: skynet_query 节点（并行执行）    │
  │  1. 解析 FieldBinding → 替换为实际值       │
  │  2. 调用天网 API                          │
  │  3. 生成天网 UI 链接                      │
  │  4. 评估健康度（红绿灯）                   │
  └──────────────────────────────────────────┘
          │
          ▼
  info / checklist 节点标记为 success（不需要 API 调用）
```

**增量执行**：只清除本次目标节点的结果，其它节点保留上次执行结果。

### 10.3 ExecutionContext 结构

```json
{
  "input": {
    "hotelId": "12345",
    "supplierName": "铂涛"
  },
  "nodes": {
    "node_1": {
      "status": "success",
      "health": "ok",
      "started_at": "2024-03-15T10:23:45.000Z",
      "finished_at": "2024-03-15T10:23:45.320Z",
      "duration_ms": 320,
      "result": {
        "count": 5,
        "list": [
          {
            "msg": "RMQ consume hotelId=12345 orderId=ORD-9876",
            "logTime": "2024-03-15 10:20:12.456",
            "priority": 3,
            "ip": "10.99.8.180",
            "category": "RMQConsume",
            "filter1": "12345"
          }
        ]
      },
      "ui_link": "https://skyeye.17usoft.com/logs/realquery?app=mvt.dc.product.mapping&data=..."
    },
    "node_2": {
      "status": "success",
      "health": "warning",
      "result": { "count": 2, "list": [...] }
    }
  },
  "global": {
    "env": "prod",
    "execution_id": "exec_20240315_102345",
    "started_at": "2024-03-15T10:23:45.000Z",
    "overall_health": "warning",
    "total_duration_ms": 1520
  }
}
```

### 10.4 FieldBinding 参数绑定引擎

**三种模式：**

| 模式 | 取值方式 | 示例 |
|------|---------|------|
| `fixed` | 字面值 | `"RMQConsume"` |
| `dynamic` | 绑定 DynamicParam.key | 绑定 `hotelId` → 运行时值 |
| `template` | `{{key}}` 插值 + 变换 | `"inc_{{hotel}}_{{rpid}}"` |

**模板变换语法：**

| 语法 | 说明 | 示例 |
|------|------|------|
| `{{key}}` | 原始值 | `{{hotelId}}` → `12345` |
| `{{key:split(delim,idx)}}` | 分割取值 | `{{id:split(_,2)}}` 取 `"9_41_42177771"` 的第 3 段 → `42177771` |

**类型自动转换（paramType）：**

DynamicParam 设置 `paramType` 后，`resolveBinding()` 自动将存储值转换为目标格式：

| paramType | 输出格式 |
|-----------|---------|
| `text` | 原样（默认） |
| `datetime` | `yyyy-MM-dd HH:mm:ss` |
| `date` | `yyyy-MM-dd` |
| `timestamp_ms` | 毫秒时间戳 |
| `timestamp_s` | 秒级时间戳 |
| `day_timestamp_s` | 当日零点秒级时间戳 |

实现：`resolveBinding()`、`applyTransform()`、`convertByParamType()` 在 `src/types/index.ts`。

### 10.5 节点健康度评估

每个节点可配置健康度规则：

```json
{
  "health_rules": {
    "ok": "result.count > 0 && !result.list.any(item => item.priority <= 1)",
    "warning": "result.list.any(item => item.priority <= 2)",
    "error": "result.count == 0"
  }
}
```

默认规则（未配置时）：
- `ok`：查询返回数据且无 FATAL/ERROR 级别日志
- `warning`：查询返回数据但含 WARN 或 ERROR 级别日志
- `error`：查询无数据（count=0）

前端展示为红绿灯图标，链路执行完后一眼定位问题节点。

### 10.6 执行模式

| 模式 | 行为 | 适用场景 |
|------|------|---------|
| 全部执行 | 按两阶段执行所有节点 | 常规排查 |
| 选中执行 | 只执行勾选的节点，其它保留上次结果 | 重试部分节点 |
| 分组快选 | 一键勾选预定义的节点组合 | 复杂链路的子流程 |

**执行面板工具栏：**
- ↻ 刷新：重新加载流程配置（不清空已填参数）
- ✕ 清空参数：一键重置所有参数为空

### 10.7 SkynetQueryNode 配置（SkynetQueryConfig）

```json
{
  "skyAppId": 1,
  "module": "",
  "category": "RMQConsume",
  "subCategory": "",
  "filter1": { "mode": "dynamic", "fixedValue": "", "paramKey": "hotelId" },
  "filter2": { "mode": "fixed", "fixedValue": "", "paramKey": "" },
  "indexContext": { "mode": "template", "fixedValue": "", "paramKey": "", "templateValue": "{{keyword}}" },
  "contextId": { "mode": "fixed", "fixedValue": "", "paramKey": "" },
  "pageSize": 100,
  "fieldHints": {
    "filter1": "三要素拼接 id_room_rpid"
  }
}
```

`skyAppId` 引用 `sky_app` 表记录（从 Pinia store 中查找 appId/token/appUk）。
时间范围由全局 TimeRangeSelector 控制，不在节点配置中。
`fieldHints` 可选，为每个字段提供参考提示。

### 10.8 天网 UI 链接生成

根据节点查询参数，自动生成天网 UI 链接：

```
基础URL: https://skyeye.17usoft.com/logs/realquery
参数:
  app = sky_app.app_uk
  data = URL编码的 JSON:
    {
      "time": { "from": beginTime, "to": endTime },
      "category": [category],
      "filter1": filter1
    }

最终: https://skyeye.17usoft.com/logs/realquery?app=mvt.dc.supplier.agent&data=%7B%22time%22%3A...%7D
```

每个天网查询节点执行后，自动在结果中附带 `ui_link`，用户可点击在浏览器中打开查看完整日志。

### 10.9 快照安全性设计

**加密方案：**
- 算法：AES-256-GCM（加密 + 完整性校验）
- 密钥：硬编码种子 + 快照专属盐值 → PBKDF2 派生
- 实现层：Rust（密钥永远不传到前端）

**快照文件格式：**
```
[8 bytes] Magic: "SKYTRACE"
[4 bytes] Version: 0x00000001
[16 bytes] Salt (随机)
[12 bytes] Nonce (随机)
[4 bytes] Encrypted data length
[N bytes] AES-256-GCM encrypted data
[16 bytes] GCM auth tag
```

**安全保证：**
- Token 等敏感信息在加密数据内部，解密后仅在 Rust 内存中
- 前端通过 Tauri IPC 只能拿到查询结果，拿不到原始 Token
- 编译时 strip symbols，增加逆向难度
- 快照文件无法被通用工具打开或分析
