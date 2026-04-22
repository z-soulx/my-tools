use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyApp {
    pub id: i64,
    pub app_id: String,
    pub app_uk: String,
    pub token: String,
    pub name: String,
    pub env: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyAppInput {
    pub app_id: String,
    pub app_uk: String,
    pub token: String,
    pub name: String,
    pub env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Supplier {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: String,
    pub service_ids: Vec<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInput {
    pub name: String,
    pub code: String,
    pub description: String,
    pub service_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceFlow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub supplier_id: Option<i64>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub sort_order: i64,
    pub dynamic_params: Vec<DynamicParam>,
    pub nodes: Vec<TraceNode>,
    #[serde(default)]
    pub node_groups: Vec<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceFlowInput {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub supplier_id: Option<i64>,
    pub tags: Vec<String>,
    pub dynamic_params: Vec<DynamicParam>,
    pub nodes: Vec<TraceNode>,
    #[serde(default)]
    pub node_groups: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicParam {
    pub key: String,
    pub label: String,
    pub required: bool,
    pub default_value: String,
    #[serde(default)]
    pub hint: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub allow_custom: Option<bool>,
    #[serde(default)]
    pub snippets: Option<Vec<String>>,
    #[serde(default)]
    pub hidden: Option<bool>,
    #[serde(default)]
    pub param_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    pub sort_order: i64,
    pub config: serde_json::Value,
    #[serde(default)]
    pub health_rules: Option<serde_json::Value>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistGroup {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub items: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistGroupInput {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub items: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryGroup {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub steps: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryGroupInput {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub steps: serde_json::Value,
}
