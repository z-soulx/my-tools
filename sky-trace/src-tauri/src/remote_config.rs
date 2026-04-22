use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Credentials obfuscated via base64 — prevents trivial `strings` extraction.
// Anyone with read access to Bitable can only READ the public config — no write risk.
// Revoke the app in Feishu Open Platform at any time to lock all clients.
const CRED: &str = "Y2xpX2E5NTZjZDM0MjNmOTVjY2I6czVndnRsM3h4ZVhXWGFsS3duejhXZFVRSGJvMnVuN2I6QnpSRndEQ0hJaUVBUXZreWdnMmM3Y2tqbktnOnRibEJ3enQwNTlOR1J1dGM=";

fn decode_creds() -> Option<(String, String, String, String)> {
    let raw = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        CRED.trim(),
    )
    .ok()?;
    let s = String::from_utf8(raw).ok()?;
    let parts: Vec<&str> = s.splitn(4, ':').collect();
    if parts.len() != 4 {
        return None;
    }
    Some((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
        parts[3].to_string(),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub text: String,
    #[serde(rename = "type")]
    pub kind: String, // "info" | "warning" | "error"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    pub enabled: bool,
    pub min_version: String,
    pub latest_version: String,
    pub message: String,
    pub update_url_mac: String,
    pub update_url_win: String,
    pub update_notes: String,
    pub features: HashMap<String, bool>,
    pub announcement: Option<Announcement>,
    pub latest_data_version: String,
    pub data_update_url: String,
    pub data_update_notes: String,
}

// ── Feishu API response shapes ──────────────────────────────────────────────

#[derive(Deserialize)]
struct TokenResp {
    code: i64,
    tenant_access_token: Option<String>,
    msg: Option<String>,
}

#[derive(Deserialize)]
struct BitableResp {
    code: i64,
    msg: Option<String>,
    data: Option<BitableData>,
}

#[derive(Deserialize)]
struct BitableData {
    items: Option<Vec<BitableRecord>>,
}

#[derive(Deserialize)]
struct BitableRecord {
    fields: serde_json::Value,
}

// ── Public API ───────────────────────────────────────────────────────────────

pub async fn fetch_config() -> Result<RemoteConfig, String> {
    let (app_id, app_secret, app_token, table_id) =
        decode_creds().ok_or_else(|| "凭证配置错误".to_string())?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTP 客户端初始化失败: {}", e))?;

    // Step 1: get tenant_access_token
    let token_body = serde_json::json!({
        "app_id": app_id,
        "app_secret": app_secret,
    });
    let token_resp: TokenResp = client
        .post("https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal/")
        .json(&token_body)
        .send()
        .await
        .map_err(|e| format!("无法连接验证服务器: {}", e))?
        .json()
        .await
        .map_err(|e| format!("验证服务器响应解析失败: {}", e))?;

    if token_resp.code != 0 {
        return Err(format!(
            "飞书认证失败: {}",
            token_resp.msg.unwrap_or_default()
        ));
    }
    let access_token = token_resp
        .tenant_access_token
        .ok_or_else(|| "未获取到访问令牌".to_string())?;

    // Step 2: read Bitable records
    let url = format!(
        "https://open.feishu.cn/open-apis/bitable/v1/apps/{}/tables/{}/records?page_size=1",
        app_token, table_id
    );
    let bitable_resp: BitableResp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("读取配置失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("配置响应解析失败: {}", e))?;

    if bitable_resp.code != 0 {
        return Err(format!(
            "读取配置失败: {}",
            bitable_resp.msg.unwrap_or_default()
        ));
    }

    let record = bitable_resp
        .data
        .as_ref()
        .and_then(|d| d.items.as_ref())
        .and_then(|items| items.first())
        .ok_or_else(|| "配置表为空".to_string())?;

    parse_record(&record.fields)
}

fn bool_field(fields: &serde_json::Value, key: &str) -> bool {
    fields.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn str_field(fields: &serde_json::Value, key: &str) -> String {
    // Feishu text fields may come as:
    // - plain string: "hello"
    // - array of {text} objects: [{"text":"hello"}]
    // - URL object: {"link":"https://...","text":"..."}
    let v = fields.get(key);
    match v {
        None => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join(""),
        Some(serde_json::Value::Object(obj)) => {
            // URL type field: {"link": "https://...", "text": "..."}
            if let Some(link) = obj.get("link").and_then(|v| v.as_str()) {
                link.to_string()
            } else if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                text.to_string()
            } else {
                serde_json::Value::Object(obj.clone()).to_string()
            }
        }
        Some(other) => other.to_string(),
    }
}

fn parse_record(fields: &serde_json::Value) -> Result<RemoteConfig, String> {
    let enabled = bool_field(fields, "enabled");
    let min_version = str_field(fields, "min_version");
    let latest_version = str_field(fields, "latest_version");
    let message = str_field(fields, "message");
    let update_url_mac = str_field(fields, "update_url_mac");
    let update_url_win = str_field(fields, "update_url_win");
    let update_notes = str_field(fields, "update_notes");

    // features: stored as JSON string like {"skynetQuery":true}
    let features_raw = str_field(fields, "features");
    let features: HashMap<String, bool> = if features_raw.is_empty() {
        HashMap::new()
    } else {
        serde_json::from_str(&features_raw).unwrap_or_default()
    };

    let announcement_text = str_field(fields, "announcement_text");
    let announcement = if announcement_text.is_empty() {
        None
    } else {
        Some(Announcement {
            text: announcement_text,
            kind: str_field(fields, "announcement_type"),
        })
    };

    let latest_data_version = str_field(fields, "latest_data_version");
    let data_update_url = str_field(fields, "data_update_url");
    let data_update_notes = str_field(fields, "data_update_notes");

    Ok(RemoteConfig {
        enabled,
        min_version,
        latest_version,
        message,
        update_url_mac,
        update_url_win,
        update_notes,
        features,
        announcement,
        latest_data_version,
        data_update_url,
        data_update_notes,
    })
}
