use crate::types::*;
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, ACCEPT, REFERER, ORIGIN};
use serde_json::json;

const BASE_URL: &str = "https://www.kimi.com";

fn resolve_token(config: &AppConfig) -> Option<String> {
    // 只使用用户配置的浏览器 token（从 kimi.com 登录后获取）
    // CLI OAuth token (~/.kimi/credentials/kimi-code.json) 对 web API 无效，不再使用
    if let Some(token) = &config.kimi_token {
        if !token.is_empty() {
            return Some(token.clone());
        }
    }
    None
}

fn resolve_device_id() -> Option<String> {
    let path = dirs::home_dir()?.join(".kimi").join("device_id");
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn extract_jwt_claims(token: &str) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".to_string());
    }
    
    let mut payload = parts[1]
        .replace('-', "+")
        .replace('_', "/");
    
    // padding
    while payload.len() % 4 != 0 {
        payload.push('=');
    }
    
    let decoded = base64::decode(&payload)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;
    
    let json: serde_json::Value = serde_json::from_slice(&decoded)
        .map_err(|e| format!("JSON parse failed: {}", e))?;
    
    Ok(json)
}

fn build_client(token: &str, device_id: &str, user_id: &str, session_id: &str) -> reqwest::Client {
    let mut headers = HeaderMap::new();
    
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap());
    headers.insert("x-msh-device-id", HeaderValue::from_str(device_id).unwrap());
    headers.insert("x-msh-session-id", HeaderValue::from_str(session_id).unwrap());
    headers.insert("x-traffic-id", HeaderValue::from_str(user_id).unwrap());
    headers.insert("x-msh-platform", HeaderValue::from_static("web"));
    headers.insert("x-msh-version", HeaderValue::from_static("1.0.0"));
    headers.insert("x-language", HeaderValue::from_static("zh-CN"));
    headers.insert("r-timezone", HeaderValue::from_static("Asia/Shanghai"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.kimi.com/code/console"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://www.kimi.com"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("connect-protocol-version", HeaderValue::from_static("1"));
    
    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap()
}

pub async fn fetch_usage_data(config: &AppConfig) -> Result<KimeUsageData, String> {
    let token = resolve_token(config)
        .ok_or("未配置 Kimi 浏览器 token。请点击下方「自动获取 Token」按钮登录获取。")?;
    let device_id = resolve_device_id()
        .unwrap_or_else(|| "unknown_device".to_string());
    
    let claims = extract_jwt_claims(&token)?;
    let user_id = claims.get("sub")
        .and_then(|v| v.as_str())
        .ok_or("JWT 中缺少 sub (user_id)")?
        .to_string();
    let session_id = claims.get("ssid")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();
    
    let client = build_client(&token, &device_id, &user_id, &session_id);
    
    // 调用 GetSubscription
    let sub_url = format!("{}/apiv2/kimi.gateway.membership.v2.MembershipService/GetSubscription", BASE_URL);
    let sub_resp = client.post(&sub_url)
        .json(&json!({}))
        .send()
        .await
        .map_err(|e| format!("GetSubscription 请求失败: {}", e))?;
    
    let sub_status = sub_resp.status();
    let sub_text = sub_resp.text().await.map_err(|e| e.to_string())?;
    
    if !sub_status.is_success() {
        return Err(format!("GetSubscription HTTP {}: {}", sub_status, &sub_text[..sub_text.len().min(200)]));
    }
    
    let sub: GetSubscriptionResponse = serde_json::from_str(&sub_text)
        .map_err(|e| format!("GetSubscription 解析失败: {}. 原始响应: {}", e, &sub_text[..sub_text.len().min(500)]))?;
    
    // 调用 GetUsages
    let usage_url = format!("{}/apiv2/kimi.gateway.billing.v1.BillingService/GetUsages", BASE_URL);
    let usage_resp = client.post(&usage_url)
        .json(&json!({"scope": ["FEATURE_CODING"]}))
        .send()
        .await
        .map_err(|e| format!("GetUsages 请求失败: {}", e))?;
    
    let usage_status = usage_resp.status();
    let usage_text = usage_resp.text().await.map_err(|e| e.to_string())?;
    
    if !usage_status.is_success() {
        return Err(format!("GetUsages HTTP {}: {}", usage_status, &usage_text[..usage_text.len().min(200)]));
    }
    
    let usage: GetUsagesResponse = serde_json::from_str(&usage_text)
        .map_err(|e| format!("GetUsages 解析失败: {}. 原始响应: {}", e, &usage_text[..usage_text.len().min(500)]))?;
    
    // 转换为前端类型
    convert_to_frontend_model(sub, usage)
}

fn convert_to_frontend_model(sub: GetSubscriptionResponse, usage: GetUsagesResponse) -> Result<KimeUsageData, String> {
    let subscription = sub.subscription.ok_or("订阅信息为空")?;
    
    // 计算有效期
    let end_time = chrono::DateTime::parse_from_rfc3339(&subscription.current_end_time)
        .map_err(|e| format!("日期解析失败: {}", e))?;
    let now = Utc::now();
    let days_remaining = (end_time.with_timezone(&Utc) - now).num_days().max(0);
    
    // 找到 CODING 相关的 usage
    let coding_usage = usage.usages.iter()
        .find(|u| u.scope == "FEATURE_CODING");
    
    // 计算额度使用比例
    // 优先级: coding_usage.detail > total_quota > balances
    // coding_usage.detail 对应控制台"本周用量"（约7天周期）
    let mut usage_ratio = 0.0;
    let mut used = 0u64;
    let mut total = 0u64;
    
    if let Some(cu) = coding_usage {
        let limit: u64 = cu.detail.limit.parse().unwrap_or(0);
        let remaining: u64 = cu.detail.remaining.parse().unwrap_or(0);
        total = limit;
        used = limit.saturating_sub(remaining);
        if limit > 0 {
            usage_ratio = used as f64 / limit as f64;
        }
    } else if let Some(ref tq) = usage.total_quota {
        let tq_limit: u64 = tq.limit.parse().unwrap_or(0);
        let tq_remaining: u64 = tq.remaining.parse().unwrap_or(0);
        total = tq_limit;
        used = tq_limit.saturating_sub(tq_remaining);
        if tq_limit > 0 {
            usage_ratio = used as f64 / tq_limit as f64;
        }
    } else if let Some(balance) = sub.balances.first() {
        usage_ratio = balance.amount_used_ratio;
        total = 100000;
        used = (total as f64 * usage_ratio) as u64;
    }
    
    // 构建频限详情
    let mut rpm = RateLimitItem { current: 0, limit: 0, remaining: 0 };
    let mut tpm = RateLimitItem { current: 0, limit: 0, remaining: 0 };
    let mut rpd = RateLimitItem { current: 0, limit: 0, remaining: 0 };
    
    if let Some(cu) = coding_usage {
        // 主 detail 可能是 RPD (daily)
        if let Ok(limit) = cu.detail.limit.parse::<u32>() {
            let remaining: u32 = cu.detail.remaining.parse().unwrap_or(0);
            rpd = RateLimitItem {
                current: limit.saturating_sub(remaining),
                limit,
                remaining,
            };
        }
        
        // limits 数组中的项
        for limit_item in &cu.limits {
            match limit_item.window.time_unit.as_str() {
                "TIME_UNIT_MINUTE" => {
                    if let Ok(lim) = limit_item.detail.limit.parse::<u32>() {
                        let rem: u32 = limit_item.detail.remaining.parse().unwrap_or(0);
                        rpm = RateLimitItem {
                            current: lim.saturating_sub(rem),
                            limit: lim,
                            remaining: rem,
                        };
                    }
                }
                "TIME_UNIT_HOUR" => {
                    if let Ok(lim) = limit_item.detail.limit.parse::<u32>() {
                        let rem: u32 = limit_item.detail.remaining.parse().unwrap_or(0);
                        tpm = RateLimitItem {
                            current: lim.saturating_sub(rem),
                            limit: lim,
                            remaining: rem,
                        };
                    }
                }
                _ => {}
            }
        }
    }
    
    // 模型权限从 capabilities 中提取
    let model_permissions: Vec<String> = sub.capabilities.iter()
        .map(|c| c.feature.clone())
        .collect();
    
    Ok(KimeUsageData {
        current_plan: subscription.goods.title.clone(),
        validity: ValidityInfo {
            current_end_time: subscription.current_end_time,
            days_remaining,
        },
        weekly_usage: UsageInfo {
            used,
            total,
            unit: "tokens".to_string(),
        },
        usage_ratio,
        rate_limit_details: RateLimitDetails { rpm, tpm, rpd },
        model_permissions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    /// 集成测试：调用真实 API，验证返回数据结构
    #[tokio::test]
    async fn test_fetch_usage_data_structure() {
        let cfg = config::read_config();
        
        // 如果没有配置 token，跳过测试
        if cfg.kimi_token.is_none() || cfg.kimi_token.as_ref().unwrap().is_empty() {
            eprintln!("SKIP: no token configured");
            return;
        }

        let result = fetch_usage_data(&cfg).await;
        assert!(result.is_ok(), "API call failed: {:?}", result.err());
        
        let data = result.unwrap();
        
        // 打印完整数据结构，供人工检查
        println!("\n========== API Response Structure ==========");
        println!("current_plan: {}", data.current_plan);
        println!("validity.current_end_time: {}", data.validity.current_end_time);
        println!("validity.days_remaining: {}", data.validity.days_remaining);
        println!("weekly_usage.used: {}", data.weekly_usage.used);
        println!("weekly_usage.total: {}", data.weekly_usage.total);
        println!("weekly_usage.unit: {}", data.weekly_usage.unit);
        println!("usage_ratio: {}", data.usage_ratio);
        println!("rate_limit_details.rpm: {}/{}/{}", 
            data.rate_limit_details.rpm.current,
            data.rate_limit_details.rpm.limit,
            data.rate_limit_details.rpm.remaining
        );
        println!("rate_limit_details.tpm: {}/{}/{}", 
            data.rate_limit_details.tpm.current,
            data.rate_limit_details.tpm.limit,
            data.rate_limit_details.tpm.remaining
        );
        println!("rate_limit_details.rpd: {}/{}/{}", 
            data.rate_limit_details.rpd.current,
            data.rate_limit_details.rpd.limit,
            data.rate_limit_details.rpd.remaining
        );
        println!("model_permissions: {:?}", data.model_permissions);
        println!("============================================\n");
        
        // 结构验证
        assert!(!data.current_plan.is_empty(), "current_plan should not be empty");
        assert!(data.usage_ratio >= 0.0 && data.usage_ratio <= 1.0, "usage_ratio should be in [0,1]");
        assert!(data.validity.days_remaining >= 0, "days_remaining should be >= 0");
    }

    /// 测试：验证 JWT 解析能正确提取 sub 和 ssid
    #[test]
    fn test_extract_jwt_claims() {
        // 用一个已知的测试 JWT (header: {"alg":"none"}, payload: {"sub":"test_user","ssid":"123"})
        let token = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJzc2lkIjoiMTIzIn0.";
        let claims = extract_jwt_claims(token).expect("should parse test JWT");
        assert_eq!(claims["sub"].as_str().unwrap(), "test_user");
        assert_eq!(claims["ssid"].as_str().unwrap(), "123");
    }
}

// base64 decoder helper (no external base64 crate needed for simple JWT)
mod base64 {
    pub fn decode(input: &str) -> Result<Vec<u8>, String> {
        use std::collections::HashMap;
        let alphabet: HashMap<char, u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            .chars().enumerate().map(|(i, c)| (c, i as u8)).collect();
        
        let mut result = Vec::new();
        let mut buffer = 0u32;
        let mut bits = 0u32;
        
        for ch in input.chars() {
            if ch == '=' { break; }
            let val = alphabet.get(&ch).ok_or("Invalid base64 char")?;
            buffer = (buffer << 6) | (*val as u32);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                result.push((buffer >> bits) as u8);
            }
        }
        Ok(result)
    }
}
