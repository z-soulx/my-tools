<script setup lang="ts">
import { ref } from "vue";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import type { DynamicParam, TraceFlow } from "@/types";

const props = withDefaults(defineProps<{
  sourceFlow?: TraceFlow | null;
}>(), { sourceFlow: null });

const emit = defineEmits<{ close: []; saved: [] }>();
const store = useAppStore();

const isEdit = !!props.sourceFlow;

const form = ref({
  name: props.sourceFlow ? props.sourceFlow.name + " (副本)" : "",
  description: props.sourceFlow?.description ?? "",
  supplierId: props.sourceFlow?.supplierId ?? null as number | null,
  tags: props.sourceFlow?.tags.join(", ") ?? "",
});

const dynamicParams = ref<DynamicParam[]>(
  props.sourceFlow ? props.sourceFlow.dynamicParams.map((p) => ({ ...p })) : []
);
const saving = ref(false);

function addParam() {
  dynamicParams.value.push({
    key: "",
    label: "",
    required: true,
    defaultValue: "",
  });
}

function removeParam(index: number) {
  dynamicParams.value.splice(index, 1);
}

async function handleSave() {
  if (!form.value.name.trim()) return;
  saving.value = true;
  try {
    await api.saveFlow({
      name: form.value.name.trim(),
      description: form.value.description.trim(),
      supplierId: form.value.supplierId,
      tags: form.value.tags.split(",").map((t) => t.trim()).filter(Boolean),
      dynamicParams: dynamicParams.value.filter((p) => p.key.trim() && p.label.trim()),
      nodes: props.sourceFlow ? props.sourceFlow.nodes : [],
    });
    emit("saved");
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" @click.self="emit('close')">
    <div class="bg-surface rounded-xl shadow-xl w-[520px] max-h-[85vh] overflow-y-auto">
      <div class="px-6 py-4 border-b border-border">
        <h3 class="text-lg font-semibold">{{ isEdit ? '复制链路 - 编辑基础信息' : '新建排查链路' }}</h3>
      </div>

      <div class="px-6 py-4 space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">链路名称 *</label>
          <input v-model="form.name" placeholder="如：铂涛 Mapping 未生效排查" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary" />
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">描述</label>
          <textarea v-model="form.description" placeholder="排查场景说明..." rows="2" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary resize-none" />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-sm font-medium mb-1">关联供应商</label>
            <select v-model="form.supplierId" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary">
              <option :value="null">不关联</option>
              <option v-for="s in store.suppliers" :key="s.id" :value="s.id">{{ s.name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium mb-1">标签</label>
            <input v-model="form.tags" placeholder="用逗号分隔" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary" />
          </div>
        </div>

        <!-- 动态参数 -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm font-medium">动态参数</label>
            <button class="text-xs text-primary hover:underline" @click="addParam">+ 添加</button>
          </div>
          <p class="text-xs text-text-secondary mb-2">执行时由用户填写的参数，可绑定到查询节点的 Filter1/Filter2/模糊查询</p>
          <div v-for="(param, i) in dynamicParams" :key="i" class="flex items-center gap-2 mb-2">
            <input v-model="param.key" placeholder="key (如 hotelId)" class="flex-1 px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary font-mono" />
            <input v-model="param.label" placeholder="显示名 (如 酒店ID)" class="flex-1 px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary" />
            <label class="flex items-center gap-1 text-xs shrink-0">
              <input type="checkbox" v-model="param.required" class="rounded" /> 必填
            </label>
            <button class="text-error text-sm px-1" @click="removeParam(i)">×</button>
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
        <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="emit('close')">取消</button>
        <button
          class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50"
          :disabled="!form.name.trim() || saving"
          @click="handleSave"
        >{{ saving ? '保存中...' : isEdit ? '复制并保存' : '创建' }}</button>
      </div>
    </div>
  </div>
</template>
