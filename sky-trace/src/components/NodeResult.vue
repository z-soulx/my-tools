<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import type { TraceNode, NodeExecResult, SkynetLogItem, PriorityLevel } from "@/types";
import { PRIORITY_MAP } from "@/types";
import { useAppStore } from "@/stores/app";
import HighlightText from "./HighlightText.vue";

const store = useAppStore();

const MSG_TRUNCATE = 200;

const props = withDefaults(defineProps<{
  node: TraceNode;
  result: NodeExecResult;
  globalSearch?: string;
  forceExpand?: boolean;
}>(), {
  globalSearch: "",
  forceExpand: false,
});

const expanded = ref(true);
const showDebug = ref(false);
const compact = ref(true);
const expandedRows = ref(new Set<string>());
const detailLog = ref<SkynetLogItem | null>(null);
const maxDisplay = ref(20);
const copied = ref("");
const showAiPanel = ref(false);
const aiPrompt = ref("");
const aiResponse = ref("");

const localSearchOpen = ref(false);
const localSearchQuery = ref("");
const activeMatchIdx = ref(0);
const scrollContainer = ref<HTMLElement | null>(null);

const effectiveSearch = computed(() => props.globalSearch || localSearchQuery.value);

const selectedPriorities = ref<Set<number>>(new Set([0, 1, 2, 3]));

function togglePriority(p: number) {
  if (selectedPriorities.value.has(p)) {
    if (selectedPriorities.value.size > 1) selectedPriorities.value.delete(p);
  } else {
    selectedPriorities.value.add(p);
  }
}

const rawLogList = computed<SkynetLogItem[]>(
  () => props.result.result?.result?.list ?? []
);

const logList = computed(() =>
  rawLogList.value.filter((log) => selectedPriorities.value.has(log.priority))
);

const filteredLogList = computed(() => {
  if (!effectiveSearch.value) return logList.value;
  const kw = effectiveSearch.value.toLowerCase();
  return logList.value.filter((log) => log.msg.toLowerCase().includes(kw));
});
const totalCount = computed(() => props.result.result?.result?.count ?? 0);
const rawResponseCode = computed(() => props.result.result?.code ?? "N/A");

const totalMatches = computed(() => {
  if (!effectiveSearch.value) return 0;
  const kw = effectiveSearch.value.toLowerCase();
  return filteredLogList.value.slice(0, maxDisplay.value).reduce(
    (sum, log) => sum + countOccurrences(log.msg.toLowerCase(), kw), 0
  );
});

function countOccurrences(src: string, kw: string): number {
  if (!kw) return 0;
  let count = 0;
  let pos = 0;
  while ((pos = src.indexOf(kw, pos)) !== -1) {
    count++;
    pos += kw.length;
  }
  return count;
}

/** 计算某条日志之前所有日志累计匹配数 */
function matchesBefore(logIndex: number): number {
  if (!effectiveSearch.value) return 0;
  const kw = effectiveSearch.value.toLowerCase();
  let count = 0;
  for (let i = 0; i < logIndex && i < maxDisplay.value; i++) {
    count += countOccurrences(filteredLogList.value[i].msg.toLowerCase(), kw);
  }
  return count;
}

function scrollToMatch(idx: number) {
  nextTick(() => {
    const el = scrollContainer.value?.querySelector(`[data-match-idx="${idx}"]`);
    el?.scrollIntoView({ block: "center", behavior: "smooth" });
  });
}

function nextMatch() {
  if (totalMatches.value === 0) return;
  activeMatchIdx.value = (activeMatchIdx.value + 1) % totalMatches.value;
  scrollToMatch(activeMatchIdx.value);
}

function prevMatch() {
  if (totalMatches.value === 0) return;
  activeMatchIdx.value = (activeMatchIdx.value - 1 + totalMatches.value) % totalMatches.value;
  scrollToMatch(activeMatchIdx.value);
}

function toggleSearch() {
  localSearchOpen.value = !localSearchOpen.value;
  if (!localSearchOpen.value) {
    localSearchQuery.value = "";
    activeMatchIdx.value = 0;
  }
}

function isTruncated(log: SkynetLogItem): boolean {
  return compact.value && !expandedRows.value.has(log.id) && log.msg.length > MSG_TRUNCATE;
}

function displayMsg(log: SkynetLogItem): string {
  return isTruncated(log) ? log.msg.slice(0, MSG_TRUNCATE) + "..." : log.msg;
}

function toggleRow(log: SkynetLogItem) {
  expandedRows.value.has(log.id) ? expandedRows.value.delete(log.id) : expandedRows.value.add(log.id);
}

function openDetail(log: SkynetLogItem) {
  detailLog.value = log;
}

function toggleCompact() {
  compact.value = !compact.value;
  if (compact.value) expandedRows.value.clear();
}

async function copyText(text: string, id: string) {
  try {
    await navigator.clipboard.writeText(text);
    copied.value = id;
    setTimeout(() => { if (copied.value === id) copied.value = ""; }, 1500);
  } catch { /* clipboard not available */ }
}

function handleKeydown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (mod && e.shiftKey && e.key === "e") {
    e.preventDefault();
    toggleCompact();
  }
  if (mod && e.key === "f" && expanded.value && props.result.status === "success") {
    e.preventDefault();
    localSearchOpen.value = true;
    nextTick(() => {
      (document.querySelector(".node-search-input") as HTMLInputElement)?.focus();
    });
  }
}

watch(() => props.forceExpand, (v) => { if (v !== undefined) expanded.value = v; });
watch(() => props.globalSearch, (v) => {
  activeMatchIdx.value = 0;
  if (v && totalMatches.value > 0) expanded.value = true;
});

onMounted(() => window.addEventListener("keydown", handleKeydown));
onUnmounted(() => window.removeEventListener("keydown", handleKeydown));

function priorityClass(priority: number): string {
  if (priority <= 0) return "text-red-600 font-bold bg-red-50";
  if (priority === 1) return "text-red-500 bg-red-50";
  if (priority === 2) return "text-amber-500 bg-amber-50";
  return "text-text-secondary";
}
</script>

<template>
  <div>
    <div
      v-if="result.status === 'running'"
      class="px-4 py-6 flex items-center justify-center text-text-secondary text-sm"
    >
      <span class="animate-pulse">查询中...</span>
    </div>

    <div
      v-else-if="result.status === 'error'"
      class="px-4 py-3 bg-red-50 text-red-600 text-sm"
    >
      <p>{{ result.error }}</p>
      <template v-if="!(store.snapshotMode && store.snapshotRestrictions.hideDebug)">
        <button class="mt-1 text-xs text-red-400 hover:underline" @click="showDebug = !showDebug">
          {{ showDebug ? '隐藏调试' : '查看请求参数' }}
        </button>
        <pre v-if="showDebug" class="mt-2 text-xs bg-red-100 rounded p-2 overflow-x-auto whitespace-pre-wrap">{{ JSON.stringify(result.requestParams, null, 2) }}</pre>
      </template>
    </div>

    <div v-else-if="result.status === 'success' && result.result" class="divide-y divide-border">
      <!-- 顶部栏 -->
      <div class="px-4 py-2 flex items-center justify-between cursor-pointer hover:bg-surface-alt" @click="expanded = !expanded">
        <span class="text-sm text-text-secondary">
          共 {{ totalCount }} 条日志，显示前 {{ Math.min(maxDisplay, filteredLogList.length) }} 条
          <span v-if="effectiveSearch && filteredLogList.length !== logList.length" class="text-amber-600">
            (搜索过滤后 {{ filteredLogList.length }} 条)
          </span>
        </span>
        <div class="flex items-center gap-2">
          <button
            class="text-[10px] px-2 py-0.5 rounded border transition-colors"
            :class="compact ? 'border-primary text-primary bg-blue-50' : 'border-border text-text-secondary'"
            @click.stop="toggleCompact"
            title="Ctrl+Shift+E"
          >{{ compact ? '精简' : '完整' }}</button>
          <button
            class="text-[10px] px-2 py-0.5 rounded border border-border text-text-secondary hover:border-primary hover:text-primary transition-colors"
            @click.stop="toggleSearch"
            title="Ctrl+F"
          >搜索</button>
          <button
            class="text-[10px] px-2 py-0.5 rounded border transition-colors"
            :class="showAiPanel ? 'border-violet-500 text-violet-600 bg-violet-50' : 'border-border text-text-secondary hover:border-violet-400 hover:text-violet-500'"
            @click.stop="showAiPanel = !showAiPanel"
          >AI 分析</button>
          <span class="text-xs text-text-secondary">{{ expanded ? '▲' : '▼' }}</span>
        </div>
      </div>

      <!-- 日志等级筛选 -->
      <div v-if="expanded" class="px-4 py-1.5 flex items-center gap-1.5 border-b border-border/50">
        <span class="text-[10px] text-text-secondary mr-1">等级:</span>
        <button
          v-for="(label, p) in { 0: 'FATAL', 1: 'ERROR', 2: 'WARN', 3: 'INFO' } as Record<number, string>"
          :key="p"
          class="text-[10px] px-2 py-0.5 rounded border transition-colors"
          :class="selectedPriorities.has(Number(p))
            ? (Number(p) === 0 ? 'border-red-600 text-red-600 bg-red-50' :
               Number(p) === 1 ? 'border-red-500 text-red-500 bg-red-50' :
               Number(p) === 2 ? 'border-amber-500 text-amber-500 bg-amber-50' :
               'border-primary text-primary bg-blue-50')
            : 'border-border text-text-secondary/40'"
          @click.stop="togglePriority(Number(p))"
        >{{ label }}</button>
        <span class="text-[10px] text-text-secondary ml-auto">{{ logList.length }}/{{ rawLogList.length }}
          <template v-if="effectiveSearch && filteredLogList.length !== logList.length"> → {{ filteredLogList.length }}</template>
        </span>
      </div>

      <!-- AI 分析面板 -->
      <div v-if="showAiPanel && expanded" class="px-4 py-3 bg-violet-50/50 border-b border-violet-200/50">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-xs font-medium text-violet-700">AI 日志分析</span>
          <span class="text-[10px] px-1.5 py-0.5 bg-violet-100 text-violet-500 rounded">Beta</span>
        </div>
        <div class="text-[10px] text-text-secondary mb-2">
          当前节点 {{ rawLogList.length }} 条日志，筛选后 {{ logList.length }} 条
          <span v-if="logList.some(l => l.priority <= 1)" class="text-red-500 ml-1">
            (含 {{ logList.filter(l => l.priority <= 1).length }} 条 ERROR/FATAL)
          </span>
        </div>
        <div class="flex flex-wrap gap-1.5 mb-2">
          <button
            v-for="preset in ['分析异常原因', '提取关键信息', '排查建议', '日志摘要']"
            :key="preset"
            class="text-[10px] px-2.5 py-1 rounded-full border border-violet-200 text-violet-600 hover:bg-violet-100 transition-colors"
            @click="aiPrompt = preset; aiResponse = '功能开发中，AI 模型即将接入...'"
          >{{ preset }}</button>
        </div>
        <div class="flex gap-2">
          <input
            v-model="aiPrompt"
            placeholder="输入分析需求，如：帮我分析这些错误日志的根因..."
            class="flex-1 px-3 py-1.5 text-xs border border-violet-200 rounded-lg outline-none focus:border-violet-400 bg-white"
            @keydown.enter="aiResponse = '功能开发中，AI 模型即将接入...'"
          />
          <button
            class="px-3 py-1.5 text-xs bg-violet-500 text-white rounded-lg hover:bg-violet-600 transition-colors disabled:opacity-50"
            :disabled="!aiPrompt.trim()"
            @click="aiResponse = '功能开发中，AI 模型即将接入...'"
          >分析</button>
        </div>
        <div v-if="aiResponse" class="mt-2 px-3 py-2 bg-white border border-violet-200/50 rounded-lg text-xs text-text-secondary leading-relaxed">
          <div class="flex items-center gap-1.5 mb-1">
            <span class="w-1.5 h-1.5 rounded-full bg-violet-400 animate-pulse" />
            <span class="text-violet-600 text-[10px] font-medium">AI 回复</span>
          </div>
          {{ aiResponse }}
        </div>
      </div>

      <!-- 局部搜索栏（全局搜索时隐藏） -->
      <div v-if="localSearchOpen && !globalSearch" class="px-4 py-2 bg-amber-50/50 flex items-center gap-2 border-b border-border">
        <input
          v-model="localSearchQuery"
          class="node-search-input flex-1 px-2 py-1 text-xs border border-border rounded outline-none focus:border-primary"
          placeholder="搜索日志内容..."
          @keydown.enter.prevent="nextMatch"
          @keydown.escape="toggleSearch"
        />
        <span v-if="localSearchQuery" class="text-[10px] text-text-secondary shrink-0">
          {{ totalMatches > 0 ? `${activeMatchIdx + 1}/${totalMatches}` : '0 结果' }}
        </span>
        <button class="text-xs text-text-secondary hover:text-primary px-1" @click="prevMatch" :disabled="totalMatches === 0">▲</button>
        <button class="text-xs text-text-secondary hover:text-primary px-1" @click="nextMatch" :disabled="totalMatches === 0">▼</button>
        <button class="text-xs text-text-secondary hover:text-text px-1" @click="toggleSearch">×</button>
      </div>
      <!-- 全局搜索匹配数 -->
      <div v-if="globalSearch && totalMatches > 0" class="px-4 py-1 bg-amber-50/30 text-[10px] text-amber-700 border-b border-border">
        此节点匹配 {{ totalMatches }} 处
      </div>

      <!-- 日志列表 -->
      <div v-if="expanded" ref="scrollContainer" class="max-h-80 overflow-y-auto">
        <div
          v-for="(log, logIdx) in filteredLogList.slice(0, maxDisplay)"
          :key="log.id"
          class="px-4 py-2 text-xs font-mono border-b border-border/50 hover:bg-surface-alt group"
          :class="expandedRows.has(log.id) ? 'select-text cursor-text' : 'select-none cursor-pointer'"
          @click="!expandedRows.has(log.id) && toggleRow(log)"
          @dblclick="openDetail(log)"
        >
          <div class="flex items-start gap-2">
            <span class="text-text-secondary shrink-0 w-40">{{ log.logTime }}</span>
            <span
              :class="priorityClass(log.priority)"
              class="shrink-0 px-1.5 py-0.5 rounded text-[10px]"
            >{{ PRIORITY_MAP[log.priority as PriorityLevel] || 'INFO' }}</span>
            <span class="text-text leading-relaxed flex-1 min-w-0" :class="isTruncated(log) ? '' : 'break-all whitespace-pre-wrap'">
              <HighlightText
                v-if="effectiveSearch"
                :text="displayMsg(log)"
                :keyword="effectiveSearch"
                :active-index="activeMatchIdx"
                :start-match-index="matchesBefore(logIdx)"
              />
              <template v-else>{{ displayMsg(log) }}</template>
            </span>
            <button
              class="shrink-0 text-[10px] text-text-secondary/40 hover:text-primary opacity-0 group-hover:opacity-100 transition-opacity px-1"
              @click.stop="copyText(log.msg, log.id)"
              title="复制全文"
            >{{ copied === log.id ? '✓' : '复制' }}</button>
            <button
              v-if="expandedRows.has(log.id)"
              class="shrink-0 text-[10px] text-text-secondary/40 hover:text-text opacity-0 group-hover:opacity-100 transition-opacity px-1"
              @click.stop="toggleRow(log)"
              title="收起"
            >收起</button>
          </div>
          <div class="mt-1 flex items-center gap-3 text-[10px] text-text-secondary">
            <span v-if="log.ip">IP: {{ log.ip }}</span>
            <span v-if="log.category">分类: {{ log.category }}</span>
            <span v-if="log.filter1">F1: {{ log.filter1 }}</span>
            <span v-if="log.filter2">F2: {{ log.filter2 }}</span>
            <span v-if="log.module">模块: {{ log.module }}</span>
          </div>
        </div>

        <div v-if="filteredLogList.length > maxDisplay" class="px-4 py-2 text-center">
          <button class="text-xs text-primary hover:underline" @click="maxDisplay += 50">
            加载更多 (还有 {{ filteredLogList.length - maxDisplay }} 条)
          </button>
        </div>
      </div>

      <div v-if="!(store.snapshotMode && store.snapshotRestrictions.hideDebug)" class="px-4 py-2 border-t border-border/50">
        <button class="text-[10px] text-text-secondary/60 hover:text-text-secondary" @click="showDebug = !showDebug">
          {{ showDebug ? '隐藏调试 ▲' : '调试信息 ▼' }}
        </button>
        <div v-if="showDebug" class="mt-2 space-y-2">
          <div>
            <span class="text-[10px] text-text-secondary block mb-0.5">请求参数：</span>
            <pre class="text-[10px] bg-surface-alt rounded p-2 overflow-x-auto whitespace-pre-wrap font-mono">{{ JSON.stringify(result.requestParams, null, 2) }}</pre>
          </div>
          <div>
            <span class="text-[10px] text-text-secondary block mb-0.5">响应概要 (code={{ rawResponseCode }}, count={{ totalCount }})：</span>
            <pre class="text-[10px] bg-surface-alt rounded p-2 overflow-x-auto whitespace-pre-wrap font-mono max-h-40">{{ JSON.stringify(result.result, null, 2)?.slice(0, 2000) }}</pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 日志详情弹窗 -->
    <Teleport to="body">
      <div
        v-if="detailLog"
        class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
        @click.self="detailLog = null"
      >
        <div class="bg-surface rounded-xl shadow-xl w-[90vw] max-w-3xl max-h-[85vh] flex flex-col">
          <div class="px-6 py-3 border-b border-border flex items-center justify-between shrink-0">
            <div class="flex items-center gap-2">
              <span
                :class="priorityClass(detailLog.priority)"
                class="px-1.5 py-0.5 rounded text-[10px]"
              >{{ PRIORITY_MAP[detailLog.priority as PriorityLevel] || 'INFO' }}</span>
              <span class="text-sm text-text-secondary">{{ detailLog.logTime }}</span>
            </div>
            <div class="flex items-center gap-2">
              <button
                class="text-xs px-2 py-1 rounded border border-border hover:border-primary hover:text-primary transition-colors"
                @click="copyText(detailLog.msg, 'detail')"
              >{{ copied === 'detail' ? '已复制 ✓' : '复制内容' }}</button>
              <button class="text-text-secondary hover:text-text text-lg" @click="detailLog = null">×</button>
            </div>
          </div>
          <div class="flex-1 overflow-y-auto p-6">
            <pre class="text-xs font-mono whitespace-pre-wrap break-all leading-relaxed text-text">{{ detailLog.msg }}</pre>
            <div class="mt-4 grid grid-cols-2 gap-2 text-[10px] text-text-secondary">
              <span v-if="detailLog.ip">IP: {{ detailLog.ip }}</span>
              <span v-if="detailLog.category">Category: {{ detailLog.category }}</span>
              <span v-if="detailLog.module">Module: {{ detailLog.module }}</span>
              <span v-if="detailLog.filter1">Filter1: {{ detailLog.filter1 }}</span>
              <span v-if="detailLog.filter2">Filter2: {{ detailLog.filter2 }}</span>
              <span v-if="detailLog.contextId">ContextId: {{ detailLog.contextId }}</span>
              <span v-if="detailLog.env">Env: {{ detailLog.env }}</span>
              <span>AppId: {{ detailLog.appId }}</span>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
