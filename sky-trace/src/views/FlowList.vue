<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import FlowCard from "@/components/FlowCard.vue";
import FlowFormDialog from "@/components/FlowFormDialog.vue";

const router = useRouter();
const store = useAppStore();
const showCreateDialog = ref(false);
const duplicateSource = ref<import("@/types").TraceFlow | null>(null);

function handleDuplicate(flowId: number) {
  const source = store.flows.find((f) => f.id === flowId);
  if (source) {
    duplicateSource.value = source;
  }
}

function handleOpen(flowId: number) {
  router.push(`/flows/${flowId}`);
}

async function handleDelete(flowId: number) {
  await api.deleteFlow(flowId);
  store.flows = store.flows.filter((f) => f.id !== flowId);
}

function handleCreated() {
  showCreateDialog.value = false;
  store.refreshFlows();
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 bg-surface border-b border-border flex items-center justify-between shrink-0">
      <div>
        <h2 class="text-xl font-semibold text-text">排查链路</h2>
        <p class="text-sm text-text-secondary mt-0.5">
          {{ store.selectedSupplierId !== null
            ? store.supplierMap.get(store.selectedSupplierId)?.name + ' - '
            : '' }}
          共 {{ store.filteredFlows.length }} 条链路
        </p>
      </div>
      <div class="flex items-center gap-3">
        <input
          v-model="store.searchQuery"
          type="text"
          placeholder="搜索链路名称、标签..."
          class="px-3 py-2 text-sm border border-border rounded-lg w-64 outline-none focus:border-primary transition-colors"
        />
        <button
          v-if="!store.snapshotMode"
          class="px-4 py-2 bg-primary text-white text-sm rounded-lg hover:bg-primary-hover transition-colors"
          @click="showCreateDialog = true"
        >
          + 新建链路
        </button>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto p-6">
      <div v-if="store.filteredFlows.length === 0" class="flex flex-col items-center justify-center h-full text-text-secondary">
        <div class="text-4xl mb-4">🔗</div>
        <p class="text-lg">暂无排查链路</p>
        <p class="text-sm mt-1">点击「新建链路」创建第一条排查流程</p>
      </div>

      <div v-else class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
        <FlowCard
          v-for="flow in store.filteredFlows"
          :key="flow.id"
          :flow="flow"
          @open="handleOpen"
          @duplicate="handleDuplicate"
          @delete="handleDelete"
          @toggle-favorite="store.toggleFavorite"
        />
      </div>
    </div>

    <FlowFormDialog
      v-if="showCreateDialog"
      @close="showCreateDialog = false"
      @saved="handleCreated"
    />

    <FlowFormDialog
      v-if="duplicateSource"
      :source-flow="duplicateSource"
      @close="duplicateSource = null"
      @saved="() => { duplicateSource = null; store.refreshFlows(); }"
    />
  </div>
</template>
