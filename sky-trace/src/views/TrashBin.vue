<script setup lang="ts">
import { ref, onMounted } from "vue";
import * as api from "@/services/tauri";
import type { TraceFlow, Supplier, RecoveryGroup } from "@/types";

const deletedFlows = ref<TraceFlow[]>([]);
const deletedSuppliers = ref<Supplier[]>([]);
const deletedRecoveryGroups = ref<RecoveryGroup[]>([]);
const loading = ref(true);

async function reload() {
  loading.value = true;
  const [flows, suppliers, recovery] = await Promise.all([
    api.getDeletedFlows(),
    api.getDeletedSuppliers(),
    api.getDeletedRecoveryGroups(),
  ]);
  deletedFlows.value = flows;
  deletedSuppliers.value = suppliers;
  deletedRecoveryGroups.value = recovery;
  loading.value = false;
}

async function restoreFlow(id: number) {
  await api.restoreFlow(id);
  await reload();
}

async function permanentlyDeleteFlow(id: number) {
  await api.permanentlyDeleteFlow(id);
  await reload();
}

async function restoreSupplier(id: number) {
  await api.restoreSupplier(id);
  await reload();
}

async function permanentlyDeleteSupplier(id: number) {
  await api.permanentlyDeleteSupplier(id);
  await reload();
}

async function restoreRecoveryGroup(id: number) {
  await api.restoreRecoveryGroup(id);
  await reload();
}

async function permanentlyDeleteRecoveryGroup(id: number) {
  await api.permanentlyDeleteRecoveryGroup(id);
  await reload();
}

async function emptyAll() {
  await api.emptyTrash();
  await reload();
}

onMounted(reload);

const isEmpty = () => deletedFlows.value.length === 0 && deletedSuppliers.value.length === 0 && deletedRecoveryGroups.value.length === 0;
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 bg-surface border-b border-border shrink-0">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold">回收站</h2>
          <p class="text-sm text-text-secondary mt-0.5">已删除的链路和供应商，可恢复或永久删除</p>
        </div>
        <button
          v-if="!isEmpty()"
          class="px-4 py-2 text-sm text-error border border-error/30 rounded-lg hover:bg-red-50 transition-colors"
          @click="emptyAll"
        >清空回收站</button>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto p-6">
      <div v-if="loading" class="text-center py-12 text-text-secondary">加载中...</div>

      <div v-else-if="isEmpty()" class="text-center py-12 text-text-secondary">
        <p class="text-lg">回收站为空</p>
          <p class="text-sm mt-1">删除的链路、供应商和恢复分组会暂存在这里</p>
      </div>

      <template v-else>
        <div v-if="deletedFlows.length > 0" class="mb-8">
          <h3 class="font-medium mb-3">已删除链路 ({{ deletedFlows.length }})</h3>
          <div class="space-y-2">
            <div
              v-for="f in deletedFlows"
              :key="f.id"
              class="bg-surface rounded-lg border border-border p-4 flex items-center justify-between"
            >
              <div>
                <div class="font-medium text-sm">{{ f.name }}</div>
                <div class="text-xs text-text-secondary mt-0.5">
                  {{ f.nodes.length }} 个节点
                  <span v-if="f.description"> · {{ f.description }}</span>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <button class="px-3 py-1 text-xs text-primary hover:bg-blue-50 rounded-lg" @click="restoreFlow(f.id)">恢复</button>
                <button class="px-3 py-1 text-xs text-error hover:bg-red-50 rounded-lg" @click="permanentlyDeleteFlow(f.id)">永久删除</button>
              </div>
            </div>
          </div>
        </div>

        <div v-if="deletedRecoveryGroups.length > 0" class="mb-8">
          <h3 class="font-medium mb-3">已删除恢复分组 ({{ deletedRecoveryGroups.length }})</h3>
          <div class="space-y-2">
            <div
              v-for="g in deletedRecoveryGroups"
              :key="g.id"
              class="bg-surface rounded-lg border border-border p-4 flex items-center justify-between"
            >
              <div>
                <div class="font-medium text-sm">{{ g.name }}</div>
                <div class="text-xs text-text-secondary mt-0.5">{{ g.steps.length }} 个步骤</div>
              </div>
              <div class="flex items-center gap-2">
                <button class="px-3 py-1 text-xs text-primary hover:bg-blue-50 rounded-lg" @click="restoreRecoveryGroup(g.id)">恢复</button>
                <button class="px-3 py-1 text-xs text-error hover:bg-red-50 rounded-lg" @click="permanentlyDeleteRecoveryGroup(g.id)">永久删除</button>
              </div>
            </div>
          </div>
        </div>

        <div v-if="deletedSuppliers.length > 0">
          <h3 class="font-medium mb-3">已删除供应商 ({{ deletedSuppliers.length }})</h3>
          <div class="space-y-2">
            <div
              v-for="s in deletedSuppliers"
              :key="s.id"
              class="bg-surface rounded-lg border border-border p-4 flex items-center justify-between"
            >
              <div>
                <div class="font-medium text-sm">{{ s.name }}</div>
                <div class="text-xs text-text-secondary mt-0.5">{{ s.code }}</div>
              </div>
              <div class="flex items-center gap-2">
                <button class="px-3 py-1 text-xs text-primary hover:bg-blue-50 rounded-lg" @click="restoreSupplier(s.id)">恢复</button>
                <button class="px-3 py-1 text-xs text-error hover:bg-red-50 rounded-lg" @click="permanentlyDeleteSupplier(s.id)">永久删除</button>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
