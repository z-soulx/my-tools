<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import type { SkynetLogResponse, PriorityLevel } from "@/types";
import { PRIORITY_MAP, resolveRelativeTime } from "@/types";
import TimeRangeSelector from "@/components/TimeRangeSelector.vue";
import { openUrl } from "@tauri-apps/plugin-opener";

const store = useAppStore();
const selectedAppId = ref<number | undefined>();
const module_ = ref("");
const category = ref("");
const subCategory = ref("");
const filter1 = ref("");
const filter2 = ref("");
const indexContext = ref("");
const contextId = ref("");
const pageSize = ref(100);
const timeFrom = ref("now-30m");
const timeTo = ref("now");
const querying = ref(false);
const result = ref<SkynetLogResponse | null>(null);
const error = ref("");
const maxDisplay = ref(50);

const selectedApp = computed(() =>
  store.skyApps.find((a) => a.id === selectedAppId.value)
);

async function handleQuery() {
  if (!selectedApp.value) return;
  querying.value = true;
  error.value = "";
  result.value = null;

  try {
    result.value = await api.querySkynetLog(
      selectedApp.value.appId,
      selectedApp.value.token,
      {
        module: module_.value,
        category: category.value,
        subCategory: subCategory.value,
        filter1: filter1.value,
        filter2: filter2.value,
        indexContext: indexContext.value,
        contextId: contextId.value,
        pageSize: pageSize.value,
        beginTime: resolveRelativeTime(timeFrom.value),
        endTime: resolveRelativeTime(timeTo.value),
      }
    );
  } catch (e) {
    error.value = String(e);
  } finally {
    querying.value = false;
  }
}

async function openInSkynet() {
  if (!selectedApp.value) return;
  const link = await api.generateSkynetUiLink(selectedApp.value.appUk, {
    module: module_.value,
    category: category.value,
    subCategory: subCategory.value,
    filter1: filter1.value,
    filter2: filter2.value,
    indexContext: indexContext.value,
    contextId: contextId.value,
    beginTime: timeFrom.value,
    endTime: timeTo.value,
  });
  await openUrl(link);
}

function priorityClass(priority: number): string {
  if (priority <= 0) return "text-red-600 font-bold bg-red-50";
  if (priority === 1) return "text-red-500 bg-red-50";
  if (priority === 2) return "text-amber-500 bg-amber-50";
  return "text-text-secondary";
}

const selectedPriorities = ref<Set<number>>(new Set([0, 1, 2, 3]));
function togglePriority(p: number) {
  if (selectedPriorities.value.has(p)) {
    if (selectedPriorities.value.size > 1) selectedPriorities.value.delete(p);
  } else {
    selectedPriorities.value.add(p);
  }
}

const filteredList = computed(() => {
  const list = result.value?.result?.list ?? [];
  return list.filter((log) => selectedPriorities.value.has(log.priority));
});
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 bg-surface border-b border-border shrink-0">
      <h2 class="text-xl font-semibold">快速查询</h2>
      <p class="text-sm text-text-secondary mt-0.5">选择服务，填参数，直接查</p>
    </header>

    <div class="flex-1 overflow-y-auto">
      <div class="p-6 space-y-4">
        <div class="bg-surface rounded-xl border border-border p-4 space-y-4">
          <div>
            <label class="block text-sm font-medium mb-1">天网应用 *</label>
            <select
              v-model="selectedAppId"
              class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary"
            >
              <option :value="undefined" disabled>选择天网应用</option>
              <option v-for="app in store.skyApps" :key="app.id" :value="app.id">
                {{ app.name || app.appUk }} ({{ app.appId }})
              </option>
            </select>
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div>
              <label class="block text-xs text-text-secondary mb-1">Module</label>
              <input v-model="module_" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-1">Category</label>
              <input v-model="category" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-1">SubCategory</label>
              <input v-model="subCategory" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-1">Filter1</label>
              <input v-model="filter1" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-1">Filter2</label>
              <input v-model="filter2" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-1">Msg 模糊查询</label>
              <input v-model="indexContext" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-1">TraceId (contextId)</label>
              <input v-model="contextId" placeholder="如 028282b9-8b1a-49b6-..." class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary font-mono" />
            </div>
          </div>

          <TimeRangeSelector v-model:model-from="timeFrom" v-model:model-to="timeTo" />

          <div>
            <label class="block text-xs text-text-secondary mb-1">每页条数</label>
            <input v-model.number="pageSize" type="number" min="1" max="500" class="w-32 px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
          </div>

          <div class="flex items-center gap-3">
            <button
              class="px-6 py-2 bg-primary text-white text-sm rounded-lg hover:bg-primary-hover transition-colors disabled:opacity-50"
              :disabled="!selectedAppId || querying"
              @click="handleQuery"
            >
              {{ querying ? '查询中...' : '查询' }}
            </button>
            <button
              v-if="selectedApp && !(store.snapshotMode && store.snapshotRestrictions.hideUiLink)"
              class="px-4 py-2 text-sm text-primary border border-primary rounded-lg hover:bg-blue-50 transition-colors"
              @click="openInSkynet"
            >
              在天网 UI 查看 ↗
            </button>
          </div>
        </div>

        <div v-if="error" class="bg-red-50 text-red-600 rounded-xl p-4 text-sm">{{ error }}</div>

        <div v-if="result" class="bg-surface rounded-xl border border-border overflow-hidden">
          <div class="px-4 py-3 bg-surface-alt border-b border-border flex items-center justify-between">
            <span class="text-sm font-medium">
              查询结果：共 {{ result.result?.count ?? 0 }} 条
            </span>
          </div>
          <div class="px-4 py-1.5 flex items-center gap-1.5 border-b border-border/50">
            <span class="text-[10px] text-text-secondary mr-1">等级:</span>
            <button
              v-for="(label, p) in ({ 0: 'FATAL', 1: 'ERROR', 2: 'WARN', 3: 'INFO' } as Record<number, string>)"
              :key="p"
              class="text-[10px] px-2 py-0.5 rounded border transition-colors"
              :class="selectedPriorities.has(Number(p))
                ? (Number(p) === 0 ? 'border-red-600 text-red-600 bg-red-50' :
                   Number(p) === 1 ? 'border-red-500 text-red-500 bg-red-50' :
                   Number(p) === 2 ? 'border-amber-500 text-amber-500 bg-amber-50' :
                   'border-primary text-primary bg-blue-50')
                : 'border-border text-text-secondary/40'"
              @click="togglePriority(Number(p))"
            >{{ label }}</button>
            <span class="text-[10px] text-text-secondary ml-auto">{{ filteredList.length }}/{{ result.result?.list?.length ?? 0 }}</span>
          </div>
          <div class="max-h-[60vh] overflow-y-auto">
            <div
              v-for="log in filteredList.slice(0, maxDisplay)"
              :key="log.id"
              class="px-4 py-2 text-xs font-mono border-b border-border/50 hover:bg-surface-alt"
            >
              <div class="flex items-start gap-2">
                <span class="text-text-secondary shrink-0 w-40">{{ log.logTime }}</span>
                <span
                  :class="priorityClass(log.priority)"
                  class="shrink-0 px-1.5 py-0.5 rounded text-[10px]"
                >{{ PRIORITY_MAP[log.priority as PriorityLevel] || 'INFO' }}</span>
                <span class="text-text break-all leading-relaxed">{{ log.msg }}</span>
              </div>
              <div class="mt-1 flex items-center gap-3 text-[10px] text-text-secondary">
                <span v-if="log.ip">IP: {{ log.ip }}</span>
                <span v-if="log.category">分类: {{ log.category }}</span>
                <span v-if="log.filter1">F1: {{ log.filter1 }}</span>
                <span v-if="log.module">模块: {{ log.module }}</span>
              </div>
            </div>
            <div v-if="filteredList.length > maxDisplay" class="px-4 py-3 text-center">
              <button class="text-xs text-primary hover:underline" @click="maxDisplay += 100">
                加载更多 (还有 {{ filteredList.length - maxDisplay }} 条)
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
