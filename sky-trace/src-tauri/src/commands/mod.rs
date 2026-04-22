use serde_json::Value;
use tauri::State;

use crate::query_engine::SkynetClient;
use crate::remote_config::RemoteConfig;
use crate::snapshot::{self, SnapshotData, SnapshotRestrictions, SnapshotState};
use crate::storage::{
    ChecklistGroup, ChecklistGroupInput, Database, RecoveryGroup, RecoveryGroupInput,
    SkyApp, SkyAppInput, Supplier, SupplierInput, TraceFlow, TraceFlowInput,
};

#[tauri::command]
pub fn get_sky_apps(db: State<'_, Database>) -> Result<Vec<SkyApp>, String> {
    db.get_sky_apps().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_sky_app(db: State<'_, Database>, app: SkyAppInput) -> Result<SkyApp, String> {
    db.save_sky_app(app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_sky_app(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_sky_app(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_suppliers(db: State<'_, Database>) -> Result<Vec<Supplier>, String> {
    db.get_suppliers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_supplier(db: State<'_, Database>, supplier: SupplierInput) -> Result<Supplier, String> {
    db.save_supplier(supplier).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_supplier(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_supplier(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_flows(db: State<'_, Database>, supplier_id: Option<i64>) -> Result<Vec<TraceFlow>, String> {
    db.get_flows(supplier_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_flow(db: State<'_, Database>, id: i64) -> Result<TraceFlow, String> {
    db.get_flow(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_flow(db: State<'_, Database>, flow: TraceFlowInput) -> Result<TraceFlow, String> {
    db.save_flow(flow).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_flow(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_flow(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn duplicate_flow(db: State<'_, Database>, id: i64) -> Result<TraceFlow, String> {
    db.duplicate_flow(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_flow_favorite(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.toggle_flow_favorite(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn query_skynet_log(
    skynet: State<'_, SkynetClient>,
    app_id: String,
    token: String,
    params: Value,
) -> Result<Value, String> {
    skynet.query_log(&app_id, &token, params).await
}

#[tauri::command]
pub fn generate_skynet_ui_link(app_uk: String, params: Value) -> Result<String, String> {
    Ok(SkynetClient::generate_ui_link(&app_uk, &params))
}

#[tauri::command]
pub fn get_deleted_flows(db: State<'_, Database>) -> Result<Vec<TraceFlow>, String> {
    db.get_deleted_flows().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_flow(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.restore_flow(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn permanently_delete_flow(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.permanently_delete_flow(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_deleted_suppliers(db: State<'_, Database>) -> Result<Vec<Supplier>, String> {
    db.get_deleted_suppliers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_supplier(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.restore_supplier(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn permanently_delete_supplier(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.permanently_delete_supplier(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn empty_trash(db: State<'_, Database>) -> Result<(), String> {
    db.empty_trash().map_err(|e| e.to_string())
}

// ── Checklist ──

#[tauri::command]
pub fn get_checklist_groups(db: State<'_, Database>) -> Result<Vec<ChecklistGroup>, String> {
    db.get_checklist_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_checklist_group(db: State<'_, Database>, id: i64) -> Result<ChecklistGroup, String> {
    db.get_checklist_group(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_checklist_group(db: State<'_, Database>, group: ChecklistGroupInput) -> Result<ChecklistGroup, String> {
    db.save_checklist_group(group).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_checklist_group(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_checklist_group(id).map_err(|e| e.to_string())
}

// ── Recovery ──

#[tauri::command]
pub fn get_recovery_groups(db: State<'_, Database>) -> Result<Vec<RecoveryGroup>, String> {
    db.get_recovery_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recovery_group(db: State<'_, Database>, id: i64) -> Result<RecoveryGroup, String> {
    db.get_recovery_group(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_recovery_group(db: State<'_, Database>, group: RecoveryGroupInput) -> Result<RecoveryGroup, String> {
    db.save_recovery_group(group).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recovery_group(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_recovery_group(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_deleted_recovery_groups(db: State<'_, Database>) -> Result<Vec<RecoveryGroup>, String> {
    db.get_deleted_recovery_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_recovery_group(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.restore_recovery_group(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn permanently_delete_recovery_group(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.permanently_delete_recovery_group(id).map_err(|e| e.to_string())
}

// ── Snapshot ──

#[tauri::command]
pub fn export_snapshot(
    db: State<'_, Database>,
    flow_ids: Vec<i64>,
    checklist_group_ids: Vec<i64>,
    recovery_group_ids: Vec<i64>,
    restrictions: SnapshotRestrictions,
    data_version: String,
    output_path: String,
) -> Result<String, String> {
    let flows: Vec<TraceFlow> = flow_ids
        .iter()
        .filter_map(|id| db.get_flow(*id).ok())
        .collect();

    let sky_apps = db.get_sky_apps().map_err(|e| e.to_string())?;
    let suppliers = db.get_suppliers().map_err(|e| e.to_string())?;
    let all_checklists = db.get_checklist_groups().map_err(|e| e.to_string())?;
    let checklist_groups = if checklist_group_ids.is_empty() {
        all_checklists
    } else {
        all_checklists.into_iter().filter(|g| checklist_group_ids.contains(&g.id)).collect()
    };
    let all_recovery = db.get_recovery_groups().map_err(|e| e.to_string())?;
    let recovery_groups = if recovery_group_ids.is_empty() {
        all_recovery
    } else {
        all_recovery.into_iter().filter(|g| recovery_group_ids.contains(&g.id)).collect()
    };

    let data = SnapshotData {
        schema_version: snapshot::CURRENT_SCHEMA_VERSION,
        data_version,
        flows,
        sky_apps,
        suppliers,
        checklist_groups,
        recovery_groups,
        restrictions,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        author: "SkyTrace".to_string(),
    };

    let bytes = snapshot::export_snapshot(&data)?;
    std::fs::write(&output_path, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(output_path)
}

#[tauri::command]
pub fn import_snapshot(path: String) -> Result<SnapshotData, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    snapshot::import_snapshot(&bytes)
}

#[tauri::command]
pub fn get_auto_snapshot(state: State<'_, SnapshotState>) -> Result<Option<SnapshotData>, String> {
    let lock = state.data.lock().map_err(|e| e.to_string())?;
    Ok(lock.clone())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppMode {
    pub snapshot_only: bool,
    pub has_snapshot: bool,
}

#[tauri::command]
pub fn get_app_mode(state: State<'_, SnapshotState>) -> Result<AppMode, String> {
    let lock = state.data.lock().map_err(|e| e.to_string())?;
    Ok(AppMode {
        snapshot_only: crate::is_snapshot_only(),
        has_snapshot: lock.is_some(),
    })
}

// ── Remote Config ──

#[tauri::command]
pub async fn check_remote_config() -> Result<RemoteConfig, String> {
    crate::remote_config::fetch_config().await
}

// ── JCP Order ──

#[tauri::command]
pub async fn query_jcp_order(body: Value) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let resp = client
        .post("http://jcp.mis.elong.com/orderparse/getBookingDetailAjax")
        .header("Content-Type", "application/json;charset=UTF-8")
        .header("X-Requested-With", "XMLHttpRequest")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("JCP请求失败: {}", e))?;

    resp.json::<Value>().await.map_err(|e| format!("解析JCP响应失败: {}", e))
}

// ── Supplier Mapping ──

#[tauri::command]
pub async fn query_supplier_mapping(body: Value) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let resp = client
        .post("http://hotedcapi.vip.elong.com:8104/rest/com/elong/hotel/dc/entity/req/mapping/GetMapping4ProductReq")
        .header("Content-Type", "application/json;charset=UTF-8")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("供应商映射请求失败: {}", e))?;

    resp.json::<Value>().await.map_err(|e| format!("解析供应商映射响应失败: {}", e))
}
