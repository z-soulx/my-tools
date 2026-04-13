<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps<{
  latestVersion: string;
  updateUrl: string;
  notes: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

async function handleDownload() {
  if (props.updateUrl) {
    await openUrl(props.updateUrl);
  }
  emit('close');
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
    <div class="bg-surface rounded-xl shadow-2xl w-full max-w-sm mx-4 p-6">
      <h2 class="text-lg font-semibold text-text mb-1">发现新版本 {{ latestVersion }}</h2>
      <p v-if="notes" class="text-sm text-text-secondary mb-4 whitespace-pre-wrap leading-relaxed">
        {{ notes }}
      </p>
      <p v-else class="text-sm text-text-secondary mb-4">
        建议更新到最新版本以获得最佳体验。
      </p>
      <div v-if="updateUrl" class="text-xs mb-4 bg-surface-alt rounded px-2 py-1.5">
        <span
          class="text-accent hover:underline cursor-pointer"
          @click="handleDownload"
        >点击快速下载 ↗</span>
        <span class="text-text-secondary/40 ml-2 break-all font-mono">{{ updateUrl }}</span>
      </div>
      <div class="flex gap-3 justify-end">
        <button
          class="px-3 py-1.5 text-sm text-text-secondary hover:text-text transition-colors"
          @click="emit('close')"
        >
          稍后再说
        </button>
        <button
          class="px-4 py-1.5 bg-accent text-white rounded-lg text-sm hover:opacity-90 transition-opacity"
          @click="handleDownload"
        >
          立即下载
        </button>
      </div>
    </div>
  </div>
</template>
