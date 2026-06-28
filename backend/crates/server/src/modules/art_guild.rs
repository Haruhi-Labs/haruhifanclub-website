use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::{AppError, AppResult};
use serde_json::{json, Value};

use crate::state::AppState;

const RATINGS: &[(&str, i64, i64)] = &[
    ("F", 0, 0),
    ("E", 100, 1),
    ("D", 300, 2),
    ("C", 800, 4),
    ("B", 1500, 7),
    ("A", 3000, 12),
    ("S", 6000, 20),
    ("X", 12000, 35),
];

#[derive(sqlx::FromRow)]
struct GuildQuestListRow {
    id: i64,
    title: String,
    description: Option<String>,
    quest_type: String,
    difficulty: String,
    required_rating: String,
    required_access: String,
    condition_kind: String,
    target_count: i64,
    reward_reputation: i64,
    reward_coins: i64,
    deadline_hours: Option<i64>,
    status: String,
    sort_order: i64,
    claim_status: Option<String>,
    claim_progress: Option<i64>,
    claim_target_count: Option<i64>,
    claim_rewarded_at: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/guild/me", get(guild_me))
        .route("/guild/terminal", get(guild_terminal))
        .route("/guild/profile/{uid}", get(guild_profile))
        .route("/guild/profile/{uid}/artworks", get(guild_profile_artworks))
        .route("/guild/quests", get(guild_quests))
        .route("/guild/quests/{id}/claim", post(guild_claim_quest))
        .route("/guild/leaderboard", get(guild_leaderboard))
        .route("/guild/coins/history", get(guild_coin_history))
        .route("/guild/rating/apply", post(guild_apply_rating))
        .route("/guild/rewards", get(guild_rewards))
        .route("/guild/rewards/{id}/redeem", post(guild_redeem_reward))
        .route("/guild/redemptions/me", get(guild_my_redemptions))
        .route(
            "/admin/guild/quests",
            get(admin_guild_quests).post(admin_create_quest),
        )
        .route(
            "/admin/guild/quests/{id}",
            put(admin_update_quest).delete(admin_delete_quest),
        )
        .route("/admin/guild/quests/{id}/status", post(admin_quest_status))
        .route(
            "/admin/guild/rewards",
            get(admin_rewards).post(admin_create_reward),
        )
        .route(
            "/admin/guild/rewards/{id}",
            put(admin_update_reward).delete(admin_delete_reward),
        )
        .route(
            "/admin/guild/rewards/{id}/status",
            post(admin_reward_status),
        )
        .route("/admin/guild/rewards/{id}/stock", post(admin_reward_stock))
        .route("/admin/guild/redemptions", get(admin_redemptions))
        .route(
            "/admin/guild/redemptions/{id}/approve",
            post(admin_approve_redemption),
        )
        .route(
            "/admin/guild/redemptions/{id}/reject",
            post(admin_reject_redemption),
        )
        .route(
            "/admin/guild/redemptions/{id}/cancel",
            post(admin_cancel_redemption),
        )
        .route(
            "/admin/guild/redemptions/{id}/fulfilled",
            post(admin_fulfill_redemption),
        )
        .route(
            "/admin/guild/rating-applications",
            get(admin_rating_applications),
        )
        .route(
            "/admin/guild/rating-applications/{id}/approve",
            post(admin_approve_rating),
        )
        .route(
            "/admin/guild/rating-applications/{id}/reject",
            post(admin_reject_rating),
        )
        .route("/admin/guild/profiles", get(admin_profiles))
        .route(
            "/admin/guild/profiles/{uid}/access",
            post(admin_profile_access),
        )
}

pub async fn record_user_event(
    state: &AppState,
    user: Option<AuthUser>,
    event_kind: &str,
    target_id: Option<i64>,
) {
    let Some(user) = user else {
        return;
    };
    let uid = uid_for_user(user.id);
    if ensure_profile_for_user(state, &user).await.is_err() {
        return;
    }
    let now = now_iso();
    let insert = sqlx::query(
        "INSERT OR IGNORE INTO guild_quest_events(uid, event_kind, target_id, created_at)
         VALUES(?,?,?,?)",
    )
    .bind(&uid)
    .bind(event_kind)
    .bind(target_id)
    .bind(&now)
    .execute(&state.pools.art)
    .await;
    if insert.is_ok() {
        let _ = refresh_claims_for_uid(state, &uid).await;
    }
}

pub async fn grant_upload_progress(
    state: &AppState,
    uid: &str,
    artwork_id: i64,
    content_type: &str,
    source_type: &str,
    created_at: &str,
    manual_note_suffix: &str,
) {
    if uid.trim().is_empty() || source_type != "personal" {
        return;
    }

    // 金币即画廊积分：投稿积分已由 art.rs 主流程写入 points_ledger，此处不再重复发放金币，
    // 仅发放公会声望（评级体系的等级分，与积分相互独立）。
    let reputation = if content_type == "haruhi" { 120 } else { 30 };
    let note = if content_type == "haruhi" {
        format!("投稿凉宫个人作品奖励{manual_note_suffix}")
    } else {
        format!("投稿其他个人作品奖励{manual_note_suffix}")
    };

    let _ = grant_reputation(
        state,
        uid,
        reputation,
        None,
        Some(artwork_id),
        &note,
        "upload_artwork",
        created_at,
    )
    .await;
    let _ = refresh_claims_for_uid(state, uid).await;
}

pub async fn guild_summary_for_uid(state: &AppState, uid: &str) -> Value {
    let row: Option<(i64, String, String)> =
        sqlx::query_as("SELECT reputation, rating, access_tier FROM guild_profiles WHERE uid=?")
            .bind(uid)
            .fetch_optional(&state.pools.art)
            .await
            .ok()
            .flatten();
    match row {
        Some((reputation, rating, access_tier)) => {
            light_summary(uid, reputation, &rating, &access_tier)
        }
        None => light_summary(uid, 0, "F", "public_archive"),
    }
}

/// 批量取多个 uid 的公会徽章概要（列表用）：一次 IN 查询，不建档、不查积分/统计，消除 N+1。
pub async fn guild_summaries_for_uids(
    state: &AppState,
    uids: &[String],
) -> std::collections::HashMap<String, Value> {
    let mut map: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    let mut uniq: Vec<String> = uids
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    uniq.sort();
    uniq.dedup();
    if uniq.is_empty() {
        return map;
    }
    // 先以默认 F 档兜底（未建档作者也显示徽章，与逐行旧行为一致），再用实际档案覆盖。
    for u in &uniq {
        map.insert(u.clone(), light_summary(u, 0, "F", "public_archive"));
    }
    let placeholders = uniq.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT uid, reputation, rating, access_tier FROM guild_profiles WHERE uid IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, (String, i64, String, String)>(&sql);
    for u in &uniq {
        q = q.bind(u);
    }
    if let Ok(rows) = q.fetch_all(&state.pools.art).await {
        for (uid, reputation, rating, access_tier) in rows {
            map.insert(
                uid.clone(),
                light_summary(&uid, reputation, &rating, &access_tier),
            );
        }
    }
    map
}

fn light_summary(uid: &str, reputation: i64, rating: &str, access_tier: &str) -> Value {
    json!({
        "uid": uid,
        "reputation": reputation,
        "level": level_from_reputation(reputation),
        "rating": rating,
        "ratingLabel": rating_label(rating),
        "accessTier": access_tier,
        "accessLabel": access_label(access_tier),
        "accessShortLabel": access_short_label(access_tier),
        "badgeLabel": rating
    })
}

async fn guild_me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let uid = ensure_profile_for_user(&state, &user).await?;
    refresh_claims_for_uid(&state, &uid).await?;
    let profile = profile_value(&state, &uid, true).await?;
    Ok(Json(json!({ "ok": true, "profile": profile })))
}

async fn guild_terminal(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let uid = ensure_profile_for_user(&state, &user).await?;
    refresh_claims_for_uid(&state, &uid).await?;
    let profile = profile_value(&state, &uid, true).await?;
    let stats = artwork_stats(&state, &uid, true).await?;
    let artworks = artworks_for_uid(&state, &uid, true, 24).await?;
    let coins = coin_history_for_uid(&state, &uid, 50).await?;
    let reputation = reputation_history_for_uid(&state, &uid, 50).await?;
    let claims = claims_for_uid(&state, &uid).await?;
    let redemptions = redemptions_for_uid(&state, &uid).await?;
    let applications = applications_for_uid(&state, &uid).await?;
    let user_profile = user_profile_value(&state, user.id).await?;

    Ok(Json(json!({
        "ok": true,
        "profile": profile,
        "user": user_profile,
        "stats": stats,
        "artworks": artworks,
        "coinsHistory": coins,
        "reputationHistory": reputation,
        "claims": claims,
        "redemptions": redemptions,
        "ratingApplications": applications
    })))
}

async fn guild_profile(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> AppResult<Json<Value>> {
    // 公开资料：只读，不为任意被访问的 uid 建档（避免未登录 GET 污染 guild_profiles）。
    let uid = normalize_uid(&uid)?;
    let profile = profile_value(&state, &uid, false).await?;
    let stats = artwork_stats(&state, &uid, false).await?;
    let artworks = artworks_for_uid(&state, &uid, false, 18).await?;
    Ok(Json(
        json!({ "ok": true, "profile": profile, "stats": stats, "artworks": artworks }),
    ))
}

async fn guild_profile_artworks(
    State(state): State<AppState>,
    Path(uid): Path<String>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let uid = normalize_uid(&uid)?;
    let limit = clamp_query_i64(q.get("limit"), 6, 60, 24);
    let artworks = artworks_for_uid(&state, &uid, false, limit).await?;
    Ok(Json(json!({ "ok": true, "data": artworks })))
}

async fn guild_quests(
    State(state): State<AppState>,
    user: Option<AuthUser>,
) -> AppResult<Json<Value>> {
    let uid = match user {
        Some(user) => Some(ensure_profile_for_user(&state, &user).await?),
        None => None,
    };
    if let Some(uid) = &uid {
        refresh_claims_for_uid(&state, uid).await?;
    }
    let profile = match &uid {
        Some(uid) => Some(profile_value(&state, uid, true).await?),
        None => None,
    };

    let rows: Vec<GuildQuestListRow> = sqlx::query_as(
        "SELECT q.id, q.title, q.description, q.quest_type, q.difficulty,
                q.required_rating, q.required_access, q.condition_kind, q.target_count,
                q.reward_reputation, q.reward_coins, q.deadline_hours, q.status, q.sort_order,
                c.status AS claim_status, c.progress AS claim_progress,
                c.target_count AS claim_target_count, c.rewarded_at AS claim_rewarded_at
         FROM guild_quests q
         LEFT JOIN guild_quest_claims c ON c.quest_id=q.id AND c.uid=?
         WHERE q.status='active'
         ORDER BY q.sort_order ASC, q.id ASC",
    )
    .bind(uid.as_deref().unwrap_or(""))
    .fetch_all(&state.pools.art)
    .await?;

    let profile_rating = profile
        .as_ref()
        .and_then(|v| v.get("rating"))
        .and_then(|v| v.as_str())
        .unwrap_or("F");
    let profile_access = profile
        .as_ref()
        .and_then(|v| v.get("accessTier"))
        .and_then(|v| v.as_str())
        .unwrap_or("public_archive");

    let data: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let unlocked = uid.is_some()
                && rating_rank(profile_rating) >= rating_rank(&row.required_rating)
                && access_rank(profile_access) >= access_rank(&row.required_access);
            let required_access_label = access_label(&row.required_access);
            json!({
                "id": row.id,
                "title": row.title,
                "description": row.description,
                "questType": row.quest_type,
                "difficulty": row.difficulty,
                "requiredRating": row.required_rating,
                "requiredAccess": row.required_access,
                "requiredAccessLabel": required_access_label,
                "conditionKind": row.condition_kind,
                "targetCount": row.target_count,
                "rewardReputation": row.reward_reputation,
                "rewardCoins": row.reward_coins,
                "deadlineHours": row.deadline_hours,
                "status": row.status,
                "sortOrder": row.sort_order,
                "unlocked": unlocked,
                "claim": row.claim_status.map(|s| json!({
                    "status": s,
                    "progress": row.claim_progress.unwrap_or(0),
                    "targetCount": row.claim_target_count.unwrap_or(row.target_count),
                    "rewarded": row.claim_rewarded_at.is_some()
                }))
            })
        })
        .collect();

    Ok(Json(
        json!({ "ok": true, "profile": profile, "data": data }),
    ))
}

async fn guild_claim_quest(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    let uid = ensure_profile_for_user(&state, &user).await?;
    let quest: Option<(String, String, i64)> = sqlx::query_as(
        "SELECT required_rating, required_access, target_count
         FROM guild_quests WHERE id=? AND status='active'",
    )
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let Some((required_rating, required_access, target_count)) = quest else {
        return Err(AppError::bad_request("委托不存在或已下架"));
    };
    let profile = profile_value(&state, &uid, true).await?;
    let rating = profile
        .get("rating")
        .and_then(|v| v.as_str())
        .unwrap_or("F");
    let access = profile
        .get("accessTier")
        .and_then(|v| v.as_str())
        .unwrap_or("observer_clearance");
    if rating_rank(rating) < rating_rank(&required_rating)
        || access_rank(access) < access_rank(&required_access)
    {
        return Err(AppError::bad_request("评级或访问许可不足"));
    }
    let now = now_iso();
    sqlx::query(
        "INSERT OR IGNORE INTO guild_quest_claims(quest_id, uid, status, progress, target_count, claimed_at)
         VALUES(?,?,?,?,?,?)",
    )
    .bind(id)
    .bind(&uid)
    .bind("active")
    .bind(0_i64)
    .bind(target_count)
    .bind(&now)
    .execute(&state.pools.art)
    .await?;
    refresh_claims_for_uid(&state, &uid).await?;
    Ok(Json(json!({ "ok": true })))
}

async fn guild_leaderboard(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<(
        String,
        Option<i64>,
        String,
        String,
        Option<i64>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT gp.uid, gp.reputation, gp.rating, gp.access_tier,
                COALESCE(SUM(pl.points), 0) AS coins, c.avatar_url
         FROM guild_profiles gp
         LEFT JOIN points_ledger pl ON pl.uid=gp.uid
         LEFT JOIN creators c ON c.uid=gp.uid
         GROUP BY gp.uid
         ORDER BY
           CASE gp.rating
             WHEN 'X' THEN 8 WHEN 'S' THEN 7 WHEN 'A' THEN 6 WHEN 'B' THEN 5
             WHEN 'C' THEN 4 WHEN 'D' THEN 3 WHEN 'E' THEN 2 ELSE 1 END DESC,
           CAST((gp.reputation / 100) AS INTEGER) DESC,
           coins DESC,
           gp.reputation DESC
         LIMIT 50",
    )
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|(uid, reputation, rating, access, coins, avatar)| {
            let rep = reputation.unwrap_or(0);
            json!({
                "uid": uid,
                "reputation": rep,
                "level": level_from_reputation(rep),
                "rating": rating,
                "accessTier": access,
                "accessLabel": access_label(&access),
                "coins": coins.unwrap_or(0),
                "avatar_url": avatar
            })
        })
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn guild_coin_history(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let uid = ensure_profile_for_user(&state, &user).await?;
    let history = coin_history_for_uid(&state, &uid, 50).await?;
    let summary = coin_summary(&state, &uid).await?;
    Ok(Json(
        json!({ "ok": true, "summary": summary, "history": history }),
    ))
}

async fn guild_apply_rating(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    let uid = ensure_profile_for_user(&state, &user).await?;
    let target = body
        .get("targetRating")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_uppercase();
    if !RATINGS.iter().any(|(r, _, _)| *r == target) {
        return Err(AppError::bad_request("评级无效"));
    }
    let profile = profile_value(&state, &uid, true).await?;
    let current = profile
        .get("rating")
        .and_then(|v| v.as_str())
        .unwrap_or("F");
    if rating_rank(&target) <= rating_rank(current) {
        return Err(AppError::bad_request("只能申请更高评级"));
    }
    let rep = profile
        .get("reputation")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let haruhi_count = approved_haruhi_personal_count(&state, &uid).await?;
    if !rating_requirements_met(&target, rep, haruhi_count) {
        return Err(AppError::bad_request("尚未满足评级申请条件"));
    }
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM guild_rating_applications WHERE uid=? AND status='pending'",
    )
    .bind(&uid)
    .fetch_optional(&state.pools.art)
    .await?;
    if exists.is_some() {
        return Err(AppError::bad_request("已有待审核评级申请"));
    }
    let now = now_iso();
    sqlx::query(
        "INSERT INTO guild_rating_applications(uid, from_rating, target_rating,
         reputation_snapshot, haruhi_count_snapshot, status, user_note, created_at)
         VALUES(?,?,?,?,?,?,?,?)",
    )
    .bind(&uid)
    .bind(current)
    .bind(&target)
    .bind(rep)
    .bind(haruhi_count)
    .bind("pending")
    .bind(body.get("note").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(&now)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn guild_rewards(
    State(state): State<AppState>,
    user: Option<AuthUser>,
) -> AppResult<Json<Value>> {
    let uid = match user {
        Some(user) => Some(ensure_profile_for_user(&state, &user).await?),
        None => None,
    };
    let profile = match &uid {
        Some(uid) => Some(profile_value(&state, uid, true).await?),
        None => None,
    };
    let rating = profile
        .as_ref()
        .and_then(|v| v.get("rating"))
        .and_then(|v| v.as_str())
        .unwrap_or("F");
    let access = profile
        .as_ref()
        .and_then(|v| v.get("accessTier"))
        .and_then(|v| v.as_str())
        .unwrap_or("public_archive");
    let rows = reward_rows(&state, "WHERE status='active'").await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|r| reward_value(r, uid.is_some(), rating, access))
        .collect();
    Ok(Json(
        json!({ "ok": true, "profile": profile, "data": data }),
    ))
}

async fn guild_redeem_reward(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    let uid = ensure_profile_for_user(&state, &user).await?;
    let reward: Option<(String, i64, Option<i64>, String, String, String)> = sqlx::query_as(
        "SELECT name, price_coins, stock, required_rating, required_access, status
         FROM guild_rewards WHERE id=?",
    )
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let Some((name, price, stock, required_rating, required_access, status)) = reward else {
        return Err(AppError::bad_request("奖励不存在"));
    };
    if status != "active" {
        return Err(AppError::bad_request("奖励已下架"));
    }
    let profile = profile_value(&state, &uid, true).await?;
    let rating = profile
        .get("rating")
        .and_then(|v| v.as_str())
        .unwrap_or("F");
    let access = profile
        .get("accessTier")
        .and_then(|v| v.as_str())
        .unwrap_or("observer_clearance");
    if rating_rank(rating) < rating_rank(&required_rating)
        || access_rank(access) < access_rank(&required_access)
    {
        return Err(AppError::bad_request("评级或访问许可不足"));
    }
    // 原子兑换临界区：BEGIN IMMEDIATE 串行化「复核库存 + 复核可用积分 + 写入冻结记录」，
    // 防并发兑换超扣/超卖（与 main 的 news prizes 原子兑换思路一致）。
    let note = body
        .get("note")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let now = now_iso();
    let mut conn = state.pools.art.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    match redeem_in_tx(&mut conn, id, &uid, price, stock, &note, &now).await {
        // conn 解引用到 &mut SqliteConnection（deref coercion）
        Ok(()) => {
            sqlx::query("COMMIT").execute(&mut *conn).await?;
            Ok(Json(
                json!({ "ok": true, "message": format!("已提交「{name}」兑换申请") }),
            ))
        }
        Err(e) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            Err(e)
        }
    }
}

/// 兑换事务体：在已开启的 BEGIN IMMEDIATE 写事务内复核库存与可用积分，并写入 pending 冻结记录。
async fn redeem_in_tx(
    conn: &mut sqlx::SqliteConnection,
    reward_id: i64,
    uid: &str,
    price: i64,
    stock: Option<i64>,
    note: &str,
    now: &str,
) -> AppResult<()> {
    if let Some(stock) = stock {
        if stock >= 0 {
            let pending: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM guild_reward_redemptions WHERE reward_id=? AND status='pending'",
            )
            .bind(reward_id)
            .fetch_one(&mut *conn)
            .await?;
            if stock - pending <= 0 {
                return Err(AppError::bad_request("库存不足"));
            }
        }
    }
    // 可用积分 = 总积分(points_ledger) - 已冻结(pending 兑换)
    let total: Option<i64> =
        sqlx::query_scalar("SELECT SUM(points) FROM points_ledger WHERE uid=?")
            .bind(uid)
            .fetch_one(&mut *conn)
            .await?;
    let frozen: Option<i64> = sqlx::query_scalar(
        "SELECT SUM(frozen_coins) FROM guild_reward_redemptions WHERE uid=? AND status='pending'",
    )
    .bind(uid)
    .fetch_one(&mut *conn)
    .await?;
    if total.unwrap_or(0) - frozen.unwrap_or(0) < price {
        return Err(AppError::bad_request("金币不足"));
    }
    sqlx::query(
        "INSERT INTO guild_reward_redemptions(reward_id, uid, frozen_coins, status, user_note, created_at)
         VALUES(?,?,?,?,?,?)",
    )
    .bind(reward_id)
    .bind(uid)
    .bind(price)
    .bind("pending")
    .bind(note)
    .bind(now)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn guild_my_redemptions(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let uid = ensure_profile_for_user(&state, &user).await?;
    Ok(Json(
        json!({ "ok": true, "data": redemptions_for_uid(&state, &uid).await? }),
    ))
}

async fn admin_guild_quests(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let data = admin_quest_rows(&state).await?;
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_create_quest(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let now = now_iso();
    sqlx::query(
        "INSERT INTO guild_quests(title, description, quest_type, difficulty, required_rating,
         required_access, condition_kind, target_count, reward_reputation, reward_coins,
         deadline_hours, status, sort_order, created_at, updated_at)
         VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(str_field(&body, "title", "未命名委托"))
    .bind(str_field(&body, "description", ""))
    .bind(str_field(&body, "questType", "daily"))
    .bind(str_field(&body, "difficulty", "normal"))
    .bind(str_field(&body, "requiredRating", "F"))
    .bind(str_field(&body, "requiredAccess", "observer_clearance"))
    .bind(str_field(&body, "conditionKind", "browse_artworks"))
    .bind(int_field(&body, "targetCount", 1))
    .bind(int_field(&body, "rewardReputation", 0))
    .bind(int_field(&body, "rewardCoins", 0))
    .bind(body.get("deadlineHours").and_then(json_num_i64))
    .bind(str_field(&body, "status", "active"))
    .bind(int_field(&body, "sortOrder", 100))
    .bind(&now)
    .bind(&now)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_update_quest(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let now = now_iso();
    sqlx::query(
        "UPDATE guild_quests SET title=?, description=?, quest_type=?, difficulty=?,
         required_rating=?, required_access=?, condition_kind=?, target_count=?,
         reward_reputation=?, reward_coins=?, deadline_hours=?, status=?, sort_order=?, updated_at=?
         WHERE id=?",
    )
    .bind(str_field(&body, "title", "未命名委托"))
    .bind(str_field(&body, "description", ""))
    .bind(str_field(&body, "questType", "daily"))
    .bind(str_field(&body, "difficulty", "normal"))
    .bind(str_field(&body, "requiredRating", "F"))
    .bind(str_field(&body, "requiredAccess", "observer_clearance"))
    .bind(str_field(&body, "conditionKind", "browse_artworks"))
    .bind(int_field(&body, "targetCount", 1))
    .bind(int_field(&body, "rewardReputation", 0))
    .bind(int_field(&body, "rewardCoins", 0))
    .bind(body.get("deadlineHours").and_then(json_num_i64))
    .bind(str_field(&body, "status", "active"))
    .bind(int_field(&body, "sortOrder", 100))
    .bind(&now)
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_delete_quest(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query("UPDATE guild_quests SET status='deleted', updated_at=? WHERE id=?")
        .bind(now_iso())
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_quest_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query("UPDATE guild_quests SET status=?, updated_at=? WHERE id=?")
        .bind(str_field(&body, "status", "active"))
        .bind(now_iso())
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_rewards(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows = reward_rows(&state, "").await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|r| reward_value(r, true, "X", "closed_space"))
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_create_reward(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let now = now_iso();
    sqlx::query(
        "INSERT INTO guild_rewards(name, description, reward_type, price_coins, stock,
         required_rating, required_access, image_url, status, sort_order, created_at, updated_at)
         VALUES(?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(str_field(&body, "name", "未命名补给"))
    .bind(str_field(&body, "description", ""))
    .bind(str_field(&body, "rewardType", "virtual"))
    .bind(int_field(&body, "priceCoins", 0))
    .bind(body.get("stock").and_then(json_num_i64))
    .bind(str_field(&body, "requiredRating", "F"))
    .bind(str_field(&body, "requiredAccess", "observer_clearance"))
    .bind(str_field(&body, "imageUrl", ""))
    .bind(str_field(&body, "status", "active"))
    .bind(int_field(&body, "sortOrder", 100))
    .bind(&now)
    .bind(&now)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_update_reward(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let now = now_iso();
    sqlx::query(
        "UPDATE guild_rewards SET name=?, description=?, reward_type=?, price_coins=?, stock=?,
         required_rating=?, required_access=?, image_url=?, status=?, sort_order=?, updated_at=?
         WHERE id=?",
    )
    .bind(str_field(&body, "name", "未命名补给"))
    .bind(str_field(&body, "description", ""))
    .bind(str_field(&body, "rewardType", "virtual"))
    .bind(int_field(&body, "priceCoins", 0))
    .bind(body.get("stock").and_then(json_num_i64))
    .bind(str_field(&body, "requiredRating", "F"))
    .bind(str_field(&body, "requiredAccess", "observer_clearance"))
    .bind(str_field(&body, "imageUrl", ""))
    .bind(str_field(&body, "status", "active"))
    .bind(int_field(&body, "sortOrder", 100))
    .bind(&now)
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_delete_reward(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query("UPDATE guild_rewards SET status='deleted', updated_at=? WHERE id=?")
        .bind(now_iso())
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_reward_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query("UPDATE guild_rewards SET status=?, updated_at=? WHERE id=?")
        .bind(str_field(&body, "status", "active"))
        .bind(now_iso())
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_reward_stock(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query("UPDATE guild_rewards SET stock=?, updated_at=? WHERE id=?")
        .bind(body.get("stock").and_then(json_num_i64))
        .bind(now_iso())
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_redemptions(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let data = all_redemptions(&state).await?;
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_approve_redemption(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let row: Option<(i64, String, i64, String)> = sqlx::query_as(
        "SELECT r.reward_id, r.uid, r.frozen_coins, gr.name
         FROM guild_reward_redemptions r JOIN guild_rewards gr ON gr.id=r.reward_id
         WHERE r.id=? AND r.status='pending'",
    )
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let Some((reward_id, uid, frozen_coins, reward_name)) = row else {
        return Err(AppError::bad_request("兑换申请不存在或状态不可批准"));
    };
    let now = now_iso();
    // 先原子置为 approved（条件更新），只有抢到的请求才扣积分、减库存，防并发双批准导致双扣。
    let affected = sqlx::query(
        "UPDATE guild_reward_redemptions SET status='approved', admin_note=?, reviewed_at=? WHERE id=? AND status='pending'",
    )
    .bind(body.get("note").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(&now)
    .bind(id)
    .execute(&state.pools.art)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::bad_request("兑换申请不存在或状态不可批准"));
    }
    grant_coins(
        &state,
        &uid,
        -frozen_coins,
        None,
        &format!("兑换「{reward_name}」扣除金币"),
        "redemption",
        &now,
    )
    .await?;
    sqlx::query(
        "UPDATE guild_rewards SET stock=CASE WHEN stock IS NULL OR stock < 0 THEN stock ELSE MAX(stock-1, 0) END,
         updated_at=? WHERE id=?",
    )
    .bind(&now)
    .bind(reward_id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_reject_redemption(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    set_redemption_status(&state, &user, id, "rejected", body).await
}

async fn admin_cancel_redemption(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    set_redemption_status(&state, &user, id, "cancelled", body).await
}

async fn admin_fulfill_redemption(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let now = now_iso();
    sqlx::query(
        "UPDATE guild_reward_redemptions SET status='fulfilled', admin_note=?, fulfilled_at=? WHERE id=? AND status IN ('approved','fulfilled')",
    )
    .bind(body.get("note").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(&now)
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_rating_applications(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows: Vec<(
        i64,
        String,
        Option<String>,
        String,
        i64,
        i64,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, uid, from_rating, target_rating, reputation_snapshot, haruhi_count_snapshot,
                    status, user_note, admin_note, created_at, reviewed_at
             FROM guild_rating_applications ORDER BY datetime(created_at) DESC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                id,
                uid,
                from_rating,
                target_rating,
                rep,
                haruhi_count,
                status,
                user_note,
                admin_note,
                created_at,
                reviewed_at,
            )| {
                json!({
                    "id": id,
                    "uid": uid,
                    "fromRating": from_rating,
                    "targetRating": target_rating,
                    "reputationSnapshot": rep,
                    "haruhiCountSnapshot": haruhi_count,
                    "status": status,
                    "userNote": user_note,
                    "adminNote": admin_note,
                    "createdAt": created_at,
                    "reviewedAt": reviewed_at
                })
            },
        )
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_approve_rating(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT uid, target_rating FROM guild_rating_applications WHERE id=? AND status='pending'",
    )
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let Some((uid, target_rating)) = row else {
        return Err(AppError::bad_request("评级申请不存在或已审核"));
    };
    let now = now_iso();
    sqlx::query("UPDATE guild_profiles SET rating=?, updated_at=? WHERE uid=?")
        .bind(&target_rating)
        .bind(&now)
        .bind(&uid)
        .execute(&state.pools.art)
        .await?;
    sqlx::query(
        "UPDATE guild_rating_applications SET status='approved', admin_note=?, reviewed_at=? WHERE id=?",
    )
    .bind(body.get("note").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(&now)
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_reject_rating(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query(
        "UPDATE guild_rating_applications SET status='rejected', admin_note=?, reviewed_at=? WHERE id=? AND status='pending'",
    )
    .bind(body.get("note").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(now_iso())
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_profiles(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows: Vec<(String, Option<i64>, i64, String, String, Option<i64>)> = sqlx::query_as(
        "SELECT gp.uid, gp.user_id, gp.reputation, gp.rating, gp.access_tier,
                COALESCE(SUM(pl.points), 0) AS coins
         FROM guild_profiles gp
         LEFT JOIN points_ledger pl ON pl.uid=gp.uid
         GROUP BY gp.uid
         ORDER BY gp.updated_at DESC, gp.uid ASC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|(uid, user_id, reputation, rating, access, coins)| {
            json!({
                "uid": uid,
                "userId": user_id,
                "reputation": reputation,
                "level": level_from_reputation(reputation),
                "rating": rating,
                "accessTier": access,
                "accessLabel": access_label(&access),
                "coins": coins.unwrap_or(0)
            })
        })
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_profile_access(
    State(state): State<AppState>,
    user: AuthUser,
    Path(uid): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let uid = normalize_uid(&uid)?;
    let access = str_field(&body, "accessTier", "observer_clearance");
    if access_rank(&access) < 0 {
        return Err(AppError::bad_request("访问许可无效"));
    }
    ensure_profile_for_uid(&state, &uid).await?;
    sqlx::query("UPDATE guild_profiles SET access_tier=?, updated_at=? WHERE uid=?")
        .bind(access)
        .bind(now_iso())
        .bind(uid)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn ensure_profile_for_user(state: &AppState, user: &AuthUser) -> AppResult<String> {
    let uid = uid_for_user(user.id);
    let now = now_iso();
    sqlx::query("INSERT OR IGNORE INTO creators(uid, avatar_url, created_at) VALUES(?,'',?)")
        .bind(&uid)
        .bind(&now)
        .execute(&state.pools.art)
        .await?;
    sqlx::query(
        "INSERT OR IGNORE INTO guild_profiles(uid, user_id, reputation, rating, access_tier, created_at, updated_at)
         VALUES(?,?,0,'F','observer_clearance',?,?)",
    )
    .bind(&uid)
    .bind(user.id)
    .bind(&now)
    .bind(&now)
    .execute(&state.pools.art)
    .await?;
    sqlx::query(
        "UPDATE guild_profiles SET user_id=COALESCE(user_id, ?), updated_at=updated_at WHERE uid=?",
    )
    .bind(user.id)
    .bind(&uid)
    .execute(&state.pools.art)
    .await?;
    Ok(uid)
}

async fn ensure_profile_for_uid(state: &AppState, uid: &str) -> AppResult<()> {
    let uid = normalize_uid(uid)?;
    let now = now_iso();
    let user_id = user_id_from_uid(&uid);
    sqlx::query(
        "INSERT OR IGNORE INTO guild_profiles(uid, user_id, reputation, rating, access_tier, created_at, updated_at)
         VALUES(?,?,0,'F','observer_clearance',?,?)",
    )
    .bind(&uid)
    .bind(user_id)
    .bind(&now)
    .bind(&now)
    .execute(&state.pools.art)
    .await?;
    Ok(())
}

async fn profile_value(state: &AppState, uid: &str, private: bool) -> AppResult<Value> {
    // 只读：不为被查询的 uid 建档；查不到则回落默认档（避免公开 GET 污染 guild_profiles）。
    let row: Option<(
        Option<i64>,
        i64,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT gp.user_id, gp.reputation, gp.rating, gp.access_tier, c.avatar_url, c.created_at, c.qq
             FROM guild_profiles gp LEFT JOIN creators c ON c.uid=gp.uid WHERE gp.uid=?",
    )
    .bind(uid)
    .fetch_optional(&state.pools.art)
    .await?;
    let (user_id, reputation, rating, access_tier, avatar_url, creator_created_at, qq) = match row {
        Some(r) => r,
        None => (
            user_id_from_uid(uid),
            0,
            "F".to_string(),
            "public_archive".to_string(),
            None,
            None,
            None,
        ),
    };
    let coins = coin_summary(state, uid).await?;
    let haruhi_count = approved_haruhi_personal_count(state, uid).await?;
    let next_rating = next_rating(&rating, reputation, haruhi_count);
    let user_profile = match (private, user_id) {
        (true, Some(id)) => Some(user_profile_value(state, id).await?),
        _ => None,
    };
    Ok(json!({
        "uid": uid,
        "userId": user_id,
        "avatar_url": avatar_url,
        "creatorCreatedAt": creator_created_at,
        "qq": if private { qq } else { None::<String> },
        "reputation": reputation,
        "level": level_from_reputation(reputation),
        "rating": rating,
        "ratingLabel": rating_label(&rating),
        "accessTier": access_tier,
        "accessLabel": access_label(&access_tier),
        "accessShortLabel": access_short_label(&access_tier),
        "badgeLabel": rating,
        "coins": coins,
        "haruhiPersonalCount": haruhi_count,
        "nextRating": next_rating,
        "user": user_profile
    }))
}

async fn user_profile_value(state: &AppState, user_id: i64) -> AppResult<Value> {
    // 显示名以权威的 nickname 为准（#19 起昵称必填唯一；display_name 是注册时冻结的旧列，
    // 改昵称不会同步，故不再用它）；老账号 nickname 为空时回落 username。
    let row: Option<(String, Option<String>, Option<String>, String)> =
        sqlx::query_as("SELECT username, nickname, email, created_at FROM users WHERE id=?")
            .bind(user_id)
            .fetch_optional(&state.pools.core)
            .await?;
    let Some((username, nickname, email, created_at)) = row else {
        return Ok(json!({ "id": user_id }));
    };
    let display_name = nickname
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| username.clone());
    Ok(json!({
        "id": user_id,
        "username": username,
        "displayName": display_name,
        "email": email,
        "createdAt": created_at
    }))
}

async fn coin_summary(state: &AppState, uid: &str) -> AppResult<Value> {
    // 金币即画廊积分：余额从 points_ledger 计；冻结为 pending 兑换占用。
    let total: Option<i64> =
        sqlx::query_scalar("SELECT SUM(points) FROM points_ledger WHERE uid=?")
            .bind(uid)
            .fetch_one(&state.pools.art)
            .await?;
    let frozen: Option<i64> = sqlx::query_scalar(
        "SELECT SUM(frozen_coins) FROM guild_reward_redemptions WHERE uid=? AND status='pending'",
    )
    .bind(uid)
    .fetch_one(&state.pools.art)
    .await?;
    let total = total.unwrap_or(0);
    let frozen = frozen.unwrap_or(0);
    Ok(json!({
        "total": total,
        "frozen": frozen,
        "available": total - frozen
    }))
}

async fn grant_coins(
    state: &AppState,
    uid: &str,
    coins: i64,
    artwork_id: Option<i64>,
    note: &str,
    _source_type: &str,
    created_at: &str,
) -> AppResult<()> {
    if coins == 0 {
        return Ok(());
    }
    // 金币即画廊积分，统一写入 points_ledger（points_ledger 无 source_type 列，来源以 note 表达）。
    sqlx::query(
        "INSERT INTO points_ledger(uid, artwork_id, points, note, created_at, granted_at)
         VALUES(?,?,?,?,?,?)",
    )
    .bind(uid)
    .bind(artwork_id)
    .bind(coins)
    .bind(note)
    .bind(created_at)
    .bind(now_iso())
    .execute(&state.pools.art)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn grant_reputation(
    state: &AppState,
    uid: &str,
    reputation: i64,
    quest_id: Option<i64>,
    artwork_id: Option<i64>,
    note: &str,
    source_type: &str,
    created_at: &str,
) -> AppResult<()> {
    if reputation == 0 {
        return Ok(());
    }
    ensure_profile_for_uid(state, uid).await?;
    sqlx::query(
        "INSERT INTO reputation_ledger(uid, quest_id, artwork_id, reputation, note, source_type, created_at)
         VALUES(?,?,?,?,?,?,?)",
    )
    .bind(uid)
    .bind(quest_id)
    .bind(artwork_id)
    .bind(reputation)
    .bind(note)
    .bind(source_type)
    .bind(created_at)
    .execute(&state.pools.art)
    .await?;
    sqlx::query("UPDATE guild_profiles SET reputation=reputation+?, updated_at=? WHERE uid=?")
        .bind(reputation)
        .bind(now_iso())
        .bind(uid)
        .execute(&state.pools.art)
        .await?;
    Ok(())
}

async fn refresh_claims_for_uid(state: &AppState, uid: &str) -> AppResult<()> {
    let rows: Vec<(i64, i64, String, i64, i64, i64, String, String)> = sqlx::query_as(
        "SELECT c.id, q.id, q.condition_kind, q.target_count, q.reward_reputation, q.reward_coins,
                c.claimed_at, q.title
         FROM guild_quest_claims c JOIN guild_quests q ON q.id=c.quest_id
         WHERE c.uid=? AND c.status='active'",
    )
    .bind(uid)
    .fetch_all(&state.pools.art)
    .await?;
    for (
        claim_id,
        quest_id,
        condition_kind,
        target_count,
        reward_rep,
        reward_coins,
        claimed_at,
        title,
    ) in rows
    {
        let progress = quest_progress(state, uid, &condition_kind, &claimed_at).await?;
        let progress = progress.min(target_count);
        if progress >= target_count {
            let now = now_iso();
            let affected = sqlx::query(
                "UPDATE guild_quest_claims SET status='completed', progress=?, completed_at=?, rewarded_at=?
                 WHERE id=? AND status='active'",
            )
            .bind(progress)
            .bind(&now)
            .bind(&now)
            .bind(claim_id)
            .execute(&state.pools.art)
            .await?
            .rows_affected();
            if affected > 0 {
                let note = format!("完成委托「{title}」奖励");
                grant_reputation(
                    state,
                    uid,
                    reward_rep,
                    Some(quest_id),
                    None,
                    &note,
                    "quest",
                    &now,
                )
                .await?;
                grant_coins(state, uid, reward_coins, None, &note, "quest", &now).await?;
            }
        } else {
            sqlx::query("UPDATE guild_quest_claims SET progress=? WHERE id=?")
                .bind(progress)
                .bind(claim_id)
                .execute(&state.pools.art)
                .await?;
        }
    }
    Ok(())
}

async fn quest_progress(
    state: &AppState,
    uid: &str,
    condition_kind: &str,
    claimed_at: &str,
) -> AppResult<i64> {
    let sql = match condition_kind {
        "upload_personal_haruhi" => {
            "SELECT COUNT(*) FROM artworks WHERE uploader_uid=? AND source_type='personal' AND content_type='haruhi' AND status='approved' AND datetime(COALESCE(reviewed_at, created_at)) >= datetime(?)"
        }
        "upload_personal_any" => {
            "SELECT COUNT(*) FROM artworks WHERE uploader_uid=? AND source_type='personal' AND status='approved' AND datetime(COALESCE(reviewed_at, created_at)) >= datetime(?)"
        }
        "upload_network" => {
            "SELECT COUNT(*) FROM artworks WHERE uploader_uid=? AND source_type='network' AND status='approved' AND datetime(COALESCE(reviewed_at, created_at)) >= datetime(?)"
        }
        "browse_artworks" => {
            "SELECT COUNT(*) FROM guild_quest_events WHERE uid=? AND event_kind='browse_artwork' AND datetime(created_at) >= datetime(?)"
        }
        "like_artworks" => {
            "SELECT COUNT(*) FROM guild_quest_events WHERE uid=? AND event_kind='like_artwork' AND datetime(created_at) >= datetime(?)"
        }
        "comment_artworks" => {
            "SELECT COUNT(*) FROM guild_quest_events WHERE uid=? AND event_kind='comment_artwork' AND datetime(created_at) >= datetime(?)"
        }
        _ => "SELECT 0",
    };
    if condition_kind == "manual_admin_verify" {
        return Ok(0);
    }
    let count: i64 = if sql == "SELECT 0" {
        0
    } else {
        sqlx::query_scalar(sql)
            .bind(uid)
            .bind(claimed_at)
            .fetch_one(&state.pools.art)
            .await?
    };
    Ok(count)
}

async fn artwork_stats(state: &AppState, uid: &str, include_private: bool) -> AppResult<Value> {
    let status_clause = if include_private {
        ""
    } else {
        "AND status='approved'"
    };
    let sql = format!(
        "SELECT
            COUNT(*) AS total,
            SUM(CASE WHEN status='approved' THEN 1 ELSE 0 END) AS approved,
            SUM(CASE WHEN status='pending' THEN 1 ELSE 0 END) AS pending,
            SUM(CASE WHEN status IN ('rejected','hidden','flagged') THEN 1 ELSE 0 END) AS restricted,
            SUM(CASE WHEN source_type='personal' THEN 1 ELSE 0 END) AS personal,
            SUM(CASE WHEN source_type='network' THEN 1 ELSE 0 END) AS network,
            SUM(CASE WHEN content_type='haruhi' THEN 1 ELSE 0 END) AS haruhi,
            SUM(CASE WHEN content_type='other' THEN 1 ELSE 0 END) AS other,
            MIN(created_at) AS first_upload,
            MAX(created_at) AS latest_upload
         FROM artworks WHERE uploader_uid=? {status_clause}"
    );
    let row: (
        i64,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as(&sql)
        .bind(uid)
        .fetch_one(&state.pools.art)
        .await?;
    Ok(json!({
        "total": row.0,
        "approved": row.1.unwrap_or(0),
        "pending": row.2.unwrap_or(0),
        "restricted": row.3.unwrap_or(0),
        "personal": row.4.unwrap_or(0),
        "network": row.5.unwrap_or(0),
        "haruhi": row.6.unwrap_or(0),
        "other": row.7.unwrap_or(0),
        "firstUploadAt": row.8,
        "latestUploadAt": row.9
    }))
}

async fn artworks_for_uid(
    state: &AppState,
    uid: &str,
    include_private: bool,
    limit: i64,
) -> AppResult<Vec<Value>> {
    let status_clause = if include_private {
        "status IN ('approved','pending','rejected','hidden','flagged')"
    } else {
        "status='approved'"
    };
    let sql = format!(
        "SELECT id, title, uploader_name, uploader_uid, source_type, content_type,
                file_path, images_json, status, like_total, created_at
         FROM artworks WHERE uploader_uid=? AND {status_clause}
         ORDER BY datetime(created_at) DESC, id DESC LIMIT ?"
    );
    let rows: Vec<(
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<i64>,
        Option<String>,
    )> = sqlx::query_as(&sql)
        .bind(uid)
        .bind(limit)
        .fetch_all(&state.pools.art)
        .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                title,
                uploader_name,
                uploader_uid,
                source_type,
                content_type,
                file_path,
                images_json,
                status,
                like_total,
                created_at,
            )| {
                json!({
                    "id": id,
                    "title": title,
                    "uploader_name": uploader_name,
                    "uploader_uid": uploader_uid,
                    "source_type": source_type,
                    "content_type": content_type,
                    "image_url": artwork_image_url(file_path.as_deref(), images_json.as_deref()),
                    "status": status,
                    "like_total": like_total.unwrap_or(0),
                    "created_at": created_at
                })
            },
        )
        .collect())
}

async fn coin_history_for_uid(state: &AppState, uid: &str, limit: i64) -> AppResult<Vec<Value>> {
    // 金币流水即画廊积分流水（points_ledger）；该表无 source_type 列，来源以 note 表达。
    let rows: Vec<(
        Option<i64>,
        Option<i64>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT points, artwork_id, note, created_at, granted_at
             FROM points_ledger WHERE uid=? ORDER BY datetime(created_at) DESC, id DESC LIMIT ?",
    )
    .bind(uid)
    .bind(limit)
    .fetch_all(&state.pools.art)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(points, artwork_id, note, created_at, granted_at)| {
            json!({
                "coins": points.unwrap_or(0),
                "artworkId": artwork_id,
                "note": note,
                "sourceType": Value::Null,
                "createdAt": created_at,
                "grantedAt": granted_at
            })
        })
        .collect())
}

async fn reputation_history_for_uid(
    state: &AppState,
    uid: &str,
    limit: i64,
) -> AppResult<Vec<Value>> {
    let rows: Vec<(i64, Option<i64>, Option<i64>, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT reputation, quest_id, artwork_id, note, source_type, created_at
             FROM reputation_ledger WHERE uid=? ORDER BY datetime(created_at) DESC, id DESC LIMIT ?",
        )
        .bind(uid)
        .bind(limit)
        .fetch_all(&state.pools.art)
        .await?;
    Ok(rows
        .into_iter()
        .map(
            |(reputation, quest_id, artwork_id, note, source_type, created_at)| {
                json!({
                    "reputation": reputation,
                    "questId": quest_id,
                    "artworkId": artwork_id,
                    "note": note,
                    "sourceType": source_type,
                    "createdAt": created_at
                })
            },
        )
        .collect())
}

async fn claims_for_uid(state: &AppState, uid: &str) -> AppResult<Vec<Value>> {
    let rows: Vec<(i64, i64, String, String, i64, i64, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT c.id, q.id, q.title, c.status, c.progress, c.target_count, c.claimed_at, c.completed_at, c.rewarded_at
             FROM guild_quest_claims c JOIN guild_quests q ON q.id=c.quest_id
             WHERE c.uid=? ORDER BY datetime(c.claimed_at) DESC",
        )
        .bind(uid)
        .fetch_all(&state.pools.art)
        .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                quest_id,
                title,
                status,
                progress,
                target_count,
                claimed_at,
                completed_at,
                rewarded_at,
            )| {
                json!({
                    "id": id,
                    "questId": quest_id,
                    "title": title,
                    "status": status,
                    "progress": progress,
                    "targetCount": target_count,
                    "claimedAt": claimed_at,
                    "completedAt": completed_at,
                    "rewardedAt": rewarded_at
                })
            },
        )
        .collect())
}

async fn redemptions_for_uid(state: &AppState, uid: &str) -> AppResult<Vec<Value>> {
    let rows: Vec<(
        i64,
        i64,
        String,
        String,
        String,
        i64,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT r.id, r.reward_id, r.uid, gr.name, gr.reward_type, r.frozen_coins, r.status,
                    r.user_note, r.admin_note, r.created_at, r.reviewed_at, r.fulfilled_at
             FROM guild_reward_redemptions r JOIN guild_rewards gr ON gr.id=r.reward_id
             WHERE r.uid=? ORDER BY datetime(r.created_at) DESC",
    )
    .bind(uid)
    .fetch_all(&state.pools.art)
    .await?;
    Ok(rows.into_iter().map(redemption_value).collect())
}

async fn all_redemptions(state: &AppState) -> AppResult<Vec<Value>> {
    let rows: Vec<(
        i64,
        i64,
        String,
        String,
        String,
        i64,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT r.id, r.reward_id, r.uid, gr.name, gr.reward_type, r.frozen_coins, r.status,
                    r.user_note, r.admin_note, r.created_at, r.reviewed_at, r.fulfilled_at
             FROM guild_reward_redemptions r JOIN guild_rewards gr ON gr.id=r.reward_id
             ORDER BY datetime(r.created_at) DESC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    Ok(rows.into_iter().map(redemption_value).collect())
}

fn redemption_value(
    row: (
        i64,
        i64,
        String,
        String,
        String,
        i64,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
) -> Value {
    let (
        id,
        reward_id,
        uid,
        reward_name,
        reward_type,
        frozen_coins,
        status,
        user_note,
        admin_note,
        created_at,
        reviewed_at,
        fulfilled_at,
    ) = row;
    json!({
        "id": id,
        "rewardId": reward_id,
        "uid": uid,
        "rewardName": reward_name,
        "rewardType": reward_type,
        "frozenCoins": frozen_coins,
        "status": status,
        "userNote": user_note,
        "adminNote": admin_note,
        "createdAt": created_at,
        "reviewedAt": reviewed_at,
        "fulfilledAt": fulfilled_at
    })
}

async fn applications_for_uid(state: &AppState, uid: &str) -> AppResult<Vec<Value>> {
    let rows: Vec<(
        i64,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, from_rating, target_rating, status, user_note, admin_note, created_at
             FROM guild_rating_applications WHERE uid=? ORDER BY datetime(created_at) DESC",
    )
    .bind(uid)
    .fetch_all(&state.pools.art)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, from_rating, target_rating, status, user_note, admin_note, created_at)| {
                json!({
                    "id": id,
                    "fromRating": from_rating,
                    "targetRating": target_rating,
                    "status": status,
                    "userNote": user_note,
                    "adminNote": admin_note,
                    "createdAt": created_at
                })
            },
        )
        .collect())
}

type RewardRow = (
    i64,
    String,
    Option<String>,
    String,
    i64,
    Option<i64>,
    String,
    String,
    Option<String>,
    String,
    i64,
);

async fn reward_rows(state: &AppState, where_sql: &str) -> AppResult<Vec<RewardRow>> {
    let sql = format!(
        "SELECT id, name, description, reward_type, price_coins, stock, required_rating,
                required_access, image_url, status, sort_order
         FROM guild_rewards {where_sql}
         ORDER BY sort_order ASC, id ASC"
    );
    Ok(sqlx::query_as(&sql).fetch_all(&state.pools.art).await?)
}

fn reward_value(row: RewardRow, logged_in: bool, rating: &str, access: &str) -> Value {
    let (
        id,
        name,
        description,
        reward_type,
        price,
        stock,
        required_rating,
        required_access,
        image_url,
        status,
        sort_order,
    ) = row;
    let unlocked = logged_in
        && rating_rank(rating) >= rating_rank(&required_rating)
        && access_rank(access) >= access_rank(&required_access);
    json!({
        "id": id,
        "name": name,
        "description": description,
        "rewardType": reward_type,
        "priceCoins": price,
        "stock": stock,
        "requiredRating": required_rating,
        "requiredAccess": required_access,
        "requiredAccessLabel": access_label(&required_access),
        "imageUrl": image_url,
        "status": status,
        "sortOrder": sort_order,
        "unlocked": unlocked
    })
}

async fn admin_quest_rows(state: &AppState) -> AppResult<Vec<Value>> {
    let rows: Vec<(
        i64,
        String,
        Option<String>,
        String,
        String,
        String,
        String,
        String,
        i64,
        i64,
        i64,
        Option<i64>,
        String,
        i64,
    )> = sqlx::query_as(
        "SELECT id, title, description, quest_type, difficulty, required_rating,
                    required_access, condition_kind, target_count, reward_reputation,
                    reward_coins, deadline_hours, status, sort_order
             FROM guild_quests ORDER BY sort_order ASC, id ASC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                title,
                description,
                quest_type,
                difficulty,
                required_rating,
                required_access,
                condition_kind,
                target_count,
                reward_reputation,
                reward_coins,
                deadline_hours,
                status,
                sort_order,
            )| {
                json!({
                    "id": id,
                    "title": title,
                    "description": description,
                    "questType": quest_type,
                    "difficulty": difficulty,
                    "requiredRating": required_rating,
                    "requiredAccess": required_access,
                    "conditionKind": condition_kind,
                    "targetCount": target_count,
                    "rewardReputation": reward_reputation,
                    "rewardCoins": reward_coins,
                    "deadlineHours": deadline_hours,
                    "status": status,
                    "sortOrder": sort_order
                })
            },
        )
        .collect())
}

async fn set_redemption_status(
    state: &AppState,
    user: &AuthUser,
    id: i64,
    status: &str,
    body: Value,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, user, "art", Action::Manage).await?;
    sqlx::query(
        "UPDATE guild_reward_redemptions SET status=?, admin_note=?, reviewed_at=? WHERE id=? AND status='pending'",
    )
    .bind(status)
    .bind(body.get("note").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(now_iso())
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn approved_haruhi_personal_count(state: &AppState, uid: &str) -> AppResult<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM artworks
         WHERE uploader_uid=? AND status='approved' AND source_type='personal' AND content_type='haruhi'",
    )
    .bind(uid)
    .fetch_one(&state.pools.art)
    .await?)
}

fn artwork_image_url(file_path: Option<&str>, images_json: Option<&str>) -> String {
    if let Some(fp) = file_path.filter(|s| !s.is_empty()) {
        return format!("uploads/{fp}");
    }
    let parsed = serde_json::from_str::<Value>(images_json.unwrap_or("[]")).unwrap_or(Value::Null);
    parsed
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|img| img.get("path"))
        .and_then(|v| v.as_str())
        .map(|p| format!("uploads/{p}"))
        .unwrap_or_default()
}

fn uid_for_user(user_id: i64) -> String {
    format!("u{user_id}")
}

fn user_id_from_uid(uid: &str) -> Option<i64> {
    uid.strip_prefix('u')?.parse::<i64>().ok()
}

fn normalize_uid(uid: &str) -> AppResult<String> {
    let uid = uid.trim();
    if uid.is_empty() || uid.len() > 80 {
        return Err(AppError::bad_request("UID 无效"));
    }
    Ok(uid.to_string())
}

fn level_from_reputation(reputation: i64) -> i64 {
    (reputation.max(0) / 100) + 1
}

fn rating_rank(rating: &str) -> i64 {
    match rating {
        "X" => 8,
        "S" => 7,
        "A" => 6,
        "B" => 5,
        "C" => 4,
        "D" => 3,
        "E" => 2,
        "F" => 1,
        _ => 0,
    }
}

fn access_rank(access: &str) -> i64 {
    match access {
        "closed_space" => 3,
        "anomaly_research" => 2,
        "observer_clearance" => 1,
        "public_archive" => 0,
        _ => -1,
    }
}

fn access_label(access: &str) -> &'static str {
    match access {
        "closed_space" => "3级闭锁空间许可",
        "anomaly_research" => "2级异常观测许可",
        "observer_clearance" => "1级观测员许可",
        _ => "0级公开档案许可",
    }
}

fn access_short_label(access: &str) -> &'static str {
    match access {
        "closed_space" => "闭锁3",
        "anomaly_research" => "异常2",
        "observer_clearance" => "观测1",
        _ => "档案0",
    }
}

fn rating_label(rating: &str) -> String {
    format!("{rating}级冒险者")
}

fn next_rating(current: &str, reputation: i64, haruhi_count: i64) -> Value {
    let current_rank = rating_rank(current);
    for (rating, required_rep, required_haruhi) in RATINGS {
        if rating_rank(rating) > current_rank {
            return json!({
                "rating": rating,
                "requiredReputation": required_rep,
                "requiredHaruhiCount": required_haruhi,
                "available": reputation >= *required_rep && haruhi_count >= *required_haruhi
            });
        }
    }
    Value::Null
}

fn rating_requirements_met(rating: &str, reputation: i64, haruhi_count: i64) -> bool {
    RATINGS
        .iter()
        .find(|(r, _, _)| *r == rating)
        .map(|(_, required_rep, required_haruhi)| {
            reputation >= *required_rep && haruhi_count >= *required_haruhi
        })
        .unwrap_or(false)
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn json_num_i64(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        Some(n)
    } else if let Some(f) = v.as_f64() {
        Some(f as i64)
    } else {
        v.as_str()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|f| f as i64)
    }
}

fn str_field(body: &Value, key: &str, fallback: &str) -> String {
    body.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn int_field(body: &Value, key: &str, fallback: i64) -> i64 {
    body.get(key).and_then(json_num_i64).unwrap_or(fallback)
}

fn clamp_query_i64(value: Option<&String>, min: i64, max: i64, fallback: i64) -> i64 {
    value
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(fallback)
        .clamp(min, max)
}
