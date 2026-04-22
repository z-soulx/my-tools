<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAppStore } from "@/stores/app";
import * as api from "@/services/tauri";
import Sidebar from "@/components/Sidebar.vue";
import SnapshotExportDialog from "@/components/SnapshotExportDialog.vue";
import RemoteLockScreen from "@/components/RemoteLockScreen.vue";
import AnnouncementBanner from "@/components/AnnouncementBanner.vue";
import UpdateDialog from "@/components/UpdateDialog.vue";
import DataUpdateDialog from "@/components/DataUpdateDialog.vue";
import { open } from "@tauri-apps/plugin-dialog";

const store = useAppStore();
const showExportDialog = ref(false);
const appVersion = __APP_VERSION__;

// Startup loading state
const booting = ref(true);
// Force-update state (version below minVersion)
const forceUpdateUrl = ref("");
// Optional update dialog
const showUpdateDialog = ref(false);
const pendingUpdateUrl = ref("");
const pendingUpdateNotes = ref("");
// Data update dialog
const showDataUpdateDialog = ref(false);
const pendingDataVersion = ref("");
const pendingDataUpdateUrl = ref("");
const pendingDataUpdateNotes = ref("");

function compareSemver(a: string, b: string): number {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    const diff = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (diff !== 0) return diff;
  }
  return 0;
}

onMounted(async () => {
  // ── Step 1: Remote config check (fail-closed) ──────────────────────────────
  try {
    const config = await api.checkRemoteConfig();
    store.remoteConfig = config;

    if (!config.enabled) {
      // Kill switch — show lock screen with message
      booting.value = false;
      return;
    }

    if (config.minVersion && compareSemver(appVersion, config.minVersion) < 0) {
      // Version too old — force update
      const isMac = navigator.userAgent.toLowerCase().includes("mac");
      forceUpdateUrl.value = isMac ? config.updateUrlMac : config.updateUrlWin;
      booting.value = false;
      return;
    }

    // ── Step 2: App mode / snapshot check ─────────────────────────────────────
    const mode = await api.getAppMode();
    store.snapshotOnlyBuild = mode.snapshotOnly;

    if (mode.hasSnapshot) {
      const snapshot = await api.getAutoSnapshot();
      if (snapshot) {
        store.enterSnapshotMode(snapshot, true);
        booting.value = false;
        // Check for optional software update
        if (config.latestVersion && compareSemver(config.latestVersion, appVersion) > 0) {
          const isMac = navigator.userAgent.toLowerCase().includes("mac");
          pendingUpdateUrl.value = isMac ? config.updateUrlMac : config.updateUrlWin;
          pendingUpdateNotes.value = config.updateNotes;
          showUpdateDialog.value = true;
        }
        // Check for data file update
        if (config.latestDataVersion && config.dataUpdateUrl &&
            store.snapshotDataVersion &&
            compareSemver(config.latestDataVersion, store.snapshotDataVersion) > 0) {
          pendingDataVersion.value = config.latestDataVersion;
          pendingDataUpdateUrl.value = config.dataUpdateUrl;
          pendingDataUpdateNotes.value = config.dataUpdateNotes;
          showDataUpdateDialog.value = true;
        }
        return;
      }
    }

    if (mode.snapshotOnly) {
      store.missingSnapshot = true;
      booting.value = false;
      return;
    }

    store.loadAll();
    booting.value = false;

    // ── Step 3: Optional update check ─────────────────────────────────────────
    if (config.latestVersion && compareSemver(config.latestVersion, appVersion) > 0) {
      const isMac = navigator.userAgent.toLowerCase().includes("mac");
      pendingUpdateUrl.value = isMac ? config.updateUrlMac : config.updateUrlWin;
      pendingUpdateNotes.value = config.updateNotes;
      showUpdateDialog.value = true;
    }
  } catch {
    store.remoteCheckFailed = true;
    booting.value = false;
  }
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
  <!-- Startup loading -->
  <div v-if="booting" class="flex h-screen w-screen items-center justify-center bg-bg">
    <div class="text-center">
      <div class="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-3"></div>
      <p class="text-sm text-text-secondary">正在验证...</p>
    </div>
  </div>

  <!-- Remote lock / force-update screen -->
  <RemoteLockScreen
    v-else-if="store.isRemoteLocked || forceUpdateUrl"
    :failed="store.remoteCheckFailed"
    :message="store.remoteConfig?.message ?? ''"
    :force-update-url="forceUpdateUrl"
    @retry="() => { store.remoteCheckFailed = false; booting = true; $nextTick(() => { booting = false; }); }"
  />

  <!-- Missing snapshot lock -->
  <div v-else-if="store.missingSnapshot" class="flex h-screen w-screen items-center justify-center bg-bg">
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

  <!-- Normal app -->
  <div v-else class="flex h-screen w-screen overflow-hidden flex-col">
    <AnnouncementBanner
      v-if="store.remoteConfig?.announcement"
      :announcement="store.remoteConfig.announcement"
    />
    <div class="flex flex-1 overflow-hidden">
      <Sidebar
        @export-snapshot="showExportDialog = true"
        @import-snapshot="handleImportSnapshot"
      />
      <main class="flex-1 overflow-hidden flex flex-col">
        <router-view />
      </main>
    </div>

    <SnapshotExportDialog
      v-if="showExportDialog"
      @close="showExportDialog = false"
    />

    <UpdateDialog
      v-if="showUpdateDialog"
      :update-url="pendingUpdateUrl"
      :notes="pendingUpdateNotes"
      :latest-version="store.remoteConfig?.latestVersion ?? ''"
      @close="showUpdateDialog = false"
    />

    <DataUpdateDialog
      v-if="showDataUpdateDialog"
      :latest-data-version="pendingDataVersion"
      :data-update-url="pendingDataUpdateUrl"
      :notes="pendingDataUpdateNotes"
      @close="showDataUpdateDialog = false"
    />
  </div>
</template>
