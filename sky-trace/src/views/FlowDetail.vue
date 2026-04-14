<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import type {
  TraceFlow, TraceNode, NodeExecResult, DynamicParam,
  SkynetQueryConfig, InfoNodeConfig, ChecklistNodeConfig, FieldBinding,
} from "@/types";
import { resolveBinding, resolveRelativeTime, extractTemplateParams } from "@/types";
import NodeEditor from "@/components/NodeEditor.vue";
import NodeResult from "@/components/NodeResult.vue";
import DynamicParamEditor from "@/components/DynamicParamEditor.vue";
import FlowFormDialog from "@/components/FlowFormDialog.vue";
import TimeRangeSelector from "@/components/TimeRangeSelector.vue";

const route = useRoute();
const router = useRouter();
const store = useAppStore();

const flow = ref<TraceFlow | null>(null);
const executing = ref(false);
const execResults = ref<Record<string, NodeExecResult>>({});
const dynamicValues = ref<Record<string, string>>({});
const timeFrom = ref("now-30m");
const timeTo = ref("now");
const showNodeEditor = ref(false);
const showParamEditor = ref(false);
const editingNode = ref<TraceNode | null>(null);
const activeTab = ref<"execute" | "edit">("execute");
const globalSearch = ref("");
const allExpanded = ref(true);
const copyFeedback = ref("");
const showImportDialog = ref(false);
const importJson = ref("");
const selectedNodeIds = ref(new Set<string>());
const showGlobalAi = ref(false);
const globalAiPrompt = ref("");
const globalAiResponse = ref("");
const dragState = ref<{
  index: number;
  startY: number;
  currentY: number;
  active: boolean;
} | null>(null);
const dropTarget = ref<number | null>(null);
const nodeRefs = ref<HTMLElement[]>([]);
const expandedNotes = ref<Record<string, boolean>>({});
const expandedNodeContent = ref<Record<string, boolean>>({});
const customInputKeys = ref<Record<string, boolean>>({});
const showFlowInfoEditor = ref(false);
const paramErrors = ref<Record<string, boolean>>({});
const expandedHints = ref<Record<string, boolean>>({});

const flowId = computed(() => Number(route.params.id));

interface ParamUsage { nodeId: string; label: string; field: string }

const FIELD_LABELS: [string, keyof Pick<SkynetQueryConfig, "filter1" | "filter2" | "indexContext" | "contextId">][] = [
  ["Filter1", "filter1"],
  ["Filter2", "filter2"],
  ["模糊查询", "indexContext"],
  ["TraceId", "contextId"],
];

/** 参数 key → 使用该参数的节点及字段 */
const paramUsageMap = computed(() => {
  const map = new Map<string, ParamUsage[]>();
  if (!flow.value) return map;
  for (const node of flow.value.nodes) {
    if (node.type !== "skynet_query") continue;
    const cfg = node.config as SkynetQueryConfig;
    for (const [fieldLabel, fieldKey] of FIELD_LABELS) {
      const b = cfg[fieldKey] as FieldBinding | undefined;
      if (!b) continue;
      if (b.mode === "dynamic" && b.paramKey) {
        if (!map.has(b.paramKey)) map.set(b.paramKey, []);
        map.get(b.paramKey)!.push({ nodeId: node.id, label: node.label, field: fieldLabel });
      } else if (b.mode === "template" && b.templateValue) {
        for (const key of extractTemplateParams(b.templateValue)) {
          if (!map.has(key)) map.set(key, []);
          map.get(key)!.push({ nodeId: node.id, label: node.label, field: fieldLabel });
        }
      }
    }
  }
  return map;
});

/** 节点 id → 引用的动态参数列表 */
const nodeParamMap = computed(() => {
  const map = new Map<string, { key: string; label: string; field: string }[]>();
  if (!flow.value) return map;
  const paramLabelMap = new Map(flow.value.dynamicParams.map((p) => [p.key, p.label]));
  for (const node of flow.value.nodes) {
    if (node.type !== "skynet_query") continue;
    const cfg = node.config as SkynetQueryConfig;
    const entries: { key: string; label: string; field: string }[] = [];
    for (const [fieldLabel, fieldKey] of FIELD_LABELS) {
      const b = cfg[fieldKey] as FieldBinding | undefined;
      if (!b) continue;
      if (b.mode === "dynamic" && b.paramKey) {
        entries.push({ key: b.paramKey, label: paramLabelMap.get(b.paramKey) || b.paramKey, field: fieldLabel });
      } else if (b.mode === "template" && b.templateValue) {
        for (const key of extractTemplateParams(b.templateValue)) {
          entries.push({ key, label: paramLabelMap.get(key) || key, field: fieldLabel });
        }
      }
    }
    if (entries.length) map.set(node.id, entries);
  }
  return map;
});

const selectAllNodes = computed({
  get: () => {
    const qNodes = flow.value?.nodes.filter((n) => n.type === "skynet_query") ?? [];
    return qNodes.length > 0 && qNodes.every((n) => selectedNodeIds.value.has(n.id));
  },
  set: (val: boolean) => {
    const qNodes = flow.value?.nodes.filter((n) => n.type === "skynet_query") ?? [];
    if (val) qNodes.forEach((n) => selectedNodeIds.value.add(n.id));
    else selectedNodeIds.value.clear();
  },
});

function toggleNodeSelection(nodeId: string) {
  selectedNodeIds.value.has(nodeId)
    ? selectedNodeIds.value.delete(nodeId)
    : selectedNodeIds.value.add(nodeId);
}

onMounted(async () => {
  try {
    if (store.snapshotMode) {
      const found = store.flows.find((f) => f.id === flowId.value);
      if (!found) throw new Error("not found");
      flow.value = JSON.parse(JSON.stringify(found));
    } else {
      flow.value = await api.getFlow(flowId.value);
    }
    flow.value!.dynamicParams.forEach((p: DynamicParam) => {
      dynamicValues.value[p.key] = p.defaultValue || "";
      // 如果有默认值且不在预定义选项中，自动切换为自定义输入模式
      if (p.options?.length && p.defaultValue && !matchesOption(p.options, p.defaultValue)) {
        customInputKeys.value[p.key] = true;
      }
    });
    flow.value!.nodes
      .filter((n) => n.type === "skynet_query")
      .forEach((n) => selectedNodeIds.value.add(n.id));
  } catch {
    router.push("/flows");
  }
});

function addNode() {
  editingNode.value = null;
  showNodeEditor.value = true;
}

function editNode(node: TraceNode) {
  editingNode.value = { ...node };
  showNodeEditor.value = true;
}

async function persistFlow() {
  if (!flow.value) return;
  await api.saveFlow({
    id: flow.value.id,
    name: flow.value.name,
    description: flow.value.description,
    supplierId: flow.value.supplierId,
    tags: flow.value.tags,
    dynamicParams: flow.value.dynamicParams,
    nodes: flow.value.nodes,
  });
}

async function saveNode(node: TraceNode) {
  if (!flow.value) return;
  const idx = flow.value.nodes.findIndex((n) => n.id === node.id);
  if (idx >= 0) {
    flow.value.nodes[idx] = node;
  } else {
    node.sortOrder = flow.value.nodes.length;
    flow.value.nodes.push(node);
  }
  await persistFlow();
  showNodeEditor.value = false;
}

async function removeNode(nodeId: string) {
  if (!flow.value) return;
  flow.value.nodes = flow.value.nodes.filter((n) => n.id !== nodeId);
  await persistFlow();
}

function onPointerDown(e: PointerEvent, index: number) {
  const target = e.target as HTMLElement;
  if (!target.closest("[data-drag-handle]")) return;
  e.preventDefault();
  dragState.value = { index, startY: e.clientY, currentY: e.clientY, active: false };
  document.addEventListener("pointermove", onPointerMove);
  document.addEventListener("pointerup", onPointerUp);
}

function onPointerMove(e: PointerEvent) {
  if (!dragState.value) return;
  dragState.value.currentY = e.clientY;
  if (!dragState.value.active && Math.abs(e.clientY - dragState.value.startY) > 5) {
    dragState.value.active = true;
  }
  if (!dragState.value.active) return;
  const els = nodeRefs.value;
  let target: number | null = null;
  for (let i = 0; i < els.length; i++) {
    if (!els[i]) continue;
    const rect = els[i].getBoundingClientRect();
    const midY = rect.top + rect.height / 2;
    if (e.clientY < midY) {
      target = i;
      break;
    }
  }
  if (target === null) target = els.length;
  dropTarget.value = target;
}

async function onPointerUp() {
  document.removeEventListener("pointermove", onPointerMove);
  document.removeEventListener("pointerup", onPointerUp);

  if (!dragState.value?.active || !flow.value || dropTarget.value === null) {
    dragState.value = null;
    dropTarget.value = null;
    return;
  }
  const from = dragState.value.index;
  const slot = dropTarget.value;
  dragState.value = null;
  dropTarget.value = null;

  if (slot === from || slot === from + 1) return;
  const nodes = flow.value.nodes;
  const [moved] = nodes.splice(from, 1);
  const insertAt = slot > from ? slot - 1 : slot;
  nodes.splice(insertAt, 0, moved);
  nodes.forEach((n, i) => { n.sortOrder = i; });
  await persistFlow();
}

async function pasteNodeAt(position: number) {
  try {
    const text = await navigator.clipboard.readText();
    const data = JSON.parse(text);
    if (data._type !== "sky_trace_node") throw new Error("格式不匹配");
    const node: TraceNode = {
      id: `node_${Date.now()}`,
      type: data.type,
      label: data.label + " (粘贴)",
      sortOrder: position,
      config: data.config,
      notes: data.notes,
    };
    flow.value?.nodes.splice(position, 0, node);
    flow.value?.nodes.forEach((n, i) => { n.sortOrder = i; });
    await persistFlow();
    showFeedback("已粘贴节点");
  } catch {
    showFeedback("粘贴失败：剪贴板内容非有效节点数据");
  }
}

async function saveDynamicParams(params: DynamicParam[]) {
  if (!flow.value) return;
  flow.value.dynamicParams = params;
  params.forEach((p) => {
    if (!(p.key in dynamicValues.value)) {
      dynamicValues.value[p.key] = p.defaultValue || "";
    }
  });
  await persistFlow();
  showParamEditor.value = false;
}

async function handleFlowInfoSaved(updated: TraceFlow) {
  if (!flow.value) return;
  flow.value.name = updated.name;
  flow.value.description = updated.description;
  flow.value.supplierId = updated.supplierId;
  flow.value.tags = updated.tags;
  showFlowInfoEditor.value = false;
  store.refreshFlows();
}

async function copyNode(node: TraceNode) {
  const data = { _type: "sky_trace_node", type: node.type, label: node.label, config: node.config, notes: node.notes };
  await clipCopy(JSON.stringify(data, null, 2), `node_${node.id}`);
}

async function pasteNode() {
  try {
    const text = await navigator.clipboard.readText();
    const data = JSON.parse(text);
    if (data._type !== "sky_trace_node") throw new Error("格式不匹配");
    const node: TraceNode = {
      id: `node_${Date.now()}`,
      type: data.type,
      label: data.label + " (粘贴)",
      sortOrder: flow.value?.nodes.length ?? 0,
      config: data.config,
      notes: data.notes,
    };
    flow.value?.nodes.push(node);
    await persistFlow();
    showFeedback("已粘贴节点");
  } catch {
    showFeedback("粘贴失败：剪贴板内容非有效节点数据");
  }
}

async function exportFlow() {
  if (!flow.value) return;
  const data = {
    _type: "sky_trace_flow",
    name: flow.value.name,
    description: flow.value.description,
    tags: flow.value.tags,
    dynamicParams: flow.value.dynamicParams,
    nodes: flow.value.nodes.map(({ type, label, config, notes }) => ({ type, label, config, notes })),
  };
  await clipCopy(JSON.stringify(data, null, 2), "flow_export");
}

async function importFlow() {
  try {
    const data = JSON.parse(importJson.value);
    if (data._type !== "sky_trace_flow") throw new Error("格式不匹配");
    const imported = await api.saveFlow({
      name: (data.name || "导入链路") + " (导入)",
      description: data.description || "",
      supplierId: flow.value?.supplierId ?? null,
      tags: data.tags || [],
      dynamicParams: data.dynamicParams || [],
      nodes: (data.nodes || []).map((n: { type: string; label: string; config: unknown; notes?: string }, i: number) => ({
        id: `node_${Date.now()}_${i}`,
        type: n.type,
        label: n.label,
        sortOrder: i,
        config: n.config,
        notes: n.notes,
      })),
    });
    showImportDialog.value = false;
    importJson.value = "";
    router.push(`/flows/${imported.id}`);
    showFeedback("链路已导入");
  } catch {
    showFeedback("导入失败：JSON 格式无效");
  }
}

async function clipCopy(text: string, id: string) {
  try {
    await navigator.clipboard.writeText(text);
    showFeedback(id.startsWith("node") ? "节点配置已复制" : "链路已复制到剪贴板");
  } catch { /* clipboard not available */ }
}

function showFeedback(msg: string) {
  copyFeedback.value = msg;
  setTimeout(() => { if (copyFeedback.value === msg) copyFeedback.value = ""; }, 2000);
}

function resolveField(binding: FieldBinding): string {
  return resolveBinding(binding, dynamicValues.value);
}

async function executeNodes(onlySelected = false) {
  if (!flow.value) return;

  // 校验必填参数
  const errors: Record<string, boolean> = {};
  for (const param of flow.value.dynamicParams) {
    if (param.required && !dynamicValues.value[param.key]?.trim()) {
      errors[param.key] = true;
    }
  }
  paramErrors.value = errors;
  if (Object.keys(errors).length > 0) {
    showFeedback("请填写所有必填参数（标 * 项）");
    return;
  }

  executing.value = true;
  execResults.value = {};

  const targetNodeIds = onlySelected
    ? new Set(selectedNodeIds.value)
    : new Set(flow.value.nodes.map((n) => n.id));

  flow.value.nodes.filter((n) => targetNodeIds.has(n.id)).forEach((node) => {
    execResults.value[node.id] = {
      nodeId: node.id,
      status: "running",
      health: "unknown",
      durationMs: 0,
      result: null,
      uiLink: "",
      error: "",
    };
  });

  const queryNodes = flow.value.nodes.filter((n) => n.type === "skynet_query" && targetNodeIds.has(n.id));

  const promises = queryNodes.map(async (node) => {
    const start = Date.now();
    const cfg = node.config as SkynetQueryConfig;
    const skyApp = store.skyAppMap.get(cfg.skyAppId);

    const resolvedBegin = resolveRelativeTime(timeFrom.value);
    const resolvedEnd = resolveRelativeTime(timeTo.value);

    const queryParams: Record<string, unknown> = skyApp ? {
      module: cfg.module,
      category: cfg.category,
      subCategory: cfg.subCategory,
      filter1: resolveField(cfg.filter1),
      filter2: resolveField(cfg.filter2),
      indexContext: resolveField(cfg.indexContext),
      contextId: resolveField(cfg.contextId ?? { mode: "fixed", fixedValue: "", paramKey: "", templateValue: "" }),
      pageSize: cfg.pageSize,
      beginTime: resolvedBegin,
      endTime: resolvedEnd,
    } : { error: `未找到天网应用配置 (id=${cfg.skyAppId})` };

    try {
      if (!skyApp) throw new Error(`未找到天网应用配置 (id=${cfg.skyAppId})`);

      const result = await api.querySkynetLog(skyApp.appId, skyApp.token, queryParams);

      const uiLink = await api.generateSkynetUiLink(skyApp.appUk, {
        module: cfg.module,
        category: cfg.category,
        subCategory: cfg.subCategory,
        filter1: resolveField(cfg.filter1),
        filter2: resolveField(cfg.filter2),
        indexContext: resolveField(cfg.indexContext),
        contextId: resolveField(cfg.contextId ?? { mode: "fixed", fixedValue: "", paramKey: "", templateValue: "" }),
        beginTime: timeFrom.value,
        endTime: timeTo.value,
      });

      const count = result?.result?.count ?? 0;
      const hasError = result?.result?.list?.some((item) => item.priority <= 1);

      execResults.value[node.id] = {
        nodeId: node.id,
        status: "success",
        health: count === 0 ? "error" : hasError ? "warning" : "ok",
        durationMs: Date.now() - start,
        result,
        uiLink,
        error: "",
        requestParams: queryParams,
      };
    } catch (err: unknown) {
      execResults.value[node.id] = {
        nodeId: node.id,
        status: "error",
        health: "error",
        durationMs: Date.now() - start,
        result: null,
        uiLink: "",
        error: String(err),
        requestParams: queryParams,
      };
    }
  });

  flow.value.nodes
    .filter((n) => n.type !== "skynet_query" && targetNodeIds.has(n.id))
    .forEach((node) => {
      execResults.value[node.id] = {
        nodeId: node.id,
        status: "success",
        health: "ok",
        durationMs: 0,
        result: null,
        uiLink: "",
        error: "",
      };
    });

  await Promise.allSettled(promises);
  executing.value = false;
}

function healthIcon(health: string) {
  return health === "ok" ? "🟢" : health === "warning" ? "🟡" : health === "error" ? "🔴" : "⚪";
}

function nodeTypeLabel(type: string) {
  return type === "skynet_query" ? "天网查询" : type === "checklist" ? "Checklist" : type === "info" ? "信息" : "链接";
}

function toggleNotes(nodeId: string) {
  expandedNotes.value[nodeId] = !expandedNotes.value[nodeId];
}

function toggleNodeContent(nodeId: string) {
  expandedNodeContent.value[nodeId] = !expandedNodeContent.value[nodeId];
}

/** 解析选项 "value|label" 格式，无 | 则 value = label */
function parseOption(opt: string): { value: string; label: string } {
  const idx = opt.indexOf("|");
  if (idx >= 0) return { value: opt.slice(0, idx), label: opt.slice(idx + 1) };
  return { value: opt, label: opt };
}

/** 判断当前值是否匹配某个选项的 value */
function matchesOption(options: string[], val: string): boolean {
  return options.some((opt) => parseOption(opt).value === val);
}

function onOptionSelect(paramKey: string, value: string) {
  if (value === "__custom__") {
    customInputKeys.value[paramKey] = true;
    dynamicValues.value[paramKey] = "";
  } else {
    customInputKeys.value[paramKey] = false;
    dynamicValues.value[paramKey] = value;
  }
}
</script>

<template>
  <div v-if="flow" class="flex flex-col h-full">
    <!-- 顶部栏 -->
    <header class="px-6 py-4 bg-surface border-b border-border shrink-0">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <button class="text-text-secondary hover:text-text" @click="router.push('/flows')">← 返回</button>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-xl font-semibold">{{ flow.name }}</h2>
              <button
                v-if="!store.snapshotMode"
                class="text-xs text-text-secondary/50 hover:text-primary transition-colors"
                title="编辑基本信息"
                @click="showFlowInfoEditor = true"
              >✏️</button>
            </div>
            <p v-if="flow.description" class="text-sm text-text-secondary">{{ flow.description }}</p>
            <div v-if="flow.tags.length || flow.supplierId" class="flex items-center gap-1.5 mt-1">
              <span v-if="flow.supplierId" class="text-[10px] px-1.5 py-0.5 bg-green-50 text-green-700 rounded">{{ store.supplierMap.get(flow.supplierId)?.name }}</span>
              <span v-for="tag in flow.tags" :key="tag" class="text-[10px] px-1.5 py-0.5 bg-surface-alt text-text-secondary rounded">{{ tag }}</span>
            </div>
          </div>
        </div>
        <div v-if="!store.snapshotMode || !store.snapshotRestrictions.hideEdit" class="flex bg-surface-alt rounded-lg p-0.5">
          <button
            class="px-3 py-1.5 text-sm rounded-md transition-colors"
            :class="activeTab === 'execute' ? 'bg-surface shadow-sm text-text' : 'text-text-secondary'"
            @click="activeTab = 'execute'"
          >执行</button>
          <button
            class="px-3 py-1.5 text-sm rounded-md transition-colors"
            :class="activeTab === 'edit' ? 'bg-surface shadow-sm text-text' : 'text-text-secondary'"
            @click="activeTab = 'edit'"
          >编排</button>
        </div>
      </div>
    </header>

    <!-- 执行模式 -->
    <div v-if="activeTab === 'execute'" class="flex-1 overflow-y-auto p-6">
      <!-- 动态参数 + 时间 -->
      <div class="bg-surface rounded-xl border border-border p-4 mb-6 space-y-4">
        <div v-if="flow.dynamicParams.length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          <div v-for="param in flow.dynamicParams" :key="param.key" class="min-w-0">
            <label class="block text-xs text-text-secondary mb-1 truncate" :title="param.label">
              {{ param.label }} <span v-if="param.required" class="text-error">*</span>
            </label>
            <!-- 有 options 且不允许自定义：纯 select -->
            <select
              v-if="param.options?.length && param.allowCustom === false"
              v-model="dynamicValues[param.key]"
              class="w-full px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
              :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
              @change="paramErrors[param.key] = false"
            >
              <option value="" disabled>请选择</option>
              <option v-for="opt in param.options" :key="opt" :value="parseOption(opt).value">{{ parseOption(opt).label }}</option>
            </select>
            <!-- 有 options 且允许自定义：select + 自定义切换 -->
            <template v-else-if="param.options?.length">
              <select
                v-if="!customInputKeys[param.key]"
                :value="matchesOption(param.options, dynamicValues[param.key]) ? dynamicValues[param.key] : ''"
                class="w-full px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
                :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
                @change="onOptionSelect(param.key, ($event.target as HTMLSelectElement).value); paramErrors[param.key] = false"
              >
                <option value="" disabled>请选择</option>
                <option v-for="opt in param.options" :key="opt" :value="parseOption(opt).value">{{ parseOption(opt).label }}</option>
                <option value="__custom__">自定义...</option>
              </select>
              <div v-else class="flex gap-1">
                <input
                  v-model="dynamicValues[param.key]"
                  :placeholder="param.defaultValue || '自定义输入'"
                  class="flex-1 min-w-0 px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
                  :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
                  @input="paramErrors[param.key] = false"
                />
                <button
                  class="shrink-0 px-2 text-xs text-text-secondary hover:text-primary border border-border rounded-lg"
                  @click="customInputKeys[param.key] = false"
                >选项</button>
              </div>
            </template>
            <!-- 无 options：普通 input -->
            <input
              v-else
              v-model="dynamicValues[param.key]"
              :placeholder="param.defaultValue || param.label"
              class="w-full px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
              :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
              @input="paramErrors[param.key] = false"
            />
            <p v-if="paramErrors[param.key]" class="mt-0.5 text-[10px] text-error">此参数为必填项</p>
            <template v-else-if="param.hint">
              <div
                class="mt-0.5 text-[10px] text-text-secondary/70 break-all cursor-pointer select-none"
                :class="expandedHints[param.key] ? 'whitespace-pre-wrap' : 'line-clamp-2'"
                :title="expandedHints[param.key] ? '点击收起' : '点击展开完整提示'"
                @click="expandedHints[param.key] = !expandedHints[param.key]"
              >{{ param.hint }}</div>
            </template>
            <div v-if="paramUsageMap.get(param.key)?.length" class="mt-1 flex flex-wrap gap-1">
              <span
                v-for="u in paramUsageMap.get(param.key)"
                :key="u.nodeId + u.field"
                class="inline-flex items-center text-[10px] px-1.5 py-0.5 bg-blue-50 text-primary/80 rounded truncate max-w-full"
                :title="`${u.label} 的 ${u.field} 字段`"
              >{{ u.label }} · {{ u.field }}</span>
            </div>
            <div v-else class="mt-1 text-[10px] text-text-secondary/50">未被任何节点引用</div>
          </div>
        </div>

        <TimeRangeSelector v-model:model-from="timeFrom" v-model:model-to="timeTo" />

        <div class="flex items-center gap-3 flex-wrap">
          <button
            class="px-5 py-2.5 bg-primary text-white rounded-lg hover:bg-primary-hover transition-colors disabled:opacity-50 font-medium text-sm"
            :disabled="executing"
            @click="executeNodes(false)"
          >{{ executing ? '执行中...' : '▶ 全部执行' }}</button>
          <button
            class="px-5 py-2.5 border border-primary text-primary rounded-lg hover:bg-blue-50 transition-colors disabled:opacity-50 font-medium text-sm"
            :disabled="executing || selectedNodeIds.size === 0"
            @click="executeNodes(true)"
          >▶ 执行选中 ({{ selectedNodeIds.size }})</button>
          <button
            v-if="Object.values(execResults).some(r => r.status === 'success')"
            class="px-4 py-2.5 border border-violet-300 text-violet-600 rounded-lg hover:bg-violet-50 transition-colors font-medium text-sm"
            @click="showGlobalAi = true"
          >AI 全局分析</button>
          <span v-if="Object.keys(execResults).length > 0" class="text-sm text-text-secondary">
            {{ Object.values(execResults).filter(r => r.status === 'success').length }}
            / {{ Object.keys(execResults).length }} 完成
          </span>
        </div>
      </div>

      <!-- 全局工具栏 -->
      <div v-if="Object.keys(execResults).length > 0" class="flex items-center gap-2 mb-4">
        <button
          class="text-xs px-3 py-1.5 rounded-lg border transition-colors"
          :class="allExpanded ? 'border-border text-text-secondary' : 'border-primary text-primary bg-blue-50'"
          @click="allExpanded = !allExpanded"
        >{{ allExpanded ? '全部收起' : '全部展开' }}</button>
        <div class="flex items-center gap-1 flex-1 max-w-sm">
          <input
            v-model="globalSearch"
            placeholder="跨节点搜索日志..."
            class="flex-1 px-3 py-1.5 text-xs border border-border rounded-lg outline-none focus:border-primary"
            @keydown.escape="globalSearch = ''"
          />
          <button
            v-if="globalSearch"
            class="text-xs text-text-secondary hover:text-text px-1"
            @click="globalSearch = ''"
          >×</button>
        </div>
      </div>

      <!-- 节点选择与结果 -->
      <div class="flex items-center gap-2 mb-4">
        <label class="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer select-none">
          <input type="checkbox" v-model="selectAllNodes" class="rounded" />
          全选查询节点 ({{ selectedNodeIds.size }}/{{ flow.nodes.filter(n => n.type === 'skynet_query').length }})
        </label>
      </div>

      <div class="space-y-4">
        <div
          v-for="node in flow.nodes"
          :key="node.id"
          class="bg-surface rounded-xl border overflow-hidden"
          :class="node.type === 'skynet_query' && selectedNodeIds.has(node.id) ? 'border-primary/40' : 'border-border'"
        >
          <div class="px-4 py-3 bg-surface-alt border-b border-border">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <input
                  v-if="node.type === 'skynet_query'"
                  type="checkbox"
                  :checked="selectedNodeIds.has(node.id)"
                  class="rounded"
                  @change="toggleNodeSelection(node.id)"
                />
                <span v-if="execResults[node.id]">{{ healthIcon(execResults[node.id].health) }}</span>
                <span v-else>⚪</span>
                <span class="font-medium text-sm">{{ node.label }}</span>
                <span class="text-xs px-2 py-0.5 bg-surface rounded-full text-text-secondary">{{ nodeTypeLabel(node.type) }}</span>
              </div>
              <div class="flex items-center gap-2 text-xs text-text-secondary">
                <span v-if="execResults[node.id]?.durationMs">{{ execResults[node.id].durationMs }}ms</span>
                <a v-if="execResults[node.id]?.uiLink && !(store.snapshotMode && store.snapshotRestrictions.hideUiLink)" :href="execResults[node.id].uiLink" target="_blank" class="text-primary hover:underline">天网UI ↗</a>
              </div>
            </div>
            <div v-if="nodeParamMap.get(node.id)?.length" class="mt-1.5 flex flex-wrap gap-1">
              <span
                v-for="p in nodeParamMap.get(node.id)"
                :key="p.key + p.field"
                class="inline-flex items-center text-[10px] px-1.5 py-0.5 rounded border border-primary/20 text-primary/70 bg-blue-50/50"
              >{{ p.field }} ← {{ p.label }}</span>
            </div>
          </div>
          <!-- 可折叠参考信息 -->
          <div v-if="node.notes || (node.type === 'skynet_query' && Object.values((node.config as SkynetQueryConfig).fieldHints ?? {}).some(v => v))" class="px-4 py-2 border-b border-border/50 bg-amber-50/30">
            <button
              class="flex items-center gap-1 text-xs text-amber-700 hover:text-amber-900"
              @click="toggleNotes(node.id)"
            >
              <span>{{ expandedNotes[node.id] ? '▾' : '▸' }}</span>
              参考信息
            </button>
            <div v-if="expandedNotes[node.id]" class="mt-2 space-y-2">
              <div v-if="node.notes" class="text-xs text-text-secondary whitespace-pre-wrap leading-relaxed">{{ node.notes }}</div>
              <template v-if="node.type === 'skynet_query'">
                <div
                  v-for="[hintLabel, hintKey] in ([['Filter1', 'filter1'], ['Filter2', 'filter2'], ['Msg 模糊查询', 'indexContext'], ['TraceId', 'contextId']] as const)"
                  :key="hintKey"
                >
                  <div v-if="(node.config as SkynetQueryConfig).fieldHints?.[hintKey]" class="flex items-start gap-1.5 text-[11px]">
                    <span class="text-amber-600 font-medium shrink-0">{{ hintLabel }}:</span>
                    <span class="text-text-secondary">{{ (node.config as SkynetQueryConfig).fieldHints![hintKey] }}</span>
                  </div>
                </div>
              </template>
            </div>
          </div>
          <NodeResult
            v-if="execResults[node.id] && node.type === 'skynet_query'"
            :node="node"
            :result="execResults[node.id]"
            :global-search="globalSearch"
            :force-expand="allExpanded"
          />
          <div v-else-if="node.type === 'info'">
            <button class="w-full px-4 py-2 flex items-center gap-1 text-xs text-text-secondary hover:bg-surface-alt transition-colors" @click="toggleNodeContent(node.id)">
              <span>{{ expandedNodeContent[node.id] ? '▾' : '▸' }}</span>
              信息内容
            </button>
            <div v-if="expandedNodeContent[node.id]" class="px-4 pb-3">
              <p class="text-sm text-text-secondary whitespace-pre-wrap">{{ (node.config as InfoNodeConfig).content }}</p>
              <div v-if="(node.config as InfoNodeConfig).links?.length" class="mt-2 flex flex-wrap gap-2">
                <a v-for="(link, i) in (node.config as InfoNodeConfig).links" :key="i" :href="link.url" target="_blank" class="text-xs text-primary hover:underline">{{ link.label }} ↗</a>
              </div>
            </div>
          </div>
          <div v-else-if="node.type === 'checklist'">
            <button class="w-full px-4 py-2 flex items-center gap-1 text-xs text-text-secondary hover:bg-surface-alt transition-colors" @click="toggleNodeContent(node.id)">
              <span>{{ expandedNodeContent[node.id] ? '▾' : '▸' }}</span>
              检查项
            </button>
            <div v-if="expandedNodeContent[node.id]" class="px-4 pb-3">
            <template v-if="store.checklistMap.get((node.config as ChecklistNodeConfig).checklistGroupId)">
              <div
                v-for="item in (
                  (node.config as ChecklistNodeConfig).itemIds.length
                    ? store.checklistMap.get((node.config as ChecklistNodeConfig).checklistGroupId)!.items.filter(
                        (i) => (node.config as ChecklistNodeConfig).itemIds.includes(i.id)
                      )
                    : store.checklistMap.get((node.config as ChecklistNodeConfig).checklistGroupId)!.items
                )"
                :key="item.id"
                class="mb-2 last:mb-0"
              >
                <div class="text-sm font-medium">{{ item.label }}</div>
                <p v-if="item.description" class="text-xs text-text-secondary">{{ item.description }}</p>
                <div v-if="item.links.length" class="flex flex-wrap gap-2 mt-1">
                  <a
                    v-for="link in item.links"
                    :key="link.id"
                    :href="link.url"
                    target="_blank"
                    class="text-xs text-primary hover:underline"
                  >🔗 {{ link.label }} ↗</a>
                </div>
              </div>
            </template>
            <p v-else class="text-sm text-text-secondary">Checklist 分组未找到</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 编排模式 -->
    <div v-else class="flex-1 overflow-y-auto p-6">
      <!-- 动态参数管理 -->
      <div class="bg-surface rounded-xl border border-border p-4 mb-6">
        <div class="flex items-center justify-between mb-3">
          <h3 class="font-medium text-sm">动态参数 ({{ flow.dynamicParams.length }})</h3>
          <button class="text-xs text-primary hover:underline" @click="showParamEditor = true">编辑参数</button>
        </div>
        <div v-if="flow.dynamicParams.length === 0" class="text-sm text-text-secondary">
          暂未定义动态参数。动态参数会在执行时由用户填写，绑定到节点的 Filter1/Filter2/模糊查询中。
        </div>
        <div v-else class="space-y-2">
          <div v-for="p in flow.dynamicParams" :key="p.key" class="flex items-start gap-2">
            <span class="shrink-0 px-2 py-1 bg-blue-50 text-primary text-xs rounded-lg">
              {{ p.label }} ({{ p.key }})
              <span v-if="p.required" class="text-error ml-1">*</span>
            </span>
            <div class="flex flex-wrap gap-1 pt-0.5">
              <span
                v-for="u in (paramUsageMap.get(p.key) || [])"
                :key="u.nodeId + u.field"
                class="text-[10px] px-1.5 py-0.5 bg-surface-alt text-text-secondary rounded border border-border"
              >→ {{ u.label }} · {{ u.field }}</span>
              <span v-if="!paramUsageMap.get(p.key)?.length" class="text-[10px] text-text-secondary/50 pt-0.5">未被引用</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 流程操作 -->
      <div class="flex items-center gap-2 mb-6">
        <button class="text-xs px-3 py-1.5 rounded-lg border border-border text-text-secondary hover:border-primary hover:text-primary transition-colors" @click="exportFlow">导出链路</button>
        <button class="text-xs px-3 py-1.5 rounded-lg border border-border text-text-secondary hover:border-primary hover:text-primary transition-colors" @click="showImportDialog = true">导入链路</button>
        <button class="text-xs px-3 py-1.5 rounded-lg border border-border text-text-secondary hover:border-primary hover:text-primary transition-colors" @click="pasteNode">粘贴节点</button>
      </div>

      <!-- 节点列表 -->
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-medium">查询节点 ({{ flow.nodes.length }})</h3>
        <button class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover" @click="addNode">+ 添加节点</button>
      </div>

      <div v-if="flow.nodes.length === 0" class="text-center py-12 text-text-secondary">
        <p class="text-lg">暂无查询节点</p>
        <p class="text-sm mt-1">添加天网查询节点来构建排查链路</p>
      </div>

      <div class="space-y-1">
        <template v-for="(node, index) in flow.nodes" :key="node.id">
          <div
            v-if="index === 0"
            class="flex items-center justify-center transition-all"
            :class="dragState?.active && dropTarget === 0 && dragState.index !== 0 ? 'py-3 border-2 border-dashed border-primary bg-blue-50/40 rounded-lg' : 'py-1 group/insert'"
          >
            <span v-if="dragState?.active && dropTarget === 0 && dragState.index !== 0" class="text-xs text-primary font-medium">放置到此处</span>
            <button
              v-else
              class="text-[10px] px-2 py-0.5 rounded border border-dashed border-transparent text-transparent group-hover/insert:border-primary/30 group-hover/insert:text-primary/50 transition-colors"
              @click="pasteNodeAt(0)"
            >粘贴到此处</button>
          </div>

          <div
            :ref="(el) => { if (el) nodeRefs[index] = el as HTMLElement }"
            class="bg-surface rounded-lg border p-4 flex items-center justify-between group transition-all select-none"
            :class="[
              dragState?.active && dragState.index === index ? 'opacity-40 border-primary/50 scale-[0.98]' : 'border-border',
            ]"
            @pointerdown="onPointerDown($event, index)"
          >
            <div class="flex items-center gap-3 min-w-0">
              <div class="flex items-center gap-1 shrink-0">
                <span data-drag-handle class="text-text-secondary/40 cursor-grab select-none text-sm" title="拖拽排序">⠿</span>
                <span class="text-text-secondary text-xs text-center w-5">{{ index + 1 }}</span>
              </div>
              <div class="min-w-0">
                <div class="font-medium text-sm">
                  {{ node.label }}
                  <span v-if="node.notes" class="text-[10px] px-1.5 py-0.5 bg-amber-50 text-amber-600 rounded ml-1" title="有参考备注">备注</span>
                </div>
                <div class="text-xs text-text-secondary mt-0.5">
                  {{ nodeTypeLabel(node.type) }}
                  <template v-if="node.type === 'skynet_query'">
                    · {{ store.skyAppMap.get((node.config as SkynetQueryConfig).skyAppId)?.name || '未配置' }}
                    <template v-if="(node.config as SkynetQueryConfig).category">
                      · {{ (node.config as SkynetQueryConfig).category }}
                    </template>
                  </template>
                </div>
                <div v-if="nodeParamMap.get(node.id)?.length" class="mt-1 flex flex-wrap gap-1">
                  <span
                    v-for="p in nodeParamMap.get(node.id)"
                    :key="p.key + p.field"
                    class="inline-flex items-center text-[10px] px-1.5 py-0.5 rounded border border-primary/20 text-primary/70 bg-blue-50/50"
                  >{{ p.field }} ← {{ p.label }}</span>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button class="px-2 py-1 text-xs text-text-secondary hover:bg-surface-alt rounded" @click="copyNode(node)">复制</button>
              <button class="px-2 py-1 text-xs text-primary hover:bg-blue-50 rounded" @click="editNode(node)">编辑</button>
              <button class="px-2 py-1 text-xs text-error hover:bg-red-50 rounded" @click="removeNode(node.id)">删除</button>
            </div>
          </div>

          <div
            class="flex items-center justify-center transition-all"
            :class="dragState?.active && dropTarget === index + 1 && dragState.index !== index + 1 && dragState.index !== index ? 'py-3 border-2 border-dashed border-primary bg-blue-50/40 rounded-lg' : 'py-1 group/insert'"
          >
            <span v-if="dragState?.active && dropTarget === index + 1 && dragState.index !== index + 1 && dragState.index !== index" class="text-xs text-primary font-medium">放置到此处</span>
            <button
              v-else
              class="text-[10px] px-2 py-0.5 rounded border border-dashed border-transparent text-transparent group-hover/insert:border-primary/30 group-hover/insert:text-primary/50 transition-colors"
              @click="pasteNodeAt(index + 1)"
            >粘贴到此处</button>
          </div>
        </template>
      </div>
    </div>

    <NodeEditor
      v-if="showNodeEditor"
      :node="editingNode"
      :dynamic-params="flow.dynamicParams"
      @close="showNodeEditor = false"
      @save="saveNode"
    />

    <DynamicParamEditor
      v-if="showParamEditor"
      :params="flow.dynamicParams"
      @close="showParamEditor = false"
      @save="saveDynamicParams"
    />

    <FlowFormDialog
      v-if="showFlowInfoEditor"
      :source-flow="flow"
      :edit-mode="true"
      @close="showFlowInfoEditor = false"
      @saved="handleFlowInfoSaved"
    />

    <!-- 导入链路弹窗 -->
    <Teleport to="body">
      <div
        v-if="showImportDialog"
        class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
        @click.self="showImportDialog = false"
      >
        <div class="bg-surface rounded-xl shadow-xl w-[560px] max-h-[80vh] overflow-y-auto">
          <div class="px-6 py-4 border-b border-border">
            <h3 class="text-lg font-semibold">导入链路</h3>
            <p class="text-sm text-text-secondary mt-1">粘贴通过「导出链路」复制的 JSON 数据</p>
          </div>
          <div class="px-6 py-4">
            <textarea
              v-model="importJson"
              rows="12"
              placeholder='{"_type":"sky_trace_flow","name":"...","nodes":[...]}'
              class="w-full px-3 py-2 border border-border rounded-lg text-xs font-mono outline-none focus:border-primary resize-none"
            />
          </div>
          <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
            <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="showImportDialog = false">取消</button>
            <button
              class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50"
              :disabled="!importJson.trim()"
              @click="importFlow"
            >导入</button>
          </div>
        </div>
      </div>

      <!-- AI 全局分析弹窗 -->
      <div
        v-if="showGlobalAi"
        class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
        @click.self="showGlobalAi = false"
      >
        <div class="bg-surface rounded-xl shadow-xl w-[640px] max-h-[80vh] overflow-y-auto">
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <div class="flex items-center gap-2">
              <h3 class="text-lg font-semibold text-violet-700">AI 全局分析</h3>
              <span class="text-[10px] px-1.5 py-0.5 bg-violet-100 text-violet-500 rounded">Beta</span>
            </div>
            <button class="text-text-secondary hover:text-text" @click="showGlobalAi = false">✕</button>
          </div>

          <div class="px-6 py-4 space-y-4">
            <div class="bg-violet-50/50 rounded-lg p-3 text-xs text-text-secondary space-y-1">
              <div class="font-medium text-violet-700 mb-1">分析范围</div>
              <div v-for="node in flow?.nodes.filter(n => n.type === 'skynet_query' && execResults[n.id]?.status === 'success')" :key="node.id" class="flex items-center gap-2">
                <span class="w-1.5 h-1.5 rounded-full" :class="(execResults[node.id]?.result?.result?.list ?? []).some((l: any) => l.priority <= 1) ? 'bg-red-400' : 'bg-green-400'" />
                <span>{{ node.label }}</span>
                <span class="text-text-secondary/50">{{ execResults[node.id]?.result?.result?.count ?? 0 }} 条日志</span>
              </div>
            </div>

            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="preset in ['全链路异常分析', '各节点健康度评估', '排查建议', '错误日志汇总']"
                :key="preset"
                class="text-xs px-3 py-1.5 rounded-full border border-violet-200 text-violet-600 hover:bg-violet-100 transition-colors"
                @click="globalAiPrompt = preset; globalAiResponse = '功能开发中，AI 模型即将接入...'"
              >{{ preset }}</button>
            </div>

            <div class="flex gap-2">
              <input
                v-model="globalAiPrompt"
                placeholder="描述分析需求：如「分析全链路是否有异常，给出排查建议」"
                class="flex-1 px-3 py-2 text-sm border border-violet-200 rounded-lg outline-none focus:border-violet-400"
                @keydown.enter="globalAiResponse = '功能开发中，AI 模型即将接入...'"
              />
              <button
                class="px-4 py-2 text-sm bg-violet-500 text-white rounded-lg hover:bg-violet-600 transition-colors disabled:opacity-50"
                :disabled="!globalAiPrompt.trim()"
                @click="globalAiResponse = '功能开发中，AI 模型即将接入...'"
              >分析</button>
            </div>

            <div v-if="globalAiResponse" class="px-4 py-3 bg-white border border-violet-200/50 rounded-lg text-sm text-text-secondary leading-relaxed">
              <div class="flex items-center gap-1.5 mb-2">
                <span class="w-2 h-2 rounded-full bg-violet-400 animate-pulse" />
                <span class="text-violet-600 text-xs font-medium">AI 回复</span>
              </div>
              {{ globalAiResponse }}
            </div>
          </div>
        </div>
      </div>

      <!-- 全局操作反馈 -->
      <Transition name="fade">
        <div
          v-if="copyFeedback"
          class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 px-4 py-2 bg-gray-900 text-white text-sm rounded-lg shadow-lg"
        >{{ copyFeedback }}</div>
      </Transition>
    </Teleport>
  </div>
</template>
