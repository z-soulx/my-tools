use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::storage::models::*;

const SNAPSHOT_MAGIC: &[u8; 8] = b"SKYTRACE";
const SNAPSHOT_VERSION: u8 = 1;
const KEY_BYTES: &[u8; 32] = b"SkyTraceSnapshotKey2026!@#$%^&*(";

/// 全局快照状态：启动时检测到 snapshot.skytrace 则填充
pub struct SnapshotState {
    pub data: Mutex<Option<SnapshotData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotData {
    pub flows: Vec<TraceFlow>,
    pub sky_apps: Vec<SkyApp>,
    pub suppliers: Vec<Supplier>,
    pub checklist_groups: Vec<ChecklistGroup>,
    #[serde(default)]
    pub recovery_groups: Vec<RecoveryGroup>,
    pub restrictions: SnapshotRestrictions,
    pub created_at: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotRestrictions {
    pub hide_edit: bool,
    pub hide_settings: bool,
    pub hide_suppliers: bool,
    pub hide_quick_query: bool,
    pub hide_checklist_edit: bool,
    pub hide_recovery_edit: bool,
    pub hide_trash: bool,
    pub hide_debug: bool,
    pub hide_ui_link: bool,
}

impl Default for SnapshotRestrictions {
    fn default() -> Self {
        Self {
            hide_edit: true,
            hide_settings: true,
            hide_suppliers: true,
            hide_quick_query: true,
            hide_checklist_edit: true,
            hide_recovery_edit: true,
            hide_trash: true,
            hide_debug: true,
            hide_ui_link: true,
        }
    }
}

pub fn export_snapshot(data: &SnapshotData) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(data).map_err(|e| format!("序列化失败: {}", e))?;

    let cipher = Aes256Gcm::new(KEY_BYTES.into());
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted = cipher
        .encrypt(nonce, json.as_ref())
        .map_err(|e| format!("加密失败: {}", e))?;

    let mut output = Vec::new();
    output.extend_from_slice(SNAPSHOT_MAGIC);
    output.push(SNAPSHOT_VERSION);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&encrypted);
    Ok(output)
}

pub fn import_snapshot(bytes: &[u8]) -> Result<SnapshotData, String> {
    if bytes.len() < 21 || &bytes[..8] != SNAPSHOT_MAGIC {
        return Err("无效的快照文件".to_string());
    }
    let version = bytes[8];
    if version != SNAPSHOT_VERSION {
        return Err(format!("不支持的快照版本: {}", version));
    }

    let nonce_bytes = &bytes[9..21];
    let encrypted = &bytes[21..];

    let cipher = Aes256Gcm::new(KEY_BYTES.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted = cipher
        .decrypt(nonce, encrypted)
        .map_err(|_| "解密失败：快照文件损坏或密钥不匹配".to_string())?;

    serde_json::from_slice(&decrypted).map_err(|e| format!("解析快照数据失败: {}", e))
}

/// 检测可执行文件同级目录下的 snapshot.skytrace
pub fn detect_snapshot() -> Option<SnapshotData> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;

    // macOS .app bundle: 可执行文件在 Contents/MacOS/ 下，快照放在 .app 同级
    let candidates = [
        exe_dir.join("snapshot.skytrace"),
        exe_dir.join("../../../snapshot.skytrace"), // macOS bundle
        exe_dir.join("../../snapshot.skytrace"),
    ];

    for path in &candidates {
        if let Ok(canonical) = path.canonicalize() {
            if canonical.exists() {
                eprintln!("[SkyTrace] 检测到快照文件: {}", canonical.display());
                match std::fs::read(&canonical) {
                    Ok(bytes) => match import_snapshot(&bytes) {
                        Ok(data) => {
                            eprintln!("[SkyTrace] 快照加载成功: {} 条链路", data.flows.len());
                            return Some(data);
                        }
                        Err(e) => eprintln!("[SkyTrace] 快照解密失败: {}", e),
                    },
                    Err(e) => eprintln!("[SkyTrace] 读取快照文件失败: {}", e),
                }
            }
        }
    }
    None
}
