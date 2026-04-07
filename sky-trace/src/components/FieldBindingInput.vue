<script setup lang="ts">
import { ref } from "vue";
import type { FieldBinding, DynamicParam } from "@/types";

const props = defineProps<{
  label: string;
  modelValue: FieldBinding;
  dynamicParams: DynamicParam[];
  hint?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: FieldBinding];
}>();

const templateInput = ref<HTMLInputElement | null>(null);

function setMode(mode: "fixed" | "dynamic" | "template") {
  emit("update:modelValue", {
    ...props.modelValue,
    mode,
  });
}

function setFixedValue(value: string) {
  emit("update:modelValue", {
    ...props.modelValue,
    mode: "fixed",
    fixedValue: value,
  });
}

function setParamKey(key: string) {
  emit("update:modelValue", {
    ...props.modelValue,
    mode: "dynamic",
    paramKey: key,
  });
}

function setTemplateValue(value: string) {
  emit("update:modelValue", {
    ...props.modelValue,
    mode: "template",
    templateValue: value,
  });
}

function insertTag(key: string) {
  const tag = `{{${key}}}`;
  const el = templateInput.value;
  if (el) {
    const start = el.selectionStart ?? el.value.length;
    const end = el.selectionEnd ?? start;
    const newVal = el.value.slice(0, start) + tag + el.value.slice(end);
    setTemplateValue(newVal);
    // restore cursor after tag
    requestAnimationFrame(() => {
      el.focus();
      const pos = start + tag.length;
      el.setSelectionRange(pos, pos);
    });
  } else {
    setTemplateValue((props.modelValue.templateValue ?? "") + tag);
  }
}

function tagLabel(key: string) {
  return "{{" + key + "}}";
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
        <button
          class="px-2 py-0.5 text-xs rounded border transition-colors"
          :class="modelValue.mode === 'template' ? 'border-primary bg-blue-50 text-primary' : 'border-border text-text-secondary'"
          :disabled="dynamicParams.length === 0"
          @click="setMode('template')"
        >模板</button>
      </div>

      <input
        v-if="modelValue.mode === 'fixed'"
        :value="modelValue.fixedValue"
        @input="setFixedValue(($event.target as HTMLInputElement).value)"
        placeholder="填写固定值"
        class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary"
      />

      <select
        v-else-if="modelValue.mode === 'dynamic'"
        :value="modelValue.paramKey"
        @change="setParamKey(($event.target as HTMLSelectElement).value)"
        class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary"
      >
        <option value="" disabled>选择动态参数</option>
        <option v-for="p in dynamicParams" :key="p.key" :value="p.key">
          {{ p.label }} ({{ p.key }})
        </option>
      </select>

      <template v-else>
        <input
          ref="templateInput"
          :value="modelValue.templateValue ?? ''"
          @input="setTemplateValue(($event.target as HTMLInputElement).value)"
          placeholder="如 inc_{{hotel}}"
          class="w-full px-3 py-1.5 border border-border rounded-lg text-sm outline-none focus:border-primary font-mono"
        />
        <div class="mt-1 flex flex-wrap items-center gap-1">
          <span class="text-[10px] text-text-secondary">插入：</span>
          <button
            v-for="p in dynamicParams"
            :key="p.key"
            class="text-[10px] px-1.5 py-0.5 rounded border border-primary/20 text-primary/80 bg-blue-50/50 hover:bg-blue-100 transition-colors"
            @click="insertTag(p.key)"
          >{{ tagLabel(p.key) }}</button>
        </div>
      </template>
      <p v-if="hint" class="mt-1 text-[10px] text-amber-600/80 leading-relaxed">{{ hint }}</p>
    </div>
  </div>
</template>
