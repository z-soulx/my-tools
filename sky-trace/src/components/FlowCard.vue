<script setup lang="ts">
import { computed } from "vue";
import type { TraceFlow } from "@/types";
import { useAppStore } from "@/stores/app";

const props = defineProps<{ flow: TraceFlow }>();
const emit = defineEmits<{
  open: [id: number];
  duplicate: [id: number];
  delete: [id: number];
  toggleFavorite: [id: number];
}>();

const store = useAppStore();
const supplierName = computed(() => {
  if (!props.flow.supplierId) return "";
  return store.supplierMap.get(props.flow.supplierId)?.name ?? "";
});
</script>

<template>
  <div
    class="bg-surface rounded-xl border border-border p-4 hover:shadow-md transition-shadow cursor-pointer group"
    @click="emit('open', flow.id)"
  >
    <div class="flex items-start justify-between">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <button
            class="text-lg shrink-0"
            :class="flow.isFavorite ? 'opacity-100' : 'opacity-0 group-hover:opacity-40'"
            @click.stop="emit('toggleFavorite', flow.id)"
          >
            {{ flow.isFavorite ? '⭐' : '☆' }}
          </button>
          <h3 class="font-medium text-text truncate">{{ flow.name }}</h3>
        </div>
        <p v-if="flow.description" class="text-sm text-text-secondary mt-1 line-clamp-2">
          {{ flow.description }}
        </p>
      </div>
      <div v-if="!store.snapshotMode" class="flex items-center gap-1 ml-2 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          class="p-1.5 text-xs rounded hover:bg-surface-alt"
          title="复制链路"
          @click.stop="emit('duplicate', flow.id)"
        >📋</button>
        <button
          class="p-1.5 text-xs rounded hover:bg-red-50 text-error"
          title="删除"
          @click.stop="emit('delete', flow.id)"
        >🗑</button>
      </div>
    </div>

    <div class="flex items-center gap-2 mt-3 flex-wrap">
      <span v-if="supplierName" class="px-2 py-0.5 bg-blue-50 text-primary text-xs rounded-full">
        {{ supplierName }}
      </span>
      <span
        v-for="tag in flow.tags"
        :key="tag"
        class="px-2 py-0.5 bg-surface-alt text-text-secondary text-xs rounded-full"
      >
        {{ tag }}
      </span>
    </div>

    <div class="flex items-center justify-between mt-3 text-xs text-text-secondary">
      <span>{{ flow.nodes.length }} 个查询节点</span>
      <span>{{ flow.updatedAt?.slice(0, 10) }}</span>
    </div>
  </div>
</template>
