<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import Sidebar from "@/components/Sidebar.vue";
import SnapshotExportDialog from "@/components/SnapshotExportDialog.vue";
import { open } from "@tauri-apps/plugin-dialog";

const store = useAppStore();
const showExportDialog = ref(false);

onMounted(async () => {
  const mode = await api.getAppMode();
  store.snapshotOnlyBuild = mode.snapshotOnly;

  if (mode.hasSnapshot) {
    const snapshot = await api.getAutoSnapshot();
    if (snapshot) {
      store.enterSnapshotMode(snapshot, true);
      return;
    }
  }

  if (mode.snapshotOnly) {
    store.missingSnapshot = true;
    return;
  }

  store.loadAll();
});

async function handleImportSnapshot() {
  try {
    const path = await open({
      filters: [{ name: "SkyTrace 快照", extensions: ["skytrace"] }],
      multiple: false,
    });
    if (!path) return;
    const data = await api.importSnapshot(path as string);
    store.enterSnapshotMode(data);
  } catch (e) {
    console.error("导入快照失败:", e);
  }
}
</script>

<template>
  <div v-if="store.missingSnapshot" class="flex h-screen w-screen items-center justify-center bg-bg">
    <div class="text-center max-w-md px-8">
      <div class="text-6xl mb-6">🔒</div>
      <h1 class="text-2xl font-bold text-text mb-3">SkyTrace 快照版</h1>
      <p class="text-text-secondary mb-6 leading-relaxed">
        未检测到快照数据文件。请将 <code class="px-1.5 py-0.5 bg-surface-alt rounded text-sm font-mono">snapshot.skytrace</code>
        文件放在应用同级目录后重新启动。
      </p>
      <p class="text-xs text-text-secondary/60">如需获取快照文件，请联系开发者。</p>
    </div>
  </div>
  <div v-else class="flex h-screen w-screen overflow-hidden">
    <Sidebar
      @export-snapshot="showExportDialog = true"
      @import-snapshot="handleImportSnapshot"
    />
    <main class="flex-1 overflow-hidden flex flex-col">
      <router-view />
    </main>

    <SnapshotExportDialog
      v-if="showExportDialog"
      @close="showExportDialog = false"
    />
  </div>
</template>
