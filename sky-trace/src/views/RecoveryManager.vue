<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import type { RecoveryGroup, RecoveryStep } from "@/types";

const store = useAppStore();
const groups = ref<RecoveryGroup[]>([]);
const editingGroup = ref<RecoveryGroup | null>(null);
const showEditor = ref(false);

const canEdit = !store.snapshotMode || !store.snapshotRestrictions.hideRecoveryEdit;

onMounted(async () => {
  await loadGroups();
});

async function loadGroups() {
  if (store.snapshotMode) {
    groups.value = store.recoveryGroups;
    return;
  }
  groups.value = await api.getRecoveryGroups();
  store.recoveryGroups = groups.value;
}

function newGroup() {
  editingGroup.value = {
    id: 0,
    name: "",
    description: "",
    steps: [],
    createdAt: "",
    updatedAt: "",
  };
  showEditor.value = true;
}

function editGroup(g: RecoveryGroup) {
  editingGroup.value = JSON.parse(JSON.stringify(g));
  showEditor.value = true;
}

async function deleteGroup(id: number) {
  await api.deleteRecoveryGroup(id);
  await loadGroups();
}

const showBatchAdd = ref(false);
const batchText = ref("");

function addStep(g: RecoveryGroup) {
  g.steps.push({
    id: `step_${Date.now()}`,
    label: "",
    description: "",
    links: [],
  });
}

function batchAddSteps(g: RecoveryGroup) {
  const lines = batchText.value.split("\n").map((l) => l.trim()).filter(Boolean);
  for (const line of lines) {
    g.steps.push({
      id: `step_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`,
      label: line,
      description: "",
      links: [],
    });
  }
  batchText.value = "";
  showBatchAdd.value = false;
}

function removeStep(g: RecoveryGroup, index: number) {
  g.steps.splice(index, 1);
}

function addLink(step: RecoveryStep) {
  step.links.push({ id: `link_${Date.now()}`, label: "", url: "" });
}

function removeLink(step: RecoveryStep, index: number) {
  step.links.splice(index, 1);
}

function duplicateStep(g: RecoveryGroup, index: number) {
  const src = g.steps[index];
  g.steps.splice(index + 1, 0, {
    id: `step_${Date.now()}`,
    label: src.label + " (副本)",
    description: src.description,
    links: src.links.map((l) => ({ ...l, id: `link_${Date.now()}_${Math.random().toString(36).slice(2, 6)}` })),
  });
}

async function saveGroup() {
  if (!editingGroup.value || !editingGroup.value.name.trim()) return;
  const g = editingGroup.value;
  await api.saveRecoveryGroup({
    id: g.id || undefined,
    name: g.name.trim(),
    description: g.description.trim(),
    steps: g.steps.filter((s) => s.label.trim()),
  });
  showEditor.value = false;
  editingGroup.value = null;
  await loadGroups();
}

const expandedGroups = ref(new Set<number>());

function toggleGroup(id: number) {
  expandedGroups.value.has(id) ? expandedGroups.value.delete(id) : expandedGroups.value.add(id);
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 bg-surface border-b border-border flex items-center justify-between shrink-0">
      <div>
        <h2 class="text-xl font-semibold text-text">快速恢复单元</h2>
        <p class="text-sm text-text-secondary mt-0.5">三层结构管理：分组 → 恢复步骤 → 工具链接</p>
      </div>
      <button v-if="canEdit" class="px-4 py-2 bg-primary text-white text-sm rounded-lg hover:bg-primary-hover" @click="newGroup">
        + 新建分组
      </button>
    </header>

    <div class="flex-1 overflow-y-auto p-6">
      <div v-if="groups.length === 0" class="flex flex-col items-center justify-center h-full text-text-secondary">
        <div class="text-4xl mb-4">🔧</div>
        <p class="text-lg">暂无恢复分组</p>
        <p class="text-sm mt-1">创建分组来管理常见故障的恢复步骤和工具链接</p>
      </div>

      <div v-else class="space-y-3">
        <div v-for="group in groups" :key="group.id" class="bg-surface rounded-xl border border-border overflow-hidden">
          <div
            class="px-4 py-3 flex items-center justify-between cursor-pointer hover:bg-surface-alt transition-colors"
            @click="toggleGroup(group.id)"
          >
            <div class="flex items-center gap-2">
              <span class="text-xs text-text-secondary">{{ expandedGroups.has(group.id) ? '▼' : '▶' }}</span>
              <span class="font-medium">{{ group.name }}</span>
              <span class="text-xs text-text-secondary">({{ group.steps.length }} 步骤)</span>
            </div>
            <div v-if="canEdit" class="flex items-center gap-1" @click.stop>
              <button class="px-2 py-1 text-xs text-primary hover:bg-blue-50 rounded" @click="editGroup(group)">编辑</button>
              <button class="px-2 py-1 text-xs text-error hover:bg-red-50 rounded" @click="deleteGroup(group.id)">删除</button>
            </div>
          </div>

          <div v-if="expandedGroups.has(group.id)" class="border-t border-border">
            <div v-for="step in group.steps" :key="step.id" class="border-b border-border last:border-b-0">
              <div class="px-6 py-2.5 flex items-center justify-between bg-surface-alt/50">
                <div>
                  <span class="text-sm font-medium">{{ step.label }}</span>
                  <span v-if="step.description" class="text-xs text-text-secondary ml-2">{{ step.description }}</span>
                </div>
                <span class="text-xs text-text-secondary">{{ step.links.length }} 个链接</span>
              </div>

              <div v-if="step.links.length > 0" class="px-8 py-2 space-y-1">
                <a
                  v-for="link in step.links"
                  :key="link.id"
                  :href="link.url"
                  target="_blank"
                  class="flex items-center gap-2 text-sm text-primary hover:underline py-0.5"
                >
                  <span class="text-text-secondary">🔗</span>
                  {{ link.label || link.url }}
                  <span class="text-xs text-text-secondary">↗</span>
                </a>
              </div>
            </div>

            <div v-if="group.steps.length === 0" class="px-6 py-4 text-sm text-text-secondary text-center">
              暂无恢复步骤
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 编辑弹窗 -->
    <Teleport to="body">
      <div
        v-if="showEditor && editingGroup"
        class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
        @click.self="showEditor = false"
      >
        <div class="bg-surface rounded-xl shadow-xl w-[640px] max-h-[85vh] overflow-y-auto">
          <div class="px-6 py-4 border-b border-border">
            <h3 class="text-lg font-semibold">{{ editingGroup.id ? '编辑恢复分组' : '新建恢复分组' }}</h3>
          </div>

          <div class="px-6 py-4 space-y-4">
            <div>
              <label class="block text-sm font-medium mb-1">分组名称 *</label>
              <input
                v-model="editingGroup.name"
                placeholder="如：Mapping 未生效恢复方案"
                class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary"
              />
            </div>
            <div>
              <label class="block text-sm font-medium mb-1">描述</label>
              <input
                v-model="editingGroup.description"
                placeholder="分组说明..."
                class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary"
              />
            </div>

            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm font-medium">恢复步骤 ({{ editingGroup.steps.length }})</label>
                <div class="flex items-center gap-2">
                  <button class="text-xs text-primary hover:underline" @click="showBatchAdd = !showBatchAdd">批量添加</button>
                  <button class="text-xs text-primary hover:underline" @click="addStep(editingGroup)">+ 添加</button>
                </div>
              </div>

              <div v-if="showBatchAdd" class="border border-primary/30 bg-blue-50/30 rounded-lg p-3 mb-3">
                <p class="text-xs text-text-secondary mb-1.5">每行一个步骤名称，回车分隔：</p>
                <textarea
                  v-model="batchText"
                  rows="4"
                  placeholder="重启 Agent 服务&#10;清除 Redis 缓存&#10;手动触发同步"
                  class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary resize-none font-mono"
                />
                <div class="flex justify-end gap-2 mt-2">
                  <button class="text-xs text-text-secondary hover:text-text" @click="showBatchAdd = false">取消</button>
                  <button
                    class="text-xs px-3 py-1 bg-primary text-white rounded hover:bg-primary-hover disabled:opacity-50"
                    :disabled="!batchText.trim()"
                    @click="batchAddSteps(editingGroup)"
                  >添加 {{ batchText.split('\n').filter(l => l.trim()).length }} 步</button>
                </div>
              </div>

              <div v-for="(step, idx) in editingGroup.steps" :key="step.id" class="border border-border rounded-lg p-3 mb-2 group/step">
                <div class="flex items-center gap-2 mb-2">
                  <span class="text-xs text-text-secondary/50 w-5 text-center shrink-0">{{ idx + 1 }}</span>
                  <input
                    v-model="step.label"
                    placeholder="步骤名称"
                    class="flex-1 px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary"
                    @keydown.enter="addStep(editingGroup)"
                  />
                  <input
                    v-model="step.description"
                    placeholder="说明 (可选)"
                    class="flex-1 px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary"
                  />
                  <div class="flex gap-0.5 opacity-0 group-hover/step:opacity-100 transition-opacity shrink-0">
                    <button class="text-xs text-text-secondary hover:text-primary px-1" title="复制" @click="duplicateStep(editingGroup, idx)">⊕</button>
                    <button class="text-error text-sm px-1" @click="removeStep(editingGroup, idx)">×</button>
                  </div>
                </div>

                <div class="pl-7 space-y-1.5">
                  <div v-for="(link, li) in step.links" :key="link.id" class="flex items-center gap-2">
                    <input
                      v-model="link.label"
                      placeholder="链接名称"
                      class="w-32 px-2 py-1 text-xs border border-border rounded outline-none focus:border-primary"
                    />
                    <input
                      v-model="link.url"
                      placeholder="https://..."
                      class="flex-1 px-2 py-1 text-xs border border-border rounded outline-none focus:border-primary font-mono"
                    />
                    <button class="text-error text-xs px-1" @click="removeLink(step, li)">×</button>
                  </div>
                  <button class="text-[10px] text-primary hover:underline" @click="addLink(step)">+ 添加链接</button>
                </div>
              </div>
            </div>
          </div>

          <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
            <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="showEditor = false">取消</button>
            <button
              class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50"
              :disabled="!editingGroup.name.trim()"
              @click="saveGroup"
            >保存</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
