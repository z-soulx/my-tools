<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";

const props = defineProps<{
  hint: string;
}>();

const emit = defineEmits<{
  apply: [value: string];
}>();

const open = ref(false);
const popoverRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLElement | null>(null);
const applyFeedback = ref<number | null>(null);

/** 按空行分段 */
const sections = computed(() => {
  return props.hint.split(/\n\s*\n/).map((s) => s.trim()).filter(Boolean);
});

/** 判断段落是否"像值"（含引号、花括号、冒号键值对等） */
function looksLikeValue(text: string): boolean {
  return /["{}[\]\\]/.test(text) || /^\S+[:=]/.test(text);
}

/** 点击段落操作按钮 */
async function applySection(text: string, index: number) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    /* clipboard not available */
  }
  emit("apply", text);
  applyFeedback.value = index;
  setTimeout(() => {
    if (applyFeedback.value === index) applyFeedback.value = null;
  }, 1500);
}

function toggle() {
  open.value = !open.value;
}

function onClickOutside(e: MouseEvent) {
  if (!open.value) return;
  const target = e.target as HTMLElement;
  if (
    popoverRef.value?.contains(target) ||
    triggerRef.value?.contains(target)
  )
    return;
  open.value = false;
}

onMounted(() => {
  document.addEventListener("mousedown", onClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", onClickOutside);
});
</script>

<template>
  <div class="relative shrink-0" ref="triggerRef">
    <button
      type="button"
      class="w-7 h-7 flex items-center justify-center rounded-lg border transition-colors text-sm"
      :class="open ? 'border-primary bg-blue-50 text-primary' : 'border-border text-text-secondary/50 hover:text-primary hover:border-primary/40'"
      title="查看提示"
      @click="toggle"
    >i</button>

    <Transition
      enter-active-class="transition-all duration-150 ease-out"
      enter-from-class="opacity-0 scale-95 -translate-y-1"
      enter-to-class="opacity-100 scale-100 translate-y-0"
      leave-active-class="transition-all duration-100 ease-in"
      leave-from-class="opacity-100 scale-100 translate-y-0"
      leave-to-class="opacity-0 scale-95 -translate-y-1"
    >
      <div
        v-if="open"
        ref="popoverRef"
        class="absolute right-0 top-full mt-1.5 z-50 w-80 max-h-72 overflow-y-auto bg-surface rounded-xl border border-border shadow-lg"
      >
        <div class="px-3 py-2.5 space-y-2">
          <template v-for="(section, idx) in sections" :key="idx">
            <!-- 像值的段落：代码块 + 操作按钮 -->
            <div v-if="looksLikeValue(section)" class="group/code">
              <div class="relative bg-surface-alt rounded-lg px-3 py-2 pr-16">
                <code class="text-[11px] text-text break-all whitespace-pre-wrap leading-relaxed select-text">{{ section }}</code>
                <button
                  class="absolute top-1.5 right-1.5 text-[10px] px-2 py-1 rounded-md border transition-all"
                  :class="applyFeedback === idx
                    ? 'border-emerald-300 bg-emerald-50 text-emerald-600'
                    : 'border-border text-text-secondary hover:border-primary hover:text-primary hover:bg-blue-50'"
                  @click="applySection(section, idx)"
                >{{ applyFeedback === idx ? '已填入' : '填入' }}</button>
              </div>
            </div>
            <!-- 普通文本段落 -->
            <p v-else class="text-xs text-text-secondary leading-relaxed whitespace-pre-wrap select-text">{{ section }}</p>
          </template>
        </div>
      </div>
    </Transition>
  </div>
</template>
