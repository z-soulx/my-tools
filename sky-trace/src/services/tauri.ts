import { invoke } from "@tauri-apps/api/core";
import type {
  SkyApp,
  Supplier,
  TraceFlow,
  TraceNode,
  DynamicParam,
  SkynetLogResponse,
  ChecklistGroup,
  ChecklistItem,
  RecoveryGroup,
  RecoveryStep,
  SnapshotData,
  SnapshotRestrictions,
  RemoteConfig,
  NodeGroup,
} from "@/types";

export async function getSkyApps(): Promise<SkyApp[]> {
  return invoke("get_sky_apps");
}

export async function saveSkyApp(
  app: Omit<SkyApp, "id" | "createdAt">
): Promise<SkyApp> {
  return invoke("save_sky_app", { app });
}

export async function deleteSkyApp(id: number): Promise<void> {
  return invoke("delete_sky_app", { id });
}

export async function getSuppliers(): Promise<Supplier[]> {
  return invoke("get_suppliers");
}

export async function saveSupplier(
  supplier: Omit<Supplier, "id" | "createdAt">
): Promise<Supplier> {
  return invoke("save_supplier", { supplier });
}

export async function deleteSupplier(id: number): Promise<void> {
  return invoke("delete_supplier", { id });
}

export async function getFlows(supplierId?: number): Promise<TraceFlow[]> {
  return invoke("get_flows", { supplierId: supplierId ?? null });
}

export async function getFlow(id: number): Promise<TraceFlow> {
  return invoke("get_flow", { id });
}

export async function saveFlow(flow: {
  id?: number;
  name: string;
  description: string;
  supplierId: number | null;
  tags: string[];
  dynamicParams: DynamicParam[];
  nodes: TraceNode[];
  nodeGroups?: NodeGroup[];
}): Promise<TraceFlow> {
  return invoke("save_flow", { flow });
}

export async function deleteFlow(id: number): Promise<void> {
  return invoke("delete_flow", { id });
}

export async function duplicateFlow(id: number): Promise<TraceFlow> {
  return invoke("duplicate_flow", { id });
}

export async function toggleFlowFavorite(id: number): Promise<void> {
  return invoke("toggle_flow_favorite", { id });
}

export async function querySkynetLog(
  appId: string,
  token: string,
  params: Record<string, unknown>
): Promise<SkynetLogResponse> {
  return invoke("query_skynet_log", { appId, token, params });
}

export async function generateSkynetUiLink(
  appUk: string,
  params: Record<string, unknown>
): Promise<string> {
  return invoke("generate_skynet_ui_link", { appUk, params });
}

export async function getDeletedFlows(): Promise<TraceFlow[]> {
  return invoke("get_deleted_flows");
}

export async function restoreFlow(id: number): Promise<void> {
  return invoke("restore_flow", { id });
}

export async function permanentlyDeleteFlow(id: number): Promise<void> {
  return invoke("permanently_delete_flow", { id });
}

export async function getDeletedSuppliers(): Promise<Supplier[]> {
  return invoke("get_deleted_suppliers");
}

export async function restoreSupplier(id: number): Promise<void> {
  return invoke("restore_supplier", { id });
}

export async function permanentlyDeleteSupplier(id: number): Promise<void> {
  return invoke("permanently_delete_supplier", { id });
}

export async function emptyTrash(): Promise<void> {
  return invoke("empty_trash");
}

// ── Checklist ──

export async function getChecklistGroups(): Promise<ChecklistGroup[]> {
  return invoke("get_checklist_groups");
}

export async function getChecklistGroup(id: number): Promise<ChecklistGroup> {
  return invoke("get_checklist_group", { id });
}

export async function saveChecklistGroup(group: {
  id?: number;
  name: string;
  description: string;
  items: ChecklistItem[];
}): Promise<ChecklistGroup> {
  return invoke("save_checklist_group", { group });
}

export async function deleteChecklistGroup(id: number): Promise<void> {
  return invoke("delete_checklist_group", { id });
}

// ── Recovery ──

export async function getRecoveryGroups(): Promise<RecoveryGroup[]> {
  return invoke("get_recovery_groups");
}

export async function getRecoveryGroup(id: number): Promise<RecoveryGroup> {
  return invoke("get_recovery_group", { id });
}

export async function saveRecoveryGroup(group: {
  id?: number;
  name: string;
  description: string;
  steps: RecoveryStep[];
}): Promise<RecoveryGroup> {
  return invoke("save_recovery_group", { group });
}

export async function deleteRecoveryGroup(id: number): Promise<void> {
  return invoke("delete_recovery_group", { id });
}

export async function getDeletedRecoveryGroups(): Promise<RecoveryGroup[]> {
  return invoke("get_deleted_recovery_groups");
}

export async function restoreRecoveryGroup(id: number): Promise<void> {
  return invoke("restore_recovery_group", { id });
}

export async function permanentlyDeleteRecoveryGroup(id: number): Promise<void> {
  return invoke("permanently_delete_recovery_group", { id });
}

// ── Snapshot ──

export async function exportSnapshot(
  flowIds: number[],
  checklistGroupIds: number[],
  recoveryGroupIds: number[],
  restrictions: SnapshotRestrictions,
  dataVersion: string,
  outputPath: string,
): Promise<string> {
  return invoke("export_snapshot", { flowIds, checklistGroupIds, recoveryGroupIds, restrictions, dataVersion, outputPath });
}

export async function importSnapshot(path: string): Promise<SnapshotData> {
  return invoke("import_snapshot", { path });
}

export async function getAutoSnapshot(): Promise<SnapshotData | null> {
  return invoke("get_auto_snapshot");
}

export async function getAppMode(): Promise<{ snapshotOnly: boolean; hasSnapshot: boolean }> {
  return invoke("get_app_mode");
}

export async function checkRemoteConfig(): Promise<RemoteConfig> {
  return invoke("check_remote_config");
}

// ── JCP Order ──

export async function queryJcpOrder(
  body: { orderId?: string; traceId?: string }
): Promise<any> {
  return invoke("query_jcp_order", { body });
}

export async function querySupplierMapping(
  body: { from: string; logId: string; realRequest: { elongHotelId: string; elongRoomId: string; elongRateplanId: string } }
): Promise<any> {
  return invoke("query_supplier_mapping", { body });
}
