//! 语音工坊（voice）：转发跑在团员本地、经 frp 暴露的 TTS（GPT-SoVITS）与 RVC 服务。
//!
//! 本模块**不落库**：后台定时探活两个本地服务并缓存在线状态（本地 Mac 关机时站点照常
//! 打开，前端据 /status 显示「服务离线」）；角色列表低频缓存；合成/变声请求带共享密钥
//! 转发到本地服务的 /hfc/* 端点，音频响应流式回传（生产 MemoryMax=512M，不整段驻内存）。
//! 每服务 1 并发信号量 + 每用户提交冷却，保护本地算力。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::header;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::{AppError, AppResult, Config};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth_routes::require_verified_member;
use crate::state::AppState;

/// RVC 上传体积上限（与本地服务 HFC_UPLOAD_MAX、前端预校验三方一致）。
const RVC_UPLOAD_MAX: usize = 50 * 1024 * 1024;
/// TTS 文本长度上限（字符数，与本地服务 HFC_TEXT_MAX 一致）。
const TTS_TEXT_MAX: usize = 500;
/// 角色列表缓存有效期。
const ROLES_TTL: Duration = Duration::from_secs(600);
/// TTS 忙时最长排队等待（RVC 长任务不排队，直接 429）。
const TTS_QUEUE_WAIT: Duration = Duration::from_secs(15);

// ============================================================
//  共享状态
// ============================================================

/// 探活结果快照。
#[derive(Default, Clone)]
pub struct VoiceStatus {
    pub tts_online: bool,
    pub rvc_online: bool,
    /// 最近一次探活时间（RFC3339）；None 表示尚未探测过。
    pub checked_at: Option<String>,
}

/// 角色列表缓存：`{"tts":[{name,refs:[…]}],"rvc":[{name}]}`。
#[derive(Default)]
struct RolesCache {
    data: Option<Arc<Value>>,
    fetched_at: Option<Instant>,
}

#[derive(Clone)]
pub struct VoiceState {
    client: reqwest::Client,
    status: Arc<RwLock<VoiceStatus>>,
    roles: Arc<RwLock<RolesCache>>,
    /// 每服务 1 并发：本地推理串行，多余请求排队/快速失败。
    tts_sem: Arc<tokio::sync::Semaphore>,
    rvc_sem: Arc<tokio::sync::Semaphore>,
    /// user_id → 最近一次成功提交时间（TTS/RVC 共用冷却）。
    cooldown: Arc<Mutex<HashMap<i64, Instant>>>,
}

impl VoiceState {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("haruhifanclub-voice/1.0")
                .connect_timeout(Duration::from_secs(10))
                // 上游是本机回环/同机 frp 隧道，绝不能被开发机的系统代理环境变量劫持：
                // 服务停机时代理会代答 5xx，把「离线(503)」污染成「上游错误(500)」，探活也会被骗
                .no_proxy()
                .build()
                .expect("voice reqwest client"),
            status: Arc::new(RwLock::new(VoiceStatus::default())),
            roles: Arc::new(RwLock::new(RolesCache::default())),
            tts_sem: Arc::new(tokio::sync::Semaphore::new(1)),
            rvc_sem: Arc::new(tokio::sync::Semaphore::new(1)),
            cooldown: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 冷却检查：返回 Err(剩余秒数) 表示仍在冷却期。
    fn check_cooldown(&self, user_id: i64, cooldown: Duration) -> Result<(), u64> {
        let map = self.cooldown.lock().unwrap();
        if let Some(last) = map.get(&user_id) {
            let elapsed = last.elapsed();
            if elapsed < cooldown {
                return Err((cooldown - elapsed).as_secs() + 1);
            }
        }
        Ok(())
    }

    /// 记录一次成功提交（失败的请求不计冷却）。顺手清掉过期条目防无限增长。
    fn touch_cooldown(&self, user_id: i64, cooldown: Duration) {
        let mut map = self.cooldown.lock().unwrap();
        if map.len() > 512 {
            map.retain(|_, t| t.elapsed() < cooldown);
        }
        map.insert(user_id, Instant::now());
    }

    fn mark_offline(&self, tts: bool) {
        let mut s = self.status.write().unwrap();
        if tts {
            s.tts_online = false;
        } else {
            s.rvc_online = false;
        }
    }
}

impl Default for VoiceState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
//  路由与 handler
// ============================================================

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/roles", get(get_roles))
        .route("/tts", post(post_tts))
        // RVC 上传：route 级 body limit 收紧到 50MB+ 表单开销（全局是 256MB）
        .route(
            "/rvc",
            post(post_rvc).layer(DefaultBodyLimit::max(RVC_UPLOAD_MAX + 2 * 1024 * 1024)),
        )
        .route("/admin/refresh", post(admin_refresh))
}

/// 服务在线状态（公开，直读探活缓存，零上游开销）。
async fn get_status(State(state): State<AppState>) -> Json<Value> {
    let s = state.voice.status.read().unwrap().clone();
    Json(json!({
        "tts": { "online": s.tts_online },
        "rvc": { "online": s.rvc_online },
        "checkedAt": s.checked_at,
    }))
}

/// 角色列表（公开）：优先返回缓存；过期/为空时现场拉取，失败回退旧值。
async fn get_roles(State(state): State<AppState>) -> Json<Arc<Value>> {
    let (cached, fresh) = {
        let cache = state.voice.roles.read().unwrap();
        let fresh = cache
            .fetched_at
            .map(|t| t.elapsed() < ROLES_TTL)
            .unwrap_or(false);
        (cache.data.clone(), fresh)
    };
    if let (Some(data), true) = (&cached, fresh) {
        return Json(data.clone());
    }
    match refresh_roles(&state.voice, &state.cfg).await {
        Some(data) => Json(data),
        // 拉取失败：有旧缓存给旧缓存，否则给空结构（前端按「服务离线」处理）
        None => Json(cached.unwrap_or_else(|| Arc::new(json!({ "tts": [], "rvc": [] })))),
    }
}

#[derive(Deserialize)]
struct TtsReq {
    character: String,
    /// 预设参考音频名（决定语气情绪），对应 characters.json 里的 refs[].name。
    #[serde(rename = "ref")]
    ref_name: String,
    text: String,
    #[serde(default = "default_text_lang")]
    text_lang: String,
    #[serde(default = "default_speed")]
    speed: f64,
}

fn default_text_lang() -> String {
    "all_ja".to_string()
}
fn default_speed() -> f64 {
    1.0
}

/// 语音合成（登录后可用）：转发本地 GPT-SoVITS 的原子角色级端点 /hfc/synth。
async fn post_tts(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<TtsReq>,
) -> AppResult<Response> {
    require_verified_member(&state.pools.core, &user).await?;

    let text = req.text.trim();
    if text.is_empty() {
        return Err(AppError::bad_request("请输入要合成的文本"));
    }
    if text.chars().count() > TTS_TEXT_MAX {
        return Err(AppError::bad_request(format!(
            "文本过长（上限 {TTS_TEXT_MAX} 字）"
        )));
    }
    if req.character.trim().is_empty() || req.ref_name.trim().is_empty() {
        return Err(AppError::bad_request("请选择角色与参考语气"));
    }

    let cooldown = Duration::from_secs(state.cfg.voice_user_cooldown_secs);
    if let Err(wait) = state.voice.check_cooldown(user.id, cooldown) {
        return Err(AppError::TooManyRequests(format!(
            "操作太频繁，请 {wait} 秒后再试"
        )));
    }

    // 单并发 + 短排队：合成通常 10~60s，15s 内等不到就请用户稍后再来
    let _permit = tokio::time::timeout(TTS_QUEUE_WAIT, state.voice.tts_sem.acquire())
        .await
        .map_err(|_| AppError::TooManyRequests("合成通道正忙，请稍后再试".to_string()))?
        .map_err(|_| AppError::internal("semaphore closed"))?;

    let speed = req.speed.clamp(0.6, 1.65);
    let mut up = state
        .voice
        .client
        .post(format!("{}/hfc/synth", state.cfg.voice_tts_base))
        .timeout(Duration::from_secs(state.cfg.voice_tts_timeout_secs))
        .json(&json!({
            "character": req.character,
            "ref": req.ref_name,
            "text": text,
            "text_lang": req.text_lang,
            "speed": speed,
        }));
    if let Some(key) = &state.cfg.voice_shared_key {
        up = up.header("X-HFC-Voice-Key", key);
    }

    let resp = up.send().await.map_err(|e| {
        tracing::warn!("[语音工坊] TTS 上游不可达：{e}");
        state.voice.mark_offline(true);
        AppError::service_unavailable("语音合成服务暂时离线，请稍后再试")
    })?;

    let resp = check_upstream(resp, true).await?;
    state.voice.touch_cooldown(user.id, cooldown);
    stream_audio(resp)
}

/// 声线转换（登录后可用）：multipart 转发本地 RVC 的 /hfc/convert。
async fn post_rvc(
    State(state): State<AppState>,
    user: AuthUser,
    mut mp: Multipart,
) -> AppResult<Response> {
    require_verified_member(&state.pools.core, &user).await?;

    let mut role = String::new();
    let mut transpose: i32 = 0;
    let mut index_rate: f64 = 0.75;
    let mut protect: f64 = 0.33;
    let mut audio: Option<(Vec<u8>, String, String)> = None; // (bytes, filename, content_type)

    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("表单解析失败：{e}")))?
    {
        match field.name().unwrap_or("") {
            "role" => role = field.text().await.unwrap_or_default(),
            "transpose" => {
                transpose = field
                    .text()
                    .await
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0)
            }
            "indexRate" => {
                index_rate = field
                    .text()
                    .await
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.75)
            }
            "protect" => {
                protect = field
                    .text()
                    .await
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.33)
            }
            "audio" => {
                let filename = field.file_name().unwrap_or("input.wav").to_string();
                let content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("音频上传失败：{e}")))?;
                if bytes.len() > RVC_UPLOAD_MAX {
                    return Err(AppError::bad_request("音频文件过大（上限 50MB）"));
                }
                audio = Some((bytes.to_vec(), filename, content_type));
            }
            _ => {}
        }
    }

    let Some((bytes, filename, content_type)) = audio else {
        return Err(AppError::bad_request("请上传要转换的音频文件"));
    };
    if role.trim().is_empty() {
        return Err(AppError::bad_request("请选择目标角色"));
    }

    let cooldown = Duration::from_secs(state.cfg.voice_user_cooldown_secs);
    if let Err(wait) = state.voice.check_cooldown(user.id, cooldown) {
        return Err(AppError::TooManyRequests(format!(
            "操作太频繁，请 {wait} 秒后再试"
        )));
    }

    // 长任务不排队：占用中直接 429，避免连接长时间挂着
    let _permit = state
        .voice
        .rvc_sem
        .try_acquire()
        .map_err(|_| AppError::TooManyRequests("有转换任务正在进行，请稍后再试".to_string()))?;

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(filename)
        .mime_str(&content_type)
        .unwrap_or_else(|_| reqwest::multipart::Part::bytes(Vec::new()));
    let form = reqwest::multipart::Form::new()
        .text("role", role)
        .text("transpose", transpose.clamp(-24, 24).to_string())
        .text("index_rate", index_rate.clamp(0.0, 1.0).to_string())
        .text("protect", protect.clamp(0.0, 0.5).to_string())
        .part("audio", part);

    let mut up = state
        .voice
        .client
        .post(format!("{}/hfc/convert", state.cfg.voice_rvc_base))
        .timeout(Duration::from_secs(state.cfg.voice_rvc_timeout_secs))
        .multipart(form);
    if let Some(key) = &state.cfg.voice_shared_key {
        up = up.header("X-HFC-Voice-Key", key);
    }

    let resp = up.send().await.map_err(|e| {
        tracing::warn!("[语音工坊] RVC 上游不可达：{e}");
        state.voice.mark_offline(false);
        AppError::service_unavailable("声线转换服务暂时离线，请稍后再试")
    })?;

    let resp = check_upstream(resp, false).await?;
    state.voice.touch_cooldown(user.id, cooldown);
    stream_audio(resp)
}

/// 强制刷新角色缓存（voice 管理角色 / 超管）。characters.json、roles.json 变更后可即时生效。
async fn admin_refresh(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "voice", Action::Manage).await?;
    let data = refresh_roles(&state.voice, &state.cfg)
        .await
        .ok_or_else(|| AppError::service_unavailable("两个语音服务均不可达，无法刷新"))?;
    Ok(Json(json!({ "ok": true, "roles": *data })))
}

// ============================================================
//  上游通用处理
// ============================================================

/// 上游响应状态归类：2xx 放行；4xx 透传上游 message（429 单独映射）；5xx 记日志掩蔽。
async fn check_upstream(resp: reqwest::Response, is_tts: bool) -> AppResult<reqwest::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let which = if is_tts { "TTS" } else { "RVC" };
    let message = resp
        .json::<Value>()
        .await
        .ok()
        .and_then(|v| v.get("message").and_then(Value::as_str).map(str::to_string))
        .unwrap_or_else(|| format!("上游返回 {status}"));
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(AppError::TooManyRequests(message));
    }
    if status.is_client_error() {
        return Err(AppError::bad_request(message));
    }
    // 5xx：推理失败等，记录细节但不向用户暴露；服务进程还在（探活会维持在线）
    tracing::error!("[语音工坊] {which} 上游 {status}：{message}");
    Err(AppError::internal(format!("{which} 上游错误：{message}")))
}

/// 把上游音频响应流式转发给客户端（不整段读进内存）。
fn stream_audio(resp: reqwest::Response) -> AppResult<Response> {
    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/wav")
        .to_string();
    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "no-store");
    if let Some(len) = resp.content_length() {
        builder = builder.header(header::CONTENT_LENGTH, len);
    }
    builder
        .body(Body::from_stream(resp.bytes_stream()))
        .map_err(|e| AppError::internal(format!("构造响应失败：{e}")))
}

// ============================================================
//  探活与角色缓存
// ============================================================

/// 拉取并合并两端角色列表；单边失败沿用旧值，双边失败返回 None（保持旧缓存）。
async fn refresh_roles(voice: &VoiceState, cfg: &Config) -> Option<Arc<Value>> {
    let tts = fetch_hfc_json(voice, cfg, &format!("{}/hfc/roles", cfg.voice_tts_base))
        .await
        .and_then(|v| v.get("characters").cloned());
    let rvc = fetch_hfc_json(voice, cfg, &format!("{}/hfc/roles", cfg.voice_rvc_base))
        .await
        .and_then(|v| v.get("roles").cloned());
    if tts.is_none() && rvc.is_none() {
        return None;
    }

    // 单边失败时保留旧缓存里对应的一半，避免「一边掉线把另一边角色也清空」
    let prev = voice.roles.read().unwrap().data.clone();
    let prev_get = |key: &str| -> Value {
        prev.as_ref()
            .and_then(|p| p.get(key).cloned())
            .unwrap_or_else(|| json!([]))
    };
    let data = Arc::new(json!({
        "tts": tts.unwrap_or_else(|| prev_get("tts")),
        "rvc": rvc.unwrap_or_else(|| prev_get("rvc")),
    }));

    let mut cache = voice.roles.write().unwrap();
    cache.data = Some(data.clone());
    cache.fetched_at = Some(Instant::now());
    Some(data)
}

async fn fetch_hfc_json(voice: &VoiceState, cfg: &Config, url: &str) -> Option<Value> {
    let mut req = voice.client.get(url).timeout(Duration::from_secs(10));
    if let Some(key) = &cfg.voice_shared_key {
        req = req.header("X-HFC-Voice-Key", key);
    }
    let resp = req.send().await.ok()?;
    if !resp.status().is_success() {
        tracing::warn!("[语音工坊] 角色拉取失败 {url}：{}", resp.status());
        return None;
    }
    resp.json().await.ok()
}

/// 启动探活后台任务：每 interval 各 GET 一次 /healthz 写状态缓存；
/// 在线且角色缓存空/过期时顺带刷新角色列表。
pub fn spawn_probe(cfg: Arc<Config>, voice: VoiceState) {
    // 最短 10 秒，防误配成 0 空转打爆日志
    let interval = Duration::from_secs(cfg.voice_probe_interval_secs.max(10));
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        // 错过的 tick 不补跑（探活只关心当下）
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let tts_online = probe(&voice, &cfg.voice_tts_base).await;
            let rvc_online = probe(&voice, &cfg.voice_rvc_base).await;
            {
                let mut s = voice.status.write().unwrap();
                s.tts_online = tts_online;
                s.rvc_online = rvc_online;
                s.checked_at = Some(chrono::Utc::now().to_rfc3339());
            }
            if tts_online || rvc_online {
                let stale = {
                    let cache = voice.roles.read().unwrap();
                    cache
                        .fetched_at
                        .map(|t| t.elapsed() >= ROLES_TTL)
                        .unwrap_or(true)
                };
                if stale {
                    refresh_roles(&voice, &cfg).await;
                }
            }
        }
    });
}

async fn probe(voice: &VoiceState, base: &str) -> bool {
    voice
        .client
        .get(format!("{base}/healthz"))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
