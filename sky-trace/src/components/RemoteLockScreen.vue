<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps<{
  failed: boolean;
  message: string;
  forceUpdateUrl: string;
}>();

const emit = defineEmits<{
  (e: 'retry'): void;
}>();

async function handleDownload() {
  if (props.forceUpdateUrl) {
    await openUrl(props.forceUpdateUrl);
  }
}
</script>

<template>
  <div class="flex h-screen w-screen items-center justify-center bg-bg">
    <div class="text-center max-w-md px-8">
      <!-- Network failure -->
      <template v-if="failed">
        <div class="text-5xl mb-6">🌐</div>
        <h1 class="text-xl font-bold text-text mb-3">无法连接验证服务器</h1>
        <p class="text-text-secondary mb-6 leading-relaxed text-sm">
          请检查网络连接后重试。
        </p>
        <button
          class="px-4 py-2 bg-accent text-white rounded-lg text-sm hover:opacity-90 transition-opacity"
          @click="emit('retry')"
        >
          重试
        </button>
      </template>

      <!-- Force update -->
      <template v-else-if="forceUpdateUrl">
        <div class="text-5xl mb-6">⬆️</div>
        <h1 class="text-xl font-bold text-text mb-3">当前版本过低，请更新</h1>
        <p class="text-text-secondary mb-6 leading-relaxed text-sm">
          您的版本不再被支持，请下载最新版本后重新启动。
        </p>
        <a
          class="inline-block px-4 py-2 bg-accent text-white rounded-lg text-sm hover:opacity-90 transition-opacity cursor-pointer"
          @click="handleDownload"
        >
          下载最新版本
        </a>
      </template>

      <!-- Kill switch -->
      <template v-else>
        <div class="text-5xl mb-6">🔒</div>
        <h1 class="text-xl font-bold text-text mb-3">软件已停用</h1>
        <p class="text-text-secondary leading-relaxed text-sm whitespace-pre-wrap">
          {{ message || '该软件当前不可用，请联系管理员。' }}
        </p>
      </template>
    </div>
  </div>
</template>
