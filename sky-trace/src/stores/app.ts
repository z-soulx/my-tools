import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { SkyApp, Supplier, TraceFlow, ChecklistGroup, RecoveryGroup, SnapshotRestrictions, SnapshotData } from "@/types";
import * as api from "@/services/tauri";

export const useAppStore = defineStore("app", () => {
  const skyApps = ref<SkyApp[]>([]);
  const suppliers = ref<Supplier[]>([]);
  const flows = ref<TraceFlow[]>([]);
  const checklists = ref<ChecklistGroup[]>([]);
  const recoveryGroups = ref<RecoveryGroup[]>([]);
  const currentEnv = ref<string>("prod");
  const searchQuery = ref("");
  const selectedSupplierId = ref<number | null>(null);
  const loading = ref(false);
  const snapshotMode = ref(false);
  const isAutoSnapshot = ref(false);
  const snapshotOnlyBuild = ref(false);
  const missingSnapshot = ref(false);
  const snapshotRestrictions = ref<SnapshotRestrictions>({
    hideEdit: true,
    hideSettings: true,
    hideSuppliers: true,
    hideQuickQuery: true,
    hideChecklistEdit: true,
    hideRecoveryEdit: true,
    hideTrash: true,
    hideDebug: true,
    hideUiLink: true,
  });

  const filteredFlows = computed(() => {
    let result = flows.value;

    if (selectedSupplierId.value !== null) {
      result = result.filter(
        (f) => f.supplierId === selectedSupplierId.value
      );
    }

    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(
        (f) =>
          f.name.toLowerCase().includes(q) ||
          f.description.toLowerCase().includes(q) ||
          f.tags.some((t) => t.toLowerCase().includes(q))
      );
    }

    return result.sort((a, b) => {
      if (a.isFavorite !== b.isFavorite) return a.isFavorite ? -1 : 1;
      return b.sortOrder - a.sortOrder;
    });
  });

  const skyAppMap = computed(() => {
    const map = new Map<number, SkyApp>();
    skyApps.value.forEach((app) => map.set(app.id, app));
    return map;
  });

  const supplierMap = computed(() => {
    const map = new Map<number, Supplier>();
    suppliers.value.forEach((s) => map.set(s.id, s));
    return map;
  });

  const checklistMap = computed(() => {
    const map = new Map<number, ChecklistGroup>();
    checklists.value.forEach((g) => map.set(g.id, g));
    return map;
  });

  async function loadAll() {
    loading.value = true;
    try {
      const [apps, sups, fls, cls, rgs] = await Promise.all([
        api.getSkyApps(),
        api.getSuppliers(),
        api.getFlows(),
        api.getChecklistGroups(),
        api.getRecoveryGroups(),
      ]);
      skyApps.value = apps;
      suppliers.value = sups;
      flows.value = fls;
      checklists.value = cls;
      recoveryGroups.value = rgs;
    } finally {
      loading.value = false;
    }
  }

  async function refreshChecklists() {
    checklists.value = await api.getChecklistGroups();
  }

  async function refreshRecoveryGroups() {
    recoveryGroups.value = await api.getRecoveryGroups();
  }

  function enterSnapshotMode(data: SnapshotData, auto = false) {
    snapshotMode.value = true;
    isAutoSnapshot.value = auto;
    snapshotRestrictions.value = data.restrictions;
    skyApps.value = data.skyApps;
    suppliers.value = data.suppliers;
    flows.value = data.flows;
    checklists.value = data.checklistGroups;
    recoveryGroups.value = data.recoveryGroups ?? [];
  }

  function exitSnapshotMode() {
    if (isAutoSnapshot.value) return;
    snapshotMode.value = false;
    loadAll();
  }

  async function refreshFlows(supplierId?: number) {
    flows.value = await api.getFlows(supplierId);
  }

  async function addSkyApp(app: Omit<SkyApp, "id" | "createdAt">) {
    const saved = await api.saveSkyApp(app);
    skyApps.value.push(saved);
    return saved;
  }

  async function removeSkyApp(id: number) {
    await api.deleteSkyApp(id);
    skyApps.value = skyApps.value.filter((a) => a.id !== id);
  }

  async function addSupplier(supplier: Omit<Supplier, "id" | "createdAt">) {
    const saved = await api.saveSupplier(supplier);
    suppliers.value.push(saved);
    return saved;
  }

  async function removeSupplier(id: number) {
    await api.deleteSupplier(id);
    suppliers.value = suppliers.value.filter((s) => s.id !== id);
  }

  async function toggleFavorite(flowId: number) {
    await api.toggleFlowFavorite(flowId);
    const flow = flows.value.find((f) => f.id === flowId);
    if (flow) flow.isFavorite = !flow.isFavorite;
  }

  return {
    skyApps,
    suppliers,
    flows,
    checklists,
    recoveryGroups,
    currentEnv,
    searchQuery,
    selectedSupplierId,
    loading,
    snapshotMode,
    isAutoSnapshot,
    snapshotOnlyBuild,
    missingSnapshot,
    snapshotRestrictions,
    filteredFlows,
    skyAppMap,
    supplierMap,
    checklistMap,
    loadAll,
    refreshFlows,
    refreshChecklists,
    refreshRecoveryGroups,
    addSkyApp,
    removeSkyApp,
    addSupplier,
    removeSupplier,
    toggleFavorite,
    enterSnapshotMode,
    exitSnapshotMode,
  };
});
