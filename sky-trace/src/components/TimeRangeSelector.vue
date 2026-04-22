<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";

const props = defineProps<{
  modelFrom: string;
  modelTo: string;
}>();

const emit = defineEmits<{
  "update:modelFrom": [value: string];
  "update:modelTo": [value: string];
}>();

type Preset = { label: string; from: string; to: string };
const presets: Preset[] = [
  { label: "最近5分钟", from: "now-5m", to: "now" },
  { label: "最近15分钟", from: "now-15m", to: "now" },
  { label: "最近30分钟", from: "now-30m", to: "now" },
  { label: "最近1小时", from: "now-1h", to: "now" },
  { label: "最近2小时", from: "now-2h", to: "now" },
  { label: "最近6小时", from: "now-6h", to: "now" },
  { label: "最近12小时", from: "now-12h", to: "now" },
  { label: "最近24小时", from: "now-24h", to: "now" },
];

const open = ref(false);
const mode = ref<"preset" | "custom">("preset");
const selectedPreset = ref(2);
const customFrom = ref("");
const customTo = ref("");
const wrapperRef = ref<HTMLElement | null>(null);
const settingFromProps = ref(false);

const displayLabel = ref("最近30分钟");

function selectPreset(index: number) {
  mode.value = "preset";
  selectedPreset.value = index;
  displayLabel.value = presets[index].label;
  emit("update:modelFrom", presets[index].from);
  emit("update:modelTo", presets[index].to);
  open.value = false;
}

function switchToCustom() {
  mode.value = "custom";
  const now = new Date();
  const thirtyAgo = new Date(now.getTime() - 30 * 60_000);
  customFrom.value = formatForInput(thirtyAgo);
  customTo.value = formatForInput(now);
  applyCustom();
}

function applyCustom() {
  if (settingFromProps.value) return;
  if (!customFrom.value || !customTo.value) return;
  displayLabel.value = `${customFrom.value.replace("T", " ")} ~ ${customTo.value.replace("T", " ")}`;
  emit("update:modelFrom", formatForApi(customFrom.value));
  emit("update:modelTo", formatForApi(customTo.value));
}

watch([customFrom, customTo], applyCustom);

watch(
  () => [props.modelFrom, props.modelTo] as const,
  ([from, to]) => {
    if (!from || !to) return;
    if (from.startsWith("now") && to.startsWith("now")) return;
    settingFromProps.value = true;
    mode.value = "custom";
    const fromShort = from.replace(/\.000$/, "").slice(0, 16);
    const toShort = to.replace(/\.000$/, "").slice(0, 16);
    customFrom.value = fromShort.replace(" ", "T");
    customTo.value = toShort.replace(" ", "T");
    displayLabel.value = `${fromShort.replace("T", " ")} ~ ${toShort.replace("T", " ")}`;
    nextTick(() => { settingFromProps.value = false; });
  },
);

function formatForInput(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function formatForApi(datetimeLocal: string): string {
  return datetimeLocal.replace("T", " ") + ":00.000";
}

function onClickOutside(e: MouseEvent) {
  if (wrapperRef.value && !wrapperRef.value.contains(e.target as Node)) {
    open.value = false;
  }
}

onMounted(() => {
  document.addEventListener("mousedown", onClickOutside);
  selectPreset(2);
});
onUnmounted(() => document.removeEventListener("mousedown", onClickOutside));
</script>

<template>
  <div ref="wrapperRef" class="relative inline-block">
    <button
      class="flex items-center gap-1.5 px-3 py-1.5 text-sm border border-border rounded-lg hover:border-primary/50 transition-colors bg-surface"
      @click="open = !open"
    >
      <span class="text-text-secondary text-xs">时间</span>
      <span class="font-medium text-text truncate max-w-[260px]">{{ displayLabel }}</span>
      <span class="text-text-secondary text-[10px]">{{ open ? '▲' : '▼' }}</span>
    </button>

    <Transition name="fade">
      <div
        v-if="open"
        class="absolute top-full left-0 mt-1 bg-surface border border-border rounded-xl shadow-lg z-30 min-w-[220px] py-1"
      >
        <button
          v-for="(p, i) in presets"
          :key="i"
          class="w-full text-left px-3 py-1.5 text-sm hover:bg-surface-alt transition-colors"
          :class="mode === 'preset' && selectedPreset === i ? 'text-primary font-medium bg-blue-50/50' : 'text-text'"
          @click="selectPreset(i)"
        >{{ p.label }}</button>

        <div class="border-t border-border my-1" />

        <button
          class="w-full text-left px-3 py-1.5 text-sm hover:bg-surface-alt transition-colors"
          :class="mode === 'custom' ? 'text-primary font-medium' : 'text-text'"
          @click="switchToCustom"
        >自定义范围</button>

        <div v-if="mode === 'custom'" class="px-3 py-2 space-y-1.5 border-t border-border mt-1">
          <input
            v-model="customFrom"
            type="datetime-local"
            class="w-full px-2 py-1 text-xs border border-border rounded outline-none focus:border-primary"
          />
          <input
            v-model="customTo"
            type="datetime-local"
            class="w-full px-2 py-1 text-xs border border-border rounded outline-none focus:border-primary"
          />
          <button
            class="w-full py-1 text-xs bg-primary text-white rounded hover:bg-primary-hover"
            @click="open = false"
          >确定</button>
        </div>
      </div>
    </Transition>
  </div>
</template>
