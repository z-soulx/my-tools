<script setup lang="ts">
import type { FieldBinding, DynamicParam } from "@/types";

const props = defineProps<{
  label: string;
  modelValue: FieldBinding;
  dynamicParams: DynamicParam[];
}>();

const emit = defineEmits<{
  "update:modelValue": [value: FieldBinding];
}>();

function setMode(mode: "fixed" | "dynamic") {
  emit("update:modelValue", {
    ...props.modelValue,
    mode,
  });
}

function setFixedValue(value: string) {
  emit("update:modelValue", {
    mode: "fixed",
    fixedValue: value,
    paramKey: props.modelValue.paramKey,
  });
}

function setParamKey(key: string) {
  emit("update:modelValue", {
    mode: "dynamic",
    fixedValue: props.modelValue.fixedValue,
    paramKey: key,
  });
}
</script>

<template>
  <div class="flex items-start gap-2">
    <div class="shrink-0 w-24 pt-1.5">
      <span class="text-xs text-text-secondary">{{ label }}</span>
    </div>
    <div class="flex-1">
      <div class="flex items-center gap-2 mb-1.5">
        <button
          class="px-2 py-0.5 text-xs rounded border transition-colors"
          :class="modelValue.mode === 'fixed' ? 'border-primary bg-blue-50 text-primary' : 'border-border text-text-secondary'"
          @click="setMode('fixed')"
        >固定值</button>
        <button
          class="px-2 py-0.5 text-xs rounded border transition-colors"
          :class="modelValue.mode === 'dynamic' ? 'border-primary bg-blue-50 text-primary' : 'border-border text-text-secondary'"
          :disabled="dynamicParams.length === 0"
          @click="setMode('dynamic')"
        >绑定参数</button>
      </div>

      <input
        v-if="modelValue.mode === 'fixed'"
        :value="modelValue.fixedValue"
        @input="setFixedValue(($event.target as HTMLInputElement).value)"
        placeholder="填写固定值"
        class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary"
      />

      <select
        v-else
        :value="modelValue.paramKey"
        @change="setParamKey(($event.target as HTMLSelectElement).value)"
        class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary"
      >
        <option value="" disabled>选择动态参数</option>
        <option v-for="p in dynamicParams" :key="p.key" :value="p.key">
          {{ p.label }} ({{ p.key }})
        </option>
      </select>
    </div>
  </div>
</template>
