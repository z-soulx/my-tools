<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import type {
  TraceFlow, TraceNode, NodeExecResult, DynamicParam, NodeGroup,
  SkynetQueryConfig, InfoNodeConfig, ChecklistNodeConfig, JcpOrderConfig, FieldBinding,
} from "@/types";
import { resolveBinding, resolveRelativeTime, extractTemplateParams } from "@/types";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { marked } from "marked";
import { buildGlobalAnalysisMessages, buildNodeAnalysisMessages } from "@/services/aiContext";
import NodeEditor from "@/components/NodeEditor.vue";
import NodeResult from "@/components/NodeResult.vue";
import JcpOrderResult from "@/components/JcpOrderResult.vue";
import DynamicParamEditor from "@/components/DynamicParamEditor.vue";
import ParamHintPopover from "@/components/ParamHintPopover.vue";
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
const globalAiThinking = ref("");
const globalAiRunning = ref(false);
const globalAiError = ref("");

// 节点级 AI 解读
const showNodeAi = ref(false);
const nodeAiTarget = ref<TraceNode | null>(null);
const nodeAiPrompt = ref("");
const nodeAiResponse = ref("");
const nodeAiThinking = ref("");
const nodeAiRunning = ref(false);
const nodeAiError = ref("");
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
const groupSectionExpanded = ref(false);
const groupNameInput = ref("");
const execEpoch = ref(0);

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
    if (node.type === "skynet_query") {
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
    } else if (node.type === "jcp_order") {
      const cfg = node.config as JcpOrderConfig;
      for (const [b, fieldLabel] of [[cfg.queryValue, "查询值"]] as [FieldBinding, string][]) {
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
  }
  return map;
});

/** 节点 id → 引用的动态参数列表 */
const nodeParamMap = computed(() => {
  const map = new Map<string, { key: string; label: string; field: string }[]>();
  if (!flow.value) return map;
  const paramLabelMap = new Map(flow.value.dynamicParams.map((p) => [p.key, p.label]));
  for (const node of flow.value.nodes) {
    const entries: { key: string; label: string; field: string }[] = [];
    if (node.type === "skynet_query") {
      const cfg = node.config as SkynetQueryConfig;
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
    } else if (node.type === "jcp_order") {
      const cfg = node.config as JcpOrderConfig;
      const b = cfg.queryValue;
      if (b?.mode === "dynamic" && b.paramKey) {
        entries.push({ key: b.paramKey, label: paramLabelMap.get(b.paramKey) || b.paramKey, field: "查询值" });
      } else if (b?.mode === "template" && b.templateValue) {
        for (const key of extractTemplateParams(b.templateValue)) {
          entries.push({ key, label: paramLabelMap.get(key) || key, field: "查询值" });
        }
      }
    }
    if (entries.length) map.set(node.id, entries);
  }
  return map;
});

const selectAllNodes = computed({
  get: () => {
    const qNodes = flow.value?.nodes.filter((n) => n.type === "skynet_query" || n.type === "jcp_order") ?? [];
    return qNodes.length > 0 && qNodes.every((n) => selectedNodeIds.value.has(n.id));
  },
  set: (val: boolean) => {
    const qNodes = flow.value?.nodes.filter((n) => n.type === "skynet_query" || n.type === "jcp_order") ?? [];
    if (val) qNodes.forEach((n) => selectedNodeIds.value.add(n.id));
    else selectedNodeIds.value.clear();
  },
});

const activeParamKeys = computed<Set<string> | null>(() => {
  if (!flow.value) return null;
  const keys = new Set<string>();
  for (const node of flow.value.nodes.filter(n => selectedNodeIds.value.has(n.id))) {
    const params = nodeParamMap.value.get(node.id);
    if (params) params.forEach(p => keys.add(p.key));
  }
  return keys.size > 0 ? keys : null;
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
      .filter((n) => n.type === "skynet_query" || n.type === "jcp_order")
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
    nodeGroups: flow.value.nodeGroups,
    aiPrompt: flow.value.aiPrompt ?? null,
    aiQuickActions: flow.value.aiQuickActions?.length ? flow.value.aiQuickActions : null,
    aiHintCollapsed: flow.value.aiHintCollapsed ?? false,
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
    aiPrompt: flow.value.aiPrompt ?? null,
    aiQuickActions: flow.value.aiQuickActions ?? [],
    nodes: flow.value.nodes.map(({ type, label, config, notes, aiPrompt, aiQuickActions }) => ({ type, label, config, notes, aiPrompt, aiQuickActions })),
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
      aiPrompt: data.aiPrompt ?? null,
      aiQuickActions: data.aiQuickActions ?? null,
      nodes: (data.nodes || []).map((n: { type: string; label: string; config: unknown; notes?: string; aiPrompt?: string; aiQuickActions?: string[] }, i: number) => ({
        id: `node_${Date.now()}_${i}`,
        type: n.type,
        label: n.label,
        sortOrder: i,
        config: n.config,
        notes: n.notes,
        aiPrompt: n.aiPrompt,
        aiQuickActions: n.aiQuickActions,
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

const paramTypeMap = computed(() => {
  const map = new Map<string, DynamicParam["paramType"]>();
  if (!flow.value) return map;
  for (const p of flow.value.dynamicParams) {
    if (p.paramType && p.paramType !== "text") map.set(p.key, p.paramType);
  }
  return map;
});

function resolveField(binding: FieldBinding): string {
  return resolveBinding(binding, dynamicValues.value, paramTypeMap.value);
}

// 将任意时间值（原始值/时间戳）转为 datetime-local 输入框格式 "yyyy-MM-ddTHH:mm"
function toDatetimeLocal(raw: string): string {
  if (!raw) return "";
  let d: Date;
  if (/^\d+$/.test(raw)) {
    const n = Number(raw);
    d = new Date(n < 1e12 ? n * 1000 : n);
  } else {
    d = new Date(raw.replace(/[/]/g, "-"));
  }
  if (isNaN(d.getTime())) return "";
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

// 将任意时间值转为 date 输入框格式 "yyyy-MM-dd"
function toDateInput(raw: string): string {
  return toDatetimeLocal(raw).slice(0, 10);
}

// datetime-local 变更：将 "yyyy-MM-ddTHH:mm" 按 paramType 存储
function onDatetimeChange(param: DynamicParam, dtLocal: string) {
  paramErrors.value[param.key] = false;
  if (!dtLocal) { dynamicValues.value[param.key] = ""; return; }
  const d = new Date(dtLocal);
  if (isNaN(d.getTime())) return;
  const p = (n: number) => String(n).padStart(2, "0");
  // 存原始 datetime 字符串，resolveBinding 在使用时按 paramType 转换
  dynamicValues.value[param.key] = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:00`;
}

// date 变更：存 yyyy-MM-dd
function onDateChange(param: DynamicParam, dateStr: string) {
  paramErrors.value[param.key] = false;
  dynamicValues.value[param.key] = dateStr;
}

function forceStopExecution() {
  // Bump epoch so in-flight promises from the previous run are ignored when they settle.
  execEpoch.value++;
  // Mark any still-running nodes as timed out.
  for (const [id, r] of Object.entries(execResults.value)) {
    if (r.status === "running") {
      execResults.value[id] = { ...r, status: "error", health: "error", error: "已手动停止" };
    }
  }
  executing.value = false;
}

async function executeNodes(onlySelected = false) {
  if (!flow.value) return;

  execEpoch.value++;
  const myEpoch = execEpoch.value;

  const targetNodeIds = onlySelected
    ? new Set(selectedNodeIds.value)
    : new Set(flow.value.nodes.map((n) => n.id));

  // 收集目标节点实际引用的 dynamic param keys
  const usedParamKeys = new Set<string>();
  for (const node of flow.value.nodes.filter((n) => targetNodeIds.has(n.id))) {
    const cfg = node.config as any;
    const bindings: FieldBinding[] = [];
    if (node.type === "skynet_query") {
      bindings.push(cfg.filter1, cfg.filter2, cfg.indexContext, cfg.contextId);
    } else if (node.type === "jcp_order") {
      bindings.push(cfg.queryValue);
    }
    for (const b of bindings) {
      if (!b) continue;
      if (b.mode === "dynamic" && b.paramKey) usedParamKeys.add(b.paramKey);
      if (b.mode === "template" && b.templateValue) {
        extractTemplateParams(b.templateValue).forEach((k) => usedParamKeys.add(k));
      }
    }
  }

  // 校验必填参数，仅限目标节点实际用到的
  const errors: Record<string, boolean> = {};
  for (const param of flow.value.dynamicParams) {
    if (param.required && usedParamKeys.has(param.key) && !dynamicValues.value[param.key]?.trim()) {
      errors[param.key] = true;
    }
  }
  paramErrors.value = errors;
  if (Object.keys(errors).length > 0) {
    showFeedback("请填写所有必填参数（标 * 项）");
    return;
  }

  executing.value = true;
  // 只清除本次要执行的节点结果，保留其它节点已有结果

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
  const jcpNodes = flow.value.nodes.filter((n) => n.type === "jcp_order" && targetNodeIds.has(n.id));

  // Phase A: execute jcp_order nodes sequentially first (they extract params for skynet queries)
  for (const node of jcpNodes) {
    const start = Date.now();
    const cfg = node.config as JcpOrderConfig;
    const queryValue = resolveField(cfg.queryValue);
    const autoDetectJcpField = (value: string): "orderId" | "traceId" =>
      /^\d{12}$/.test(value) ? "orderId" : "traceId";
    const field = cfg.queryField === "runtime"
      ? autoDetectJcpField(queryValue)
      : cfg.queryField;
    const body: Record<string, string> = { [field]: queryValue };
    try {
      const resp = await api.queryJcpOrder(body);
      const extracted: Record<string, string> = {};
      const findDeep = (obj: any, key: string): any => {
        if (!obj || typeof obj !== "object") return undefined;
        if (!Array.isArray(obj) && key in obj) return obj[key];
        const items = Array.isArray(obj) ? obj : Object.values(obj);
        for (const v of items) {
          if (v && typeof v === "object") {
            const found = findDeep(v, key);
            if (found !== undefined) return found;
          }
        }
        return undefined;
      };
      // dot-path 提取：先用 findDeep 定位第一段（支持任意深度嵌套），再按剩余路径走
      // 数字段视为数组下标，例如 "classMessageVoList.0.messageLogEntity.traceId"
      const findAtPath = (obj: any, path: string[]): any => {
        if (path.length === 0) return undefined;
        let cur = findDeep(obj, path[0]);
        for (const seg of path.slice(1)) {
          if (cur === null || cur === undefined) return undefined;
          if (Array.isArray(cur)) {
            const idx = Number(seg);
            cur = cur[isNaN(idx) ? 0 : idx];
          } else {
            cur = cur[seg];
          }
        }
        return cur;
      };

      const TIME_FIELDS = new Set(["checkInDate", "checkOutDate", "requestTime", "createDate"]);

      const tryParseDate = (raw: string): Date | null => {
        if (!raw) return null;
        if (/^\d+$/.test(raw)) {
          const n = Number(raw);
          const d = new Date(n < 1e12 ? n * 1000 : n);
          return isNaN(d.getTime()) ? null : d;
        }
        const d = new Date(raw.replace(/[/]/g, "-"));
        return isNaN(d.getTime()) ? null : d;
      };

      const pad = (n: number, len = 2) => String(n).padStart(len, "0");

      const deriveTimeFormats = (paramKey: string, raw: string) => {
        const d = tryParseDate(raw);
        if (!d) return;
        const ymd = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
        const full = `${ymd} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
        const tsMs = String(d.getTime());
        const tsSec = String(Math.floor(d.getTime() / 1000));
        const dayStart = new Date(d.getFullYear(), d.getMonth(), d.getDate());
        const dayTs = String(Math.floor(dayStart.getTime() / 1000));

        const derivations: Record<string, string> = {
          [`${paramKey}_ymd`]: ymd,
          [`${paramKey}_full`]: full,
          [`${paramKey}_ts`]: tsMs,
          [`${paramKey}_tsSec`]: tsSec,
          [`${paramKey}_dayTs`]: dayTs,
        };
        for (const [dk, dv] of Object.entries(derivations)) {
          extracted[dk] = dv;
          dynamicValues.value[dk] = dv;
        }
      };

      for (const m of (cfg.extractMappings ?? [])) {
        if (!m.targetParamKey) continue;
        // dot-path fields (e.g. "bookingVo.orderId") use findAtPath; plain keys use findDeep
        const val = m.sourceField.includes(".")
          ? findAtPath(resp, m.sourceField.split("."))
          : findDeep(resp, m.sourceField);
        if (val !== undefined && val !== null && val !== "") {
          const strVal = String(val);
          extracted[m.targetParamKey] = strVal;
          dynamicValues.value[m.targetParamKey] = strVal;
          if (TIME_FIELDS.has(m.sourceField)) {
            deriveTimeFormats(m.targetParamKey, strVal);
          }
        }
      }

      // 供应商映射查询：用已提取的 shotelId/roomTypeId/ratePlanId 调第二个 API
      if (cfg.supplierMappingEnabled) {
        const shotelIdKey = (cfg.extractMappings ?? []).find(m => m.sourceField === "shotelId")?.targetParamKey;
        const roomTypeIdKey = (cfg.extractMappings ?? []).find(m => m.sourceField === "roomTypeId")?.targetParamKey;
        const ratePlanIdKey = (cfg.extractMappings ?? []).find(m => m.sourceField === "ratePlanId")?.targetParamKey;
        const elongHotelId = shotelIdKey ? (dynamicValues.value[shotelIdKey] || "") : String(findDeep(resp, "shotelId") ?? "");
        const elongRoomId = roomTypeIdKey ? (dynamicValues.value[roomTypeIdKey] || "") : String(findDeep(resp, "roomTypeId") ?? "");
        const elongRateplanId = ratePlanIdKey ? (dynamicValues.value[ratePlanIdKey] || "") : String(findDeep(resp, "ratePlanId") ?? "");
        console.log("[SupplierMapping] Input:", { elongHotelId, elongRoomId, elongRateplanId });
        if (elongHotelId && elongRoomId && elongRateplanId) {
          try {
            const guid = findDeep(resp, "logId") || findDeep(resp, "guid") || "";
            const mappingResp = await api.querySupplierMapping({
              from: "",
              logId: String(guid),
              realRequest: { elongHotelId, elongRoomId, elongRateplanId },
            });
            console.log("[SupplierMapping] Response:", mappingResp);
            for (const m of (cfg.supplierExtractMappings ?? [])) {
              const val = findDeep(mappingResp, m.sourceField);
              if (val !== undefined && val !== null && val !== "") {
                const strVal = String(val);
                // always store in extracted by field name so it shows in result
                extracted[m.sourceField] = strVal;
                if (m.targetParamKey) {
                  extracted[m.targetParamKey] = strVal;
                  dynamicValues.value[m.targetParamKey] = strVal;
                }
              }
            }
          } catch (e) {
            console.warn("[SupplierMapping] Error:", e);
          }
        } else {
          console.log("[SupplierMapping] Skipped — missing values:", { elongHotelId, elongRoomId, elongRateplanId });
        }
      }

      // requestTime 特殊处理：自动设置查询时间范围（无需配置提取映射）
      const rtRaw = findDeep(resp, "createDate");
      if (rtRaw) {
        const rtDate = tryParseDate(String(rtRaw));
        if (rtDate) {
          const windowBefore = cfg.requestTimeWindowBefore ?? cfg.requestTimeWindow ?? 5;
          const windowAfter = cfg.requestTimeWindowAfter ?? cfg.requestTimeWindow ?? 5;
          const before = new Date(rtDate.getTime() - windowBefore * 60_000);
          const after = new Date(rtDate.getTime() + windowAfter * 60_000);
          const fmt = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}.000`;
          timeFrom.value = fmt(before);
          timeTo.value = fmt(after);
        }
      }
      execResults.value[node.id] = {
        nodeId: node.id,
        status: "success",
        health: "ok",
        durationMs: Date.now() - start,
        result: null,
        jcpResult: resp,
        uiLink: "",
        error: "",
        requestParams: body,
        extractedParams: extracted,
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
        requestParams: body,
      };
    }
  }

  // Phase B: execute skynet_query nodes in parallel (dynamic values now include jcp extracted params)
  const NODE_TIMEOUT_MS = 45_000;

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
      ...(cfg.advancedSearchItems?.length ? {
        advancedSearchItems: cfg.advancedSearchItems.map(item => ({
          filter: item.filter,
          compare: item.compare,
          value: [resolveField(item.value)],
        }))
      } : {}),
    } : { error: `未找到天网应用配置 (id=${cfg.skyAppId})` };

    const timeoutPromise = new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error("请求超时（45s）")), NODE_TIMEOUT_MS)
    );

    try {
      if (!skyApp) throw new Error(`未找到天网应用配置 (id=${cfg.skyAppId})`);

      const result = await Promise.race([api.querySkynetLog(skyApp.appId, skyApp.token, queryParams), timeoutPromise]);

      // If a force-stop happened while we were awaiting, discard result.
      if (execEpoch.value !== myEpoch) return;

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

      if (execEpoch.value !== myEpoch) return;

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
      if (execEpoch.value !== myEpoch) return;
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
    .filter((n) => n.type !== "skynet_query" && n.type !== "jcp_order" && targetNodeIds.has(n.id))
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
  // Only clear executing flag if no force-stop happened after us.
  if (execEpoch.value === myEpoch) executing.value = false;
}

function healthIcon(health: string) {
  return health === "ok" ? "🟢" : health === "warning" ? "🟡" : health === "error" ? "🔴" : "⚪";
}

async function openSkynetLink(node: TraceNode) {
  // If already executed, reuse the stored link
  if (execResults.value[node.id]?.uiLink) {
    await openUrl(execResults.value[node.id].uiLink);
    return;
  }
  const cfg = node.config as SkynetQueryConfig;
  const skyApp = store.skyAppMap.get(cfg.skyAppId);
  if (!skyApp) return;
  try {
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
    await openUrl(uiLink);
  } catch (e) {
    console.warn("[openSkynetLink] failed:", e);
  }
}

/** 刷新流程配置（不清空已填参数和执行结果） */
async function refreshFlow() {
  if (!flow.value || store.snapshotMode) return;
  const savedValues = { ...dynamicValues.value };
  try {
    flow.value = await api.getFlow(flowId.value);
    // 恢复已填值，仅对仍存在的参数 key 恢复
    flow.value.dynamicParams.forEach((p: DynamicParam) => {
      dynamicValues.value[p.key] = savedValues[p.key] ?? p.defaultValue ?? "";
    });
    showFeedback("已刷新节点配置");
  } catch {
    showFeedback("刷新失败");
  }
}

/** 快速清空面板参数 */
function clearParams() {
  if (!flow.value) return;
  flow.value.dynamicParams.forEach((p: DynamicParam) => {
    dynamicValues.value[p.key] = "";
  });
  paramErrors.value = {};
  showFeedback("参数已清空");
}

function nodeTypeLabel(type: string) {
  return type === "skynet_query" ? "天网查询" : type === "checklist" ? "Checklist" : type === "info" ? "信息" : type === "jcp_order" ? "产品组成单" : "链接";
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

function applyNodeGroup(group: NodeGroup) {
  selectedNodeIds.value = new Set(group.nodeIds);
}

function saveCurrentAsGroup() {
  const name = groupNameInput.value.trim();
  if (!name || !flow.value) return;
  const group: NodeGroup = {
    id: Date.now().toString(36),
    name,
    nodeIds: [...selectedNodeIds.value],
  };
  flow.value.nodeGroups = [...(flow.value.nodeGroups ?? []), group];
  groupNameInput.value = "";
  persistFlow();
}

function deleteNodeGroup(groupId: string) {
  if (!flow.value) return;
  flow.value.nodeGroups = (flow.value.nodeGroups ?? []).filter((g) => g.id !== groupId);
  persistFlow();
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

async function applySnippet(paramKey: string, value: string) {
  dynamicValues.value[paramKey] = value;
  paramErrors.value[paramKey] = false;
  // 如果当前是 select 模式且值不在 options 中，自动切到自定义输入
  const param = flow.value?.dynamicParams.find((p) => p.key === paramKey);
  if (param?.options?.length && !matchesOption(param.options, value)) {
    customInputKeys.value[paramKey] = true;
  }
  // 同时复制到剪贴板
  try {
    await navigator.clipboard.writeText(value);
  } catch {
    /* clipboard not available */
  }
}

// ── AI 流式调用辅助 ──

marked.setOptions({ breaks: true, gfm: true });

function renderMd(src: string): string {
  if (!src) return "";
  return marked.parse(src) as string;
}

async function streamAi(
  messages: { role: 'system' | 'user' | 'assistant'; content: string }[],
  onChunk: (s: string) => void,
  onThinking?: (s: string) => void,
): Promise<void> {
  const sessionId = (crypto.randomUUID && crypto.randomUUID()) || `ai_${Date.now()}_${Math.random()}`;
  const unlisteners: UnlistenFn[] = [];
  return new Promise<void>(async (resolve, reject) => {
    try {
      unlisteners.push(await listen<string>(`ai:chunk:${sessionId}`, (e) => onChunk(String(e.payload ?? ""))));
      if (onThinking) {
        unlisteners.push(await listen<string>(`ai:thinking:${sessionId}`, (e) => onThinking(String(e.payload ?? ""))));
      }
      unlisteners.push(await listen<string>(`ai:done:${sessionId}`, () => {
        unlisteners.forEach((u) => u());
        resolve();
      }));
      unlisteners.push(await listen<string>(`ai:error:${sessionId}`, (e) => {
        unlisteners.forEach((u) => u());
        reject(new Error(String(e.payload ?? "AI 调用失败")));
      }));
      api.aiChatStream(sessionId, messages).catch((err) => {
        unlisteners.forEach((u) => u());
        reject(err);
      });
    } catch (err) {
      unlisteners.forEach((u) => u());
      reject(err);
    }
  });
}

async function runGlobalAi(promptOverride?: string) {
  if (!flow.value) return;
  if (!store.aiAvailable) {
    globalAiError.value = "AI 未启用：请联系管理员开启远程 AI 配置";
    return;
  }
  const userPrompt = (promptOverride ?? globalAiPrompt.value).trim();
  if (!userPrompt) return;
  globalAiPrompt.value = userPrompt;
  globalAiResponse.value = "";
  globalAiThinking.value = "";
  globalAiError.value = "";
  globalAiRunning.value = true;
  try {
    const messages = buildGlobalAnalysisMessages(
      flow.value,
      execResults.value,
      dynamicValues.value,
      userPrompt,
      store.remoteConfig?.aiDefaultSystemPrompt,
    );
    await streamAi(messages, (c) => { globalAiResponse.value += c; }, (t) => { globalAiThinking.value += t; });
  } catch (e: any) {
    globalAiError.value = e?.message ?? String(e);
  } finally {
    globalAiRunning.value = false;
  }
}

function openNodeAi(node: TraceNode) {
  nodeAiTarget.value = node;
  nodeAiPrompt.value = "";
  nodeAiResponse.value = "";
  nodeAiThinking.value = "";
  nodeAiError.value = "";
  showNodeAi.value = true;
}

async function runNodeAi(promptOverride?: string) {
  if (!flow.value || !nodeAiTarget.value) return;
  if (!store.aiAvailable) {
    nodeAiError.value = "AI 未启用：请联系管理员开启远程 AI 配置";
    return;
  }
  const node = nodeAiTarget.value;
  const result = execResults.value[node.id];
  if (!result) {
    nodeAiError.value = "该节点尚未执行，请先运行后再分析";
    return;
  }
  const userPrompt = (promptOverride ?? nodeAiPrompt.value).trim();
  if (promptOverride) nodeAiPrompt.value = promptOverride;
  nodeAiResponse.value = "";
  nodeAiThinking.value = "";
  nodeAiError.value = "";
  nodeAiRunning.value = true;
  try {
    const messages = buildNodeAnalysisMessages(
      flow.value,
      node,
      result,
      dynamicValues.value,
      userPrompt,
      store.remoteConfig?.aiDefaultSystemPrompt,
    );
    await streamAi(messages, (c) => { nodeAiResponse.value += c; }, (t) => { nodeAiThinking.value += t; });
  } catch (e: any) {
    nodeAiError.value = e?.message ?? String(e);
  } finally {
    nodeAiRunning.value = false;
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
      <!-- AI 提示词只读提示 -->
      <div v-if="flow.aiPrompt?.trim() || flow.nodes.some(n => n.aiPrompt?.trim())" class="bg-violet-50/50 rounded-xl border border-violet-200/60 px-4 py-2.5 mb-4 flex items-start gap-2">
        <span class="text-violet-500 shrink-0 text-sm mt-0.5">✨</span>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-xs font-medium text-violet-600">已配置 AI 提示词</span>
            <span v-if="flow.aiPrompt?.trim()" class="text-[10px] px-1.5 py-0.5 bg-violet-100 text-violet-500 rounded">流程级</span>
            <span v-if="flow.nodes.some(n => n.aiPrompt?.trim())" class="text-[10px] px-1.5 py-0.5 bg-violet-100 text-violet-500 rounded">{{ flow.nodes.filter(n => n.aiPrompt?.trim()).length }} 个节点</span>
          </div>
          <p v-if="!flow.aiHintCollapsed && flow.aiPrompt?.trim()" class="text-xs text-text-secondary mt-1 line-clamp-2">{{ flow.aiPrompt }}</p>
        </div>
      </div>
      <!-- 动态参数 + 时间 -->
      <div class="bg-surface rounded-xl border border-border p-4 mb-6 space-y-4">
        <div v-if="flow.dynamicParams.filter(p => !p.hidden).length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          <div v-for="param in flow.dynamicParams.filter(p => !p.hidden)" :key="param.key" class="min-w-0 transition-opacity" :class="{ 'opacity-35 pointer-events-none': activeParamKeys && !activeParamKeys.has(param.key) }">
            <label class="block text-xs text-text-secondary mb-1 truncate" :title="param.label">
              {{ param.label }} <span v-if="param.required" :class="activeParamKeys && !activeParamKeys.has(param.key) ? 'text-text-secondary/50' : 'text-error'">*</span>
              <span v-if="param.paramType && param.paramType !== 'text'" class="ml-1 text-[10px] px-1 py-0.5 bg-cyan-50 text-cyan-600 rounded">{{ param.paramType === 'datetime' ? '日期时间' : param.paramType === 'date' ? '日期' : param.paramType === 'timestamp_ms' ? '毫秒戳' : param.paramType === 'timestamp_s' ? '秒戳' : param.paramType === 'day_timestamp_s' ? '天戳' : param.paramType }}</span>
            </label>
            <!-- 输入行：input + hint popover -->
            <div class="flex items-center gap-1">
              <!-- 有 options 且不允许自定义：纯 select -->
              <select
                v-if="param.options?.length && param.allowCustom === false"
                v-model="dynamicValues[param.key]"
                class="flex-1 min-w-0 px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
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
                  :value="dynamicValues[param.key]"
                  class="flex-1 min-w-0 px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
                  :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
                  @change="onOptionSelect(param.key, ($event.target as HTMLSelectElement).value); paramErrors[param.key] = false"
                >
                  <option value="" disabled>请选择</option>
                  <option v-for="opt in param.options" :key="opt" :value="parseOption(opt).value">{{ parseOption(opt).label }}</option>
                  <option value="__custom__">自定义...</option>
                </select>
                <div v-else class="flex-1 min-w-0 flex gap-1">
                  <input
                    v-model="dynamicValues[param.key]"
                    :placeholder="param.defaultValue || '自定义输入'"
                    class="flex-1 min-w-0 px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
                    :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
                    @input="paramErrors[param.key] = false"
                    @focus="($event.target as HTMLInputElement).select()"
                  />
                  <button
                    class="shrink-0 px-2 text-xs text-text-secondary hover:text-primary border border-border rounded-lg"
                    @click="customInputKeys[param.key] = false"
                  >选项</button>
                </div>
              </template>
              <!-- 无 options：按类型显示输入 -->
              <template v-else>
                <!-- 日期时间类型：用 datetime-local，显示为人类可读 -->
                <input
                  v-if="param.paramType === 'datetime' || param.paramType === 'timestamp_ms' || param.paramType === 'timestamp_s' || param.paramType === 'day_timestamp_s'"
                  type="datetime-local"
                  :value="toDatetimeLocal(dynamicValues[param.key])"
                  class="flex-1 min-w-0 px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
                  :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
                  @change="onDatetimeChange(param, ($event.target as HTMLInputElement).value)"
                />
                <input
                  v-else-if="param.paramType === 'date'"
                  type="date"
                  :value="toDateInput(dynamicValues[param.key])"
                  class="flex-1 min-w-0 px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
                  :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
                  @change="onDateChange(param, ($event.target as HTMLInputElement).value)"
                />
                <input
                  v-else
                  v-model="dynamicValues[param.key]"
                  :placeholder="param.defaultValue || param.label"
                  class="flex-1 min-w-0 px-3 py-1.5 text-sm border rounded-lg outline-none focus:border-primary"
                  :class="paramErrors[param.key] ? 'border-error bg-red-50' : 'border-border'"
                  @input="paramErrors[param.key] = false"
                  @focus="($event.target as HTMLInputElement).select()"
                />
              </template>
              <!-- Hint Popover -->
              <ParamHintPopover
                v-if="param.hint"
                :hint="param.hint"
                @apply="applySnippet(param.key, $event)"
              />
            </div>
            <p v-if="paramErrors[param.key]" class="mt-0.5 text-[10px] text-error">此参数为必填项</p>
            <!-- Snippets 快捷填入芯片 -->
            <div v-if="param.snippets?.length" class="mt-1 flex flex-wrap gap-1">
              <button
                v-for="(snippet, i) in param.snippets" :key="i"
                class="text-[10px] px-2 py-0.5 rounded-full border border-emerald-200 text-emerald-700 bg-emerald-50 hover:bg-emerald-100 truncate max-w-[200px] transition-colors"
                :title="parseOption(snippet).value"
                @click="applySnippet(param.key, parseOption(snippet).value)"
              >{{ parseOption(snippet).label }}</button>
            </div>
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
            v-if="executing"
            class="px-4 py-2.5 border border-red-300 text-red-500 rounded-lg hover:bg-red-50 transition-colors font-medium text-sm"
            title="强制停止当前执行"
            @click="forceStopExecution"
          >⏹ 停止</button>
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
          <div class="ml-auto flex items-center gap-2">
            <button
              v-if="!store.snapshotMode"
              class="px-3 py-1.5 text-xs border border-border text-text-secondary rounded-lg hover:border-primary hover:text-primary transition-colors"
              title="刷新节点配置，不清空已填参数"
              @click="refreshFlow"
            >↻ 刷新</button>
            <button
              class="px-3 py-1.5 text-xs border border-border text-text-secondary rounded-lg hover:border-red-300 hover:text-red-500 transition-colors"
              title="清空所有参数输入"
              @click="clearParams"
            >✕ 清空参数</button>
          </div>
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
      <!-- 节点分组快选（只读，分组在编排模式管理） -->
      <div v-if="(flow.nodeGroups ?? []).length" class="flex items-center gap-2 mb-3 flex-wrap">
        <span class="text-xs text-text-secondary shrink-0">分组:</span>
        <button
          v-for="group in (flow.nodeGroups ?? [])"
          :key="group.id"
          class="inline-flex items-center gap-1 px-3 py-1 text-xs rounded-full border border-primary/30 text-primary hover:bg-blue-50 transition-colors"
          @click="applyNodeGroup(group)"
        >{{ group.name }} ({{ group.nodeIds.length }})</button>
      </div>

      <div class="flex items-center gap-2 mb-4">
        <label class="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer select-none">
          <input type="checkbox" v-model="selectAllNodes" class="rounded" />
          全选查询节点 ({{ selectedNodeIds.size }}/{{ flow.nodes.filter(n => n.type === 'skynet_query' || n.type === 'jcp_order').length }})
        </label>
      </div>

      <div class="space-y-4">
        <div
          v-for="node in flow.nodes"
          :key="node.id"
          class="bg-surface rounded-xl border overflow-hidden"
          :class="(node.type === 'skynet_query' || node.type === 'jcp_order') && selectedNodeIds.has(node.id) ? 'border-primary/40' : 'border-border'"
        >
          <div class="px-4 py-3 bg-surface-alt border-b border-border">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <input
                  v-if="node.type === 'skynet_query' || node.type === 'jcp_order'"
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
                <button v-else-if="node.type === 'skynet_query' && !(store.snapshotMode && store.snapshotRestrictions.hideUiLink)" class="text-primary hover:underline" @click.stop="openSkynetLink(node)">天网UI ↗</button>
                <button
                  v-if="execResults[node.id]?.status === 'success' && store.aiAvailable"
                  class="text-violet-500 hover:text-violet-700 hover:underline"
                  title="基于该节点结果让 AI 解读"
                  @click.stop="openNodeAi(node)"
                >✨ AI 解读</button>
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
          <JcpOrderResult
            v-else-if="node.type === 'jcp_order'"
            :node="node"
            :result="execResults[node.id] ?? { nodeId: node.id, status: 'pending', health: 'unknown', durationMs: 0, result: null, uiLink: '', error: '' }"
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
      <!-- 流程级 AI 提示词 -->
      <div class="bg-surface rounded-xl border border-violet-200 p-4 mb-6">
        <div class="flex items-center justify-between mb-2">
          <h3 class="font-medium text-sm">
            <span class="text-violet-600">✨ 流程级 AI 提示词</span>
            <span class="text-xs text-text-secondary font-normal ml-2">（AI 全局/节点分析时作为整体业务上下文）</span>
          </h3>
          <label class="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer select-none" title="关闭后执行界面仅显示摘要，不展示提示词具体内容">
            <input type="checkbox" :checked="!flow.aiHintCollapsed" class="rounded accent-violet-500" @change="flow.aiHintCollapsed = !flow.aiHintCollapsed; persistFlow()" />
            <span>执行界面展示详情</span>
          </label>
        </div>
        <textarea
          v-model="flow.aiPrompt"
          rows="3"
          placeholder="例如：本流程用于排查铂涛 Mapping 不生效问题，关注 priority<=1 的日志、确认 hotelId 是否成功映射等..."
          class="w-full px-3 py-2 border border-violet-200 rounded-lg text-sm outline-none focus:border-violet-400 resize-none"
          @blur="persistFlow"
        />
        <div class="mt-3">
          <label class="block text-xs text-violet-600 font-medium mb-1.5">全局 AI 快捷问题</label>
          <p class="text-[10px] text-text-secondary mb-2">自定义「AI 全局分析」弹窗中的快捷按钮，留空则使用默认</p>
          <div class="space-y-1.5">
            <div v-for="(_, i) in (flow.aiQuickActions ?? [])" :key="i" class="flex items-center gap-1.5">
              <input
                v-model="flow.aiQuickActions![i]"
                class="flex-1 px-2 py-1 text-xs border border-violet-200 rounded outline-none focus:border-violet-400"
                placeholder="快捷问题文本"
                @blur="persistFlow"
              />
              <button class="text-error text-sm px-1" @click="flow.aiQuickActions!.splice(i, 1); persistFlow()">×</button>
            </div>
          </div>
          <button class="mt-1.5 text-xs text-violet-500 hover:underline" @click="if (!flow.aiQuickActions) flow.aiQuickActions = []; flow.aiQuickActions.push('')">+ 添加快捷问题</button>
        </div>
      </div>

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

      <!-- 节点分组管理 -->
      <div class="bg-surface rounded-xl border border-border mb-6">
        <button
          class="w-full px-4 py-3 flex items-center justify-between text-sm font-medium"
          @click="groupSectionExpanded = !groupSectionExpanded"
        >
          <span>节点分组 <span class="text-text-secondary font-normal">({{ (flow.nodeGroups ?? []).length }} 个)</span></span>
          <span class="text-text-secondary text-xs">{{ groupSectionExpanded ? '▾' : '▸' }}</span>
        </button>
        <div v-if="groupSectionExpanded" class="px-4 pb-4 space-y-4 border-t border-border">
          <!-- 已有分组 -->
          <div class="pt-3">
            <div v-if="!(flow.nodeGroups ?? []).length" class="text-xs text-text-secondary">暂无分组</div>
            <div v-else class="flex flex-wrap gap-2">
              <div
                v-for="group in (flow.nodeGroups ?? [])"
                :key="group.id"
                class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg border border-border bg-surface-alt"
              >
                <span class="font-medium">{{ group.name }}</span>
                <span class="text-text-secondary/60">({{ group.nodeIds.length }} 节点)</span>
                <button class="ml-1 text-text-secondary/50 hover:text-red-500 transition-colors" @click="deleteNodeGroup(group.id)">&times;</button>
              </div>
            </div>
          </div>
          <!-- 勾选节点 -->
          <div class="space-y-1">
            <div class="flex items-center justify-between mb-1">
              <span class="text-xs text-text-secondary font-medium">勾选节点</span>
              <button class="text-[10px] text-primary hover:underline" @click="selectAllNodes = !selectAllNodes">全选/全不选</button>
            </div>
            <label
              v-for="node in flow.nodes.filter(n => n.type === 'skynet_query' || n.type === 'jcp_order')"
              :key="node.id"
              class="flex items-center gap-2 py-1 px-2 rounded hover:bg-surface-alt cursor-pointer text-sm select-none"
            >
              <input
                type="checkbox"
                :checked="selectedNodeIds.has(node.id)"
                class="rounded accent-primary"
                @change="toggleNodeSelection(node.id)"
              />
              <span class="truncate">{{ node.label }}</span>
              <span class="text-[10px] text-text-secondary shrink-0">{{ nodeTypeLabel(node.type) }}</span>
            </label>
          </div>
          <!-- 保存分组 -->
          <div class="flex items-center gap-2">
            <input
              v-model="groupNameInput"
              placeholder="分组名称"
              class="flex-1 px-3 py-1.5 text-sm border border-border rounded-lg outline-none focus:border-primary"
              @keydown.enter="saveCurrentAsGroup"
            />
            <button
              class="px-3 py-1.5 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-40 transition-colors"
              :disabled="!groupNameInput.trim() || selectedNodeIds.size === 0"
              @click="saveCurrentAsGroup"
            >保存分组</button>
          </div>
          <div v-if="selectedNodeIds.size === 0" class="text-[11px] text-text-secondary/60">请先勾选节点</div>
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
                v-for="preset in (flow?.aiQuickActions?.length ? flow.aiQuickActions : ['全链路异常分析', '各节点健康度评估', '排查建议', '错误日志汇总'])"
                :key="preset"
                class="text-xs px-3 py-1.5 rounded-full border border-violet-200 text-violet-600 hover:bg-violet-100 transition-colors disabled:opacity-50"
                :disabled="globalAiRunning"
                @click="runGlobalAi(preset)"
              >{{ preset }}</button>
            </div>

            <div class="flex gap-2">
              <input
                v-model="globalAiPrompt"
                placeholder="描述分析需求：如「分析全链路是否有异常，给出排查建议」"
                class="flex-1 px-3 py-2 text-sm border border-violet-200 rounded-lg outline-none focus:border-violet-400"
                :disabled="globalAiRunning"
                @keydown.enter="runGlobalAi()"
              />
              <button
                class="px-4 py-2 text-sm bg-violet-500 text-white rounded-lg hover:bg-violet-600 transition-colors disabled:opacity-50"
                :disabled="!globalAiPrompt.trim() || globalAiRunning"
                @click="runGlobalAi()"
              >{{ globalAiRunning ? '分析中…' : '分析' }}</button>
            </div>

            <div v-if="!store.aiAvailable" class="px-4 py-3 bg-amber-50 border border-amber-200 rounded-lg text-xs text-amber-700">
              AI 未启用：请联系管理员在远程配置中开启 AI（base url / token / model）。
            </div>

            <div v-if="globalAiError" class="px-4 py-3 bg-red-50 border border-red-200 rounded-lg text-xs text-red-600">
              {{ globalAiError }}
            </div>

            <div v-if="globalAiThinking || globalAiResponse || globalAiRunning" class="px-4 py-3 bg-white border border-violet-200/50 rounded-lg text-sm text-text-secondary leading-relaxed">
              <details v-if="globalAiThinking || (globalAiRunning && !globalAiResponse)" class="mb-3" :open="!globalAiResponse">
                <summary class="cursor-pointer text-xs text-violet-500 font-medium select-none flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full bg-violet-300" :class="{ 'animate-pulse': globalAiRunning && !globalAiResponse }" />
                  思考过程
                  <span v-if="globalAiThinking" class="text-[10px] text-text-secondary font-normal ml-1">{{ globalAiThinking.length }} 字</span>
                </summary>
                <div v-if="globalAiThinking" class="mt-2 pl-3 border-l-2 border-violet-100 text-xs text-text-secondary/70 max-h-40 overflow-y-auto ai-markdown" v-html="renderMd(globalAiThinking)" />
                <span v-else class="mt-2 block pl-3 text-xs text-text-secondary/50 animate-pulse">正在思考中…</span>
              </details>
              <div class="flex items-center gap-1.5 mb-2">
                <span class="w-2 h-2 rounded-full bg-violet-400" :class="{ 'animate-pulse': globalAiRunning }" />
                <span class="text-violet-600 text-xs font-medium">AI 回复</span>
              </div>
              <div v-if="globalAiResponse" class="ai-markdown" v-html="renderMd(globalAiResponse)" />
              <span v-else class="text-text-secondary/50">等待回复…</span>
            </div>
          </div>
        </div>
      </div>

      <!-- AI 节点解读弹窗 -->
      <div
        v-if="showNodeAi && nodeAiTarget"
        class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
        @click.self="showNodeAi = false"
      >
        <div class="bg-surface rounded-xl shadow-xl w-[640px] max-h-[80vh] overflow-y-auto">
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <div class="flex items-center gap-2">
              <h3 class="text-lg font-semibold text-violet-700">✨ AI 节点解读</h3>
              <span class="text-xs text-text-secondary">{{ nodeAiTarget.label }}</span>
            </div>
            <button class="text-text-secondary hover:text-text" @click="showNodeAi = false">✕</button>
          </div>
          <div class="px-6 py-4 space-y-4">
            <div v-if="!flow.aiHintCollapsed && nodeAiTarget.aiPrompt" class="bg-violet-50/50 rounded-lg p-3 text-xs text-text-secondary whitespace-pre-wrap">
              <div class="font-medium text-violet-700 mb-1">节点提示词</div>
              {{ nodeAiTarget.aiPrompt }}
            </div>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="preset in (nodeAiTarget.aiQuickActions?.length ? nodeAiTarget.aiQuickActions : ['判断该节点是否异常', '总结关键日志', '可能的根因'])"
                :key="preset"
                class="text-xs px-3 py-1.5 rounded-full border border-violet-200 text-violet-600 hover:bg-violet-100 transition-colors disabled:opacity-50"
                :disabled="nodeAiRunning"
                @click="runNodeAi(preset)"
              >{{ preset }}</button>
            </div>
            <div class="flex gap-2">
              <input
                v-model="nodeAiPrompt"
                placeholder="例如：这个节点的错误日志说明了什么？"
                class="flex-1 px-3 py-2 text-sm border border-violet-200 rounded-lg outline-none focus:border-violet-400"
                :disabled="nodeAiRunning"
                @keydown.enter="runNodeAi()"
              />
              <button
                class="px-4 py-2 text-sm bg-violet-500 text-white rounded-lg hover:bg-violet-600 transition-colors disabled:opacity-50"
                :disabled="nodeAiRunning"
                @click="runNodeAi()"
              >{{ nodeAiRunning ? '分析中…' : '分析' }}</button>
            </div>
            <div v-if="nodeAiError" class="px-4 py-3 bg-red-50 border border-red-200 rounded-lg text-xs text-red-600">
              {{ nodeAiError }}
            </div>
            <div v-if="nodeAiThinking || nodeAiResponse || nodeAiRunning" class="px-4 py-3 bg-white border border-violet-200/50 rounded-lg text-sm text-text-secondary leading-relaxed">
              <details v-if="nodeAiThinking || (nodeAiRunning && !nodeAiResponse)" class="mb-3" :open="!nodeAiResponse">
                <summary class="cursor-pointer text-xs text-violet-500 font-medium select-none flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full bg-violet-300" :class="{ 'animate-pulse': nodeAiRunning && !nodeAiResponse }" />
                  思考过程
                  <span v-if="nodeAiThinking" class="text-[10px] text-text-secondary font-normal ml-1">{{ nodeAiThinking.length }} 字</span>
                </summary>
                <div v-if="nodeAiThinking" class="mt-2 pl-3 border-l-2 border-violet-100 text-xs text-text-secondary/70 max-h-40 overflow-y-auto ai-markdown" v-html="renderMd(nodeAiThinking)" />
                <span v-else class="mt-2 block pl-3 text-xs text-text-secondary/50 animate-pulse">正在思考中…</span>
              </details>
              <div class="flex items-center gap-1.5 mb-2">
                <span class="w-2 h-2 rounded-full bg-violet-400" :class="{ 'animate-pulse': nodeAiRunning }" />
                <span class="text-violet-600 text-xs font-medium">AI 回复</span>
              </div>
              <div v-if="nodeAiResponse" class="ai-markdown" v-html="renderMd(nodeAiResponse)" />
              <span v-else class="text-text-secondary/50">等待回复…</span>
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
