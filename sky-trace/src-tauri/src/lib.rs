mod commands;
mod query_engine;
pub mod snapshot;
mod storage;

use query_engine::SkynetClient;
use snapshot::SnapshotState;
use storage::Database;
use std::sync::Mutex;
use tauri::Manager;

pub fn is_snapshot_only() -> bool {
    cfg!(feature = "snapshot-only")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let auto_snapshot = snapshot::detect_snapshot();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("无法获取应用数据目录");

            let db = Database::new(data_dir).expect("数据库初始化失败");
            app.manage(db);
            app.manage(SkynetClient::new());
            app.manage(SnapshotState {
                data: Mutex::new(auto_snapshot),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_sky_apps,
            commands::save_sky_app,
            commands::delete_sky_app,
            commands::get_suppliers,
            commands::save_supplier,
            commands::delete_supplier,
            commands::get_flows,
            commands::get_flow,
            commands::save_flow,
            commands::delete_flow,
            commands::duplicate_flow,
            commands::toggle_flow_favorite,
            commands::query_skynet_log,
            commands::generate_skynet_ui_link,
            commands::get_deleted_flows,
            commands::restore_flow,
            commands::permanently_delete_flow,
            commands::get_deleted_suppliers,
            commands::restore_supplier,
            commands::permanently_delete_supplier,
            commands::empty_trash,
            commands::get_checklist_groups,
            commands::get_checklist_group,
            commands::save_checklist_group,
            commands::delete_checklist_group,
            commands::get_recovery_groups,
            commands::get_recovery_group,
            commands::save_recovery_group,
            commands::delete_recovery_group,
            commands::get_deleted_recovery_groups,
            commands::restore_recovery_group,
            commands::permanently_delete_recovery_group,
            commands::export_snapshot,
            commands::import_snapshot,
            commands::get_auto_snapshot,
            commands::get_app_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
