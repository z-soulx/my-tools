export interface SkyApp {
  id: number;
  appId: string;
  appUk: string;
  token: string;
  name: string;
  env: string;
  createdAt: string;
}

export interface Supplier {
  id: number;
  name: string;
  code: string;
  description: string;
  serviceIds: number[];
  createdAt: string;
}

/**
 * 流程自定义的动态参数定义
 * 用户编排流程时创建，执行时填值
 */
export interface DynamicParam {
  key: string;
  label: string;
  required: boolean;
  defaultValue: string;
  hint?: string;
  options?: string[];
  allowCustom?: boolean;
  snippets?: string[];
  hidden?: boolean;
  /** 参数值类型，用于自动格式转换。
   * text: 原样使用（默认）
   * datetime: yyyy-MM-dd HH:mm:ss
   * date: yyyy-MM-dd
   * timestamp_ms: 毫秒时间戳
   * timestamp_s: 秒级时间戳
   * day_timestamp_s: 当日零点秒级时间戳
   */
  paramType?: "text" | "datetime" | "date" | "timestamp_ms" | "timestamp_s" | "day_timestamp_s";
}

/**
 * 节点中某个字段如何取值：固定值 or 绑定动态参数
 */
export interface FieldBinding {
  mode: "fixed" | "dynamic" | "template";
  fixedValue: string;
  paramKey: string; // 绑定的 DynamicParam.key
  templateValue?: string; // 模板字符串，如 "inc_{{hotel}}"
}

export interface ChecklistSubItem {
  id: string;
  label: string;
  url: string;
}

export interface ChecklistItem {
  id: string;
  label: string;
  description: string;
  links: ChecklistSubItem[];
}

export interface ChecklistGroup {
  id: number;
  name: string;
  description: string;
  items: ChecklistItem[];
  createdAt: string;
  updatedAt: string;
}

export interface ChecklistNodeConfig {
  checklistGroupId: number;
  itemIds: string[];
}

export interface JcpExtractMapping {
  sourceField: string;
  targetParamKey: string;
}

export interface JcpOrderConfig {
  queryField: "orderId" | "traceId" | "runtime";
  queryValue: FieldBinding;
  extractMappings: JcpExtractMapping[];
  /** requestTime 聚焦向前窗口（分钟），默认 5 */
  requestTimeWindowBefore?: number;
  /** requestTime 聚焦向后窗口（分钟），默认 5 */
  requestTimeWindowAfter?: number;
  /** @deprecated 用 requestTimeWindowBefore/After 替代 */
  requestTimeWindow?: number;
  /** 是否启用供应商映射查询（自动用 shotelId/roomTypeId/ratePlanId 调第二个 API） */
  supplierMappingEnabled?: boolean;
  /** 供应商映射提取字段 */
  supplierExtractMappings?: JcpExtractMapping[];
}

export interface TraceNode {
  id: string;
  type: "skynet_query" | "info" | "link" | "checklist" | "jcp_order";
  label: string;
  sortOrder: number;
  config: SkynetQueryConfig | InfoNodeConfig | LinkNodeConfig | ChecklistNodeConfig | JcpOrderConfig;
  notes?: string;
}

/**
 * 天网查询节点配置
 * 固定值直接填写，动态值绑定到流程的 DynamicParam
 */
export interface SkynetQueryConfig {
  skyAppId: number;
  module: string;
  category: string;
  subCategory: string;
  filter1: FieldBinding;
  filter2: FieldBinding;
  indexContext: FieldBinding;
  contextId: FieldBinding;
  pageSize: number;
  fieldHints?: Record<string, string>;
}

export interface InfoNodeConfig {
  content: string;
  links: ExternalLink[];
}

export interface LinkNodeConfig {
  skyAppId: number;
  category: string;
  filter1: FieldBinding;
}

export interface ExternalLink {
  label: string;
  url: string;
}

export interface NodeGroup {
  id: string;
  name: string;
  nodeIds: string[];
}

export interface TraceFlow {
  id: number;
  name: string;
  description: string;
  supplierId: number | null;
  tags: string[];
  isFavorite: boolean;
  sortOrder: number;
  dynamicParams: DynamicParam[];
  nodes: TraceNode[];
  nodeGroups: NodeGroup[];
  createdAt: string;
  updatedAt: string;
}

export interface NodeExecResult {
  nodeId: string;
  status: "pending" | "running" | "success" | "error";
  health: "ok" | "warning" | "error" | "unknown";
  durationMs: number;
  result: SkynetLogResponse | null;
  jcpResult?: any;
  uiLink: string;
  error: string;
  requestParams?: Record<string, unknown>;
  extractedParams?: Record<string, string>;
}

export interface FlowExecContext {
  dynamicValues: Record<string, string>;
  timeFrom: string; // "now-30m" or absolute
  timeTo: string;   // "now" or absolute
  env: string;
}

export interface SkynetLogResponse {
  code: number;
  message: string | null;
  result: {
    count: number;
    list: SkynetLogItem[];
  };
}

export interface SkynetLogItem {
  id: string;
  msg: string;
  subCategory: string;
  module: string;
  ip: string;
  filter1: string;
  filter2: string;
  contextId: string;
  priority: number;
  env: string;
  logTime: string;
  kms: boolean;
  logicIdcUk: string;
  appId: number;
  domainName: string;
  appUk: string;
  category: string;
  extraInfo: string;
  timestamp: number;
}

export interface RecoveryLink {
  id: string;
  label: string;
  url: string;
}

export interface RecoveryStep {
  id: string;
  label: string;
  description: string;
  links: RecoveryLink[];
}

export interface RecoveryGroup {
  id: number;
  name: string;
  description: string;
  steps: RecoveryStep[];
  createdAt: string;
  updatedAt: string;
}

export interface SnapshotRestrictions {
  hideEdit: boolean;
  hideSettings: boolean;
  hideSuppliers: boolean;
  hideQuickQuery: boolean;
  hideChecklistEdit: boolean;
  hideRecoveryEdit: boolean;
  hideTrash: boolean;
  hideDebug: boolean;
  hideUiLink: boolean;
}

export interface SnapshotData {
  schemaVersion?: number;
  dataVersion?: string;
  flows: TraceFlow[];
  skyApps: SkyApp[];
  suppliers: Supplier[];
  checklistGroups: ChecklistGroup[];
  recoveryGroups: RecoveryGroup[];
  restrictions: SnapshotRestrictions;
  createdAt: string;
  author: string;
}

export interface RemoteConfig {
  enabled: boolean;
  minVersion: string;
  latestVersion: string;
  message: string;
  updateUrlMac: string;
  updateUrlWin: string;
  updateNotes: string;
  features: Record<string, boolean>;
  announcement: { text: string; type: 'info' | 'warning' | 'error' } | null;
  latestDataVersion: string;
  dataUpdateUrl: string;
  dataUpdateNotes: string;
}

export type PriorityLevel = 0 | 1 | 2 | 3;
export const PRIORITY_MAP: Record<PriorityLevel, string> = {
  0: "FATAL",
  1: "ERROR",
  2: "WARN",
  3: "INFO",
};

/** 尝试将任意时间表示解析为 Date */
function tryParseTime(raw: string): Date | null {
  if (!raw) return null;
  if (/^\d+$/.test(raw)) {
    const n = Number(raw);
    const d = new Date(n < 1e12 ? n * 1000 : n);
    return isNaN(d.getTime()) ? null : d;
  }
  const d = new Date(raw.replace(/[/]/g, "-"));
  return isNaN(d.getTime()) ? null : d;
}

/** 按 paramType 转换值 */
export function convertByParamType(value: string, paramType?: DynamicParam["paramType"]): string {
  if (!paramType || paramType === "text") return value;
  const d = tryParseTime(value);
  if (!d) return value;
  const p = (n: number, len = 2) => String(n).padStart(len, "0");
  const ymd = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
  switch (paramType) {
    case "date": return ymd;
    case "datetime": return `${ymd} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
    case "timestamp_ms": return String(d.getTime());
    case "timestamp_s": return String(Math.floor(d.getTime() / 1000));
    case "day_timestamp_s": return String(Math.floor(new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime() / 1000));
    default: return value;
  }
}

/** 根据 FieldBinding 解析实际值 */
export function resolveBinding(
  binding: FieldBinding,
  dynamicValues: Record<string, string>,
  paramTypeMap?: Map<string, DynamicParam["paramType"]>,
): string {
  if (binding.mode === "fixed") return binding.fixedValue;
  if (binding.mode === "template") {
    return (binding.templateValue ?? "").replace(
      /\{\{([\w-]+)(?::([^}]+))?\}\}/g,
      (_, key, transform) => {
        const val = dynamicValues[key] ?? "";
        if (!transform) return val;
        return applyTransform(val, transform);
      }
    );
  }
  const raw = dynamicValues[binding.paramKey] ?? "";
  const pType = paramTypeMap?.get(binding.paramKey);
  return convertByParamType(raw, pType);
}

/** 应用模板变换函数，如 split(_,0) */
function applyTransform(value: string, transform: string): string {
  const splitMatch = transform.match(/^split\((.+),\s*(\d+)\)$/);
  if (splitMatch) {
    const delimiter = splitMatch[1];
    const index = parseInt(splitMatch[2], 10);
    const parts = value.split(delimiter);
    return index < parts.length ? parts[index] : "";
  }
  return value;
}

export function emptyBinding(): FieldBinding {
  return { mode: "fixed", fixedValue: "", paramKey: "", templateValue: "" };
}

/** 从模板字符串中提取所有 {{key}} 或 {{key:transform}} 引用的参数 key */
export function extractTemplateParams(tpl: string): string[] {
  const keys: string[] = [];
  const re = /\{\{([\w-]+)(?::[^}]+)?\}\}/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(tpl)) !== null) {
    if (!keys.includes(m[1])) keys.push(m[1]);
  }
  return keys;
}

/** 将 "now-30m" 等相对时间解析为绝对时间字符串 yyyy-MM-dd HH:mm:ss.SSS */
export function resolveRelativeTime(expr: string): string {
  if (!expr.startsWith("now")) return expr;
  const now = new Date();
  const offset = expr.slice(3);
  if (!offset) return formatTimestamp(now);
  const sign = offset.startsWith("-") ? -1 : 1;
  const rest = sign === -1 ? offset.slice(1) : offset;
  const unit = rest.slice(-1);
  const num = parseInt(rest.slice(0, -1), 10) || 30;
  const ms =
    unit === "m" ? num * 60_000 :
    unit === "h" ? num * 3_600_000 :
    unit === "d" ? num * 86_400_000 :
    num * 60_000;
  return formatTimestamp(new Date(now.getTime() + sign * ms));
}

function formatTimestamp(d: Date): string {
  const p = (n: number, len = 2) => String(n).padStart(len, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}.${p(d.getMilliseconds(), 3)}`;
}
