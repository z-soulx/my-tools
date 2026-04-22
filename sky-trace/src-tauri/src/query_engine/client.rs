use reqwest::Client;
use serde_json::Value;

const SKYNET_API_URL: &str = "http://skynetapi.dss.17usoft.com/log/real/list";
const SKYNET_UI_BASE: &str = "https://skyeye.17usoft.com/logs/realquery";

pub struct SkynetClient {
    http: Client,
}

impl SkynetClient {
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn query_log(
        &self,
        app_id: &str,
        token: &str,
        params: Value,
    ) -> Result<Value, String> {
        let mut body = params;
        if let Some(obj) = body.as_object_mut() {
            if !obj.contains_key("appIds") {
                obj.insert(
                    "appIds".to_string(),
                    Value::Array(vec![Value::String(app_id.to_string())]),
                );
            }
            if !obj.contains_key("tokens") {
                obj.insert(
                    "tokens".to_string(),
                    Value::Array(vec![Value::String(token.to_string())]),
                );
            }
            Self::apply_time_defaults(obj);
            Self::ensure_api_defaults(obj);

            // 天网 API 要求 indexContext 中的双引号以 \" 形式传递。
            // 支持用户用单引号 ' 书写模板（更友好），自动转换为双引号再转义。
            // 例如用户输入 'innId':'WYN5300072' → \"innId\":\"WYN5300072\"
            if let Some(val) = obj.get("indexContext").and_then(|v| v.as_str()) {
                if !val.is_empty() {
                    let normalized = val.replace('\'', "\"");
                    let escaped = normalized.replace('"', "\\\"");
                    obj.insert("indexContext".to_string(), Value::String(escaped));
                }
            }
        }

        eprintln!("[SkyTrace] 请求 URL: {}", SKYNET_API_URL);
        eprintln!("[SkyTrace] 请求体: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

        let resp = self
            .http
            .post(SKYNET_API_URL)
            .header("Content-Type", "application/json")
            .header("token", token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP请求失败: {}", e))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("读取响应失败: {}", e))?;

        eprintln!("[SkyTrace] 响应状态: {} | 长度: {} 字节", status.as_u16(), text.len());
        if text.len() < 2000 {
            eprintln!("[SkyTrace] 响应体: {}", text);
        } else {
            eprintln!("[SkyTrace] 响应体(截断): {}...", &text[..500]);
        }

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status.as_u16(), text));
        }

        serde_json::from_str(&text).map_err(|e| format!("解析JSON失败: {}", e))
    }

    pub fn generate_ui_link(app_uk: &str, params: &Value) -> String {
        let mut data = serde_json::Map::new();

        if let Some(obj) = params.as_object() {
            let time_from = obj
                .get("beginTime")
                .and_then(|v| v.as_str())
                .unwrap_or("now-30m")
                .to_string();
            let time_to = obj
                .get("endTime")
                .and_then(|v| v.as_str())
                .unwrap_or("now")
                .to_string();

            let mut time = serde_json::Map::new();
            time.insert("from".to_string(), Value::String(time_from.clone()));
            time.insert("to".to_string(), Value::String(time_to.clone()));
            let mut raw = serde_json::Map::new();
            raw.insert("from".to_string(), Value::String(time_from));
            raw.insert("to".to_string(), Value::String(time_to));
            time.insert("raw".to_string(), Value::Object(raw));
            data.insert("time".to_string(), Value::Object(time));

            let mut cats: Vec<Value> = Vec::new();
            for key in ["module", "category", "subCategory"] {
                if let Some(val) = obj.get(key).and_then(|v| v.as_str()) {
                    if !val.is_empty() {
                        cats.push(Value::String(val.to_string()));
                    }
                }
            }
            data.insert("category".to_string(), Value::Array(cats));

            let f1 = obj.get("filter1").and_then(|v| v.as_str()).unwrap_or("").to_string();
            data.insert("filter1".to_string(), Value::String(f1));

            let f2 = obj.get("filter2").and_then(|v| v.as_str()).unwrap_or("").to_string();
            data.insert("filter2".to_string(), Value::String(f2));

            let msg = obj.get("indexContext").and_then(|v| v.as_str()).unwrap_or("").to_string();
            // ' → " 规范化，然后将 \ 转义为 \\ (Lucene 特殊字符)。
            // " 本身不转义：用户写 "asd 保持 "asd，写 \"asd 变成 \\"asd。
            let escaped_msg = msg.replace('\'', "\"").replace('\\', "\\\\");
            data.insert("message".to_string(), Value::String(escaped_msg));
            if let Some(cid) = obj.get("contextId").and_then(|v| v.as_str()) {
                if !cid.is_empty() {
                    data.insert("flowID".to_string(), Value::String(cid.to_string()));
                }
            }
        }

        let data_json = serde_json::to_string(&Value::Object(data)).unwrap_or_default();
        let encoded = urlencoding::encode(&data_json);
        format!("{}?app={}&data={}", SKYNET_UI_BASE, app_uk, encoded)
    }

    fn apply_time_defaults(obj: &mut serde_json::Map<String, Value>) {
        let now = chrono::Local::now();

        let resolve_time = |val: &str| -> String {
            if val.starts_with("now") {
                let offset = &val[3..];
                let dt = if offset.is_empty() {
                    now
                } else {
                    let (sign, rest) = if offset.starts_with('-') {
                        (-1i64, &offset[1..])
                    } else {
                        (1, offset)
                    };
                    let (num_str, unit) = rest.split_at(rest.len() - 1);
                    let num: i64 = num_str.parse().unwrap_or(30);
                    let minutes = match unit {
                        "m" => num,
                        "h" => num * 60,
                        "d" => num * 60 * 24,
                        _ => num,
                    };
                    now + chrono::Duration::minutes(sign * minutes)
                };
                dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
            } else {
                val.to_string()
            }
        };

        if let Some(begin) = obj.get("beginTime").and_then(|v| v.as_str()) {
            let resolved = resolve_time(begin);
            obj.insert("beginTime".to_string(), Value::String(resolved));
        } else {
            let thirty_ago = (now - chrono::Duration::minutes(30))
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string();
            obj.insert("beginTime".to_string(), Value::String(thirty_ago));
        }

        if let Some(end) = obj.get("endTime").and_then(|v| v.as_str()) {
            let resolved = resolve_time(end);
            obj.insert("endTime".to_string(), Value::String(resolved));
        } else {
            let now_str = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            obj.insert("endTime".to_string(), Value::String(now_str));
        }
    }

    fn ensure_api_defaults(obj: &mut serde_json::Map<String, Value>) {
        let empty_str = || Value::String(String::new());
        let empty_arr = || Value::Array(Vec::new());

        for key in ["module", "category", "subCategory", "contextId", "priority", "env"] {
            obj.entry(key.to_string()).or_insert_with(empty_str);
        }
        for key in ["filter1s", "filter2s", "modules", "categories", "subCategories", "ips"] {
            obj.entry(key.to_string()).or_insert_with(empty_arr);
        }
        for key in ["filter1", "filter2", "indexContext"] {
            obj.entry(key.to_string()).or_insert_with(empty_str);
        }
        obj.entry("pageSize".to_string())
            .or_insert(Value::Number(100.into()));
        obj.entry("operationType".to_string())
            .or_insert(Value::String("next".to_string()));
        obj.entry("lastRowTime".to_string())
            .or_insert(Value::Number(0.into()));
        obj.entry("lastRowData".to_string())
            .or_insert(Value::String(String::new()));
    }
}
