//! 轻量内存版登录限流（per-IP 滑动窗口），替代统一鉴权时丢失的旧 shop 登录限流。
//! 单进程内存即可——单机部署无需 Redis。

use std::collections::HashMap;
use std::sync::Mutex;

use axum::http::HeaderMap;

pub struct LoginLimiter {
    inner: Mutex<HashMap<String, (u32, i64)>>, // ip -> (窗口内尝试数, 窗口起点 epoch 秒)
    max: u32,
    window_secs: i64,
}

impl LoginLimiter {
    pub fn new(max: u32, window_secs: i64) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            max,
            window_secs,
        }
    }

    /// 记录一次尝试。窗口内超过上限返回 Err(剩余冷却秒数)。
    pub fn check_and_record(&self, ip: &str) -> Result<(), i64> {
        let now = chrono::Utc::now().timestamp();
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        // 顺手清理过期项，避免内存无限增长
        map.retain(|_, (_, start)| now - *start < self.window_secs);
        let entry = map.entry(ip.to_string()).or_insert((0, now));
        if now - entry.1 >= self.window_secs {
            *entry = (0, now);
        }
        if entry.0 >= self.max {
            return Err((self.window_secs - (now - entry.1)).max(1));
        }
        entry.0 += 1;
        Ok(())
    }

    /// 登录成功后清除该 IP 计数。
    pub fn reset(&self, ip: &str) {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(ip);
    }
}

/// 取客户端 IP：优先 X-Forwarded-For 首个（nginx 反代时），退而 X-Real-IP。
pub fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}
