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

| 类型 | 说明 | 示例 |
|------|------|------|
| 天网查询节点 | 调用 API 查询日志 | 查 agent 侧 RMQ consume 日志 |
| 信息节点 | 展示固定文本、说明、外部链接 | "确认 Redis botao_brand{innId} 品牌数据" |
| 链接节点 | 生成天网 UI 查询链接，可直接点击跳转 | 跳转天网查看详细日志 |
| 条件节点 | 根据上一步结果分支 | 如果查到记录 → A路径，否则 → B路径 |
| 输入节点 | 流程开始时收集动态参数 | 输入酒店ID、供应商名称 |

**参数模板机制：**
```
变量语法：{{scope.field}}

内置变量：
  {{input.hotelId}}        - 用户输入的动态参数
  {{input.supplierName}}   - 用户输入的动态参数
  {{node.1.filter1}}       - 引用节点1的查询参数filter1
  {{node.1.result.count}}  - 引用节点1的查询结果条数
  {{node.1.result.list[0].msg}} - 引用节点1第一条日志的msg
  {{now}}                  - 当前时间
  {{now-30m}}              - 30分钟前
  {{env}}                  - 当前环境
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

### 3.4 快照系统（Snapshot System）

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

### 3.5 配置中心（Config Center）

- 天网应用管理（appId、token、appUk）
- 环境切换（UAT/生产）
- 服务列表管理（agent、mapping 等可选服务）
- 全局参数预设

### 3.6 快速查询入口（Quick Query）

不是每次排查都需要走完整链路，提供独立的快速查询模式：
- 选服务 → 填参数 → 查，三步完成
- 查询参数可保存为"查询片段"，下次直接调用
- 相当于一个更好用的天网查询客户端
- 查询结果可一键"转为链路节点"，方便从临时查询演变为正式链路

### 3.7 排查知识库（Troubleshoot Knowledge Base）

每次链路执行完毕后，可记录排查结论：
- 问题定位到哪个节点
- 根因描述
- 解决方案
- 状态（已解决/待跟进）

结论关联到链路+供应商维度，下次同类问题可先查历史结论，避免重复排查。

---

## 四、扩展功能（未提到但很有价值的）

| 功能 | 说明 | 优先级 |
|------|------|--------|
| 执行历史 | 记录每次链路执行的时间、参数、结果摘要 | P1 |
| 日志结果高亮 | 对 msg 中的关键字高亮、ERROR 标红 | P1 |
| 常用链路收藏/置顶 | 高频使用的链路快速访问 | P1 |
| 快捷键支持 | Ctrl+Enter 执行、Tab 切换节点等 | P1 |
| 环境切换 | UAT/生产一键切换，自动调整 API 地址 | P1 |
| 链路执行报告导出 | 将排查过程导出为 HTML/PDF 报告 | P2 |
| 定时巡检 | 周期性执行链路，异常时桌面通知 | P2 |
| 日志结果对比 | 两次执行结果的 diff 对比 | P2 |
| 链路分组/标签 | 按业务场景分组管理 | P2 |
| 全文搜索 | 对历史日志结果全文检索 | P3 |
| 智能参数提取 | 从上一步 msg 中正则提取字段传递给下一节点 | P1 |
| 节点健康度指标 | 每个节点执行后自动判定红绿灯状态 | P1 |
| 并行执行模式 | 无依赖节点同时查询，加速排查 | P1 |
| 链路模板分享 | 导出链路配置（自动剥离Token），开发者间复用排查经验 | P1 |
| 排查知识库 | 记录排查结论，关联链路+供应商，沉淀排查经验 | P2 |
| 快速查询入口 | 不编排也好用，三步查日志 | P0 |

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

-- 供应商关联服务
CREATE TABLE supplier_service (
    supplier_id INTEGER REFERENCES supplier(id),
    sky_app_id  INTEGER REFERENCES sky_app(id),
    PRIMARY KEY (supplier_id, sky_app_id)
);

-- 链路流程
CREATE TABLE trace_flow (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,             -- 如 "铂涛Mapping未生效排查"
    description TEXT,
    supplier_id INTEGER REFERENCES supplier(id),
    tags        TEXT,                      -- JSON array
    is_favorite BOOLEAN DEFAULT FALSE,
    sort_order  INTEGER DEFAULT 0,
    flow_data   TEXT NOT NULL,             -- JSON: nodes + edges + input_schema
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
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

**flow_data JSON 结构：**
```json
{
  "input_schema": [
    { "key": "hotelId", "label": "酒店ID", "type": "string", "required": true },
    { "key": "supplierName", "label": "供应商", "type": "string", "default": "铂涛" }
  ],
  "nodes": [
    {
      "id": "node_1",
      "type": "skynet_query",
      "label": "查 RMQ consume",
      "position": { "x": 100, "y": 200 },
      "config": {
        "app_id": "106676",
        "module": "",
        "category": "RMQConsume",
        "filter1": "{{input.hotelId}}",
        "time_range": { "from": "now-30m", "to": "now" },
        "page_size": 100
      }
    },
    {
      "id": "node_2",
      "type": "info",
      "label": "检查 Redis 品牌数据",
      "position": { "x": 100, "y": 400 },
      "config": {
        "content": "确认 Redis key: botao_brand_{{input.hotelId}}",
        "links": [
          { "label": "Redis 管理台", "url": "https://redis-admin.17usoft.com/..." }
        ]
      }
    },
    {
      "id": "node_3",
      "type": "condition",
      "label": "日志是否存在?",
      "position": { "x": 100, "y": 600 },
      "config": {
        "expression": "{{node.node_1.result.count}} > 0",
        "true_target": "node_4",
        "false_target": "node_5"
      }
    }
  ],
  "edges": [
    { "source": "node_1", "target": "node_2" },
    { "source": "node_2", "target": "node_3" }
  ]
}
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

### P0 - 核心 MVP（预计 2-3 周）
- [ ] Tauri 2.0 + Vue 3 项目脚手架搭建
- [ ] SQLite 数据层（sky_app、trace_flow 表）
- [ ] 天网 API 查询封装（Rust 侧 HTTP 客户端）
- [ ] 基础链路编排（列表式，非拖拽，顺序节点）
- [ ] 参数模板变量解析引擎
- [ ] 链路执行器（顺序执行所有节点）
- [ ] 日志结果展示面板
- [ ] 天网 UI 链接生成与跳转

### P1 - 增强体验（预计 2-3 周）
- [ ] 供应商维度管理
- [ ] 可视化拖拽编排（Vue Flow）
- [ ] 链路复制/导入/导出
- [ ] 自定义信息节点（文本+链接）
- [ ] 快照生成与加载（AES-256 加密）
- [ ] 执行历史记录
- [ ] 日志关键字高亮
- [ ] 环境切换（UAT/PROD）

### P2 - 进阶功能（预计 2-3 周）
- [ ] 条件分支节点
- [ ] 链路执行报告导出
- [ ] 常用链路收藏/置顶
- [ ] 快捷键系统
- [ ] 定时巡检 + 桌面通知
- [ ] 执行结果 diff 对比
- [ ] 链路分组/标签

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
├─ InputSchema        定义用户需要填写的动态参数
├─ Node[]             节点列表
│   ├─ SkynetQueryNode    天网 API 查询
│   ├─ InfoNode           固定信息展示（文本+链接）
│   ├─ LinkNode           天网 UI 跳转链接
│   ├─ ConditionNode      条件分支判断
│   ├─ InputNode          运行时收集额外参数
│   └─ ExtractNode        从上一步结果提取数据
├─ Edge[]             节点连接关系（支持分支）
└─ HealthRules        节点健康度评估规则
```

### 10.2 执行引擎流程

```
用户填写 InputSchema 参数
          │
          ▼
  构建 ExecutionContext（执行上下文）
          │
          ▼
  拓扑排序获取执行顺序（支持并行分组）
          │
          ▼
  ┌───────────────────────────────┐
  │  逐节点/逐组执行：             │
  │  1. 解析模板变量 → 替换为实际值  │
  │  2. 执行节点逻辑               │
  │  3. 将结果写入 ExecutionContext │
  │  4. 评估健康度                 │
  │  5. 条件节点 → 决定下一跳       │
  │  6. 推送实时状态到前端          │
  └───────────────────────────────┘
          │
          ▼
  汇总所有节点结果 → 生成执行报告
```

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

### 10.4 模板变量引擎

**语法规则：**

| 语法 | 说明 | 示例 |
|------|------|------|
| `{{input.key}}` | 用户输入参数 | `{{input.hotelId}}` → `12345` |
| `{{node.ID.result.path}}` | 引用节点结果 | `{{node.node_1.result.count}}` → `5` |
| `{{node.ID.result.list[N].field}}` | 引用列表元素 | `{{node.node_1.result.list[0].msg}}` |
| `{{extract(source, regex)}}` | 正则提取 | `{{extract(node.node_1.result.list[0].msg, "orderId=(\\w+-\\d+)")}}` → `ORD-9876` |
| `{{time.now}}` | 当前时间 | `2024-03-15 10:23:45.000` |
| `{{time.now-30m}}` | 相对时间 | `2024-03-15 09:53:45.000` |
| `{{time.now-1h}}` | 相对时间 | `2024-03-15 09:23:45.000` |
| `{{env}}` | 当前环境 | `prod` |
| `{{value \| default}}` | 默认值 | `{{input.env \| "prod"}}` |

**解析流程（Rust 侧实现）：**
1. 正则扫描 `{{...}}` 占位符
2. 解析 scope（input/node/time/env/extract）
3. 从 ExecutionContext 中按路径取值
4. 如果是 extract 函数，执行正则匹配
5. 替换占位符为实际值
6. 未解析的变量 → 报错并标注位置

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
| 自动顺序 | 按拓扑序逐节点执行，自动进入下一步 | 常规排查 |
| 自动并行 | 无依赖的节点同时执行 | 需要同时查多个服务 |
| 手动逐步 | 每个节点需手动点击"执行"才进入下一步 | 需要中间检查、调试 |

并行执行的判定：根据 Edge 的拓扑关系，如果两个节点没有直接或间接的依赖关系，则可以并行。

### 10.7 SkynetQueryNode 详细配置

```json
{
  "id": "node_1",
  "type": "skynet_query",
  "label": "查 RMQ consume 日志",
  "config": {
    "sky_app_ref": "mapping",
    "query": {
      "module": "",
      "category": "RMQConsume",
      "subCategory": "",
      "filter1": "{{input.hotelId}}",
      "filter2": "",
      "filter1s": [],
      "filter2s": [],
      "modules": [],
      "categories": [],
      "subCategories": [],
      "contextId": "",
      "priority": "",
      "env": "{{env}}",
      "ips": [],
      "indexContext": "",
      "pageSize": 100,
      "beginTime": "{{time.now-30m}}",
      "endTime": "{{time.now}}"
    },
    "auto_page": false,
    "max_pages": 1,
    "generate_ui_link": true
  },
  "health_rules": {
    "ok": "result.count > 0",
    "error": "result.count == 0"
  }
}
```

`sky_app_ref` 引用 sky_app 表的记录，避免在节点中重复存储 appId/token。

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
