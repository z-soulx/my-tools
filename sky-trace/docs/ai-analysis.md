# AI 分析功能

SkyTrace 的 AI 分析基于一个核心思路：**用户编排的排查流程获取真实数据 + AI 解读**。AI 不替代排查，而是在用户精心编排的上下文（流程说明 + 节点提示词 + 实际查询结果）之上做归因和建议。

---

## 快速上手

### 1. 启用 AI

在飞书多维表格中填写 5 个 AI 字段（详见 [feishu-bitable-setup.md](feishu-bitable-setup.md)）：

| 字段 | 示例 |
|------|------|
| `ai_enabled` | 勾选 |
| `ai_base_url` | `https://api.openai.com/v1` |
| `ai_token` | `sk-...` |
| `ai_model` | `gpt-4o-mini` |
| `ai_default_system_prompt` | （可选） |

重启客户端后，AI 按钮自动亮起。

### 2. 编排提示词

在流程编排页面：

- **流程级提示词** — 顶部紫色卡片，描述整体业务背景（如"本流程排查铂涛 Mapping 不生效，关注 priority<=1"）
- **节点级提示词** — 每个节点编辑面板底部，描述该节点产出的判断标准
- **快捷问题** — 自定义「AI 分析」弹窗中的预设按钮（留空则用默认值）
- **显示控制** — "执行界面展示详情"开关控制执行页是否展示提示词原文

### 3. 使用 AI 分析

执行流程后：

- 点击 **"AI 全局分析"** — 基于全部已执行节点数据，由 AI 给出跨节点归因
- 点击节点结果旁的 **"✨ AI 解读"** — 聚焦单个节点的数据分析
- 输入自定义问题或点击快捷按钮发起分析
- AI 回复支持 Markdown 格式（标题、列表、代码块、表格等）

---

## 架构设计

### 安全模型

```
飞书 Bitable ──远程配置──> Rust 进程内存 (AiConfig)
                              │
                              │  token 仅在这里
                              │
                              v
前端 invoke("ai_chat_stream") ──> Rust POST /chat/completions
                                       │
              Tauri events <───────────┘
         ai:chunk / ai:done / ai:error
```

Token 不进 webview、不持久化、不写日志。前端只能看到 `AiStatus { enabled, hasToken, model, baseUrl }`。

### 流式传输

使用 Tauri 事件系统（而非 invoke 返回值）实现 SSE 流式传输：

1. 前端生成 `sessionId = crypto.randomUUID()`
2. 注册三个 listener：`ai:chunk:{id}`, `ai:done:{id}`, `ai:error:{id}`
3. 调用 `invoke("ai_chat_stream", { sessionId, messages })`
4. Rust 端解析 SSE `data: {...}` 行，提取 `choices[0].delta.content`，通过 `app.emit()` 推送
5. 前端累积文本 → `marked.parse()` → `v-html` 渲染

### Prompt 组装

`src/services/aiContext.ts` 按层级拼接 system prompt：

```
[远程默认 prompt]           ← remoteConfig.aiDefaultSystemPrompt
[流程上下文]                ← flow.name + flow.description
[流程级提示词]              ← flow.aiPrompt
[聚焦节点 + 节点提示词]     ← 仅节点分析时
```

User message 包含：
- `dynamicValues`（用户填写的参数）
- 每个已执行节点的摘要（requestParams + logs/jcpResult + extractedParams）
- 用户的分析需求

截断策略：每节点最多 30 条日志、每条 msg 最多 500 字符、JCP 响应最多 2KB。

---

## 数据模型

### trace_flow 表新增列

| 列名 | 类型 | 说明 |
|------|------|------|
| `ai_prompt` | TEXT | 流程级 AI 提示词 |
| `ai_quick_actions` | TEXT | JSON 数组，全局分析快捷按钮 |
| `ai_hint_collapsed` | INTEGER | 是否折叠执行页提示词详情 |

### TraceNode（JSON 内嵌字段）

| 字段 | 说明 |
|------|------|
| `aiPrompt` | 节点级 AI 提示词 |
| `aiQuickActions` | 节点分析快捷按钮 |

所有新增字段均使用 `#[serde(default)]`，向后兼容旧数据和旧快照。

### 快照兼容

- `SnapshotData.flows` 包含 `TraceFlow`，新字段随 serde 自动序列化
- 旧快照导入时，缺失字段取默认值（空/false）
- AI token 不进快照（仅在 Rust 进程内存中）

---

## 文件清单

| 文件 | 职责 |
|------|------|
| `src-tauri/src/ai/mod.rs` | 模块入口 |
| `src-tauri/src/ai/config.rs` | `AiConfig` 内存缓存 + `set_cached` / `get_cached` |
| `src-tauri/src/ai/client.rs` | `chat_stream()` SSE 流式调用 + `status()` |
| `src-tauri/src/remote_config.rs` | 5 个 AI 字段的解析 |
| `src-tauri/src/commands/mod.rs` | `ai_chat_stream` / `ai_status` 命令 |
| `src-tauri/src/storage/models.rs` | `TraceFlow.ai_prompt` / `ai_quick_actions` / `ai_hint_collapsed`; `TraceNode.ai_prompt` / `ai_quick_actions` |
| `src-tauri/src/storage/db.rs` | 3 列迁移 + SELECT/INSERT/UPDATE |
| `src/types/index.ts` | `AiStatus`, `ChatMessage`, TraceFlow/TraceNode AI 字段 |
| `src/services/tauri.ts` | `aiChatStream()`, `aiStatus()` |
| `src/services/aiContext.ts` | `buildGlobalAnalysisMessages()`, `buildNodeAnalysisMessages()` |
| `src/stores/app.ts` | `aiStatus`, `aiAvailable`, `refreshAiStatus()` |
| `src/views/FlowDetail.vue` | 全局/节点 AI 弹窗、编排区 AI 卡片、执行区只读提示 |
| `src/components/NodeEditor.vue` | 节点 AI 提示词 + 快捷问题编辑 |
| `src/styles.css` | `.ai-markdown` 样式 |
