<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAppStore } from "@/stores/app";
import type { TraceNode, SkynetQueryConfig, InfoNodeConfig, ChecklistNodeConfig, JcpOrderConfig, JcpExtractMapping, DynamicParam, FieldBinding, AdvancedSearchItem } from "@/types";
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
const advancedSearchItems = ref<AdvancedSearchItem[]>([]);
const showAdvancedSearch = ref(false);

// InfoNodeConfig fields
const infoContent = ref("");
const infoLinks = ref<{ label: string; url: string }[]>([]);

// ChecklistNodeConfig fields
const checklistGroupId = ref<number | undefined>();
const checklistItemIds = ref<string[]>([]);

// JcpOrderConfig fields
const jcpQueryField = ref<"orderId" | "traceId" | "runtime">("traceId");
const jcpQueryValue = ref<FieldBinding>(emptyBinding());
const jcpRequestTimeWindowBefore = ref(5);
const jcpRequestTimeWindowAfter = ref(5);
const JCP_EXTRACT_FIELDS = ["roomTypeId", "shotelId", "ratePlanId", "checkInDate", "checkOutDate", "requestTime", "createDate"] as const;
const jcpExtractMappings = ref<JcpExtractMapping[]>(
  JCP_EXTRACT_FIELDS.map((f) => ({ sourceField: f, targetParamKey: "" }))
);
const jcpSupplierMappingEnabled = ref(false);
const SUPPLIER_EXTRACT_FIELDS = ["supplierHotelId", "supplierRatePlanId", "supplierRoomTypeId"] as const;
const jcpSupplierExtractMappings = ref<JcpExtractMapping[]>(
  SUPPLIER_EXTRACT_FIELDS.map((f) => ({ sourceField: f, targetParamKey: "" }))
);

// Common fields
const notes = ref("");

const derivedFormatsHelp = [
  "{{参数key}}      — 原始值",
  "{{参数key}}_ymd  — yyyy-MM-dd 日期",
  "{{参数key}}_full — yyyy-MM-dd HH:mm:ss 完整时间",
  "{{参数key}}_ts   — 毫秒时间戳",
  "{{参数key}}_tsSec — 秒级时间戳",
  "{{参数key}}_dayTs — 当日零点秒级时间戳",
  "",
  "分割语法: {{参数key:split(分隔符,索引)}}",
  "例: 值 \"9_41_42177771\" 用 {{key:split(_,0)}} 取 \"9\"",
  "    {{key:split(_,1)}} 取 \"41\"",
  "    {{key:split(_,2)}} 取 \"42177771\"",
].join("\n");

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
    advancedSearchItems.value = cfg.advancedSearchItems ? cfg.advancedSearchItems.map(item => ({ ...item, value: { ...item.value } })) : [];
    showAdvancedSearch.value = advancedSearchItems.value.length > 0;
  } else if (props.node.type === "info") {
    const cfg = props.node.config as InfoNodeConfig;
    infoContent.value = cfg.content;
    infoLinks.value = [...cfg.links];
  } else if (props.node.type === "checklist") {
    const cfg = props.node.config as ChecklistNodeConfig;
    checklistGroupId.value = cfg.checklistGroupId;
    checklistItemIds.value = [...(cfg.itemIds || [])];
  } else if (props.node.type === "jcp_order") {
    const cfg = props.node.config as JcpOrderConfig;
    jcpQueryField.value = cfg.queryField;
    jcpQueryValue.value = cfg.queryValue;
    const legacy = cfg.requestTimeWindow ?? 5;
    jcpRequestTimeWindowBefore.value = cfg.requestTimeWindowBefore ?? legacy;
    jcpRequestTimeWindowAfter.value = cfg.requestTimeWindowAfter ?? legacy;
    jcpExtractMappings.value = JCP_EXTRACT_FIELDS.map((f) => {
      const existing = cfg.extractMappings?.find((m) => m.sourceField === f);
      return { sourceField: f, targetParamKey: existing?.targetParamKey ?? "" };
    });
    jcpSupplierMappingEnabled.value = cfg.supplierMappingEnabled ?? false;
    jcpSupplierExtractMappings.value = SUPPLIER_EXTRACT_FIELDS.map((f) => {
      const existing = cfg.supplierExtractMappings?.find((m) => m.sourceField === f);
      return { sourceField: f, targetParamKey: existing?.targetParamKey ?? "" };
    });
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
      advancedSearchItems: advancedSearchItems.value.length > 0 ? advancedSearchItems.value : undefined,
    } as SkynetQueryConfig;
  } else if (nodeType.value === "checklist") {
    config = {
      checklistGroupId: checklistGroupId.value!,
      itemIds: checklistItemIds.value,
    } as ChecklistNodeConfig;
  } else if (nodeType.value === "jcp_order") {
    config = {
      queryField: jcpQueryField.value,
      queryValue: jcpQueryValue.value,
      extractMappings: jcpExtractMappings.value.filter((m) => m.targetParamKey),
      requestTimeWindowBefore: jcpRequestTimeWindowBefore.value,
      requestTimeWindowAfter: jcpRequestTimeWindowAfter.value,
      supplierMappingEnabled: jcpSupplierMappingEnabled.value || undefined,
      supplierExtractMappings: jcpSupplierMappingEnabled.value
        ? jcpSupplierExtractMappings.value.filter((m) => m.targetParamKey)
        : undefined,
    } as JcpOrderConfig;
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

function addAdvancedSearchItem() {
  advancedSearchItems.value.push({ filter: "indexContext", compare: "like", value: emptyBinding() });
}

function removeAdvancedSearchItem(index: number) {
  advancedSearchItems.value.splice(index, 1);
}
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
          <div class="flex gap-2 flex-wrap">
            <button
              v-for="t in (['skynet_query', 'info', 'checklist', 'jcp_order'] as const)"
              :key="t"
              class="px-3 py-1.5 text-sm rounded-lg border transition-colors"
              :class="nodeType === t ? 'border-primary bg-blue-50 text-primary' : 'border-border text-text-secondary hover:border-primary/30'"
              @click="nodeType = t"
            >{{ t === 'skynet_query' ? '天网查询' : t === 'checklist' ? '监控Checklist' : t === 'jcp_order' ? '产品组成单分析' : '信息节点' }}</button>
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

          <div class="border border-border rounded-lg p-3 space-y-3">
            <label class="flex items-center gap-2 cursor-pointer select-none">
              <input type="checkbox" v-model="showAdvancedSearch" class="rounded" />
              <span class="text-sm font-medium">高级搜索条件 (advancedSearchItems)</span>
              <span class="text-xs text-text-secondary">多条 like / nlike 并发过滤</span>
            </label>

            <template v-if="showAdvancedSearch">
              <div v-for="(item, idx) in advancedSearchItems" :key="idx" class="space-y-1">
                <div class="flex items-center gap-2">
                  <select v-model="item.filter" class="px-2 py-1.5 border border-border rounded-lg text-xs outline-none focus:border-primary">
                    <option value="indexContext">indexContext (Msg)</option>
                    <option value="filter1">filter1</option>
                    <option value="filter2">filter2</option>
                  </select>
                  <select v-model="item.compare" class="px-2 py-1.5 border border-border rounded-lg text-xs outline-none focus:border-primary">
                    <option value="like">like（包含）</option>
                    <option value="nlike">nlike（排除）</option>
                  </select>
                  <button @click="removeAdvancedSearchItem(idx)" class="ml-auto text-xs text-red-400 hover:text-red-600 px-2 py-1 rounded hover:bg-red-50 transition-colors">删除</button>
                </div>
                <FieldBindingInput
                  label="值"
                  :model-value="item.value"
                  @update:model-value="item.value = $event"
                  :dynamic-params="dynamicParams"
                />
              </div>

              <button @click="addAdvancedSearchItem" class="w-full text-xs text-primary border border-dashed border-primary/40 rounded-lg py-1.5 hover:bg-blue-50 transition-colors">
                + 添加条件
              </button>
            </template>
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

        <template v-else-if="nodeType === 'jcp_order'">
          <div>
            <label class="block text-sm font-medium mb-1">查询维度</label>
            <div class="flex gap-3">
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input type="radio" v-model="jcpQueryField" value="traceId" class="accent-primary" />
                traceId
              </label>
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input type="radio" v-model="jcpQueryField" value="orderId" class="accent-primary" />
                订单号 (orderId)
              </label>
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input type="radio" v-model="jcpQueryField" value="runtime" class="accent-primary" />
                运行时选择
              </label>
            </div>
          </div>
          <FieldBindingInput
            :label="jcpQueryField === 'traceId' ? 'traceId' : jcpQueryField === 'orderId' ? '订单号' : '查询值'"
            v-model="jcpQueryValue"
            :dynamic-params="dynamicParams"
          />
          <div>
            <label class="block text-sm font-medium mb-2">提取参数映射 <span class="text-xs text-text-secondary font-normal">(提取后自动填入动态参数)</span></label>
            <div class="space-y-2 border border-border rounded-lg p-3 bg-surface-alt/30">
              <div
                v-for="(m, i) in jcpExtractMappings"
                :key="m.sourceField"
                class="flex items-center gap-2"
              >
                <span class="text-xs font-mono text-text-secondary w-28 shrink-0">{{ m.sourceField }}</span>
                <span class="text-xs text-text-secondary">→</span>
                <select
                  v-model="jcpExtractMappings[i].targetParamKey"
                  class="flex-1 px-2 py-1 border border-border rounded text-sm outline-none focus:border-primary"
                >
                  <option value="">不提取</option>
                  <option v-for="p in dynamicParams" :key="p.key" :value="p.key">{{ p.label }} ({{ p.key }})</option>
                </select>
              </div>
            </div>
            <div class="mt-3 p-3 bg-amber-50/60 border border-amber-200/60 rounded-lg text-xs text-amber-800 leading-relaxed space-y-1.5">
              <div class="font-medium">使用说明</div>
              <div class="flex items-center gap-2 pt-0.5 flex-wrap">
                <label class="shrink-0">requestTime 聚焦</label>
                <span>前</span>
                <input
                  v-model.number="jcpRequestTimeWindowBefore"
                  type="number"
                  min="0"
                  max="120"
                  class="w-14 px-2 py-0.5 border border-amber-300 rounded text-xs outline-none focus:border-amber-500 bg-white/60 text-center"
                />
                <span>分钟 / 后</span>
                <input
                  v-model.number="jcpRequestTimeWindowAfter"
                  type="number"
                  min="0"
                  max="120"
                  class="w-14 px-2 py-0.5 border border-amber-300 rounded text-xs outline-none focus:border-amber-500 bg-white/60 text-center"
                />
                <span>分钟（时间范围 = requestTime/createDate - 前 ~ requestTime/createDate + 后）</span>
              </div>
              <div>时间字段 (checkInDate, checkOutDate, requestTime, createDate) 提取后自动派生多种格式的隐藏参数，可在后续天网节点的模板中直接引用：</div>
              <pre class="font-mono bg-white/50 rounded px-2 py-1.5 text-xs whitespace-pre-wrap">{{ derivedFormatsHelp }}</pre>
              <div>例: 动态参数 key 为 <span class="font-mono">cin</span>，模板中用 <span class="font-mono text-amber-600" v-text="'{{cin_ymd}}'"></span> 取日期。</div>
              <div class="pt-1 border-t border-amber-200/60">requestTime / createDate 提取后，<span class="font-medium">查询时间范围自动聚焦到请求时间窗口</span>（无需配置提取映射即可生效）。</div>
            </div>
          </div>

          <div>
            <label class="flex items-center gap-2 text-sm font-medium cursor-pointer">
              <input type="checkbox" v-model="jcpSupplierMappingEnabled" class="rounded accent-primary" />
              启用供应商映射查询
              <span class="text-xs text-text-secondary font-normal">（自动用 shotelId/roomTypeId/ratePlanId 查询供应商侧 ID）</span>
            </label>
            <div v-if="jcpSupplierMappingEnabled" class="mt-2 space-y-2 border border-border rounded-lg p-3 bg-surface-alt/30">
              <div class="text-xs text-text-secondary mb-1">供应商映射提取</div>
              <div
                v-for="(m, i) in jcpSupplierExtractMappings"
                :key="m.sourceField"
                class="flex items-center gap-2"
              >
                <span class="text-xs font-mono text-text-secondary w-36 shrink-0">{{ m.sourceField }}</span>
                <span class="text-xs text-text-secondary">→</span>
                <select
                  v-model="jcpSupplierExtractMappings[i].targetParamKey"
                  class="flex-1 px-2 py-1 border border-border rounded text-sm outline-none focus:border-primary"
                >
                  <option value="">不提取</option>
                  <option v-for="p in dynamicParams" :key="p.key" :value="p.key">{{ p.label }} ({{ p.key }})</option>
                </select>
              </div>
              <div class="text-[11px] text-text-secondary/70 pt-1">
                需要先配置 shotelId、roomTypeId、ratePlanId 的提取映射，供应商映射才能自动调用。
              </div>
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
