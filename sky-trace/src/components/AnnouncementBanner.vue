<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  announcement: { text: string; type: 'info' | 'warning' | 'error' };
}>();

const dismissed = ref(false);

const colorClass = {
  info: "bg-blue-500/10 border-blue-500/30 text-blue-700 dark:text-blue-300",
  warning: "bg-yellow-500/10 border-yellow-500/30 text-yellow-700 dark:text-yellow-300",
  error: "bg-red-500/10 border-red-500/30 text-red-700 dark:text-red-300",
} as const;
</script>

<template>
  <div
    v-if="!dismissed"
    :class="['flex items-center gap-3 px-4 py-2 border-b text-sm', colorClass[announcement.type]]"
  >
    <span class="flex-1">{{ announcement.text }}</span>
    <button
      class="shrink-0 opacity-60 hover:opacity-100 transition-opacity text-xs"
      @click="dismissed = true"
    >
      ✕
    </button>
  </div>
</template>
