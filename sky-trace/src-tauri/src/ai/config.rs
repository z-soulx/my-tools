use std::sync::RwLock;

use crate::remote_config::RemoteConfig;

/// 进程内缓存的 AI 配置（来自远程飞书 Bitable）
/// Token 仅驻内存，不持久化；每次 check_remote_config 时刷新。
static AI_CONFIG: RwLock<Option<AiConfig>> = RwLock::new(None);

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub enabled: bool,
    pub base_url: String,
    pub token: String,
    pub model: String,
    pub default_system_prompt: String,
}

impl AiConfig {
    pub fn from_remote(rc: &RemoteConfig) -> Self {
        Self {
            enabled: rc.ai_enabled,
            base_url: rc.ai_base_url.trim_end_matches('/').to_string(),
            token: rc.ai_token.clone(),
            model: rc.ai_model.clone(),
            default_system_prompt: rc.ai_default_system_prompt.clone(),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.enabled && !self.base_url.is_empty() && !self.token.is_empty() && !self.model.is_empty()
    }
}

pub fn set_cached(rc: &RemoteConfig) {
    if let Ok(mut w) = AI_CONFIG.write() {
        *w = Some(AiConfig::from_remote(rc));
    }
}

pub fn get_cached() -> Option<AiConfig> {
    AI_CONFIG.read().ok().and_then(|r| r.clone())
}
