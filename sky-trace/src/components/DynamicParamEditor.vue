<script setup lang="ts">
import { ref } from "vue";
import type { DynamicParam } from "@/types";

const props = defineProps<{ params: DynamicParam[] }>();
const emit = defineEmits<{ close: []; save: [params: DynamicParam[]] }>();

const list = ref<DynamicParam[]>(
  props.params.map((p) => ({ ...p }))
);

function addParam() {
  list.value.push({
    key: "",
    label: "",
    required: true,
    defaultValue: "",
  });
}

function removeParam(index: number) {
  list.value.splice(index, 1);
}

function handleSave() {
  const valid = list.value.filter((p) => p.key.trim() && p.label.trim());
  emit("save", valid);
}
</script>

<template>
  <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" @click.self="emit('close')">
    <div class="bg-surface rounded-xl shadow-xl w-[520px] max-h-[80vh] overflow-y-auto">
      <div class="px-6 py-4 border-b border-border">
        <h3 class="text-lg font-semibold">编辑动态参数</h3>
        <p class="text-sm text-text-secondary mt-1">
          动态参数在执行时由用户填写，可绑定到节点的 Filter1、Filter2、模糊查询字段
        </p>
      </div>

      <div class="px-6 py-4 space-y-3">
        <div
          v-for="(param, i) in list"
          :key="i"
          class="flex items-start gap-2 p-3 bg-surface-alt rounded-lg"
        >
          <div class="flex-1 grid grid-cols-2 gap-2">
            <div>
              <label class="block text-xs text-text-secondary mb-0.5">参数标识 (key)</label>
              <input
                v-model="param.key"
                placeholder="如 hotelId"
                class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary font-mono"
              />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-0.5">显示名称</label>
              <input
                v-model="param.label"
                placeholder="如 酒店ID"
                class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary"
              />
            </div>
            <div>
              <label class="block text-xs text-text-secondary mb-0.5">默认值</label>
              <input
                v-model="param.defaultValue"
                placeholder="可选"
                class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary"
              />
            </div>
            <div class="flex items-end">
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input type="checkbox" v-model="param.required" class="rounded" />
                必填
              </label>
            </div>
          </div>
          <button class="text-error text-sm mt-5 px-1 shrink-0" @click="removeParam(i)">×</button>
        </div>

        <button
          class="w-full py-2 border border-dashed border-border rounded-lg text-sm text-text-secondary hover:border-primary hover:text-primary transition-colors"
          @click="addParam"
        >
          + 添加参数
        </button>
      </div>

      <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
        <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="emit('close')">取消</button>
        <button class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover" @click="handleSave">保存</button>
      </div>
    </div>
  </div>
</template>
