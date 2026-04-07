<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAppStore } from "@/stores/app";
import type { TraceNode, SkynetQueryConfig, InfoNodeConfig, ChecklistNodeConfig, DynamicParam, FieldBinding } from "@/types";
import { emptyBinding } from "@/types";
import FieldBindingInput from "@/components/FieldBindingInput.vue";

const props = defineProps<{
  node: TraceNode | null;
  dynamicParams: DynamicParam[];
}>();

const emit = defineEmits<{ close: []; save: [node: TraceNode] }>();
const store = useAppStore();

const nodeType = ref<TraceNode["type"]>("skynet_query");
const label = ref("");

// SkynetQueryConfig fields
const skyAppId = ref<number | undefined>();
const module_ = ref("");
const category = ref("");
const subCategory = ref("");
const filter1 = ref<FieldBinding>(emptyBinding());
const filter2 = ref<FieldBinding>(emptyBinding());
const indexContext = ref<FieldBinding>(emptyBinding());
const contextId = ref<FieldBinding>(emptyBinding());
const pageSize = ref(100);
const fieldHints = ref<Record<string, string>>({});

// InfoNodeConfig fields
const infoContent = ref("");
const infoLinks = ref<{ label: string; url: string }[]>([]);

// ChecklistNodeConfig fields
const checklistGroupId = ref<number | undefined>();
const checklistItemIds = ref<string[]>([]);

// Common fields
const notes = ref("");

onMounted(() => {
  if (!props.node) return;
  nodeType.value = props.node.type;
  label.value = props.node.label;
  notes.value = props.node.notes ?? "";

  if (props.node.type === "skynet_query") {
    const cfg = props.node.config as SkynetQueryConfig;
    skyAppId.value = cfg.skyAppId;
    module_.value = cfg.module;
    category.value = cfg.category;
    subCategory.value = cfg.subCategory;
    filter1.value = cfg.filter1;
    filter2.value = cfg.filter2;
    indexContext.value = cfg.indexContext;
    contextId.value = cfg.contextId ?? emptyBinding();
    pageSize.value = cfg.pageSize;
    fieldHints.value = cfg.fieldHints ?? {};
  } else if (props.node.type === "info") {
    const cfg = props.node.config as InfoNodeConfig;
    infoContent.value = cfg.content;
    infoLinks.value = [...cfg.links];
  } else if (props.node.type === "checklist") {
    const cfg = props.node.config as ChecklistNodeConfig;
    checklistGroupId.value = cfg.checklistGroupId;
    checklistItemIds.value = [...(cfg.itemIds || [])];
  }
});

function addLink() {
  infoLinks.value.push({ label: "", url: "" });
}

function removeLink(index: number) {
  infoLinks.value.splice(index, 1);
}

function toggleChecklistItem(itemId: string) {
  const idx = checklistItemIds.value.indexOf(itemId);
  if (idx >= 0) {
    checklistItemIds.value.splice(idx, 1);
  } else {
    checklistItemIds.value.push(itemId);
  }
}

function handleSave() {
  const id = props.node?.id || `node_${Date.now()}`;

  let config: TraceNode["config"];
  if (nodeType.value === "skynet_query") {
    config = {
      skyAppId: skyAppId.value!,
      module: module_.value,
      category: category.value,
      subCategory: subCategory.value,
      filter1: filter1.value,
      filter2: filter2.value,
      indexContext: indexContext.value,
      contextId: contextId.value,
      pageSize: pageSize.value,
      fieldHints: Object.keys(fieldHints.value).some((k) => fieldHints.value[k]) ? fieldHints.value : undefined,
    } as SkynetQueryConfig;
  } else if (nodeType.value === "checklist") {
    config = {
      checklistGroupId: checklistGroupId.value!,
      itemIds: checklistItemIds.value,
    } as ChecklistNodeConfig;
  } else {
    config = {
      content: infoContent.value,
      links: infoLinks.value.filter((l) => l.label && l.url),
    } as InfoNodeConfig;
  }

  emit("save", {
    id,
    type: nodeType.value,
    label: label.value,
    sortOrder: props.node?.sortOrder ?? 0,
    config,
    notes: notes.value || undefined,
  });
}

const saveDisabled = () => {
  if (!label.value.trim()) return true;
  if (nodeType.value === "skynet_query" && !skyAppId.value) return true;
  if (nodeType.value === "checklist" && !checklistGroupId.value) return true;
  return false;
};
</script>

<template>
  <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" @click.self="emit('close')">
    <div class="bg-surface rounded-xl shadow-xl w-[580px] max-h-[85vh] overflow-y-auto">
      <div class="px-6 py-4 border-b border-border">
        <h3 class="text-lg font-semibold">{{ node ? '编辑节点' : '添加节点' }}</h3>
      </div>

      <div class="px-6 py-4 space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">节点类型</label>
          <div class="flex gap-2">
            <button
              v-for="t in (['skynet_query', 'info', 'checklist'] as const)"
              :key="t"
              class="px-3 py-1.5 text-sm rounded-lg border transition-colors"
              :class="nodeType === t ? 'border-primary bg-blue-50 text-primary' : 'border-border text-text-secondary hover:border-primary/30'"
              @click="nodeType = t"
            >{{ t === 'skynet_query' ? '天网查询' : t === 'checklist' ? '监控Checklist' : '信息节点' }}</button>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">节点名称 *</label>
          <input v-model="label" placeholder="如：查 RMQ consume 日志" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary" />
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">参考备注</label>
          <textarea v-model="notes" rows="2" placeholder="排查参考信息，如错误码映射、注意事项等..." class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary resize-none" />
        </div>

        <template v-if="nodeType === 'skynet_query'">
          <div>
            <label class="block text-sm font-medium mb-1">天网应用 *</label>
            <select v-model="skyAppId" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary">
              <option :value="undefined" disabled>选择天网应用</option>
              <option v-for="app in store.skyApps" :key="app.id" :value="app.id">{{ app.name || app.appUk }} ({{ app.appId }})</option>
            </select>
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div>
              <label class="block text-xs text-text-secondary mb-1">Module (固定)</label>
              <input v-model="module_" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-1">Category (固定)</label>
              <input v-model="category" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-1">SubCategory (固定)</label>
              <input v-model="subCategory" class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
            </div>
          </div>

          <div class="space-y-3">
            <div v-for="[fieldLabel, fieldKey] in ([['Filter1', 'filter1'], ['Filter2', 'filter2'], ['Msg 模糊查询', 'indexContext'], ['TraceId (contextId)', 'contextId']] as const)" :key="fieldKey">
              <FieldBindingInput
                :label="fieldLabel"
                :model-value="fieldKey === 'filter1' ? filter1 : fieldKey === 'filter2' ? filter2 : fieldKey === 'indexContext' ? indexContext : contextId"
                @update:model-value="fieldKey === 'filter1' ? filter1 = $event : fieldKey === 'filter2' ? filter2 = $event : fieldKey === 'indexContext' ? indexContext = $event : contextId = $event"
                :dynamic-params="dynamicParams"
                :hint="fieldHints[fieldKey]"
              />
              <div class="ml-24 mt-1">
                <input
                  :value="fieldHints[fieldKey] ?? ''"
                  @input="fieldHints[fieldKey] = ($event.target as HTMLInputElement).value"
                  placeholder="参考提示 (可选)，如: 三要素拼接 id_room_rpid"
                  class="w-full px-2 py-1 text-[11px] border border-dashed border-border rounded outline-none focus:border-amber-400 text-text-secondary"
                />
              </div>
            </div>
          </div>

          <div>
            <label class="block text-xs text-text-secondary mb-1">每页条数</label>
            <input v-model.number="pageSize" type="number" min="1" max="500" class="w-32 px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
          </div>
        </template>

        <template v-else-if="nodeType === 'checklist'">
          <div>
            <label class="block text-sm font-medium mb-1">Checklist 分组 *</label>
            <select v-model="checklistGroupId" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary">
              <option :value="undefined" disabled>选择分组</option>
              <option v-for="g in store.checklists" :key="g.id" :value="g.id">{{ g.name }} ({{ g.items.length }} 项)</option>
            </select>
          </div>
          <div v-if="checklistGroupId">
            <label class="block text-sm font-medium mb-1">选择检查项 <span class="text-xs text-text-secondary font-normal">(不选则显示全部)</span></label>
            <div class="space-y-1 max-h-40 overflow-y-auto border border-border rounded-lg p-2">
              <label
                v-for="item in store.checklistMap.get(checklistGroupId)?.items ?? []"
                :key="item.id"
                class="flex items-center gap-2 py-1 px-1 hover:bg-surface-alt rounded cursor-pointer text-sm"
              >
                <input
                  type="checkbox"
                  :checked="checklistItemIds.includes(item.id)"
                  class="rounded"
                  @change="toggleChecklistItem(item.id)"
                />
                {{ item.label }}
                <span class="text-xs text-text-secondary">({{ item.links.length }} 链接)</span>
              </label>
            </div>
          </div>
        </template>

        <template v-else-if="nodeType === 'info'">
          <div>
            <label class="block text-sm font-medium mb-1">信息内容</label>
            <textarea v-model="infoContent" rows="4" placeholder="排查说明、SQL模板、Redis命令等..." class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary resize-none font-mono" />
          </div>
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="text-sm font-medium">外部链接</label>
              <button class="text-xs text-primary hover:underline" @click="addLink">+ 添加链接</button>
            </div>
            <div v-for="(link, i) in infoLinks" :key="i" class="flex gap-2 mb-2">
              <input v-model="link.label" placeholder="链接名称" class="flex-1 px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
              <input v-model="link.url" placeholder="URL" class="flex-[2] px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary" />
              <button class="text-error text-sm px-2" @click="removeLink(i)">×</button>
            </div>
          </div>
        </template>
      </div>

      <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
        <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="emit('close')">取消</button>
        <button
          class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50"
          :disabled="saveDisabled()"
          @click="handleSave"
        >保存</button>
      </div>
    </div>
  </div>
</template>
