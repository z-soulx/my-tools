<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import type { ChecklistGroup, ChecklistItem } from "@/types";

const store = useAppStore();
const groups = ref<ChecklistGroup[]>([]);
const editingGroup = ref<ChecklistGroup | null>(null);
const showEditor = ref(false);

const canEdit = (!store.snapshotMode || !store.snapshotRestrictions.hideChecklistEdit) && store.featureEnabled('checklistEdit');

onMounted(async () => {
  await loadGroups();
});

async function loadGroups() {
  if (store.snapshotMode) {
    groups.value = store.checklists;
    return;
  }
  groups.value = await api.getChecklistGroups();
  store.checklists = groups.value;
}

function newGroup() {
  editingGroup.value = {
    id: 0,
    name: "",
    description: "",
    items: [],
    createdAt: "",
    updatedAt: "",
  };
  showEditor.value = true;
}

function editGroup(g: ChecklistGroup) {
  editingGroup.value = JSON.parse(JSON.stringify(g));
  showEditor.value = true;
}

async function deleteGroup(id: number) {
  await api.deleteChecklistGroup(id);
  await loadGroups();
}

const showBatchAdd = ref(false);
const batchText = ref("");

function addItem(g: ChecklistGroup) {
  g.items.push({
    id: `item_${Date.now()}`,
    label: "",
    description: "",
    links: [],
  });
}

function batchAddItems(g: ChecklistGroup) {
  const lines = batchText.value.split("\n").map((l) => l.trim()).filter(Boolean);
  for (const line of lines) {
    g.items.push({
      id: `item_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`,
      label: line,
      description: "",
      links: [],
    });
  }
  batchText.value = "";
  showBatchAdd.value = false;
}

function removeItem(g: ChecklistGroup, index: number) {
  g.items.splice(index, 1);
}

function addLink(item: ChecklistItem) {
  item.links.push({ id: `link_${Date.now()}`, label: "", url: "" });
}

function removeLink(item: ChecklistItem, index: number) {
  item.links.splice(index, 1);
}

function duplicateItem(g: ChecklistGroup, index: number) {
  const src = g.items[index];
  g.items.splice(index + 1, 0, {
    id: `item_${Date.now()}`,
    label: src.label + " (副本)",
    description: src.description,
    links: src.links.map((l) => ({ ...l, id: `link_${Date.now()}_${Math.random().toString(36).slice(2, 6)}` })),
  });
}

async function saveGroup() {
  if (!editingGroup.value || !editingGroup.value.name.trim()) return;
  const g = editingGroup.value;
  await api.saveChecklistGroup({
    id: g.id || undefined,
    name: g.name.trim(),
    description: g.description.trim(),
    items: g.items.filter((i) => i.label.trim()),
  });
  showEditor.value = false;
  editingGroup.value = null;
  await loadGroups();
}

const expandedGroups = ref(new Set<number>());

function toggleGroup(id: number) {
  if (expandedGroups.value.has(id)) {
    expandedGroups.value.delete(id);
  } else {
    expandedGroups.value.add(id);
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 bg-surface border-b border-border flex items-center justify-between shrink-0">
      <div>
        <h2 class="text-xl font-semibold text-text">监控 Checklist</h2>
        <p class="text-sm text-text-secondary mt-0.5">三层结构管理：分组 → 检查项 → 链接</p>
      </div>
      <button v-if="canEdit" class="px-4 py-2 bg-primary text-white text-sm rounded-lg hover:bg-primary-hover" @click="newGroup">
        + 新建分组
      </button>
    </header>

    <div class="flex-1 overflow-y-auto p-6">
      <div v-if="groups.length === 0" class="flex flex-col items-center justify-center h-full text-text-secondary">
        <div class="text-4xl mb-4">📋</div>
        <p class="text-lg">暂无 Checklist 分组</p>
        <p class="text-sm mt-1">创建分组来管理监控检查项和链接</p>
      </div>

      <div v-else class="space-y-3">
        <div v-for="group in groups" :key="group.id" class="bg-surface rounded-xl border border-border overflow-hidden">
          <!-- 分组标题 (第一层) -->
          <div
            class="px-4 py-3 flex items-center justify-between cursor-pointer hover:bg-surface-alt transition-colors"
            @click="toggleGroup(group.id)"
          >
            <div class="flex items-center gap-2">
              <span class="text-xs text-text-secondary">{{ expandedGroups.has(group.id) ? '▼' : '▶' }}</span>
              <span class="font-medium">{{ group.name }}</span>
              <span class="text-xs text-text-secondary">({{ group.items.length }} 项)</span>
            </div>
            <div v-if="canEdit" class="flex items-center gap-1" @click.stop>
              <button class="px-2 py-1 text-xs text-primary hover:bg-blue-50 rounded" @click="editGroup(group)">编辑</button>
              <button class="px-2 py-1 text-xs text-error hover:bg-red-50 rounded" @click="deleteGroup(group.id)">删除</button>
            </div>
          </div>

          <!-- 检查项列表 (第二层) -->
          <div v-if="expandedGroups.has(group.id)" class="border-t border-border">
            <div v-for="item in group.items" :key="item.id" class="border-b border-border last:border-b-0">
              <div class="px-6 py-2.5 flex items-center justify-between bg-surface-alt/50">
                <div>
                  <span class="text-sm font-medium">{{ item.label }}</span>
                  <span v-if="item.description" class="text-xs text-text-secondary ml-2">{{ item.description }}</span>
                </div>
                <span class="text-xs text-text-secondary">{{ item.links.length }} 个链接</span>
              </div>

              <!-- 链接列表 (第三层) -->
              <div v-if="item.links.length > 0" class="px-8 py-2 space-y-1">
                <a
                  v-for="link in item.links"
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

            <div v-if="group.items.length === 0" class="px-6 py-4 text-sm text-text-secondary text-center">
              暂无检查项
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
            <h3 class="text-lg font-semibold">{{ editingGroup.id ? '编辑分组' : '新建分组' }}</h3>
          </div>

          <div class="px-6 py-4 space-y-4">
            <div>
              <label class="block text-sm font-medium mb-1">分组名称 *</label>
              <input
                v-model="editingGroup.name"
                placeholder="如：铂涛监控"
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

            <!-- 检查项列表 -->
            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm font-medium">检查项 ({{ editingGroup.items.length }})</label>
                <div class="flex items-center gap-2">
                  <button class="text-xs text-primary hover:underline" @click="showBatchAdd = !showBatchAdd">批量添加</button>
                  <button class="text-xs text-primary hover:underline" @click="addItem(editingGroup)">+ 添加</button>
                </div>
              </div>

              <!-- 批量添加面板 -->
              <div v-if="showBatchAdd" class="border border-primary/30 bg-blue-50/30 rounded-lg p-3 mb-3">
                <p class="text-xs text-text-secondary mb-1.5">每行一个检查项名称，回车分隔：</p>
                <textarea
                  v-model="batchText"
                  rows="4"
                  placeholder="RMQ consume 确认&#10;Redis 品牌检查&#10;活动列表查询"
                  class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary resize-none font-mono"
                />
                <div class="flex justify-end gap-2 mt-2">
                  <button class="text-xs text-text-secondary hover:text-text" @click="showBatchAdd = false">取消</button>
                  <button
                    class="text-xs px-3 py-1 bg-primary text-white rounded hover:bg-primary-hover disabled:opacity-50"
                    :disabled="!batchText.trim()"
                    @click="batchAddItems(editingGroup)"
                  >添加 {{ batchText.split('\n').filter(l => l.trim()).length }} 项</button>
                </div>
              </div>

              <div v-for="(item, idx) in editingGroup.items" :key="item.id" class="border border-border rounded-lg p-3 mb-2 group/item">
                <div class="flex items-center gap-2 mb-2">
                  <span class="text-xs text-text-secondary/50 w-5 text-center shrink-0">{{ idx + 1 }}</span>
                  <input
                    v-model="item.label"
                    placeholder="检查项名称"
                    class="flex-1 px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary"
                    @keydown.enter="addItem(editingGroup)"
                  />
                  <input
                    v-model="item.description"
                    placeholder="说明 (可选)"
                    class="flex-1 px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary"
                  />
                  <div class="flex gap-0.5 opacity-0 group-hover/item:opacity-100 transition-opacity shrink-0">
                    <button class="text-xs text-text-secondary hover:text-primary px-1" title="复制" @click="duplicateItem(editingGroup, idx)">⊕</button>
                    <button class="text-error text-sm px-1" @click="removeItem(editingGroup, idx)">×</button>
                  </div>
                </div>

                <!-- 链接 -->
                <div class="pl-7 space-y-1.5">
                  <div v-for="(link, li) in item.links" :key="link.id" class="flex items-center gap-2">
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
                    <button class="text-error text-xs px-1" @click="removeLink(item, li)">×</button>
                  </div>
                  <button class="text-[10px] text-primary hover:underline" @click="addLink(item)">+ 添加链接</button>
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
