<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import type { SnapshotRestrictions } from "@/types";
import { save } from "@tauri-apps/plugin-dialog";

const emit = defineEmits<{ close: [] }>();
const store = useAppStore();

const selectedFlowIds = ref<Set<number>>(new Set());
const selectedChecklistIds = ref<Set<number>>(new Set());
const selectedRecoveryIds = ref<Set<number>>(new Set());
const restrictions = ref<SnapshotRestrictions>({
  hideEdit: true,
  hideSettings: true,
  hideSuppliers: true,
  hideQuickQuery: true,
  hideChecklistEdit: true,
  hideRecoveryEdit: true,
  hideTrash: true,
  hideDebug: true,
  hideUiLink: true,
});
const exporting = ref(false);
const feedback = ref("");

const selectAllFlows = computed({
  get: () => selectedFlowIds.value.size === store.flows.length && store.flows.length > 0,
  set: (val: boolean) => {
    if (val) store.flows.forEach((f) => selectedFlowIds.value.add(f.id));
    else selectedFlowIds.value.clear();
  },
});

const selectAllChecklists = computed({
  get: () => selectedChecklistIds.value.size === store.checklists.length && store.checklists.length > 0,
  set: (val: boolean) => {
    if (val) store.checklists.forEach((g) => selectedChecklistIds.value.add(g.id));
    else selectedChecklistIds.value.clear();
  },
});

const selectAllRecovery = computed({
  get: () => selectedRecoveryIds.value.size === store.recoveryGroups.length && store.recoveryGroups.length > 0,
  set: (val: boolean) => {
    if (val) store.recoveryGroups.forEach((g) => selectedRecoveryIds.value.add(g.id));
    else selectedRecoveryIds.value.clear();
  },
});

function toggleFlow(id: number) {
  selectedFlowIds.value.has(id) ? selectedFlowIds.value.delete(id) : selectedFlowIds.value.add(id);
}

function toggleChecklist(id: number) {
  selectedChecklistIds.value.has(id) ? selectedChecklistIds.value.delete(id) : selectedChecklistIds.value.add(id);
}

function toggleRecovery(id: number) {
  selectedRecoveryIds.value.has(id) ? selectedRecoveryIds.value.delete(id) : selectedRecoveryIds.value.add(id);
}

const restrictionItems = [
  { key: "hideEdit" as keyof SnapshotRestrictions, label: "隐藏编排功能" },
  { key: "hideSettings" as keyof SnapshotRestrictions, label: "隐藏天网配置" },
  { key: "hideSuppliers" as keyof SnapshotRestrictions, label: "隐藏供应商管理" },
  { key: "hideQuickQuery" as keyof SnapshotRestrictions, label: "隐藏快速查询" },
  { key: "hideChecklistEdit" as keyof SnapshotRestrictions, label: "隐藏Checklist编辑" },
  { key: "hideRecoveryEdit" as keyof SnapshotRestrictions, label: "隐藏恢复单元编辑" },
  { key: "hideTrash" as keyof SnapshotRestrictions, label: "隐藏回收站" },
  { key: "hideDebug" as keyof SnapshotRestrictions, label: "隐藏调试信息" },
  { key: "hideUiLink" as keyof SnapshotRestrictions, label: "隐藏天网UI跳转" },
];

async function handleExport() {
  if (selectedFlowIds.value.size === 0 && selectedChecklistIds.value.size === 0 && selectedRecoveryIds.value.size === 0) return;
  exporting.value = true;
  feedback.value = "";

  try {
    const path = await save({
      defaultPath: `skytrace-snapshot-${Date.now()}.skytrace`,
      filters: [{ name: "SkyTrace 快照", extensions: ["skytrace"] }],
    });

    if (!path) {
      exporting.value = false;
      return;
    }

    await api.exportSnapshot(
      Array.from(selectedFlowIds.value),
      Array.from(selectedChecklistIds.value),
      Array.from(selectedRecoveryIds.value),
      restrictions.value,
      path,
    );
    feedback.value = `快照已导出到: ${path}`;
    setTimeout(() => emit("close"), 2000);
  } catch (e) {
    feedback.value = `导出失败: ${e}`;
  } finally {
    exporting.value = false;
  }
}

const totalSelected = computed(() => selectedFlowIds.value.size + selectedChecklistIds.value.size + selectedRecoveryIds.value.size);
</script>

<template>
  <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" @click.self="emit('close')">
    <div class="bg-surface rounded-xl shadow-xl w-[560px] max-h-[85vh] overflow-y-auto">
      <div class="px-6 py-4 border-b border-border">
        <h3 class="text-lg font-semibold">导出快照</h3>
        <p class="text-sm text-text-secondary mt-0.5">选择要包含的链路、Checklist 和权限设置</p>
      </div>

      <div class="px-6 py-4 space-y-4">
        <!-- 选择链路 -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm font-medium">排查链路</label>
            <label class="flex items-center gap-1 text-xs">
              <input type="checkbox" v-model="selectAllFlows" class="rounded" />
              全选 ({{ selectedFlowIds.size }}/{{ store.flows.length }})
            </label>
          </div>
          <div class="border border-border rounded-lg max-h-40 overflow-y-auto">
            <label
              v-for="flow in store.flows"
              :key="flow.id"
              class="flex items-center gap-2 px-3 py-2 hover:bg-surface-alt cursor-pointer border-b border-border/50 last:border-b-0"
            >
              <input type="checkbox" :checked="selectedFlowIds.has(flow.id)" class="rounded" @change="toggleFlow(flow.id)" />
              <div class="min-w-0">
                <div class="text-sm truncate">{{ flow.name }}</div>
                <div class="text-[10px] text-text-secondary">{{ flow.nodes.length }} 节点</div>
              </div>
            </label>
          </div>
        </div>

        <!-- 选择 Checklist -->
        <div v-if="store.checklists.length > 0">
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm font-medium">监控 Checklist</label>
            <label class="flex items-center gap-1 text-xs">
              <input type="checkbox" v-model="selectAllChecklists" class="rounded" />
              全选 ({{ selectedChecklistIds.size }}/{{ store.checklists.length }})
            </label>
          </div>
          <div class="border border-border rounded-lg max-h-40 overflow-y-auto">
            <label
              v-for="group in store.checklists"
              :key="group.id"
              class="flex items-center gap-2 px-3 py-2 hover:bg-surface-alt cursor-pointer border-b border-border/50 last:border-b-0"
            >
              <input type="checkbox" :checked="selectedChecklistIds.has(group.id)" class="rounded" @change="toggleChecklist(group.id)" />
              <div class="min-w-0">
                <div class="text-sm truncate">{{ group.name }}</div>
                <div class="text-[10px] text-text-secondary">{{ group.items.length }} 检查项</div>
              </div>
            </label>
          </div>
        </div>

        <!-- 选择恢复单元 -->
        <div v-if="store.recoveryGroups.length > 0">
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm font-medium">快速恢复单元</label>
            <label class="flex items-center gap-1 text-xs">
              <input type="checkbox" v-model="selectAllRecovery" class="rounded" />
              全选 ({{ selectedRecoveryIds.size }}/{{ store.recoveryGroups.length }})
            </label>
          </div>
          <div class="border border-border rounded-lg max-h-40 overflow-y-auto">
            <label
              v-for="group in store.recoveryGroups"
              :key="group.id"
              class="flex items-center gap-2 px-3 py-2 hover:bg-surface-alt cursor-pointer border-b border-border/50 last:border-b-0"
            >
              <input type="checkbox" :checked="selectedRecoveryIds.has(group.id)" class="rounded" @change="toggleRecovery(group.id)" />
              <div class="min-w-0">
                <div class="text-sm truncate">{{ group.name }}</div>
                <div class="text-[10px] text-text-secondary">{{ group.steps.length }} 步骤</div>
              </div>
            </label>
          </div>
        </div>

        <!-- 权限限制 -->
        <div>
          <label class="text-sm font-medium block mb-2">快照权限（勾选 = 对快照用户隐藏）</label>
          <div class="grid grid-cols-2 gap-2">
            <label
              v-for="item in restrictionItems"
              :key="item.key"
              class="flex items-center gap-2 text-sm cursor-pointer"
            >
              <input type="checkbox" v-model="restrictions[item.key]" class="rounded" />
              {{ item.label }}
            </label>
          </div>
        </div>

        <div v-if="feedback" class="text-sm" :class="feedback.startsWith('导出失败') ? 'text-error' : 'text-green-600'">
          {{ feedback }}
        </div>
      </div>

      <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
        <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="emit('close')">取消</button>
        <button
          class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50"
          :disabled="totalSelected === 0 || exporting"
          @click="handleExport"
        >{{ exporting ? '导出中...' : `导出快照 (${totalSelected} 项)` }}</button>
      </div>
    </div>
  </div>
</template>
