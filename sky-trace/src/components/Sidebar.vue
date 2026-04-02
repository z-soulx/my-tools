<script setup lang="ts">
import { computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useAppStore } from "@/stores/app";

const emit = defineEmits<{ "export-snapshot": []; "import-snapshot": [] }>();

const router = useRouter();
const route = useRoute();
const store = useAppStore();

function selectSupplier(id: number | null) {
  store.selectedSupplierId = id;
  router.push("/flows");
}

function isActive(path: string) {
  return route.path.startsWith(path);
}

const allNavItems = [
  { path: "/flows", label: "排查链路", icon: "🔗", restrictionKey: null },
  { path: "/quick-query", label: "快速查询", icon: "⚡", restrictionKey: "hideQuickQuery" as const },
  { path: "/checklists", label: "监控Checklist", icon: "📋", restrictionKey: null },
  { path: "/recovery", label: "快速恢复", icon: "🔧", restrictionKey: null },
  { path: "/suppliers", label: "供应商管理", icon: "🏢", restrictionKey: "hideSuppliers" as const },
  { path: "/settings", label: "天网配置", icon: "⚙", restrictionKey: "hideSettings" as const },
  { path: "/trash", label: "回收站", icon: "🗑", restrictionKey: "hideTrash" as const },
];

const navItems = computed(() => {
  if (!store.snapshotMode) return allNavItems;
  return allNavItems.filter((item) => {
    if (!item.restrictionKey) return true;
    return !store.snapshotRestrictions[item.restrictionKey];
  });
});
</script>

<template>
  <aside
    class="w-56 bg-sidebar text-white flex flex-col shrink-0 select-none"
  >
    <div class="px-4 py-4 border-b border-white/10">
      <h1 class="text-lg font-bold tracking-wide">SkyTrace</h1>
      <p v-if="store.snapshotMode" class="text-xs text-amber-300 mt-0.5">快照版 (只读)</p>
      <p v-else class="text-xs text-white/50 mt-0.5">天网日志链路追踪</p>
    </div>

    <nav class="flex-1 overflow-y-auto py-2">
      <div
        v-for="item in navItems"
        :key="item.path"
        class="px-3 py-2 mx-2 rounded-md cursor-pointer text-sm flex items-center gap-2 transition-colors"
        :class="
          isActive(item.path)
            ? 'bg-primary text-white'
            : 'text-white/70 hover:bg-sidebar-hover hover:text-white'
        "
        @click="router.push(item.path)"
      >
        <span>{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </div>

      <div class="mt-4 px-4">
        <div class="text-xs text-white/40 uppercase tracking-wider mb-2">
          供应商筛选
        </div>
        <div
          class="px-3 py-1.5 rounded cursor-pointer text-sm transition-colors"
          :class="
            store.selectedSupplierId === null
              ? 'bg-white/10 text-white'
              : 'text-white/60 hover:text-white hover:bg-white/5'
          "
          @click="selectSupplier(null)"
        >
          全部
        </div>
        <div
          v-for="supplier in store.suppliers"
          :key="supplier.id"
          class="px-3 py-1.5 rounded cursor-pointer text-sm transition-colors"
          :class="
            store.selectedSupplierId === supplier.id
              ? 'bg-white/10 text-white'
              : 'text-white/60 hover:text-white hover:bg-white/5'
          "
          @click="selectSupplier(supplier.id)"
        >
          {{ supplier.name }}
        </div>
      </div>

      <div class="mt-4 px-4" v-if="store.flows.some((f) => f.isFavorite)">
        <div class="text-xs text-white/40 uppercase tracking-wider mb-2">
          收藏链路
        </div>
        <div
          v-for="flow in store.flows.filter((f) => f.isFavorite)"
          :key="flow.id"
          class="px-3 py-1.5 rounded cursor-pointer text-sm text-white/60 hover:text-white hover:bg-white/5 transition-colors truncate"
          @click="router.push(`/flows/${flow.id}`)"
        >
          {{ flow.name }}
        </div>
      </div>
    </nav>

    <div class="px-4 py-3 border-t border-white/10 space-y-2">
      <div v-if="store.snapshotMode" class="flex items-center gap-1.5 text-xs">
        <span class="w-2 h-2 rounded-full bg-amber-400 shrink-0" />
        <span class="text-amber-300">快照模式</span>
        <button
          v-if="!store.isAutoSnapshot"
          class="ml-auto text-white/50 hover:text-white text-[10px]"
          @click="store.exitSnapshotMode()"
        >退出</button>
      </div>
      <div v-if="!store.snapshotMode" class="flex items-center gap-1">
        <button
          class="flex-1 text-[10px] px-2 py-1 rounded bg-white/10 text-white/70 hover:bg-white/20 hover:text-white transition-colors"
          @click="emit('export-snapshot')"
        >导出快照</button>
        <button
          class="flex-1 text-[10px] px-2 py-1 rounded bg-white/10 text-white/70 hover:bg-white/20 hover:text-white transition-colors"
          @click="emit('import-snapshot')"
        >导入快照</button>
      </div>
      <div class="flex items-center gap-2 text-xs">
        <span class="text-white/40">环境:</span>
        <select
          v-model="store.currentEnv"
          class="bg-sidebar-hover text-white text-xs rounded px-2 py-1 border border-white/10 outline-none"
        >
          <option value="prod">PROD</option>
          <option value="uat">UAT</option>
        </select>
      </div>
    </div>
  </aside>
</template>
