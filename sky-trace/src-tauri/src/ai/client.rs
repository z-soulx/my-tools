use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

use super::config;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(120))
            .no_proxy()
            .build()
            .expect("Failed to build HTTP client")
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub enabled: bool,
    pub has_token: bool,
    pub model: String,
    pub base_url: String,
}

pub fn status() -> AiStatus {
    let cfg = config::get_cached();
    match cfg {
        Some(c) => AiStatus {
            enabled: c.is_ready(),
            has_token: !c.token.is_empty(),
            model: c.model,
            base_url: c.base_url,
        },
        None => AiStatus {
            enabled: false,
            has_token: false,
            model: String::new(),
            base_url: String::new(),
        },
    }
}

/// 调用 OpenAI 兼容 /v1/chat/completions 流式接口
/// 通过 Tauri Event 推送：
///   - `ai:chunk:{session_id}` (string payload，每个 token / delta)
///   - `ai:done:{session_id}`
///   - `ai:error:{session_id}` (string payload，错误信息)
pub async fn chat_stream(
    app: AppHandle,
    session_id: String,
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    let cfg = config::get_cached().ok_or_else(|| "AI 未启用：未加载远程配置".to_string())?;
    if !cfg.is_ready() {
        return Err("AI 未启用或配置不完整".to_string());
    }

    let url = format!("{}/chat/completions", cfg.base_url);
    let body = json!({
        "model": cfg.model,
        "messages": messages,
        "stream": true,
    });

    #[cfg(debug_assertions)]
    {
        let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
        eprintln!("[AI] 请求 URL: {}", url);
        eprintln!("[AI] payload 总字符数: {} ({:.1} KB)", total_chars, total_chars as f64 / 1024.0);
    }

    let resp = get_http_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", cfg.token))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Accept-Encoding", "identity")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI 请求失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let msg = format!("AI 服务错误 {}: {}", status, text);
        let _ = app.emit(&format!("ai:error:{}", session_id), msg.clone());
        return Err(msg);
    }

    #[cfg(debug_assertions)]
    eprintln!("[AI] 响应状态: {} — 开始读取流 (Content-Encoding: {:?})", resp.status().as_u16(), resp.headers().get("content-encoding"));

    let chunk_event = format!("ai:chunk:{}", session_id);
    let thinking_event = format!("ai:thinking:{}", session_id);
    let done_event = format!("ai:done:{}", session_id);
    let error_event = format!("ai:error:{}", session_id);

    let mut stream = resp.bytes_stream();
    let mut buf = String::new();
    #[cfg(debug_assertions)]
    let mut first_chunk = true;

    while let Some(item) = stream.next().await {
        match item {
            Err(e) => {
                let msg = format!("流式读取失败: {}", e);
                let _ = app.emit(&error_event, msg.clone());
                return Err(msg);
            }
            Ok(bytes) => {
                #[cfg(debug_assertions)]
                if first_chunk {
                    eprintln!("[AI] 收到首个 chunk ({} bytes)", bytes.len());
                    eprintln!("[AI] 首个 chunk 原始内容: {:?}", String::from_utf8_lossy(&bytes).chars().take(200).collect::<String>());
                    first_chunk = false;
                }
                let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n").replace('\r', "\n");
                buf.push_str(&text);

                while let Some(idx) = buf.find("\n\n") {
                    let event = buf[..idx].to_string();
                    buf.drain(..idx + 2);

                    for line in event.lines() {
                        let line = line.trim_start();
                        let data = match line.strip_prefix("data:") {
                            Some(d) => d.trim(),
                            None => continue,
                        };
                        if data == "[DONE]" {
                            let _ = app.emit(&done_event, ());
                            return Ok(());
                        }
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(delta) = v
                                .get("choices")
                                .and_then(|c| c.get(0))
                                .and_then(|c0| c0.get("delta"))
                            {
                                if let Some(rc) = delta.get("reasoning_content").and_then(|s| s.as_str()) {
                                    if !rc.is_empty() {
                                        let _ = app.emit(&thinking_event, rc.to_string());
                                    }
                                }
                                if let Some(content) = delta.get("content").and_then(|s| s.as_str()) {
                                    if !content.is_empty() {
                                        let _ = app.emit(&chunk_event, content.to_string());
                                    }
                                }
                            }
                            if let Some(reason) = v
                                .get("choices")
                                .and_then(|c| c.get(0))
                                .and_then(|c0| c0.get("finish_reason"))
                                .and_then(|s| s.as_str())
                            {
                                if reason == "stop" || reason == "length" {
                                    let _ = app.emit(&done_event, ());
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 流自然结束
    let _ = app.emit(&done_event, ());
    Ok(())
}
