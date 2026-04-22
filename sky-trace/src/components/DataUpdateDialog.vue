<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps<{
  latestDataVersion: string;
  dataUpdateUrl: string;
  notes: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

async function handleDownload() {
  if (props.dataUpdateUrl) {
    await openUrl(props.dataUpdateUrl);
  }
  emit("close");
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
    <div class="bg-surface rounded-xl shadow-2xl w-full max-w-sm mx-4 p-6">
      <h2 class="text-lg font-semibold text-text mb-1">发现新数据版本 {{ latestDataVersion }}</h2>
      <p v-if="notes" class="text-sm text-text-secondary mb-4 whitespace-pre-wrap leading-relaxed">
        {{ notes }}
      </p>
      <p v-else class="text-sm text-text-secondary mb-4">
        有新的数据文件可用，建议更新以获得最新排查链路。
      </p>
      <div v-if="dataUpdateUrl" class="text-xs mb-4 bg-surface-alt rounded px-2 py-1.5">
        <span class="text-accent hover:underline cursor-pointer" @click="handleDownload">点击下载 .skytrace 文件 ↗</span>
        <span class="text-text-secondary/40 ml-2 break-all font-mono">{{ dataUpdateUrl }}</span>
      </div>
      <p class="text-xs text-text-secondary mb-4">下载后从侧边栏底部「导入快照」加载新文件。</p>
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
