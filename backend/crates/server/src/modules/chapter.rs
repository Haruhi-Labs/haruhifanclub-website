//! Chapter 地方支部子站：公开目录、内容聚合与实例级管理 API。
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::time::Duration;

use axum::extract::{Multipart, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, patch, post, put};
use axum::{Json, Router};
use haruhi_auth::{
    authorize_capability, capability_grants, require_super, verify_password, AuthUser,
};
use haruhi_core::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{self as stream, StreamExt};

use crate::ratelimit::client_ip;
use crate::state::AppState;

const ADMIN_CAPABILITIES: &[&str] = &[
    "branch.profile.manage",
    "branch.brand.manage",
    "branch.contacts.manage",
    "branch.members.manage",
    "branch.organization.manage",
    "branch.timeline.write",
    "branch.timeline.publish",
    "branch.events.write",
    "branch.events.publish",
    "branch.events.attendees.manage",
    "branch.permissions.manage",
    "branch.audit.read",
    "branch.lifecycle.manage",
    "branch.platform.intervene",
];

const MEDIA_UPLOAD_CAPABILITIES: &[&str] = &[
    "branch.profile.manage",
    "branch.brand.manage",
    "branch.contacts.manage",
    "branch.timeline.write",
    "branch.events.write",
];

const CHAPTER_UTC_OFFSET_HOURS: i64 = 8;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/branches", get(list_branches))
        .route("/posts", get(aggregate_timeline))
        .route("/timeline", get(aggregate_timeline))
        .route("/timeline/stream", get(timeline_stream))
        .route("/events", get(aggregate_events))
        .route("/branches/{slug}", get(get_branch))
        .route("/branches/{slug}/members", get(public_members))
        .route(
            "/branches/{slug}/official-members",
            get(public_official_members),
        )
        .route(
            "/branches/{slug}/membership-summary",
            get(public_membership_summary),
        )
        .route("/branches/{slug}/join", post(join_branch))
        .route("/branches/{slug}/contacts", get(public_contacts))
        .route("/branches/{slug}/qq-groups", get(public_qq_groups))
        .route("/branches/{slug}/organization", get(public_organization))
        .route("/branches/{slug}/merchandise", get(public_merchandise))
        .route("/branches/{slug}/posts", get(branch_timeline))
        .route("/branches/{slug}/timeline", get(branch_timeline))
        .route("/branches/{slug}/events", get(branch_events))
        .route("/branches/{slug}/events/{event_slug}", get(get_event))
        .route(
            "/branches/{slug}/events/{event_slug}/photos",
            get(event_photos),
        )
        .route(
            "/branches/{slug}/events/{event_slug}/registration",
            get(my_event_registration)
                .post(register_event)
                .patch(update_my_registration),
        )
        .route("/membership", get(my_membership))
        .route("/membership/leave-request", post(request_membership_leave))
        .route("/admin/branches", get(admin_branches).post(create_branch))
        .route(
            "/admin/branches/{id}",
            get(admin_branch).patch(update_branch),
        )
        .route("/admin/branches/{id}/brand", put(update_brand))
        .route("/admin/branches/{id}/sections", put(replace_sections))
        .route("/admin/branches/{id}/merchandise", put(replace_merchandise))
        .route("/admin/branches/{id}/members", put(replace_members))
        .route("/admin/branches/{id}/memberships", get(admin_memberships))
        .route(
            "/admin/branches/{id}/leave-requests/{request_id}",
            patch(review_leave_request),
        )
        .route("/admin/branches/{id}/contacts", put(replace_contacts))
        .route("/admin/branches/{id}/qq-groups", put(replace_qq_groups))
        .route(
            "/admin/branches/{id}/organization",
            put(replace_organization),
        )
        .route(
            "/admin/branches/{id}/timeline",
            get(admin_timeline).post(create_timeline_entry),
        )
        .route(
            "/admin/branches/{id}/timeline/{item_id}",
            patch(update_timeline_entry).delete(delete_timeline_entry),
        )
        .route(
            "/admin/branches/{id}/albums",
            get(admin_timeline).post(create_timeline_entry),
        )
        .route(
            "/admin/branches/{id}/albums/{item_id}",
            patch(update_timeline_entry).delete(delete_timeline_entry),
        )
        .route(
            "/admin/branches/{id}/events",
            get(admin_events).post(create_event),
        )
        .route(
            "/admin/branches/{id}/events/{item_id}",
            patch(update_event).delete(delete_event),
        )
        .route(
            "/admin/branches/{id}/events/{item_id}/operations",
            get(admin_event_operations).put(replace_event_operations),
        )
        .route(
            "/admin/branches/{id}/cohost-invitations",
            get(admin_cohost_invitations),
        )
        .route(
            "/admin/branches/{id}/events/{item_id}/registrations",
            get(admin_event_registrations),
        )
        .route(
            "/admin/branches/{id}/events/{item_id}/registrations/{registration_id}",
            patch(review_event_registration),
        )
        .route(
            "/admin/branches/{id}/cohost-invitations/{cohost_id}",
            patch(respond_cohost_invitation),
        )
        .route(
            "/admin/branches/{id}/grants",
            get(list_branch_grants).put(set_branch_grants),
        )
        .route("/admin/branches/{id}/handover", post(handover_branch))
        .route("/admin/branches/{id}/moderation", patch(moderate_branch))
        .route("/admin/branches/{id}/audit", get(branch_audit))
        .route("/admin/branches/{id}/media", post(upload_media))
        .route("/admin/platform/grants", put(set_platform_grants))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct Branch {
    id: i64,
    slug: String,
    name: String,
    short_name: Option<String>,
    summary: Option<String>,
    about_text: Option<String>,
    join_text: Option<String>,
    country_code: String,
    locality_name: Option<String>,
    founded_on: Option<String>,
    status: String,
    default_post_aggregate: bool,
    default_event_aggregate: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct Brand {
    branch_id: i64,
    logo_path: Option<String>,
    logo_alt: Option<String>,
    cover_path: Option<String>,
    cover_focal_x: f64,
    cover_focal_y: f64,
    tagline: Option<String>,
    accent_key: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct Section {
    section_key: String,
    label: Option<String>,
    enabled: bool,
    visibility: String,
    sort_order: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct MerchandiseRow {
    id: i64,
    name: String,
    description: Option<String>,
    image_path: Option<String>,
    tags_json: String,
    status: String,
    sort_order: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct Member {
    id: i64,
    user_id: Option<i64>,
    display_name: String,
    avatar_path: Option<String>,
    bio: Option<String>,
    status: String,
    joined_on: Option<String>,
    left_on: Option<String>,
    is_public: bool,
    sort_order: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct QqGroup {
    id: i64,
    name: String,
    group_number: String,
    description: Option<String>,
    audience_label: Option<String>,
    join_url: Option<String>,
    qr_image_path: Option<String>,
    join_instructions: Option<String>,
    is_primary: bool,
    status: String,
    sort_order: i64,
    last_verified_at: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct ContactPerson {
    id: i64,
    member_id: Option<i64>,
    display_name: String,
    role_title: Option<String>,
    responsibility: Option<String>,
    is_primary: bool,
    is_public: bool,
    consent_confirmed_at: Option<String>,
    status: String,
    sort_order: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct ContactMethod {
    id: i64,
    person_id: i64,
    method_type: String,
    label: Option<String>,
    value: String,
    url: Option<String>,
    is_public: bool,
    sort_order: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct EventItem {
    id: i64,
    branch_id: i64,
    branch_slug: String,
    branch_name: String,
    slug: String,
    title: String,
    summary: Option<String>,
    content: Option<String>,
    cover_path: Option<String>,
    event_type: Option<String>,
    venue_name: Option<String>,
    address: Option<String>,
    online_url: Option<String>,
    starts_at: String,
    ends_at: Option<String>,
    registration_url: Option<String>,
    format: String,
    registration_mode: String,
    admission_mode: String,
    capacity: Option<i64>,
    registration_opens_at: Option<String>,
    registration_closes_at: Option<String>,
    status: String,
    visibility: String,
    aggregate_mode: String,
    moderation_state: String,
    published_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct TimelineItem {
    id: i64,
    branch_id: i64,
    branch_slug: String,
    branch_name: String,
    event_id: i64,
    event_slug: String,
    event_title: String,
    title: String,
    content: Option<String>,
    image_path: Option<String>,
    happened_at: String,
    location_name: Option<String>,
    status: String,
    moderation_state: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct TimelineEventItem {
    id: i64,
    branch_slug: String,
    branch_locality_name: Option<String>,
    event_slug: String,
    title: String,
    starts_at: String,
    ends_at: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct EventPhotoItem {
    id: i64,
    event_id: i64,
    title: String,
    content: Option<String>,
    image_path: String,
    happened_at: String,
    created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
struct RegistrationEvent {
    id: i64,
    branch_id: i64,
    registration_mode: String,
    admission_mode: String,
    capacity: Option<i64>,
    registration_opens_at: Option<String>,
    registration_closes_at: Option<String>,
    starts_at: String,
    ends_at: Option<String>,
    status: String,
    visibility: String,
    moderation_state: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    q: Option<String>,
    branch: Option<String>,
    city: Option<String>,
    event_type: Option<String>,
    format: Option<String>,
    topic: Option<String>,
    period: Option<String>,
    event: Option<String>,
    from: Option<String>,
    to: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

fn list_pagination(query: &ListQuery) -> (i64, i64, i64) {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(24).clamp(1, 50);
    (page, page_size, (page - 1) * page_size)
}

fn event_time_values(body: &Value, status: &str) -> AppResult<(String, Option<String>)> {
    let starts_at = required_string(body, "startsAt", "开始时间")?;
    let ends_at = opt_string(body, "endsAt");
    if status == "published" && ends_at.is_none() {
        return Err(AppError::bad_request("发布活动前请填写结束时间"));
    }
    if let Some(end) = &ends_at {
        if end <= &starts_at {
            return Err(AppError::bad_request("结束时间必须晚于开始时间"));
        }
    }
    Ok((starts_at, ends_at))
}

async fn active_branch(state: &AppState, slug: &str) -> AppResult<Branch> {
    let branch = sqlx::query_as::<_, Branch>(
        "SELECT id, slug, name, short_name, summary, about_text, join_text, country_code, \
         locality_name, founded_on, status, default_post_aggregate, \
         default_event_aggregate, created_at, updated_at FROM branches \
         WHERE slug = ? AND status IN ('active','paused')",
    )
    .bind(slug)
    .fetch_optional(&state.pools.chapter)
    .await?;
    if let Some(branch) = branch {
        return Ok(branch);
    }
    let alias: Option<String> = sqlx::query_scalar(
        "SELECT b.slug FROM branch_slug_aliases a JOIN branches b ON b.id = a.branch_id \
         WHERE a.slug = ? AND b.status IN ('active','paused')",
    )
    .bind(slug)
    .fetch_optional(&state.pools.chapter)
    .await?;
    match alias {
        Some(canonical) => Box::pin(active_branch(state, &canonical)).await,
        None => Err(AppError::not_found("支部不存在或尚未公开")),
    }
}

async fn brand(state: &AppState, branch_id: i64) -> AppResult<Option<Brand>> {
    Ok(sqlx::query_as::<_, Brand>(
        "SELECT branch_id, logo_path, logo_alt, \
         cover_path, cover_focal_x, cover_focal_y, tagline, accent_key \
         FROM branch_brand WHERE branch_id = ?",
    )
    .bind(branch_id)
    .fetch_optional(&state.pools.chapter)
    .await?)
}

async fn sections(state: &AppState, branch_id: i64, public_only: bool) -> AppResult<Vec<Section>> {
    let sql = if public_only {
        "SELECT section_key, label, enabled, visibility, sort_order FROM branch_sections \
         WHERE branch_id = ? AND enabled = 1 AND visibility = 'public' ORDER BY sort_order"
    } else {
        "SELECT section_key, label, enabled, visibility, sort_order FROM branch_sections \
         WHERE branch_id = ? ORDER BY sort_order"
    };
    Ok(sqlx::query_as(sql)
        .bind(branch_id)
        .fetch_all(&state.pools.chapter)
        .await?)
}

async fn list_branches(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    let q = format!("%{}%", query.q.unwrap_or_default().trim());
    let items: Vec<Branch> = sqlx::query_as(
        "SELECT id, slug, name, short_name, summary, about_text, join_text, country_code, \
         locality_name, founded_on, status, default_post_aggregate, \
         default_event_aggregate, created_at, updated_at FROM branches \
         WHERE status IN ('active','paused') \
         AND (? = '%%' OR name LIKE ? OR short_name LIKE ? OR locality_name LIKE ?) \
         ORDER BY status = 'paused', locality_name, name",
    )
    .bind(&q)
    .bind(&q)
    .bind(&q)
    .bind(&q)
    .fetch_all(&state.pools.chapter)
    .await?;
    let mut data = Vec::with_capacity(items.len());
    for item in items {
        let item_brand = brand(&state, item.id).await?;
        let member_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM branch_memberships WHERE branch_id=? AND ended_at IS NULL",
        )
        .bind(item.id)
        .fetch_one(&state.pools.chapter)
        .await?;
        data.push(json!({ "branch": item, "brand": item_brand, "memberCount":member_count }));
    }
    Ok(Json(json!({ "items": data, "total": data.len() })))
}

async fn get_branch(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let item_brand = brand(&state, branch.id).await?;
    let item_sections = sections(&state, branch.id, true).await?;
    let has_merchandise: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM branch_merchandise WHERE branch_id=? AND status='published')",
    )
    .bind(branch.id)
    .fetch_one(&state.pools.chapter)
    .await?;
    Ok(Json(json!({
        "branch": branch,
        "brand": item_brand,
        "sections": item_sections,
        "hasMerchandise": has_merchandise,
    })))
}

fn merchandise_value(item: MerchandiseRow) -> Value {
    let tags = serde_json::from_str::<Vec<String>>(&item.tags_json).unwrap_or_default();
    json!({
        "id": item.id,
        "name": item.name,
        "description": item.description,
        "imagePath": item.image_path,
        "tags": tags,
        "status": item.status,
        "sortOrder": item.sort_order,
        "createdAt": item.created_at,
        "updatedAt": item.updated_at,
    })
}

async fn merchandise_for(
    state: &AppState,
    branch_id: i64,
    public_only: bool,
) -> AppResult<Vec<Value>> {
    let sql = if public_only {
        "SELECT id,name,description,image_path,tags_json,status,sort_order,created_at,updated_at \
         FROM branch_merchandise WHERE branch_id=? AND status='published' ORDER BY sort_order,id"
    } else {
        "SELECT id,name,description,image_path,tags_json,status,sort_order,created_at,updated_at \
         FROM branch_merchandise WHERE branch_id=? ORDER BY sort_order,id"
    };
    let items: Vec<MerchandiseRow> = sqlx::query_as(sql)
        .bind(branch_id)
        .fetch_all(&state.pools.chapter)
        .await?;
    Ok(items.into_iter().map(merchandise_value).collect())
}

async fn public_merchandise(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let items = merchandise_for(&state, branch.id, true).await?;
    Ok(Json(json!({ "items": items })))
}

async fn members_for(state: &AppState, id: i64, public_only: bool) -> AppResult<Vec<Member>> {
    let sql = if public_only {
        "SELECT id, user_id, display_name, avatar_path, bio, status, joined_on, left_on, \
         is_public, sort_order FROM branch_members WHERE branch_id = ? AND is_public = 1 \
         AND status IN ('active','alumni') ORDER BY sort_order, id"
    } else {
        "SELECT id, user_id, display_name, avatar_path, bio, status, joined_on, left_on, \
         is_public, sort_order FROM branch_members WHERE branch_id = ? ORDER BY sort_order, id"
    };
    Ok(sqlx::query_as(sql)
        .bind(id)
        .fetch_all(&state.pools.chapter)
        .await?)
}

async fn public_members(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let q = query.q.unwrap_or_default().trim().to_lowercase();
    let page = query.page.unwrap_or(1).max(1);
    let rows: Vec<(i64, i64, String, String)> = sqlx::query_as(
        "SELECT membership.id,membership.user_id,membership.state,membership.joined_at \
         FROM branch_memberships membership WHERE membership.branch_id=? \
         AND membership.ended_at IS NULL AND NOT EXISTS ( \
           SELECT 1 FROM branch_members official WHERE official.branch_id=membership.branch_id \
           AND official.user_id=membership.user_id AND official.status='active' \
         ) ORDER BY membership.joined_at,membership.id",
    )
    .bind(branch.id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let mut all = Vec::with_capacity(rows.len());
    for (membership_id, user_id, membership_state, joined_at) in rows {
        if let Some((display_name, avatar)) = public_account(&state, user_id).await? {
            if q.is_empty() || display_name.to_lowercase().contains(&q) {
                all.push(json!({
                    "membershipId":membership_id,"displayName":display_name,"avatar":avatar,
                    "state":membership_state,"joinedAt":joined_at
                }));
            }
        }
    }
    let total = all.len();
    let start = ((page - 1) * 48) as usize;
    let items = all.into_iter().skip(start).take(48).collect::<Vec<_>>();
    Ok(Json(
        json!({"items":items,"total":total,"page":page,"pageSize":48}),
    ))
}

async fn public_membership_summary(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let member_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM branch_memberships WHERE branch_id=? AND ended_at IS NULL",
    )
    .bind(branch.id)
    .fetch_one(&state.pools.chapter)
    .await?;
    let active_member_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT official.user_id) FROM branch_members official \
         JOIN branch_memberships membership ON membership.branch_id=official.branch_id \
           AND membership.user_id=official.user_id AND membership.ended_at IS NULL \
         WHERE official.branch_id=? AND official.status='active' AND official.user_id IS NOT NULL",
    )
    .bind(branch.id)
    .fetch_one(&state.pools.chapter)
    .await?;
    let alumni_member_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM branch_members WHERE branch_id=? AND status='alumni' AND is_public=1",
    )
    .bind(branch.id)
    .fetch_one(&state.pools.chapter)
    .await?;
    Ok(Json(json!({
        "branchId":branch.id,
        "memberCount":member_count,
        "ordinaryMemberCount":member_count.saturating_sub(active_member_count),
        "activeMemberCount":active_member_count,
        "alumniMemberCount":alumni_member_count
    })))
}

async fn public_official_members(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let rows = members_for(&state, branch.id, true).await?;
    let mut items = Vec::with_capacity(rows.len());
    for member in rows {
        let account = if let Some(user_id) = member.user_id {
            public_account(&state, user_id).await?
        } else {
            None
        };
        let (display_name, avatar) =
            account.unwrap_or_else(|| (member.display_name.clone(), member.avatar_path.clone()));
        items.push(json!({
            "id":member.id,"userId":member.user_id,"displayName":display_name,"avatar":avatar,
            "bio":member.bio,"status":member.status,"joinedOn":member.joined_on,
            "leftOn":member.left_on,"sortOrder":member.sort_order
        }));
    }
    Ok(Json(json!({"items":items})))
}

#[derive(Deserialize)]
struct JoinBranchInput {
    password: String,
}

async fn join_branch(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<JoinBranchInput>,
) -> AppResult<Json<Value>> {
    let ip = client_ip(&headers);
    state
        .login_limiter
        .check_and_record(&ip)
        .map_err(|seconds| {
            AppError::TooManyRequests(format!("尝试次数过多，请在 {seconds} 秒后重试"))
        })?;
    let branch: Branch = active_branch(&state, &slug).await?;
    if branch.status != "active" {
        return Err(AppError::bad_request("该支部当前暂停接受新成员"));
    }
    let hash: Option<String> = sqlx::query_scalar(
        "SELECT password_hash FROM users WHERE id=? AND status='active' AND deleted_at IS NULL",
    )
    .bind(user.id)
    .fetch_optional(&state.pools.core)
    .await?;
    if !hash
        .as_deref()
        .is_some_and(|stored| verify_password(&body.password, stored))
    {
        return Err(AppError::bad_request("当前账号密码不正确"));
    }
    let existing: Option<(i64, i64)> = sqlx::query_as(
        "SELECT m.id,m.branch_id FROM branch_memberships m WHERE m.user_id=? AND m.ended_at IS NULL",
    )
    .bind(user.id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    if let Some((_, existing_branch)) = existing {
        if existing_branch == branch.id {
            return Err(AppError::conflict("你已经加入该支部"));
        }
        let name: String = sqlx::query_scalar("SELECT name FROM branches WHERE id=?")
            .bind(existing_branch)
            .fetch_one(&state.pools.chapter)
            .await?;
        return Err(AppError::conflict(format!(
            "你已经加入{name}，退出申请获批前不能加入其他支部"
        )));
    }
    let inserted = sqlx::query_scalar::<_, i64>(
        "INSERT INTO branch_memberships(branch_id,user_id) VALUES (?,?) RETURNING id",
    )
    .bind(branch.id)
    .bind(user.id)
    .fetch_one(&state.pools.chapter)
    .await;
    let membership_id = match inserted {
        Ok(id) => id,
        Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
            return Err(AppError::conflict("该账号已经加入其他支部"));
        }
        Err(error) => return Err(error.into()),
    };
    state.login_limiter.reset(&ip);
    audit(
        &state,
        Some(branch.id),
        user.id,
        "membership.join",
        "membership",
        membership_id,
        None,
    )
    .await;
    Ok(Json(
        json!({"id":membership_id,"branchId":branch.id,"branchSlug":branch.slug,"state":"active"}),
    ))
}

async fn my_membership(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let row: Option<(i64, i64, String, String, String, String)> = sqlx::query_as(
        "SELECT m.id,b.id,b.slug,b.name,m.state,m.joined_at FROM branch_memberships m \
         JOIN branches b ON b.id=m.branch_id WHERE m.user_id=? AND m.ended_at IS NULL",
    )
    .bind(user.id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    let Some((id, branch_id, branch_slug, branch_name, membership_state, joined_at)) = row else {
        return Ok(Json(json!({"membership":null,"leaveRequest":null})));
    };
    let leave: Option<(i64, String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id,reason,state,requested_at,review_note,reviewed_at FROM branch_membership_leave_requests \
         WHERE membership_id=? ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    Ok(Json(json!({
        "membership":{"id":id,"branchId":branch_id,"branchSlug":branch_slug,"branchName":branch_name,"state":membership_state,"joinedAt":joined_at},
        "leaveRequest":leave.map(|(id,reason,state,requested_at,review_note,reviewed_at)|json!({"id":id,"reason":reason,"state":state,"requestedAt":requested_at,"reviewNote":review_note,"reviewedAt":reviewed_at}))
    })))
}

#[derive(Deserialize)]
struct LeaveRequestInput {
    reason: String,
}

async fn request_membership_leave(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<LeaveRequestInput>,
) -> AppResult<Json<Value>> {
    let reason = body.reason.trim();
    if reason.is_empty() {
        return Err(AppError::bad_request("请填写退出原因"));
    }
    let membership: Option<(i64, i64, String)> = sqlx::query_as(
        "SELECT id,branch_id,state FROM branch_memberships WHERE user_id=? AND ended_at IS NULL",
    )
    .bind(user.id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    let Some((membership_id, branch_id, membership_state)) = membership else {
        return Err(AppError::bad_request("当前没有已加入的支部"));
    };
    if membership_state == "leave_requested" {
        return Err(AppError::conflict("已有退出申请正在处理"));
    }
    let mut tx = state.pools.chapter.begin().await?;
    let request_id: i64 = sqlx::query_scalar(
        "INSERT INTO branch_membership_leave_requests(membership_id,reason) VALUES (?,?) RETURNING id",
    )
    .bind(membership_id)
    .bind(reason)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("UPDATE branch_memberships SET state='leave_requested',updated_at=datetime('now') WHERE id=?")
        .bind(membership_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    audit(
        &state,
        Some(branch_id),
        user.id,
        "membership.leave_request",
        "leave_request",
        request_id,
        None,
    )
    .await;
    Ok(Json(json!({"id":request_id,"state":"pending"})))
}

async fn qq_for(state: &AppState, id: i64, public_only: bool) -> AppResult<Vec<QqGroup>> {
    let sql = if public_only {
        "SELECT id, name, group_number, description, audience_label, join_url, qr_image_path, \
         join_instructions, is_primary, status, sort_order, last_verified_at FROM branch_qq_groups \
         WHERE branch_id = ? AND status = 'active' ORDER BY is_primary DESC, sort_order, id"
    } else {
        "SELECT id, name, group_number, description, audience_label, join_url, qr_image_path, \
         join_instructions, is_primary, status, sort_order, last_verified_at FROM branch_qq_groups \
         WHERE branch_id = ? ORDER BY is_primary DESC, sort_order, id"
    };
    Ok(sqlx::query_as(sql)
        .bind(id)
        .fetch_all(&state.pools.chapter)
        .await?)
}

async fn public_qq_groups(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    Ok(Json(
        json!({ "items": qq_for(&state, branch.id, true).await? }),
    ))
}

async fn contacts_for(
    state: &AppState,
    id: i64,
    public_only: bool,
) -> AppResult<(Vec<ContactPerson>, Vec<ContactMethod>)> {
    let person_sql = if public_only {
        "SELECT id, member_id, display_name, role_title, responsibility, is_primary, is_public, \
         consent_confirmed_at, status, sort_order FROM branch_contact_people \
         WHERE branch_id = ? AND status = 'active' AND is_public = 1 \
         AND consent_confirmed_at IS NOT NULL ORDER BY is_primary DESC, sort_order, id"
    } else {
        "SELECT id, member_id, display_name, role_title, responsibility, is_primary, is_public, \
         consent_confirmed_at, status, sort_order FROM branch_contact_people \
         WHERE branch_id = ? ORDER BY is_primary DESC, sort_order, id"
    };
    let people: Vec<ContactPerson> = sqlx::query_as(person_sql)
        .bind(id)
        .fetch_all(&state.pools.chapter)
        .await?;
    let mut methods = Vec::new();
    for person in &people {
        let sql = if public_only {
            "SELECT id, person_id, method_type, label, value, url, is_public, sort_order \
             FROM branch_contact_methods WHERE person_id = ? AND is_public = 1 ORDER BY sort_order, id"
        } else {
            "SELECT id, person_id, method_type, label, value, url, is_public, sort_order \
             FROM branch_contact_methods WHERE person_id = ? ORDER BY sort_order, id"
        };
        let mut rows: Vec<ContactMethod> = sqlx::query_as(sql)
            .bind(person.id)
            .fetch_all(&state.pools.chapter)
            .await?;
        methods.append(&mut rows);
    }
    Ok((people, methods))
}

async fn public_contacts(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let (people, methods) = contacts_for(&state, branch.id, true).await?;
    Ok(Json(json!({ "people": people, "methods": methods })))
}

async fn public_organization(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let version: Option<(i64, String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, name, summary, state, display_mode FROM organization_versions \
         WHERE branch_id = ? AND state = 'current' ORDER BY id DESC LIMIT 1",
    )
    .bind(branch.id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    let Some((version_id, name, summary, state_name, display_mode)) = version else {
        return Ok(Json(
            json!({ "version": null, "units": [], "assignments": [] }),
        ));
    };
    if display_mode == "hidden" {
        return Ok(Json(
            json!({ "version": { "name": name, "summary": summary, "state": state_name, "displayMode": display_mode }, "units": [], "assignments": [] }),
        ));
    }
    let units: Vec<(i64, Option<i64>, String, String, Option<String>, i64)> = sqlx::query_as(
        "SELECT id, parent_id, name, kind, description, sort_order FROM organization_units \
         WHERE version_id = ? AND is_public = 1 ORDER BY sort_order, id",
    )
    .bind(version_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let assignment_rows: Vec<(i64, Option<i64>, i64, Option<i64>, String, Option<String>, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT a.id, a.unit_id, a.member_id, m.user_id, m.display_name, a.title, a.term_start, a.term_end, a.sort_order \
         FROM organization_assignments a JOIN branch_members m ON m.id = a.member_id \
         WHERE a.version_id = ? AND a.is_public = 1 AND m.is_public = 1 ORDER BY a.sort_order, a.id",
    )
    .bind(version_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let mut assignments = Vec::with_capacity(assignment_rows.len());
    for (id, unit_id, member_id, user_id, snapshot_name, title, term_start, term_end, sort_order) in
        assignment_rows
    {
        let account: Option<(String, Option<String>)> = if let Some(user_id) = user_id {
            sqlx::query_as("SELECT COALESCE(NULLIF(nickname,''),NULLIF(display_name,''),username),avatar FROM users WHERE id=? AND status='active' AND deleted_at IS NULL")
                .bind(user_id).fetch_optional(&state.pools.core).await?
        } else {
            None
        };
        let (display_name, avatar) = account.unwrap_or((snapshot_name, None));
        assignments.push(json!({"id":id,"unitId":unit_id,"memberId":member_id,"displayName":display_name,"avatar":avatar,"title":title,"termStart":term_start,"termEnd":term_end,"sortOrder":sort_order}));
    }
    Ok(Json(json!({
        "version": { "id": version_id, "name": name, "summary": summary, "state": state_name, "displayMode": display_mode },
        "units": units.into_iter().map(|(id,parent_id,name,kind,description,sort_order)| json!({"id":id,"parentId":parent_id,"name":name,"kind":kind,"description":description,"sortOrder":sort_order})).collect::<Vec<_>>(),
        "assignments": assignments
    })))
}

async fn admin_organization(state: &AppState, id: i64) -> AppResult<Value> {
    let version: Option<(i64, String, Option<String>, Option<String>, String)> = sqlx::query_as(
        "SELECT id, name, summary, effective_on, display_mode FROM organization_versions \
         WHERE branch_id = ? AND state = 'current' ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    let Some((version_id, name, summary, effective_on, display_mode)) = version else {
        return Ok(json!({
            "name": "现行组织架构",
            "displayMode": "tree",
            "summary": "",
            "units": [],
            "assignments": []
        }));
    };
    let units: Vec<(i64, Option<i64>, String, String, Option<String>, bool, i64)> = sqlx::query_as(
        "SELECT id, parent_id, name, kind, description, is_public, sort_order \
             FROM organization_units WHERE version_id = ? ORDER BY sort_order, id",
    )
    .bind(version_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let unit_keys: HashMap<i64, String> = units
        .iter()
        .map(|(unit_id, ..)| (*unit_id, format!("unit-{unit_id}")))
        .collect();
    let unit_items = units
        .into_iter()
        .map(
            |(unit_id, parent_id, unit_name, kind, description, is_public, sort_order)| {
                json!({
                    "key": unit_keys[&unit_id],
                    "parentKey": parent_id.and_then(|parent| unit_keys.get(&parent).cloned()),
                    "name": unit_name,
                    "kind": kind,
                    "description": description,
                    "isPublic": is_public,
                    "sortOrder": sort_order
                })
            },
        )
        .collect::<Vec<_>>();
    let assignments: Vec<(
        i64,
        Option<i64>,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        bool,
        bool,
        i64,
    )> = sqlx::query_as(
        "SELECT id, unit_id, member_id, title, term_start, term_end, is_public, is_contact, \
         sort_order FROM organization_assignments WHERE version_id = ? ORDER BY sort_order, id",
    )
    .bind(version_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let assignment_items = assignments
        .into_iter()
        .map(
            |(
                assignment_id,
                unit_id,
                member_id,
                title,
                term_start,
                term_end,
                is_public,
                is_contact,
                sort_order,
            )| {
                json!({
                    "id": assignment_id,
                    "unitKey": unit_id.and_then(|unit| unit_keys.get(&unit).cloned()),
                    "memberId": member_id,
                    "title": title,
                    "termStart": term_start,
                    "termEnd": term_end,
                    "isPublic": is_public,
                    "isContact": is_contact,
                    "sortOrder": sort_order
                })
            },
        )
        .collect::<Vec<_>>();
    Ok(json!({
        "id": version_id,
        "name": name,
        "effectiveOn": effective_on,
        "displayMode": display_mode,
        "summary": summary,
        "units": unit_items,
        "assignments": assignment_items
    }))
}

const EVENT_SELECT: &str = "SELECT e.id, e.branch_id, b.slug branch_slug, b.name branch_name, e.slug, e.title, \
 e.summary, e.content, e.cover_path, e.event_type, e.venue_name, e.address, e.online_url, e.starts_at, \
 e.ends_at, e.registration_url, e.format, e.registration_mode, e.admission_mode, e.capacity, \
 e.registration_opens_at, e.registration_closes_at, e.status, e.visibility, e.aggregate_mode, e.moderation_state, \
 e.published_at, e.created_at, e.updated_at FROM branch_events e JOIN branches b ON b.id = e.branch_id";
const TIMELINE_SELECT: &str = "SELECT t.id, t.branch_id, b.slug branch_slug, b.name branch_name, \
 t.event_id, e.slug event_slug, e.title event_title, t.title, t.content, t.image_path, \
 t.happened_at, COALESCE(t.location_name, e.venue_name) location_name, t.status, \
 t.moderation_state, t.created_at, t.updated_at FROM branch_event_timeline_entries t \
 JOIN branches b ON b.id=t.branch_id JOIN branch_events e ON e.id=t.event_id";
const TIMELINE_EVENT_SELECT: &str = "SELECT e.id, b.slug branch_slug, \
 b.locality_name branch_locality_name, e.slug event_slug, e.title, e.starts_at, e.ends_at \
 FROM branch_events e JOIN branches b ON b.id=e.branch_id";

async fn branch_timeline(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let event_slug = query.event.unwrap_or_default();
    let sql = format!(
        "{TIMELINE_EVENT_SELECT} WHERE (e.branch_id=? OR EXISTS(SELECT 1 FROM branch_event_cohosts c \
         WHERE c.event_id=e.id AND c.branch_id=? AND c.state='accepted')) \
         AND e.status='published' AND e.visibility='public' AND e.moderation_state='normal' \
         AND (?='' OR e.slug=?) ORDER BY e.starts_at DESC,e.id DESC LIMIT 200"
    );
    let items: Vec<TimelineEventItem> = sqlx::query_as(&sql)
        .bind(branch.id)
        .bind(branch.id)
        .bind(&event_slug)
        .bind(&event_slug)
        .fetch_all(&state.pools.chapter)
        .await?;
    Ok(Json(json!({"items":items,"total":items.len()})))
}

async fn aggregate_timeline(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    let (page, page_size, offset) = list_pagination(&query);
    let branch = query.branch.unwrap_or_default();
    let city = query.city.unwrap_or_default();
    let from = query.from.unwrap_or_default();
    let to = query.to.unwrap_or_default();
    let q = format!("%{}%", query.q.unwrap_or_default().trim());
    let where_sql = "FROM branch_events e JOIN branches b ON b.id=e.branch_id \
        WHERE b.status='active' AND e.status='published' \
        AND e.visibility='public' AND e.moderation_state='normal' \
        AND (e.aggregate_mode='include' OR (e.aggregate_mode='inherit' AND b.default_event_aggregate=1)) \
        AND (?='' OR b.slug=?) AND (?='' OR b.locality_name=?) \
        AND (?='' OR e.starts_at>=?) AND (?='' OR e.starts_at<=?) \
        AND (?='%%' OR e.title LIKE ?)";
    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) {where_sql}"))
        .bind(&branch)
        .bind(&branch)
        .bind(&city)
        .bind(&city)
        .bind(&from)
        .bind(&from)
        .bind(&to)
        .bind(&to)
        .bind(&q)
        .bind(&q)
        .fetch_one(&state.pools.chapter)
        .await?;
    let sql = format!(
        "{TIMELINE_EVENT_SELECT} {} ORDER BY e.starts_at DESC,e.id DESC LIMIT ? OFFSET ?",
        where_sql.trim_start_matches("FROM branch_events e JOIN branches b ON b.id=e.branch_id ")
    );
    let items: Vec<TimelineEventItem> = sqlx::query_as(&sql)
        .bind(&branch)
        .bind(&branch)
        .bind(&city)
        .bind(&city)
        .bind(&from)
        .bind(&from)
        .bind(&to)
        .bind(&to)
        .bind(&q)
        .bind(&q)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pools.chapter)
        .await?;
    Ok(Json(
        json!({"items":items,"total":total,"page":page,"pageSize":page_size}),
    ))
}

async fn timeline_stream(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>> {
    let after_id = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let replay: Vec<i64> = if after_id > 0 {
        let sql = "SELECT t.id FROM branch_event_timeline_entries t \
            JOIN branches b ON b.id=t.branch_id JOIN branch_events e ON e.id=t.event_id \
            WHERE t.id>? AND b.status='active' \
            AND b.default_post_aggregate=1 AND t.status='published' AND t.moderation_state='normal' \
            AND e.status='published' AND e.visibility='public' AND e.moderation_state='normal' \
            ORDER BY t.id LIMIT 200";
        sqlx::query_scalar(sql)
            .bind(after_id)
            .fetch_all(&state.pools.chapter)
            .await?
    } else {
        Vec::new()
    };
    let initial = replay.into_iter().map(|item_id| {
        let id = item_id.to_string();
        let payload = json!({"action":"refresh","id":item_id}).to_string();
        Ok(Event::default().id(id).event("timeline").data(payload))
    });
    let live = BroadcastStream::new(state.chapter_timeline_tx.subscribe()).filter_map(|result| {
        result.ok().map(|payload| {
            let item_id = serde_json::from_str::<Value>(&payload)
                .ok()
                .and_then(|value| value.get("id").and_then(Value::as_i64))
                .unwrap_or(0);
            let id = item_id.to_string();
            let data = json!({"action":"refresh","id":item_id}).to_string();
            Ok(Event::default().id(id).event("timeline").data(data))
        })
    });
    Ok(Sse::new(stream::iter(initial).chain(live)).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("chapter-timeline"),
    ))
}

async fn branch_events(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let sql = format!("{EVENT_SELECT} WHERE (e.branch_id=? OR EXISTS(SELECT 1 FROM branch_event_cohosts c WHERE c.event_id=e.id AND c.branch_id=? AND c.state='accepted')) AND e.status='published' AND e.visibility='public' AND e.moderation_state='normal' ORDER BY e.starts_at,e.id");
    let items: Vec<EventItem> = sqlx::query_as(&sql)
        .bind(branch.id)
        .bind(branch.id)
        .fetch_all(&state.pools.chapter)
        .await?;
    Ok(Json(json!({ "items": items })))
}

async fn get_event(
    State(state): State<AppState>,
    user: Option<AuthUser>,
    Path((slug, event_slug)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let sql = format!("{EVENT_SELECT} WHERE (e.branch_id=? OR EXISTS(SELECT 1 FROM branch_event_cohosts c WHERE c.event_id=e.id AND c.branch_id=? AND c.state='accepted')) AND e.slug=? AND e.status='published' AND e.visibility='public' AND e.moderation_state='normal'");
    let item: EventItem = sqlx::query_as(&sql)
        .bind(branch.id)
        .bind(branch.id)
        .bind(event_slug)
        .fetch_one(&state.pools.chapter)
        .await?;
    let operations = public_event_operations(&state, item.id).await?;
    let registration = registration_summary(&state, &item).await?;
    let attendees = public_event_attendees(&state, item.id).await?;
    let mine = if let Some(user) = user {
        event_registration_for_user(&state, item.id, user.id).await?
    } else {
        Value::Null
    };
    Ok(Json(json!({
        "item": item,
        "operations": operations,
        "registration": registration,
        "attendees": attendees,
        "myRegistration": mine
    })))
}

async fn event_photos(
    State(state): State<AppState>,
    Path((slug, event_slug)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    let branch = active_branch(&state, &slug).await?;
    let items: Vec<EventPhotoItem> = sqlx::query_as(
        "SELECT t.id, t.event_id, t.title, t.content, t.image_path, t.happened_at, t.created_at \
         FROM branch_event_timeline_entries t JOIN branch_events e ON e.id=t.event_id \
         WHERE (e.branch_id=? OR EXISTS(SELECT 1 FROM branch_event_cohosts c \
         WHERE c.event_id=e.id AND c.branch_id=? AND c.state='accepted')) AND e.slug=? \
         AND e.status='published' AND e.visibility='public' AND e.moderation_state='normal' \
         AND t.status='published' AND t.moderation_state='normal' \
         AND t.image_path IS NOT NULL AND TRIM(t.image_path)!='' \
         ORDER BY t.happened_at,t.id",
    )
    .bind(branch.id)
    .bind(branch.id)
    .bind(event_slug)
    .fetch_all(&state.pools.chapter)
    .await?;
    Ok(Json(json!({"items":items})))
}

async fn aggregate_events(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    let (page, page_size, offset) = list_pagination(&query);
    let branch = query.branch.unwrap_or_default();
    let city = query.city.unwrap_or_default();
    let event_type = query.event_type.unwrap_or_default();
    let format_name = query.format.unwrap_or_default();
    let topic = query.topic.unwrap_or_default();
    let period = query.period.unwrap_or_default();
    let from = query.from.unwrap_or_default();
    let to = query.to.unwrap_or_default();
    let q = format!("%{}%", query.q.unwrap_or_default().trim());
    let where_sql = "FROM branch_events e JOIN branches b ON b.id=e.branch_id WHERE b.status='active' AND e.status='published' AND e.visibility='public' AND e.moderation_state='normal' AND (e.aggregate_mode='include' OR (e.aggregate_mode='inherit' AND b.default_event_aggregate=1)) AND (?='' OR b.slug=?) AND (?='' OR b.locality_name=?) AND (?='' OR e.event_type=?) AND (?='' OR e.format=?) AND (?='' OR EXISTS(SELECT 1 FROM branch_event_topics t WHERE t.event_id=e.id AND t.topic=?)) AND (?='' OR e.starts_at>=?) AND (?='' OR e.starts_at<=?) AND (?='%%' OR e.title LIKE ? OR e.summary LIKE ?) AND (?='' OR (?='upcoming' AND datetime(COALESCE(e.ends_at,e.starts_at))>=datetime('now','+8 hours')) OR (?='past' AND datetime(COALESCE(e.ends_at,e.starts_at))<datetime('now','+8 hours')) OR ?='all')";
    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) {where_sql}"))
        .bind(&branch)
        .bind(&branch)
        .bind(&city)
        .bind(&city)
        .bind(&event_type)
        .bind(&event_type)
        .bind(&format_name)
        .bind(&format_name)
        .bind(&topic)
        .bind(&topic)
        .bind(&from)
        .bind(&from)
        .bind(&to)
        .bind(&to)
        .bind(&q)
        .bind(&q)
        .bind(&q)
        .bind(&period)
        .bind(&period)
        .bind(&period)
        .bind(&period)
        .fetch_one(&state.pools.chapter)
        .await?;
    let sql = format!("{EVENT_SELECT} {} ORDER BY CASE WHEN datetime(COALESCE(e.ends_at,e.starts_at))>=datetime('now','+8 hours') THEN 0 ELSE 1 END, CASE WHEN datetime(COALESCE(e.ends_at,e.starts_at))>=datetime('now','+8 hours') THEN datetime(e.starts_at) END ASC, CASE WHEN datetime(COALESCE(e.ends_at,e.starts_at))<datetime('now','+8 hours') THEN datetime(e.starts_at) END DESC, e.id LIMIT ? OFFSET ?", where_sql.trim_start_matches("FROM branch_events e JOIN branches b ON b.id=e.branch_id "));
    let items: Vec<EventItem> = sqlx::query_as(&sql)
        .bind(&branch)
        .bind(&branch)
        .bind(&city)
        .bind(&city)
        .bind(&event_type)
        .bind(&event_type)
        .bind(&format_name)
        .bind(&format_name)
        .bind(&topic)
        .bind(&topic)
        .bind(&from)
        .bind(&from)
        .bind(&to)
        .bind(&to)
        .bind(&q)
        .bind(&q)
        .bind(&q)
        .bind(&period)
        .bind(&period)
        .bind(&period)
        .bind(&period)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pools.chapter)
        .await?;
    Ok(Json(
        json!({ "items": items, "total": total, "page":page, "pageSize":page_size }),
    ))
}

async fn public_event_operations(state: &AppState, event_id: i64) -> AppResult<Value> {
    let topics: Vec<String> = sqlx::query_scalar(
        "SELECT topic FROM branch_event_topics WHERE event_id=? ORDER BY sort_order,topic",
    )
    .bind(event_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let people: Vec<(
        i64,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        i64,
    )> = sqlx::query_as(
        "SELECT id,role,name,title,organization,avatar_path,bio,url,sort_order \
         FROM branch_event_people WHERE event_id=? ORDER BY sort_order,id",
    )
    .bind(event_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let partners: Vec<(i64, String, String, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT id,partner_type,name,logo_path,url,sort_order FROM branch_event_partners \
         WHERE event_id=? ORDER BY sort_order,id",
    )
    .bind(event_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let cohosts: Vec<(i64, i64, String, String, Option<String>)> = sqlx::query_as(
        "SELECT c.id,b.id,b.slug,b.name,bb.logo_path FROM branch_event_cohosts c \
         JOIN branches b ON b.id=c.branch_id LEFT JOIN branch_brand bb ON bb.branch_id=b.id \
         WHERE c.event_id=? AND c.state='accepted' ORDER BY b.name",
    )
    .bind(event_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let questions: Vec<(i64, String, String, bool, Option<String>, i64)> = sqlx::query_as(
        "SELECT id,question_type,label,required,options_json,sort_order \
         FROM branch_event_registration_questions WHERE event_id=? ORDER BY sort_order,id",
    )
    .bind(event_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    Ok(json!({
        "topics": topics,
        "people": people.into_iter().map(|(id,role,name,title,organization,avatar_path,bio,url,sort_order)|json!({"id":id,"role":role,"name":name,"title":title,"organization":organization,"avatarPath":avatar_path,"bio":bio,"url":url,"sortOrder":sort_order})).collect::<Vec<_>>(),
        "partners": partners.into_iter().map(|(id,partner_type,name,logo_path,url,sort_order)|json!({"id":id,"partnerType":partner_type,"name":name,"logoPath":logo_path,"url":url,"sortOrder":sort_order})).collect::<Vec<_>>(),
        "cohosts": cohosts.into_iter().map(|(id,branch_id,slug,name,logo_path)|json!({"id":id,"branchId":branch_id,"slug":slug,"name":name,"logoPath":logo_path})).collect::<Vec<_>>(),
        "questions": questions.into_iter().map(|(id,question_type,label,required,options_json,sort_order)|json!({"id":id,"questionType":question_type,"label":label,"required":required,"options":options_json.and_then(|raw|serde_json::from_str::<Value>(&raw).ok()).unwrap_or_else(||json!([])),"sortOrder":sort_order})).collect::<Vec<_>>()
    }))
}

async fn registration_summary(state: &AppState, event: &EventItem) -> AppResult<Value> {
    let (confirmed, waitlisted): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*) FILTER (WHERE state='confirmed'),COUNT(*) FILTER (WHERE state='waitlisted') \
         FROM branch_event_registrations WHERE event_id=?",
    )
    .bind(event.id)
    .fetch_one(&state.pools.chapter)
    .await?;
    let now = chapter_now_string();
    let state_name = if event.registration_mode == "none" {
        "disabled"
    } else if event
        .registration_opens_at
        .as_deref()
        .is_some_and(|value| comparable_datetime(value).as_str() > now.as_str())
    {
        "not_open"
    } else if event
        .registration_closes_at
        .as_deref()
        .is_some_and(|value| comparable_datetime(value).as_str() < now.as_str())
        || comparable_datetime(event.ends_at.as_deref().unwrap_or(&event.starts_at)).as_str()
            < now.as_str()
    {
        "closed"
    } else if event.capacity.is_some_and(|capacity| confirmed >= capacity) {
        "waitlist"
    } else {
        "open"
    };
    Ok(json!({
        "state":state_name,
        "confirmed":confirmed,
        "waitlisted":waitlisted,
        "capacity":event.capacity,
        "mode":event.registration_mode,
        "admissionMode":event.admission_mode
    }))
}

async fn public_event_attendees(state: &AppState, event_id: i64) -> AppResult<Vec<Value>> {
    let rows: Vec<(i64, i64, String, i64)> = sqlx::query_as(
        "SELECT id,user_id,public_mode,anonymous_number FROM branch_event_registrations \
         WHERE event_id=? AND state='confirmed' ORDER BY created_at,id LIMIT 500",
    )
    .bind(event_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let mut items = Vec::with_capacity(rows.len());
    for (id, user_id, public_mode, anonymous_number) in rows {
        if public_mode == "anonymous" {
            items.push(json!({"id":id,"anonymous":true,"displayName":format!("匿名参与者 {:03}",anonymous_number),"avatar":null}));
            continue;
        }
        if let Some((display_name, avatar)) = public_account(state, user_id).await? {
            items.push(
                json!({"id":id,"anonymous":false,"displayName":display_name,"avatar":avatar}),
            );
        }
    }
    Ok(items)
}

async fn public_account(
    state: &AppState,
    user_id: i64,
) -> AppResult<Option<(String, Option<String>)>> {
    Ok(sqlx::query_as(
        "SELECT COALESCE(NULLIF(nickname,''),NULLIF(display_name,''),username),avatar \
         FROM users WHERE id=? AND status='active' AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(&state.pools.core)
    .await?)
}

async fn event_registration_for_user(
    state: &AppState,
    event_id: i64,
    user_id: i64,
) -> AppResult<Value> {
    let row: Option<(
        i64,
        String,
        Option<String>,
        String,
        i64,
        Option<String>,
        Option<String>,
        String,
        String,
    )> = sqlx::query_as(
        "SELECT id,state,answers_json,public_mode,anonymous_number,review_note,checked_in_at,created_at,updated_at \
         FROM branch_event_registrations WHERE event_id=? AND user_id=?",
    )
    .bind(event_id)
    .bind(user_id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    Ok(row.map_or(Value::Null, |(id,state,answers_json,public_mode,anonymous_number,review_note,checked_in_at,created_at,updated_at)|json!({
        "id":id,"state":state,"answers":answers_json.and_then(|raw|serde_json::from_str::<Value>(&raw).ok()).unwrap_or_else(||json!({})),"publicMode":public_mode,"anonymousNumber":anonymous_number,"reviewNote":review_note,"checkedInAt":checked_in_at,"createdAt":created_at,"updatedAt":updated_at
    })))
}

fn branch_scope(id: i64) -> String {
    id.to_string()
}

async fn allow(state: &AppState, user: &AuthUser, id: i64, capability: &str) -> AppResult<()> {
    authorize_capability(
        &state.pools.core,
        user,
        capability,
        "branch",
        &branch_scope(id),
    )
    .await
}

async fn allow_any_branch_access(state: &AppState, user: &AuthUser, id: i64) -> AppResult<()> {
    if user.is_super {
        return Ok(());
    }
    let grants = capability_grants(&state.pools.core, user.id).await?;
    let scope = id.to_string();
    if grants.iter().any(|grant| {
        (grant.scope_type == "branch" && grant.scope_id == scope)
            || (grant.scope_type == "platform" && grant.scope_id == "chapter")
    }) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

async fn allow_media_upload(state: &AppState, user: &AuthUser, id: i64) -> AppResult<()> {
    if user.is_super {
        return Ok(());
    }
    let grants = capability_grants(&state.pools.core, user.id).await?;
    let scope = id.to_string();
    if grants.iter().any(|grant| {
        MEDIA_UPLOAD_CAPABILITIES.contains(&grant.capability.as_str())
            && ((grant.scope_type == "branch" && grant.scope_id == scope)
                || (grant.scope_type == "platform" && grant.scope_id == "chapter"))
    }) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

async fn audit(
    state: &AppState,
    branch_id: Option<i64>,
    actor_id: i64,
    action: &str,
    entity_type: &str,
    entity_id: impl ToString,
    detail: Option<&Value>,
) {
    let _ = sqlx::query(
        "INSERT INTO branch_audit_log (branch_id, actor_id, action, entity_type, entity_id, detail_json) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(branch_id)
    .bind(actor_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id.to_string())
    .bind(detail.map(Value::to_string))
    .execute(&state.pools.chapter)
    .await;
}

async fn admin_branches(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let items: Vec<Branch> = if user.is_super {
        sqlx::query_as("SELECT id, slug, name, short_name, summary, about_text, join_text, country_code, locality_name, founded_on, status, default_post_aggregate, default_event_aggregate, created_at, updated_at FROM branches ORDER BY name")
            .fetch_all(&state.pools.chapter).await?
    } else {
        let grants = capability_grants(&state.pools.core, user.id).await?;
        if grants
            .iter()
            .any(|g| g.scope_type == "platform" && g.scope_id == "chapter")
        {
            sqlx::query_as("SELECT id, slug, name, short_name, summary, about_text, join_text, country_code, locality_name, founded_on, status, default_post_aggregate, default_event_aggregate, created_at, updated_at FROM branches ORDER BY name")
                .fetch_all(&state.pools.chapter).await?
        } else {
            let mut rows = Vec::new();
            let mut ids: Vec<i64> = grants
                .iter()
                .filter(|g| g.scope_type == "branch")
                .filter_map(|g| g.scope_id.parse().ok())
                .collect();
            ids.sort_unstable();
            ids.dedup();
            for id in ids {
                if let Some(row) = sqlx::query_as("SELECT id, slug, name, short_name, summary, about_text, join_text, country_code, locality_name, founded_on, status, default_post_aggregate, default_event_aggregate, created_at, updated_at FROM branches WHERE id = ?")
                    .bind(id).fetch_optional(&state.pools.chapter).await? { rows.push(row); }
            }
            rows
        }
    };
    Ok(Json(json!({ "items": items })))
}

fn required_string(body: &Value, key: &str, label: &str) -> AppResult<String> {
    let value = body.get(key).and_then(Value::as_str).unwrap_or("").trim();
    if value.is_empty() {
        Err(AppError::bad_request(format!("{label}不能为空")))
    } else {
        Ok(value.to_string())
    }
}

fn opt_string(body: &Value, key: &str) -> Option<String> {
    body.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn validated_url(
    body: &Value,
    key: &str,
    label: &str,
    allowed_schemes: &[&str],
) -> AppResult<Option<String>> {
    let Some(value) = opt_string(body, key) else {
        return Ok(None);
    };
    let Some((scheme, _)) = value.split_once(':') else {
        return Err(AppError::bad_request(format!("{label}必须是完整链接")));
    };
    if !allowed_schemes
        .iter()
        .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
    {
        return Err(AppError::bad_request(format!(
            "{label}仅支持 {} 协议",
            allowed_schemes.join("、")
        )));
    }
    Ok(Some(value))
}

fn public_url(body: &Value, key: &str, label: &str) -> AppResult<Option<String>> {
    validated_url(body, key, label, &["http", "https"])
}

fn contact_url(body: &Value, key: &str, label: &str) -> AppResult<Option<String>> {
    validated_url(body, key, label, &["http", "https", "mailto", "tel"])
}

fn chapter_now_string() -> String {
    // Chapter 已移除支部独立时区，所有无时区业务时间统一按 Asia/Shanghai 解释。
    (chrono::Utc::now() + chrono::Duration::hours(CHAPTER_UTC_OFFSET_HOURS))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 64
        && slug
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

async fn create_branch(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize_capability(
        &state.pools.core,
        &user,
        "branch.lifecycle.manage",
        "platform",
        "chapter",
    )
    .await?;
    let slug = required_string(&body, "slug", "Slug")?.to_lowercase();
    let name = required_string(&body, "name", "支部名称")?;
    if !valid_slug(&slug) {
        return Err(AppError::bad_request("Slug 只能包含小写字母、数字和连字符"));
    }
    let mut tx = state.pools.chapter.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO branches (slug, name, short_name, summary, locality_name, status) \
         VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&slug)
    .bind(&name)
    .bind(opt_string(&body, "shortName"))
    .bind(opt_string(&body, "summary"))
    .bind(opt_string(&body, "localityName"))
    .bind(
        body.get("status")
            .and_then(Value::as_str)
            .unwrap_or("draft"),
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO branch_brand (branch_id, logo_alt) VALUES (?, ?)")
        .bind(id)
        .bind(&name)
        .execute(&mut *tx)
        .await?;
    for (order, key) in ["organization", "members", "events", "join"]
        .iter()
        .enumerate()
    {
        sqlx::query(
            "INSERT INTO branch_sections (branch_id, section_key, sort_order) VALUES (?, ?, ?)",
        )
        .bind(id)
        .bind(key)
        .bind(order as i64)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "branch.create",
        "branch",
        id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({ "id": id, "slug": slug })))
}

async fn admin_branch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    allow_any_branch_access(&state, &user, id).await?;
    let grants = capability_grants(&state.pools.core, user.id).await?;
    let branch_scope = id.to_string();
    let can = |capability: &str| {
        user.is_super
            || grants.iter().any(|grant| {
                grant.capability == capability
                    && ((grant.scope_type == "branch" && grant.scope_id == branch_scope)
                        || (grant.scope_type == "platform" && grant.scope_id == "chapter"))
            })
    };
    let branch: Branch = sqlx::query_as("SELECT id, slug, name, short_name, summary, about_text, join_text, country_code, locality_name, founded_on, status, default_post_aggregate, default_event_aggregate, created_at, updated_at FROM branches WHERE id = ?")
        .bind(id).fetch_one(&state.pools.chapter).await?;
    let (people, methods) = if can("branch.contacts.manage") {
        contacts_for(&state, id, false).await?
    } else {
        (Vec::new(), Vec::new())
    };
    let can_read_members = can("branch.members.manage")
        || can("branch.contacts.manage")
        || can("branch.organization.manage");
    Ok(Json(json!({
        "branch": branch,
        "brand": brand(&state, id).await?,
        "sections": if can("branch.profile.manage") { sections(&state, id, false).await? } else { Vec::new() },
        "merchandise": if can("branch.profile.manage") { merchandise_for(&state, id, false).await? } else { Vec::new() },
        "members": if can_read_members { members_for(&state, id, false).await? } else { Vec::new() },
        "qqGroups": if can("branch.contacts.manage") { qq_for(&state, id, false).await? } else { Vec::new() },
        "contacts": { "people": people, "methods": methods },
        "organization": if can("branch.organization.manage") { admin_organization(&state, id).await? } else { Value::Null }
    })))
}

async fn update_branch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.profile.manage").await?;
    let name = required_string(&body, "name", "支部名称")?;
    let current_status: String = sqlx::query_scalar("SELECT status FROM branches WHERE id=?")
        .bind(id)
        .fetch_optional(&state.pools.chapter)
        .await?
        .ok_or_else(|| AppError::not_found("支部不存在"))?;
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or(&current_status);
    if !["draft", "active", "paused", "archived"].contains(&status) {
        return Err(AppError::bad_request("支部状态无效"));
    }
    if status != current_status {
        allow(&state, &user, id, "branch.lifecycle.manage").await?;
    }
    sqlx::query("UPDATE branches SET name=?, short_name=?, summary=?, about_text=?, join_text=?, country_code=?, locality_name=?, founded_on=?, status=?, default_post_aggregate=?, default_event_aggregate=?, updated_at=datetime('now') WHERE id=?")
        .bind(name)
        .bind(opt_string(&body,"shortName")).bind(opt_string(&body,"summary"))
        .bind(opt_string(&body,"aboutText")).bind(opt_string(&body,"joinText"))
        .bind(body.get("countryCode").and_then(Value::as_str).unwrap_or("CN"))
        .bind(opt_string(&body,"localityName"))
        .bind(opt_string(&body,"foundedOn"))
        .bind(status)
        .bind(body.get("defaultPostAggregate").and_then(Value::as_bool).unwrap_or(true))
        .bind(body.get("defaultEventAggregate").and_then(Value::as_bool).unwrap_or(true))
        .bind(id).execute(&state.pools.chapter).await?;
    audit(
        &state,
        Some(id),
        user.id,
        "branch.update",
        "branch",
        id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}

async fn update_brand(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.brand.manage").await?;
    let accent = body
        .get("accentKey")
        .and_then(Value::as_str)
        .unwrap_or("blue");
    if !["blue", "red", "amber", "green", "purple", "cyan"].contains(&accent) {
        return Err(AppError::bad_request("强调色不在允许列表中"));
    }
    let logo_path = opt_string(&body, "logoPath");
    sqlx::query("INSERT INTO branch_brand (branch_id,logo_path,logo_alt,cover_path,cover_focal_x,cover_focal_y,tagline,accent_key) VALUES (?,?,?,?,?,?,?,?) ON CONFLICT(branch_id) DO UPDATE SET logo_path=excluded.logo_path,logo_alt=excluded.logo_alt,cover_path=excluded.cover_path,cover_focal_x=excluded.cover_focal_x,cover_focal_y=excluded.cover_focal_y,tagline=excluded.tagline,accent_key=excluded.accent_key,updated_at=datetime('now')")
        .bind(id).bind(logo_path).bind(opt_string(&body,"logoAlt"))
        .bind(opt_string(&body,"coverPath"))
        .bind(body.get("coverFocalX").and_then(Value::as_f64).unwrap_or(0.5).clamp(0.0,1.0))
        .bind(body.get("coverFocalY").and_then(Value::as_f64).unwrap_or(0.5).clamp(0.0,1.0))
        .bind(opt_string(&body,"tagline")).bind(accent).execute(&state.pools.chapter).await?;
    audit(
        &state,
        Some(id),
        user.id,
        "brand.update",
        "brand",
        id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn replace_sections(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.profile.manage").await?;
    let items = body
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::bad_request("items 必须是数组"))?;
    let mut tx = state.pools.chapter.begin().await?;
    for item in items {
        sqlx::query("UPDATE branch_sections SET label=?, enabled=?, visibility=?, sort_order=? WHERE branch_id=? AND section_key=?")
            .bind(opt_string(item,"label"))
            .bind(item.get("enabled").and_then(Value::as_bool).unwrap_or(true))
            .bind(item.get("visibility").and_then(Value::as_str).unwrap_or("public"))
            .bind(item.get("sortOrder").and_then(Value::as_i64).unwrap_or(0))
            .bind(id).bind(required_string(item,"sectionKey","栏目")?).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "sections.replace",
        "section",
        id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn replace_merchandise(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.profile.manage").await?;
    let items = body
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::bad_request("items 必须是数组"))?;
    let mut tx = state.pools.chapter.begin().await?;
    let existing_ids: HashSet<i64> =
        sqlx::query_scalar("SELECT id FROM branch_merchandise WHERE branch_id=?")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .collect();
    let mut retained_ids = HashSet::new();

    for (index, item) in items.iter().enumerate() {
        let item_id = item.get("id").and_then(Value::as_i64);
        let name = required_string(item, "name", "周边名称")?;
        let description = opt_string(item, "description");
        let image_path = opt_string(item, "imagePath");
        let status = item
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("draft");
        if !["draft", "published"].contains(&status) {
            return Err(AppError::bad_request("周边状态无效"));
        }
        if status == "published" && image_path.is_none() {
            return Err(AppError::bad_request("公开周边前请上传展示图片"));
        }

        let mut seen_tags = HashSet::new();
        let mut tags = Vec::new();
        if let Some(values) = item.get("tags").and_then(Value::as_array) {
            for value in values {
                let tag = value.as_str().unwrap_or_default().trim();
                if !tag.is_empty() && seen_tags.insert(tag.to_string()) {
                    tags.push(tag.to_string());
                }
            }
        }
        let tags_json =
            serde_json::to_string(&tags).map_err(|_| AppError::bad_request("周边标签格式无效"))?;
        let sort_order = item
            .get("sortOrder")
            .and_then(Value::as_i64)
            .unwrap_or(index as i64);

        let saved_id = if let Some(item_id) = item_id {
            if !existing_ids.contains(&item_id) {
                return Err(AppError::bad_request("周边不属于当前支部"));
            }
            sqlx::query(
                "UPDATE branch_merchandise SET name=?,description=?,image_path=?,tags_json=?,status=?,sort_order=?,updated_at=datetime('now') WHERE id=? AND branch_id=?",
            )
            .bind(name)
            .bind(description)
            .bind(image_path)
            .bind(tags_json)
            .bind(status)
            .bind(sort_order)
            .bind(item_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
            item_id
        } else {
            sqlx::query_scalar(
                "INSERT INTO branch_merchandise(branch_id,name,description,image_path,tags_json,status,sort_order) VALUES (?,?,?,?,?,?,?) RETURNING id",
            )
            .bind(id)
            .bind(name)
            .bind(description)
            .bind(image_path)
            .bind(tags_json)
            .bind(status)
            .bind(sort_order)
            .fetch_one(&mut *tx)
            .await?
        };
        retained_ids.insert(saved_id);
    }

    for item_id in existing_ids.difference(&retained_ids) {
        sqlx::query("DELETE FROM branch_merchandise WHERE id=? AND branch_id=?")
            .bind(item_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "merchandise.replace",
        "merchandise",
        id,
        Some(&body),
    )
    .await;
    let saved = merchandise_for(&state, id, false).await?;
    Ok(Json(json!({"ok":true,"items":saved})))
}

async fn replace_members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.members.manage").await?;
    let items = body
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::bad_request("items 必须是数组"))?;
    let mut tx = state.pools.chapter.begin().await?;
    let existing_ids: HashSet<i64> =
        sqlx::query_scalar("SELECT id FROM branch_members WHERE branch_id=?")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .collect();
    let mut retained_ids = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        let member_id = item.get("id").and_then(Value::as_i64);
        let status = item
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("active");
        let user_id = item.get("userId").and_then(Value::as_i64);
        let account = if let Some(user_id) = user_id {
            public_account(&state, user_id).await?
        } else {
            None
        };
        if status == "active" {
            let user_id = user_id
                .ok_or_else(|| AppError::bad_request("只有已加入本支部的账号才能设置为在任成员"))?;
            let is_joined: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM branch_memberships WHERE branch_id=? AND user_id=? AND ended_at IS NULL)",
            )
            .bind(id)
            .bind(user_id)
            .fetch_one(&state.pools.chapter)
            .await?;
            if !is_joined || account.is_none() {
                return Err(AppError::bad_request(
                    "只有当前已加入本支部的有效账号才能设置为在任成员",
                ));
            }
        }
        let (display_name, avatar_path) = account.unwrap_or_else(|| {
            (
                item.get("displayName")
                    .and_then(Value::as_str)
                    .unwrap_or("历史成员")
                    .trim()
                    .to_string(),
                opt_string(item, "avatarPath"),
            )
        });
        if display_name.is_empty() {
            return Err(AppError::bad_request("成员名称不能为空"));
        }
        let sort_order = item
            .get("sortOrder")
            .and_then(Value::as_i64)
            .unwrap_or(index as i64);
        let saved_id = if let Some(member_id) = member_id {
            if !existing_ids.contains(&member_id) {
                return Err(AppError::bad_request("成员不属于当前支部"));
            }
            sqlx::query("UPDATE branch_members SET user_id=?,display_name=?,avatar_path=?,bio=?,status=?,joined_on=?,left_on=?,is_public=?,sort_order=?,updated_at=datetime('now') WHERE id=? AND branch_id=?")
                .bind(user_id)
                .bind(display_name).bind(avatar_path).bind(opt_string(item,"bio"))
                .bind(status)
                .bind(opt_string(item,"joinedOn")).bind(opt_string(item,"leftOn"))
                .bind(item.get("isPublic").and_then(Value::as_bool).unwrap_or(true))
                .bind(sort_order).bind(member_id).bind(id).execute(&mut *tx).await?;
            member_id
        } else {
            sqlx::query_scalar("INSERT INTO branch_members (branch_id,user_id,display_name,avatar_path,bio,status,joined_on,left_on,is_public,sort_order) VALUES (?,?,?,?,?,?,?,?,?,?) RETURNING id")
                .bind(id).bind(user_id)
                .bind(display_name).bind(avatar_path).bind(opt_string(item,"bio"))
                .bind(status)
                .bind(opt_string(item,"joinedOn")).bind(opt_string(item,"leftOn"))
                .bind(item.get("isPublic").and_then(Value::as_bool).unwrap_or(true))
                .bind(sort_order).fetch_one(&mut *tx).await?
        };
        retained_ids.insert(saved_id);
    }
    for member_id in existing_ids.difference(&retained_ids) {
        sqlx::query("DELETE FROM branch_members WHERE id=? AND branch_id=?")
            .bind(member_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "members.replace",
        "member",
        id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({"ok":true,"ids":retained_ids})))
}

async fn admin_memberships(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.members.manage").await?;
    let rows: Vec<(i64, i64, String, String)> = sqlx::query_as(
        "SELECT id,user_id,state,joined_at FROM branch_memberships \
         WHERE branch_id=? AND ended_at IS NULL ORDER BY joined_at,id",
    )
    .bind(id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let mut items = Vec::with_capacity(rows.len());
    for (membership_id, user_id, membership_state, joined_at) in rows {
        let account: Option<(String, Option<String>, String, String)> = sqlx::query_as(
            "SELECT COALESCE(NULLIF(nickname,''),NULLIF(display_name,''),username),avatar,username,email \
             FROM users WHERE id=? AND status='active' AND deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_optional(&state.pools.core)
        .await?;
        let Some((display_name, avatar, username, email)) = account else {
            continue;
        };
        let official: Option<(i64, String)> =
            sqlx::query_as("SELECT id,status FROM branch_members WHERE branch_id=? AND user_id=?")
                .bind(id)
                .bind(user_id)
                .fetch_optional(&state.pools.chapter)
                .await?;
        items.push(json!({
            "membershipId":membership_id,"userId":user_id,"state":membership_state,
            "joinedAt":joined_at,"displayName":display_name,"avatar":avatar,
            "username":username,"email":email,
            "officialMemberId":official.as_ref().map(|row|row.0),
            "officialMemberStatus":official.map(|row|row.1)
        }));
    }
    let requests: Vec<(i64, i64, String, String, String)> = sqlx::query_as(
        "SELECT r.id,r.membership_id,r.reason,r.state,r.requested_at \
         FROM branch_membership_leave_requests r JOIN branch_memberships m ON m.id=r.membership_id \
         WHERE m.branch_id=? AND r.state='pending' ORDER BY r.requested_at,r.id",
    )
    .bind(id)
    .fetch_all(&state.pools.chapter)
    .await?;
    Ok(Json(json!({
        "items":items,
        "leaveRequests":requests.into_iter().map(|(request_id,membership_id,reason,state_name,requested_at)|json!({"id":request_id,"membershipId":membership_id,"reason":reason,"state":state_name,"requestedAt":requested_at})).collect::<Vec<_>>()
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewLeaveInput {
    action: String,
    review_note: Option<String>,
}

async fn review_leave_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, request_id)): Path<(i64, i64)>,
    Json(body): Json<ReviewLeaveInput>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.members.manage").await?;
    if !["approve", "reject"].contains(&body.action.as_str()) {
        return Err(AppError::bad_request("操作必须是 approve 或 reject"));
    }
    let row: Option<(i64, i64, i64)> = sqlx::query_as(
        "SELECT r.membership_id,m.user_id,m.branch_id FROM branch_membership_leave_requests r \
         JOIN branch_memberships m ON m.id=r.membership_id \
         WHERE r.id=? AND r.state='pending' AND m.branch_id=? AND m.ended_at IS NULL",
    )
    .bind(request_id)
    .bind(id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    let Some((membership_id, member_user_id, _)) = row else {
        return Err(AppError::not_found("退出申请不存在或已经处理"));
    };
    let mut tx = state.pools.chapter.begin().await?;
    if body.action == "approve" {
        sqlx::query("UPDATE branch_membership_leave_requests SET state='approved',reviewed_by=?,review_note=?,reviewed_at=datetime('now') WHERE id=?")
            .bind(user.id).bind(body.review_note.as_deref()).bind(request_id).execute(&mut *tx).await?;
        sqlx::query("UPDATE branch_memberships SET state='ended',ended_at=datetime('now'),ended_by=?,end_reason='leave_approved',updated_at=datetime('now') WHERE id=?")
            .bind(user.id).bind(membership_id).execute(&mut *tx).await?;
        let official_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT id FROM branch_members WHERE branch_id=? AND user_id=? AND status='active'",
        )
        .bind(id)
        .bind(member_user_id)
        .fetch_all(&mut *tx)
        .await?;
        for official_id in official_ids {
            sqlx::query("UPDATE branch_members SET status='alumni',left_on=date('now'),updated_at=datetime('now') WHERE id=?")
                .bind(official_id).execute(&mut *tx).await?;
            sqlx::query("UPDATE organization_assignments SET term_end=COALESCE(term_end,date('now')) WHERE member_id=?")
                .bind(official_id).execute(&mut *tx).await?;
        }
    } else {
        sqlx::query("UPDATE branch_membership_leave_requests SET state='rejected',reviewed_by=?,review_note=?,reviewed_at=datetime('now') WHERE id=?")
            .bind(user.id).bind(body.review_note.as_deref()).bind(request_id).execute(&mut *tx).await?;
        sqlx::query(
            "UPDATE branch_memberships SET state='active',updated_at=datetime('now') WHERE id=?",
        )
        .bind(membership_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    let detail = json!({"hasReviewNote":body.review_note.as_deref().is_some_and(|note|!note.trim().is_empty())});
    audit(
        &state,
        Some(id),
        user.id,
        if body.action == "approve" {
            "membership.leave_approve"
        } else {
            "membership.leave_reject"
        },
        "leave_request",
        request_id,
        Some(&detail),
    )
    .await;
    Ok(Json(
        json!({"ok":true,"state":if body.action=="approve"{"approved"}else{"rejected"}}),
    ))
}

async fn replace_qq_groups(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.contacts.manage").await?;
    let items = body
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::bad_request("items 必须是数组"))?;
    let mut tx = state.pools.chapter.begin().await?;
    sqlx::query("DELETE FROM branch_qq_groups WHERE branch_id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for (index, item) in items.iter().enumerate() {
        let join_url = public_url(item, "joinUrl", "加群链接")?;
        sqlx::query("INSERT INTO branch_qq_groups (branch_id,name,group_number,description,audience_label,join_url,qr_image_path,join_instructions,is_primary,status,sort_order,last_verified_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(id).bind(required_string(item,"name","群名称")?).bind(required_string(item,"groupNumber","QQ群号")?)
            .bind(opt_string(item,"description")).bind(opt_string(item,"audienceLabel")).bind(join_url)
            .bind(opt_string(item,"qrImagePath")).bind(opt_string(item,"joinInstructions"))
            .bind(item.get("isPrimary").and_then(Value::as_bool).unwrap_or(false))
            .bind(item.get("status").and_then(Value::as_str).unwrap_or("active"))
            .bind(item.get("sortOrder").and_then(Value::as_i64).unwrap_or(index as i64)).bind(opt_string(item,"lastVerifiedAt"))
            .execute(&mut *tx).await?;
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "qq.replace",
        "qq_group",
        id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn replace_contacts(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.contacts.manage").await?;
    let people = body
        .get("people")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::bad_request("people 必须是数组"))?;
    let mut tx = state.pools.chapter.begin().await?;
    sqlx::query("DELETE FROM branch_contact_people WHERE branch_id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for (index, person) in people.iter().enumerate() {
        let public = person
            .get("isPublic")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let consent = opt_string(person, "consentConfirmedAt");
        if public && consent.is_none() {
            return Err(AppError::bad_request("公开负责人必须确认公开授权"));
        }
        let person_id:i64=sqlx::query_scalar("INSERT INTO branch_contact_people (branch_id,member_id,display_name,role_title,responsibility,is_primary,is_public,consent_confirmed_at,status,sort_order) VALUES (?,?,?,?,?,?,?,?,?,?) RETURNING id")
            .bind(id).bind(person.get("memberId").and_then(Value::as_i64)).bind(required_string(person,"displayName","负责人名称")?)
            .bind(opt_string(person,"roleTitle")).bind(opt_string(person,"responsibility"))
            .bind(person.get("isPrimary").and_then(Value::as_bool).unwrap_or(false)).bind(public).bind(consent)
            .bind(person.get("status").and_then(Value::as_str).unwrap_or("active"))
            .bind(person.get("sortOrder").and_then(Value::as_i64).unwrap_or(index as i64)).fetch_one(&mut *tx).await?;
        if let Some(methods) = person.get("methods").and_then(Value::as_array) {
            for (method_index, method) in methods.iter().enumerate() {
                let method_url = contact_url(method, "url", "联系方式链接")?;
                sqlx::query("INSERT INTO branch_contact_methods (person_id,method_type,label,value,url,is_public,sort_order) VALUES (?,?,?,?,?,?,?)")
                    .bind(person_id).bind(required_string(method,"methodType","联系方式类型")?).bind(opt_string(method,"label"))
                    .bind(required_string(method,"value","联系方式")?).bind(method_url)
                    .bind(method.get("isPublic").and_then(Value::as_bool).unwrap_or(true))
                    .bind(method.get("sortOrder").and_then(Value::as_i64).unwrap_or(method_index as i64)).execute(&mut *tx).await?;
            }
        }
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "contacts.replace",
        "contact",
        id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn replace_organization(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.organization.manage").await?;
    let name = required_string(&body, "name", "组织版本名称")?;
    let units = body
        .get("units")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let assignments = body
        .get("assignments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut tx = state.pools.chapter.begin().await?;
    sqlx::query("UPDATE organization_versions SET state='historical', ended_on=date('now') WHERE branch_id=? AND state='current'").bind(id).execute(&mut *tx).await?;
    let version_id:i64=sqlx::query_scalar("INSERT INTO organization_versions (branch_id,name,effective_on,state,display_mode,summary) VALUES (?,?,date('now'),'current',?,?) RETURNING id")
        .bind(id).bind(name).bind(body.get("displayMode").and_then(Value::as_str).unwrap_or("tree")).bind(opt_string(&body,"summary")).fetch_one(&mut *tx).await?;
    let mut keys = HashMap::<String, i64>::new();
    for (index, unit) in units.iter().enumerate() {
        let key = required_string(unit, "key", "组织单元 key")?;
        let parent_id = unit
            .get("parentKey")
            .and_then(Value::as_str)
            .and_then(|k| keys.get(k))
            .copied();
        let unit_id:i64=sqlx::query_scalar("INSERT INTO organization_units (version_id,parent_id,name,kind,description,is_public,sort_order) VALUES (?,?,?,?,?,?,?) RETURNING id")
            .bind(version_id).bind(parent_id).bind(required_string(unit,"name","组织单元名称")?)
            .bind(unit.get("kind").and_then(Value::as_str).unwrap_or("group")).bind(opt_string(unit,"description"))
            .bind(unit.get("isPublic").and_then(Value::as_bool).unwrap_or(true))
            .bind(unit.get("sortOrder").and_then(Value::as_i64).unwrap_or(index as i64)).fetch_one(&mut *tx).await?;
        keys.insert(key, unit_id);
    }
    for (index, item) in assignments.iter().enumerate() {
        let unit_id = item
            .get("unitKey")
            .and_then(Value::as_str)
            .and_then(|k| keys.get(k))
            .copied();
        let member_id = item
            .get("memberId")
            .and_then(Value::as_i64)
            .ok_or_else(|| AppError::bad_request("任职缺少 memberId"))?;
        let eligible:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM branch_members m JOIN branch_memberships bm ON bm.branch_id=m.branch_id AND bm.user_id=m.user_id AND bm.ended_at IS NULL WHERE m.id=? AND m.branch_id=? AND m.status='active' AND m.user_id IS NOT NULL)")
            .bind(member_id).bind(id).fetch_one(&mut *tx).await?;
        if !eligible {
            return Err(AppError::bad_request(
                "组织任职只能选择当前已加入且处于在任状态的成员",
            ));
        }
        sqlx::query("INSERT INTO organization_assignments (version_id,unit_id,member_id,title,term_start,term_end,is_public,is_contact,sort_order) VALUES (?,?,?,?,?,?,?,?,?)")
            .bind(version_id).bind(unit_id).bind(member_id)
            .bind(opt_string(item,"title")).bind(opt_string(item,"termStart")).bind(opt_string(item,"termEnd"))
            .bind(item.get("isPublic").and_then(Value::as_bool).unwrap_or(true)).bind(item.get("isContact").and_then(Value::as_bool).unwrap_or(false))
            .bind(item.get("sortOrder").and_then(Value::as_i64).unwrap_or(index as i64)).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "organization.publish",
        "organization",
        version_id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({"id":version_id})))
}

async fn admin_timeline(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.timeline.write").await?;
    let sql = format!(
        "{TIMELINE_SELECT} WHERE t.branch_id=? AND t.status!='deleted' \
         ORDER BY t.happened_at DESC,t.id DESC"
    );
    let items: Vec<TimelineItem> = sqlx::query_as(&sql)
        .bind(id)
        .fetch_all(&state.pools.chapter)
        .await?;
    Ok(Json(json!({"items":items})))
}

async fn validate_timeline_event(
    state: &AppState,
    branch_id: i64,
    event_id: i64,
    published: bool,
) -> AppResult<()> {
    let event_status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM branch_events WHERE id=? AND branch_id=? AND moderation_state='normal'",
    )
    .bind(event_id)
    .bind(branch_id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    let Some(event_status) = event_status else {
        return Err(AppError::bad_request("相册照片只能关联本支部活动"));
    };
    if published && event_status != "published" {
        return Err(AppError::bad_request("只有已发布活动才能公开相册照片"));
    }
    Ok(())
}

async fn broadcast_timeline(state: &AppState, item_id: i64, action: &str) {
    let payload = json!({"id":item_id,"action":action,"refresh":true}).to_string();
    let _ = state.chapter_timeline_tx.send(payload);
}

async fn create_timeline_entry(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.timeline.write").await?;
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft");
    let event_id = body
        .get("eventId")
        .and_then(Value::as_i64)
        .ok_or_else(|| AppError::bad_request("请选择关联活动"))?;
    validate_timeline_event(&state, id, event_id, status == "published").await?;
    let image_path = opt_string(&body, "imagePath");
    if status == "published" && image_path.is_none() {
        return Err(AppError::bad_request("公开相册照片前请先上传图片"));
    }
    if status == "published" {
        allow(&state, &user, id, "branch.timeline.publish").await?;
    }
    let item_id:i64=sqlx::query_scalar("INSERT INTO branch_event_timeline_entries(branch_id,event_id,title,content,image_path,happened_at,location_name,status,created_by,updated_by) VALUES (?,?,?,?,?,?,?,?,?,?) RETURNING id")
        .bind(id).bind(event_id).bind(required_string(&body,"title","照片标题")?)
        .bind(opt_string(&body,"content")).bind(image_path)
        .bind(required_string(&body,"happenedAt","拍摄时间")?).bind(opt_string(&body,"locationName"))
        .bind(status).bind(user.id).bind(user.id).fetch_one(&state.pools.chapter).await?;
    audit(
        &state,
        Some(id),
        user.id,
        "timeline.create",
        "timeline",
        item_id,
        Some(&body),
    )
    .await;
    broadcast_timeline(&state, item_id, "upsert").await;
    Ok(Json(json!({"id":item_id})))
}

async fn update_timeline_entry(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, item_id)): Path<(i64, i64)>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.timeline.write").await?;
    let current_status: String = sqlx::query_scalar(
        "SELECT status FROM branch_event_timeline_entries WHERE id=? AND branch_id=?",
    )
    .bind(item_id)
    .bind(id)
    .fetch_optional(&state.pools.chapter)
    .await?
    .ok_or_else(|| AppError::not_found("相册照片不存在"))?;
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or(&current_status);
    let event_id = body
        .get("eventId")
        .and_then(Value::as_i64)
        .ok_or_else(|| AppError::bad_request("请选择关联活动"))?;
    validate_timeline_event(&state, id, event_id, status == "published").await?;
    let image_path = opt_string(&body, "imagePath");
    if status == "published" && image_path.is_none() {
        return Err(AppError::bad_request("公开相册照片前请先上传图片"));
    }
    if current_status == "published" || status == "published" {
        allow(&state, &user, id, "branch.timeline.publish").await?;
    }
    let result = sqlx::query("UPDATE branch_event_timeline_entries SET event_id=?,title=?,content=?,image_path=?,happened_at=?,location_name=?,status=?,updated_by=?,updated_at=datetime('now') WHERE id=? AND branch_id=?")
        .bind(event_id).bind(required_string(&body,"title","照片标题")?).bind(opt_string(&body,"content"))
        .bind(image_path).bind(required_string(&body,"happenedAt","拍摄时间")?)
        .bind(opt_string(&body,"locationName")).bind(status).bind(user.id).bind(item_id).bind(id)
        .execute(&state.pools.chapter).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("相册照片不存在"));
    }
    audit(
        &state,
        Some(id),
        user.id,
        "timeline.update",
        "timeline",
        item_id,
        Some(&body),
    )
    .await;
    broadcast_timeline(&state, item_id, "upsert").await;
    Ok(Json(json!({"ok":true})))
}

async fn delete_timeline_entry(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, item_id)): Path<(i64, i64)>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.timeline.write").await?;
    let current_status: String = sqlx::query_scalar(
        "SELECT status FROM branch_event_timeline_entries WHERE id=? AND branch_id=?",
    )
    .bind(item_id)
    .bind(id)
    .fetch_optional(&state.pools.chapter)
    .await?
    .ok_or_else(|| AppError::not_found("相册照片不存在"))?;
    if current_status == "published" {
        allow(&state, &user, id, "branch.timeline.publish").await?;
    }
    sqlx::query("UPDATE branch_event_timeline_entries SET status='deleted',updated_by=?,updated_at=datetime('now') WHERE id=? AND branch_id=?").bind(user.id).bind(item_id).bind(id).execute(&state.pools.chapter).await?;
    audit(
        &state,
        Some(id),
        user.id,
        "timeline.delete",
        "timeline",
        item_id,
        None,
    )
    .await;
    broadcast_timeline(&state, item_id, "delete").await;
    Ok(Json(json!({"ok":true})))
}

async fn admin_events(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.write").await?;
    let sql = format!(
        "{EVENT_SELECT} WHERE e.branch_id=? AND e.status!='deleted' ORDER BY e.starts_at DESC"
    );
    let items: Vec<EventItem> = sqlx::query_as(&sql)
        .bind(id)
        .fetch_all(&state.pools.chapter)
        .await?;
    Ok(Json(json!({"items":items})))
}

async fn create_event(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.write").await?;
    let online_url = public_url(&body, "onlineUrl", "线上活动链接")?;
    let registration_url = public_url(&body, "registrationUrl", "外部报名链接")?;
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft");
    if status == "published" {
        allow(&state, &user, id, "branch.events.publish").await?;
    }
    let (starts_at, ends_at) = event_time_values(&body, status)?;
    let format_name = body
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("in_person");
    let registration_mode = body
        .get("registrationMode")
        .and_then(Value::as_str)
        .unwrap_or_else(|| {
            if registration_url.is_some() {
                "external"
            } else {
                "none"
            }
        });
    let admission_mode = body
        .get("admissionMode")
        .and_then(Value::as_str)
        .unwrap_or("automatic");
    if !["in_person", "online", "hybrid"].contains(&format_name)
        || !["none", "internal", "external", "both"].contains(&registration_mode)
        || !["automatic", "review"].contains(&admission_mode)
    {
        return Err(AppError::bad_request("活动形式或报名设置无效"));
    }
    if ["external", "both"].contains(&registration_mode) && registration_url.is_none() {
        return Err(AppError::bad_request("外部报名方式必须填写有效链接"));
    }
    let item_id:i64=sqlx::query_scalar("INSERT INTO branch_events (branch_id,slug,title,summary,content,cover_path,event_type,venue_name,address,online_url,starts_at,ends_at,registration_url,format,registration_mode,admission_mode,capacity,registration_opens_at,registration_closes_at,status,visibility,aggregate_mode,published_at,created_by,updated_by) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,CASE WHEN ?='published' THEN datetime('now') ELSE NULL END,?,?) RETURNING id")
        .bind(id).bind(required_string(&body,"slug","Slug")?).bind(required_string(&body,"title","标题")?)
        .bind(opt_string(&body,"summary")).bind(opt_string(&body,"content")).bind(opt_string(&body,"coverPath"))
        .bind(opt_string(&body,"eventType")).bind(opt_string(&body,"venueName")).bind(opt_string(&body,"address")).bind(online_url)
        .bind(starts_at).bind(ends_at).bind(registration_url)
        .bind(format_name).bind(registration_mode).bind(admission_mode).bind(body.get("capacity").and_then(Value::as_i64))
        .bind(opt_string(&body,"registrationOpensAt")).bind(opt_string(&body,"registrationClosesAt"))
        .bind(status).bind(body.get("visibility").and_then(Value::as_str).unwrap_or("public"))
        .bind(body.get("aggregateMode").and_then(Value::as_str).unwrap_or("inherit")).bind(status).bind(user.id).bind(user.id)
        .fetch_one(&state.pools.chapter).await?;
    audit(
        &state,
        Some(id),
        user.id,
        "event.create",
        "event",
        item_id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({"id":item_id})))
}

async fn update_event(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, item_id)): Path<(i64, i64)>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.write").await?;
    let current_status: String =
        sqlx::query_scalar("SELECT status FROM branch_events WHERE id=? AND branch_id=?")
            .bind(item_id)
            .bind(id)
            .fetch_optional(&state.pools.chapter)
            .await?
            .ok_or_else(|| AppError::not_found("活动不存在"))?;
    let online_url = public_url(&body, "onlineUrl", "线上活动链接")?;
    let registration_url = public_url(&body, "registrationUrl", "外部报名链接")?;
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or(&current_status);
    if current_status == "published" || status == "published" {
        allow(&state, &user, id, "branch.events.publish").await?;
    }
    let (starts_at, ends_at) = event_time_values(&body, status)?;
    let format_name = body
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("in_person");
    let registration_mode = body
        .get("registrationMode")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let admission_mode = body
        .get("admissionMode")
        .and_then(Value::as_str)
        .unwrap_or("automatic");
    if !["in_person", "online", "hybrid"].contains(&format_name)
        || !["none", "internal", "external", "both"].contains(&registration_mode)
        || !["automatic", "review"].contains(&admission_mode)
    {
        return Err(AppError::bad_request("活动形式或报名设置无效"));
    }
    if ["external", "both"].contains(&registration_mode) && registration_url.is_none() {
        return Err(AppError::bad_request("外部报名方式必须填写有效链接"));
    }
    sqlx::query("UPDATE branch_events SET slug=?,title=?,summary=?,content=?,cover_path=?,event_type=?,venue_name=?,address=?,online_url=?,starts_at=?,ends_at=?,registration_url=?,format=?,registration_mode=?,admission_mode=?,capacity=?,registration_opens_at=?,registration_closes_at=?,status=?,visibility=?,aggregate_mode=?,published_at=CASE WHEN ?='published' THEN COALESCE(published_at,datetime('now')) ELSE published_at END,updated_by=?,updated_at=datetime('now') WHERE id=? AND branch_id=?")
        .bind(required_string(&body,"slug","Slug")?).bind(required_string(&body,"title","标题")?).bind(opt_string(&body,"summary")).bind(opt_string(&body,"content")).bind(opt_string(&body,"coverPath"))
        .bind(opt_string(&body,"eventType")).bind(opt_string(&body,"venueName")).bind(opt_string(&body,"address")).bind(online_url)
        .bind(starts_at).bind(ends_at).bind(registration_url)
        .bind(format_name).bind(registration_mode).bind(admission_mode).bind(body.get("capacity").and_then(Value::as_i64))
        .bind(opt_string(&body,"registrationOpensAt")).bind(opt_string(&body,"registrationClosesAt"))
        .bind(status).bind(body.get("visibility").and_then(Value::as_str).unwrap_or("public")).bind(body.get("aggregateMode").and_then(Value::as_str).unwrap_or("inherit"))
        .bind(status).bind(user.id).bind(item_id).bind(id).execute(&state.pools.chapter).await?;
    audit(
        &state,
        Some(id),
        user.id,
        "event.update",
        "event",
        item_id,
        Some(&body),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn delete_event(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, item_id)): Path<(i64, i64)>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.write").await?;
    let current_status: String =
        sqlx::query_scalar("SELECT status FROM branch_events WHERE id=? AND branch_id=?")
            .bind(item_id)
            .bind(id)
            .fetch_optional(&state.pools.chapter)
            .await?
            .ok_or_else(|| AppError::not_found("活动不存在"))?;
    if current_status == "published" {
        allow(&state, &user, id, "branch.events.publish").await?;
    }
    sqlx::query("UPDATE branch_events SET status='deleted',updated_by=?,updated_at=datetime('now') WHERE id=? AND branch_id=?").bind(user.id).bind(item_id).bind(id).execute(&state.pools.chapter).await?;
    audit(
        &state,
        Some(id),
        user.id,
        "event.delete",
        "event",
        item_id,
        None,
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn admin_event_operations(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, item_id)): Path<(i64, i64)>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.write").await?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM branch_events WHERE id=? AND branch_id=?)")
            .bind(item_id)
            .bind(id)
            .fetch_one(&state.pools.chapter)
            .await?;
    if !exists {
        return Err(AppError::not_found("活动不存在"));
    }
    let mut operations = public_event_operations(&state, item_id).await?;
    let cohosts: Vec<(i64, i64, String, String, String)> = sqlx::query_as(
        "SELECT c.id,b.id,b.slug,b.name,c.state FROM branch_event_cohosts c \
         JOIN branches b ON b.id=c.branch_id WHERE c.event_id=? ORDER BY c.id",
    )
    .bind(item_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    operations["cohosts"] = json!(cohosts.into_iter().map(|(cohost_id,branch_id,slug,name,state_name)|json!({"id":cohost_id,"branchId":branch_id,"slug":slug,"name":name,"state":state_name})).collect::<Vec<_>>());
    Ok(Json(operations))
}

async fn admin_cohost_invitations(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.write").await?;
    let rows: Vec<(i64, i64, String, String, String, String, String)> = sqlx::query_as(
        "SELECT c.id,e.id,e.slug,e.title,b.slug,b.name,c.invited_at \
         FROM branch_event_cohosts c JOIN branch_events e ON e.id=c.event_id \
         JOIN branches b ON b.id=e.branch_id WHERE c.branch_id=? AND c.state='pending' \
         AND e.status!='deleted' ORDER BY c.invited_at DESC,c.id DESC",
    )
    .bind(id)
    .fetch_all(&state.pools.chapter)
    .await?;
    Ok(Json(json!({
        "items": rows.into_iter().map(|(id,event_id,event_slug,event_title,owner_slug,owner_name,invited_at)|json!({
            "id":id,"eventId":event_id,"eventSlug":event_slug,"eventTitle":event_title,
            "ownerSlug":owner_slug,"ownerName":owner_name,"invitedAt":invited_at
        })).collect::<Vec<_>>()
    })))
}

async fn replace_event_operations(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, item_id)): Path<(i64, i64)>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.write").await?;
    let event_status: String =
        sqlx::query_scalar("SELECT status FROM branch_events WHERE id=? AND branch_id=?")
            .bind(item_id)
            .bind(id)
            .fetch_optional(&state.pools.chapter)
            .await?
            .ok_or_else(|| AppError::not_found("活动不存在"))?;
    if event_status == "published" {
        allow(&state, &user, id, "branch.events.publish").await?;
    }
    let topics = body
        .get("topics")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let people = body
        .get("people")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let partners = body
        .get("partners")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let questions = body
        .get("questions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let cohosts = body
        .get("cohosts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut tx = state.pools.chapter.begin().await?;
    let existing_question_rows: Vec<(i64, String, String, bool, Option<String>)> = sqlx::query_as(
        "SELECT id,question_type,label,required,options_json \
             FROM branch_event_registration_questions WHERE event_id=?",
    )
    .bind(item_id)
    .fetch_all(&mut *tx)
    .await?;
    let existing_questions: HashMap<i64, (String, String, bool, Value)> = existing_question_rows
        .into_iter()
        .map(
            |(question_id, question_type, label, required, options_json)| {
                let options = options_json
                    .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
                    .unwrap_or_else(|| json!([]));
                (question_id, (question_type, label, required, options))
            },
        )
        .collect();
    let has_registrations: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM branch_event_registrations WHERE event_id=?)",
    )
    .bind(item_id)
    .fetch_one(&mut *tx)
    .await?;
    for table in [
        "branch_event_topics",
        "branch_event_people",
        "branch_event_partners",
    ] {
        let sql = format!("DELETE FROM {table} WHERE event_id=?");
        sqlx::query(&sql).bind(item_id).execute(&mut *tx).await?;
    }
    for (index, topic) in topics.iter().enumerate() {
        let topic = topic.as_str().unwrap_or("").trim();
        if topic.is_empty() {
            continue;
        }
        sqlx::query("INSERT INTO branch_event_topics(event_id,topic,sort_order) VALUES (?,?,?)")
            .bind(item_id)
            .bind(topic)
            .bind(index as i64)
            .execute(&mut *tx)
            .await?;
    }
    for (index, person) in people.iter().enumerate() {
        let role = person
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("speaker");
        if !["speaker", "host", "facilitator", "volunteer"].contains(&role) {
            return Err(AppError::bad_request("活动人员角色无效"));
        }
        let person_url = public_url(person, "url", "活动人员链接")?;
        sqlx::query("INSERT INTO branch_event_people(event_id,role,name,title,organization,avatar_path,bio,url,sort_order) VALUES (?,?,?,?,?,?,?,?,?)")
            .bind(item_id).bind(role).bind(required_string(person,"name","活动人员姓名")?)
            .bind(opt_string(person,"title")).bind(opt_string(person,"organization")).bind(opt_string(person,"avatarPath"))
            .bind(opt_string(person,"bio")).bind(person_url).bind(index as i64).execute(&mut *tx).await?;
    }
    for (index, partner) in partners.iter().enumerate() {
        let partner_type = partner
            .get("partnerType")
            .and_then(Value::as_str)
            .unwrap_or("community");
        if !["community", "venue", "sponsor", "media", "other"].contains(&partner_type) {
            return Err(AppError::bad_request("合作方类型无效"));
        }
        let partner_url = public_url(partner, "url", "合作方链接")?;
        sqlx::query("INSERT INTO branch_event_partners(event_id,partner_type,name,logo_path,url,sort_order) VALUES (?,?,?,?,?,?)")
            .bind(item_id).bind(partner_type).bind(required_string(partner,"name","合作方名称")?)
            .bind(opt_string(partner,"logoPath")).bind(partner_url).bind(index as i64).execute(&mut *tx).await?;
    }
    let mut retained_question_ids = HashSet::new();
    for (index, question) in questions.iter().enumerate() {
        let question_type = question
            .get("questionType")
            .and_then(Value::as_str)
            .unwrap_or("short_text");
        if !["short_text", "single", "multiple"].contains(&question_type) {
            return Err(AppError::bad_request("报名问题类型无效"));
        }
        let options = question
            .get("options")
            .cloned()
            .unwrap_or_else(|| json!([]));
        let label = required_string(question, "label", "报名问题")?;
        let required = question
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if let Some(question_id) = question.get("id").and_then(Value::as_i64) {
            let Some((old_type, old_label, old_required, old_options)) =
                existing_questions.get(&question_id)
            else {
                return Err(AppError::bad_request("报名问题不属于当前活动"));
            };
            if !retained_question_ids.insert(question_id) {
                return Err(AppError::bad_request("报名问题重复"));
            }
            if has_registrations
                && (old_type != question_type
                    || old_label != &label
                    || *old_required != required
                    || old_options != &options)
            {
                return Err(AppError::bad_request("已有报名记录，不能修改报名问题内容"));
            }
            sqlx::query("UPDATE branch_event_registration_questions SET question_type=?,label=?,required=?,options_json=?,sort_order=? WHERE id=? AND event_id=?")
                .bind(question_type).bind(label).bind(required).bind(options.to_string())
                .bind(index as i64).bind(question_id).bind(item_id).execute(&mut *tx).await?;
        } else {
            if has_registrations {
                return Err(AppError::bad_request("已有报名记录，不能新增报名问题"));
            }
            sqlx::query("INSERT INTO branch_event_registration_questions(event_id,question_type,label,required,options_json,sort_order) VALUES (?,?,?,?,?,?)")
                .bind(item_id).bind(question_type).bind(label).bind(required)
                .bind(options.to_string()).bind(index as i64).execute(&mut *tx).await?;
        }
    }
    for question_id in existing_questions.keys() {
        if retained_question_ids.contains(question_id) {
            continue;
        }
        if has_registrations {
            return Err(AppError::bad_request("已有报名记录，不能删除报名问题"));
        }
        sqlx::query("DELETE FROM branch_event_registration_questions WHERE id=? AND event_id=?")
            .bind(question_id)
            .bind(item_id)
            .execute(&mut *tx)
            .await?;
    }
    let mut requested = HashSet::new();
    for cohost in cohosts {
        let branch_id = cohost
            .get("branchId")
            .and_then(Value::as_i64)
            .or_else(|| cohost.as_i64())
            .ok_or_else(|| AppError::bad_request("联合主办支部无效"))?;
        if branch_id == id {
            return Err(AppError::bad_request("主办支部不能同时作为联合主办"));
        }
        let valid: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM branches WHERE id=? AND status IN ('active','paused'))",
        )
        .bind(branch_id)
        .fetch_one(&mut *tx)
        .await?;
        if !valid {
            return Err(AppError::bad_request("联合主办支部不存在"));
        }
        requested.insert(branch_id);
        sqlx::query("INSERT INTO branch_event_cohosts(event_id,branch_id,state,invited_by) VALUES (?,?,'pending',?) ON CONFLICT(event_id,branch_id) DO UPDATE SET state=CASE WHEN branch_event_cohosts.state='accepted' THEN 'accepted' ELSE 'pending' END,invited_by=excluded.invited_by,invited_at=datetime('now'),responded_by=CASE WHEN branch_event_cohosts.state='accepted' THEN branch_event_cohosts.responded_by ELSE NULL END,responded_at=CASE WHEN branch_event_cohosts.state='accepted' THEN branch_event_cohosts.responded_at ELSE NULL END")
            .bind(item_id).bind(branch_id).bind(user.id).execute(&mut *tx).await?;
    }
    let existing:Vec<(i64,i64)>=sqlx::query_as("SELECT id,branch_id FROM branch_event_cohosts WHERE event_id=? AND state IN ('pending','accepted')").bind(item_id).fetch_all(&mut *tx).await?;
    for (cohost_id, branch_id) in existing {
        if !requested.contains(&branch_id) {
            sqlx::query("UPDATE branch_event_cohosts SET state='revoked',responded_by=?,responded_at=datetime('now') WHERE id=?")
                .bind(user.id).bind(cohost_id).execute(&mut *tx).await?;
        }
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "event.operations.replace",
        "event",
        item_id,
        None,
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
struct CohostResponseInput {
    action: String,
}

async fn respond_cohost_invitation(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, cohost_id)): Path<(i64, i64)>,
    Json(body): Json<CohostResponseInput>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.write").await?;
    let state_name = match body.action.as_str() {
        "accept" => "accepted",
        "reject" => "rejected",
        _ => return Err(AppError::bad_request("操作必须是 accept 或 reject")),
    };
    let result=sqlx::query("UPDATE branch_event_cohosts SET state=?,responded_by=?,responded_at=datetime('now') WHERE id=? AND branch_id=? AND state='pending'")
        .bind(state_name).bind(user.id).bind(cohost_id).bind(id).execute(&state.pools.chapter).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("联合主办邀请不存在或已经处理"));
    }
    let detail = json!({"state":state_name});
    audit(
        &state,
        Some(id),
        user.id,
        "event.cohost.respond",
        "cohost",
        cohost_id,
        Some(&detail),
    )
    .await;
    Ok(Json(json!({"ok":true,"state":state_name})))
}

async fn registration_event_by_slug(
    state: &AppState,
    branch_slug: &str,
    event_slug: &str,
) -> AppResult<RegistrationEvent> {
    sqlx::query_as(
        "SELECT e.id,e.branch_id,e.registration_mode,e.admission_mode,e.capacity, \
         e.registration_opens_at,e.registration_closes_at,e.starts_at,e.ends_at,e.status, \
         e.visibility,e.moderation_state FROM branch_events e JOIN branches b ON b.id=e.branch_id \
         WHERE b.slug=? AND e.slug=?",
    )
    .bind(branch_slug)
    .bind(event_slug)
    .fetch_one(&state.pools.chapter)
    .await
    .map_err(Into::into)
}

fn comparable_datetime(value: &str) -> String {
    value.trim().replace('T', " ")
}

fn ensure_registration_open(event: &RegistrationEvent) -> AppResult<()> {
    if event.status != "published"
        || event.visibility != "public"
        || event.moderation_state != "normal"
    {
        return Err(AppError::bad_request("该活动当前不可报名"));
    }
    if !["internal", "both"].contains(&event.registration_mode.as_str()) {
        return Err(AppError::bad_request("该活动未启用站内报名"));
    }
    let now = chapter_now_string();
    if event
        .registration_opens_at
        .as_deref()
        .is_some_and(|value| comparable_datetime(value).as_str() > now.as_str())
    {
        return Err(AppError::bad_request("报名尚未开放"));
    }
    if event
        .registration_closes_at
        .as_deref()
        .is_some_and(|value| comparable_datetime(value).as_str() < now.as_str())
        || comparable_datetime(event.ends_at.as_deref().unwrap_or(&event.starts_at)).as_str()
            < now.as_str()
    {
        return Err(AppError::bad_request("报名已经结束"));
    }
    Ok(())
}

async fn validate_registration_answers(
    state: &AppState,
    event_id: i64,
    answers: &Value,
) -> AppResult<()> {
    let questions: Vec<(i64, String, bool, Option<String>)> = sqlx::query_as(
        "SELECT id,question_type,required,options_json FROM branch_event_registration_questions \
         WHERE event_id=? ORDER BY sort_order,id",
    )
    .bind(event_id)
    .fetch_all(&state.pools.chapter)
    .await?;
    let object = answers
        .as_object()
        .ok_or_else(|| AppError::bad_request("报名回答格式无效"))?;
    for (question_id, question_type, required, options_json) in questions {
        let key = question_id.to_string();
        let answer = object.get(&key);
        let missing = match answer {
            None | Some(Value::Null) => true,
            Some(Value::String(value)) => value.trim().is_empty(),
            Some(Value::Array(values)) => values.is_empty(),
            _ => false,
        };
        if required && missing {
            return Err(AppError::bad_request(format!(
                "请完成必填报名问题 #{question_id}"
            )));
        }
        if missing || question_type == "short_text" {
            continue;
        }
        let options: Vec<String> = options_json
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();
        let selected = match answer {
            Some(Value::String(value)) => vec![value.clone()],
            Some(Value::Array(values)) => values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect(),
            _ => return Err(AppError::bad_request("报名选项格式无效")),
        };
        if selected.iter().any(|value| !options.contains(value)) {
            return Err(AppError::bad_request("报名回答包含无效选项"));
        }
        if question_type == "single" && selected.len() > 1 {
            return Err(AppError::bad_request("单选问题只能选择一项"));
        }
    }
    Ok(())
}

async fn my_event_registration(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, event_slug)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    let event = registration_event_by_slug(&state, &slug, &event_slug).await?;
    Ok(Json(json!({
        "registration":event_registration_for_user(&state,event.id,user.id).await?
    })))
}

async fn register_event(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, event_slug)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    let verified: bool = sqlx::query_scalar(
        "SELECT email_verified FROM users WHERE id=? AND status='active' AND deleted_at IS NULL",
    )
    .bind(user.id)
    .fetch_one(&state.pools.core)
    .await?;
    if !verified {
        return Err(AppError::bad_request("请先验证邮箱再报名活动"));
    }
    let event = registration_event_by_slug(&state, &slug, &event_slug).await?;
    ensure_registration_open(&event)?;
    let public_mode = body
        .get("publicMode")
        .and_then(Value::as_str)
        .unwrap_or("named");
    if !["named", "anonymous"].contains(&public_mode) {
        return Err(AppError::bad_request("公开方式无效"));
    }
    let answers = body.get("answers").cloned().unwrap_or_else(|| json!({}));
    validate_registration_answers(&state, event.id, &answers).await?;
    let existing: Option<(i64, String, i64)> = sqlx::query_as(
        "SELECT id,state,anonymous_number FROM branch_event_registrations WHERE event_id=? AND user_id=?",
    )
    .bind(event.id)
    .bind(user.id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    if existing
        .as_ref()
        .is_some_and(|(_, state_name, _)| !["cancelled", "rejected"].contains(&state_name.as_str()))
    {
        return Err(AppError::conflict("你已经报名该活动"));
    }
    let mut tx = state.pools.chapter.begin().await?;
    let confirmed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM branch_event_registrations WHERE event_id=? AND state='confirmed'",
    )
    .bind(event.id)
    .fetch_one(&mut *tx)
    .await?;
    let state_name = if event.admission_mode == "review" {
        "pending"
    } else if event.capacity.is_some_and(|capacity| confirmed >= capacity) {
        "waitlisted"
    } else {
        "confirmed"
    };
    let registration_id = if let Some((registration_id, _, anonymous_number)) = existing {
        sqlx::query("UPDATE branch_event_registrations SET state=?,answers_json=?,public_mode=?,review_note=NULL,reviewed_by=NULL,reviewed_at=NULL,checked_in_at=NULL,checked_in_by=NULL,updated_at=datetime('now') WHERE id=?")
            .bind(state_name).bind(answers.to_string()).bind(public_mode).bind(registration_id).execute(&mut *tx).await?;
        let _ = anonymous_number;
        registration_id
    } else {
        let anonymous_number:i64=sqlx::query_scalar("SELECT COALESCE(MAX(anonymous_number),0)+1 FROM branch_event_registrations WHERE event_id=?")
            .bind(event.id).fetch_one(&mut *tx).await?;
        sqlx::query_scalar("INSERT INTO branch_event_registrations(event_id,user_id,state,answers_json,public_mode,anonymous_number) VALUES (?,?,?,?,?,?) RETURNING id")
            .bind(event.id).bind(user.id).bind(state_name).bind(answers.to_string()).bind(public_mode).bind(anonymous_number).fetch_one(&mut *tx).await?
    };
    tx.commit().await?;
    let detail = json!({"state":state_name,"publicMode":public_mode});
    audit(
        &state,
        Some(event.branch_id),
        user.id,
        "event.registration.create",
        "event_registration",
        registration_id,
        Some(&detail),
    )
    .await;
    Ok(Json(
        json!({"id":registration_id,"state":state_name,"publicMode":public_mode}),
    ))
}

async fn update_my_registration(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, event_slug)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    let event = registration_event_by_slug(&state, &slug, &event_slug).await?;
    let row: Option<(i64, String)> = sqlx::query_as(
        "SELECT id,state FROM branch_event_registrations WHERE event_id=? AND user_id=?",
    )
    .bind(event.id)
    .bind(user.id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    let Some((registration_id, old_state)) = row else {
        return Err(AppError::not_found("尚未报名该活动"));
    };
    let mut tx = state.pools.chapter.begin().await?;
    if body.get("action").and_then(Value::as_str) == Some("cancel") {
        if ["cancelled", "rejected"].contains(&old_state.as_str()) {
            return Err(AppError::conflict("该报名已经结束"));
        }
        sqlx::query("UPDATE branch_event_registrations SET state='cancelled',checked_in_at=NULL,checked_in_by=NULL,updated_at=datetime('now') WHERE id=?")
            .bind(registration_id).execute(&mut *tx).await?;
        if old_state == "confirmed" {
            if let Some(next_id) = sqlx::query_scalar::<_,i64>("SELECT id FROM branch_event_registrations WHERE event_id=? AND state='waitlisted' ORDER BY created_at,id LIMIT 1")
                .bind(event.id).fetch_optional(&mut *tx).await? {
                sqlx::query("UPDATE branch_event_registrations SET state='confirmed',reviewed_at=datetime('now'),updated_at=datetime('now') WHERE id=?")
                    .bind(next_id).execute(&mut *tx).await?;
            }
        }
    } else if let Some(public_mode) = body.get("publicMode").and_then(Value::as_str) {
        if !["named", "anonymous"].contains(&public_mode) {
            return Err(AppError::bad_request("公开方式无效"));
        }
        sqlx::query("UPDATE branch_event_registrations SET public_mode=?,updated_at=datetime('now') WHERE id=?")
            .bind(public_mode).bind(registration_id).execute(&mut *tx).await?;
    } else {
        return Err(AppError::bad_request("没有可更新的报名信息"));
    }
    tx.commit().await?;
    audit(
        &state,
        Some(event.branch_id),
        user.id,
        "event.registration.update",
        "event_registration",
        registration_id,
        None,
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn admin_event_registrations(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, item_id)): Path<(i64, i64)>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.attendees.manage").await?;
    let valid: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM branch_events WHERE id=? AND branch_id=?)")
            .bind(item_id)
            .bind(id)
            .fetch_one(&state.pools.chapter)
            .await?;
    if !valid {
        return Err(AppError::not_found("活动不存在"));
    }
    let rows:Vec<(i64,i64,String,Option<String>,String,i64,Option<String>,Option<String>,String,String)>=sqlx::query_as("SELECT id,user_id,state,answers_json,public_mode,anonymous_number,review_note,checked_in_at,created_at,updated_at FROM branch_event_registrations WHERE event_id=? ORDER BY created_at,id")
        .bind(item_id).fetch_all(&state.pools.chapter).await?;
    let mut items = Vec::with_capacity(rows.len());
    for (
        registration_id,
        user_id,
        state_name,
        answers_json,
        public_mode,
        anonymous_number,
        review_note,
        checked_in_at,
        created_at,
        updated_at,
    ) in rows
    {
        let account:Option<(String,Option<String>,String,String)>=sqlx::query_as("SELECT COALESCE(NULLIF(nickname,''),NULLIF(display_name,''),username),avatar,username,email FROM users WHERE id=?").bind(user_id).fetch_optional(&state.pools.core).await?;
        let Some((display_name, avatar, username, email)) = account else {
            continue;
        };
        items.push(json!({"id":registration_id,"userId":user_id,"state":state_name,"answers":answers_json.and_then(|raw|serde_json::from_str::<Value>(&raw).ok()).unwrap_or_else(||json!({})),"publicMode":public_mode,"anonymousNumber":anonymous_number,"reviewNote":review_note,"checkedInAt":checked_in_at,"createdAt":created_at,"updatedAt":updated_at,"displayName":display_name,"avatar":avatar,"username":username,"email":email}));
    }
    Ok(Json(json!({"items":items})))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewRegistrationInput {
    action: String,
    review_note: Option<String>,
}

async fn review_event_registration(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, item_id, registration_id)): Path<(i64, i64, i64)>,
    Json(body): Json<ReviewRegistrationInput>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.events.attendees.manage").await?;
    let event: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT capacity FROM branch_events WHERE id=? AND branch_id=?")
            .bind(item_id)
            .bind(id)
            .fetch_optional(&state.pools.chapter)
            .await?;
    let Some((capacity,)) = event else {
        return Err(AppError::not_found("活动不存在"));
    };
    let old_state: Option<String> = sqlx::query_scalar(
        "SELECT state FROM branch_event_registrations WHERE id=? AND event_id=?",
    )
    .bind(registration_id)
    .bind(item_id)
    .fetch_optional(&state.pools.chapter)
    .await?;
    let Some(old_state) = old_state else {
        return Err(AppError::not_found("报名记录不存在"));
    };
    let mut tx = state.pools.chapter.begin().await?;
    let new_state = match body.action.as_str() {
        "approve" => {
            let confirmed:i64=sqlx::query_scalar("SELECT COUNT(*) FROM branch_event_registrations WHERE event_id=? AND state='confirmed' AND id<>?").bind(item_id).bind(registration_id).fetch_one(&mut *tx).await?;
            if capacity.is_some_and(|limit| confirmed >= limit) {
                "waitlisted"
            } else {
                "confirmed"
            }
        }
        "reject" => "rejected",
        "waitlist" => "waitlisted",
        "cancel" => "cancelled",
        "checkin" => {
            if old_state != "confirmed" {
                return Err(AppError::bad_request("只有已确认参与者可以签到"));
            }
            sqlx::query("UPDATE branch_event_registrations SET checked_in_at=datetime('now'),checked_in_by=?,updated_at=datetime('now') WHERE id=?").bind(user.id).bind(registration_id).execute(&mut *tx).await?;
            "confirmed"
        }
        "uncheckin" => {
            sqlx::query("UPDATE branch_event_registrations SET checked_in_at=NULL,checked_in_by=NULL,updated_at=datetime('now') WHERE id=?").bind(registration_id).execute(&mut *tx).await?;
            old_state.as_str()
        }
        _ => return Err(AppError::bad_request("不支持的报名操作")),
    };
    if !["checkin", "uncheckin"].contains(&body.action.as_str()) {
        sqlx::query("UPDATE branch_event_registrations SET state=?,review_note=?,reviewed_by=?,reviewed_at=datetime('now'),checked_in_at=CASE WHEN ?='confirmed' THEN checked_in_at ELSE NULL END,checked_in_by=CASE WHEN ?='confirmed' THEN checked_in_by ELSE NULL END,updated_at=datetime('now') WHERE id=?")
            .bind(new_state).bind(body.review_note.as_deref()).bind(user.id).bind(new_state).bind(new_state).bind(registration_id).execute(&mut *tx).await?;
    }
    if old_state == "confirmed" && new_state != "confirmed" {
        if let Some(next_id)=sqlx::query_scalar::<_,i64>("SELECT id FROM branch_event_registrations WHERE event_id=? AND state='waitlisted' AND id<>? ORDER BY created_at,id LIMIT 1").bind(item_id).bind(registration_id).fetch_optional(&mut *tx).await? {
            sqlx::query("UPDATE branch_event_registrations SET state='confirmed',reviewed_at=datetime('now'),updated_at=datetime('now') WHERE id=?").bind(next_id).execute(&mut *tx).await?;
        }
    }
    tx.commit().await?;
    let detail = json!({"action":body.action,"state":new_state,"hasReviewNote":body.review_note.as_deref().is_some_and(|note|!note.trim().is_empty())});
    audit(
        &state,
        Some(id),
        user.id,
        "event.registration.review",
        "event_registration",
        registration_id,
        Some(&detail),
    )
    .await;
    Ok(Json(json!({"ok":true,"state":new_state})))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrantInput {
    user_id: Option<i64>,
    username: Option<String>,
    capabilities: Vec<String>,
    super_password: Option<String>,
}

async fn resolve_target_user_id(
    state: &AppState,
    user_id: Option<i64>,
    username: Option<&str>,
) -> AppResult<i64> {
    let user_id = if let Some(user_id) = user_id {
        sqlx::query_scalar(
            "SELECT id FROM users WHERE id=? AND status='active' AND deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_optional(&state.pools.core)
        .await?
    } else if let Some(username) = username.map(str::trim).filter(|v| !v.is_empty()) {
        sqlx::query_scalar(
            "SELECT id FROM users WHERE username=? AND status='active' AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(&state.pools.core)
        .await?
    } else {
        None
    };
    user_id.ok_or_else(|| AppError::bad_request("找不到该有效用户"))
}

async fn grant_target_user_id(state: &AppState, body: &GrantInput) -> AppResult<i64> {
    resolve_target_user_id(state, body.user_id, body.username.as_deref()).await
}

async fn confirm_current_super_password(
    state: &AppState,
    user: &AuthUser,
    password: Option<&str>,
) -> AppResult<()> {
    let Some(owner_username) = state.cfg.superadmin_user.as_deref() else {
        return Err(AppError::bad_request("未配置总负责人超管账号"));
    };
    if !user.is_super {
        return Err(AppError::bad_request(
            "该操作只能由总负责人使用自己的超管账号确认",
        ));
    }
    let password = password
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::bad_request("请输入当前超管账号密码"))?;
    let hash: Option<String> = sqlx::query_scalar(
        "SELECT password_hash FROM users WHERE id=? AND username=? AND is_super_admin=1 AND status='active' \
         AND deleted_at IS NULL",
    )
    .bind(user.id)
    .bind(owner_username)
    .fetch_optional(&state.pools.core)
    .await?;
    if hash
        .as_deref()
        .is_some_and(|stored| verify_password(password, stored))
    {
        Ok(())
    } else {
        Err(AppError::bad_request(
            "超管密码不正确，或当前账号不是总负责人账号",
        ))
    }
}

async fn needs_multi_branch_override(
    state: &AppState,
    target_user_id: i64,
    branch_id: i64,
) -> AppResult<bool> {
    let target_is_super: bool = sqlx::query_scalar(
        "SELECT is_super_admin FROM users WHERE id=? AND status='active' AND deleted_at IS NULL",
    )
    .bind(target_user_id)
    .fetch_one(&state.pools.core)
    .await?;
    if target_is_super {
        return Ok(false);
    }
    let has_platform_access: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM capability_grants WHERE user_id=? AND scope_type='platform' \
         AND scope_id='chapter' AND (expires_at IS NULL OR expires_at > datetime('now')))",
    )
    .bind(target_user_id)
    .fetch_one(&state.pools.core)
    .await?;
    if has_platform_access {
        return Ok(false);
    }
    let has_other_branch: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM capability_grants WHERE user_id=? AND scope_type='branch' \
         AND scope_id<>? AND (expires_at IS NULL OR expires_at > datetime('now')))",
    )
    .bind(target_user_id)
    .bind(branch_id.to_string())
    .fetch_one(&state.pools.core)
    .await?;
    Ok(has_other_branch)
}

fn validate_capabilities(items: &[String]) -> AppResult<()> {
    if items
        .iter()
        .all(|c| ADMIN_CAPABILITIES.contains(&c.as_str()))
    {
        Ok(())
    } else {
        Err(AppError::bad_request("包含未知能力"))
    }
}

async fn list_branch_grants(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.permissions.manage").await?;
    let rows:Vec<(i64,String,String,Option<String>)>=sqlx::query_as("SELECT g.user_id,u.username,g.capability,g.expires_at FROM capability_grants g JOIN users u ON u.id=g.user_id WHERE g.scope_type='branch' AND g.scope_id=? ORDER BY u.username,g.capability")
        .bind(id.to_string()).fetch_all(&state.pools.core).await?;
    Ok(Json(
        json!({"items":rows.into_iter().map(|(user_id,username,capability,expires_at)|json!({"userId":user_id,"username":username,"capability":capability,"expiresAt":expires_at})).collect::<Vec<_>>() }),
    ))
}

async fn set_branch_grants(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<GrantInput>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.permissions.manage").await?;
    validate_capabilities(&body.capabilities)?;
    let target_user_id = grant_target_user_id(&state, &body).await?;
    let override_used = !body.capabilities.is_empty()
        && needs_multi_branch_override(&state, target_user_id, id).await?;
    if override_used {
        confirm_current_super_password(&state, &user, body.super_password.as_deref()).await?;
    }
    let mut tx = state.pools.core.begin().await?;
    sqlx::query(
        "DELETE FROM capability_grants WHERE user_id=? AND scope_type='branch' AND scope_id=?",
    )
    .bind(target_user_id)
    .bind(id.to_string())
    .execute(&mut *tx)
    .await?;
    for capability in &body.capabilities {
        sqlx::query("INSERT INTO capability_grants (user_id,capability,scope_type,scope_id,granted_by) VALUES (?,?,'branch',?,?)")
            .bind(target_user_id).bind(capability).bind(id.to_string()).bind(user.id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    audit(
        &state,
        Some(id),
        user.id,
        "grants.replace",
        "grant",
        target_user_id,
        Some(&json!({"capabilities":body.capabilities,"multiBranchOverride":override_used})),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn set_platform_grants(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<GrantInput>,
) -> AppResult<Json<Value>> {
    require_super(&user)?;
    validate_capabilities(&body.capabilities)?;
    let target_user_id = grant_target_user_id(&state, &body).await?;
    let override_used = !body.capabilities.is_empty();
    if override_used {
        confirm_current_super_password(&state, &user, body.super_password.as_deref()).await?;
    }
    let mut tx = state.pools.core.begin().await?;
    sqlx::query("DELETE FROM capability_grants WHERE user_id=? AND scope_type='platform' AND scope_id='chapter'").bind(target_user_id).execute(&mut *tx).await?;
    for capability in &body.capabilities {
        sqlx::query("INSERT INTO capability_grants (user_id,capability,scope_type,scope_id,granted_by) VALUES (?,?,'platform','chapter',?)")
            .bind(target_user_id).bind(capability).bind(user.id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    audit(
        &state,
        None,
        user.id,
        "platform_grants.replace",
        "grant",
        target_user_id,
        Some(&json!({"capabilities":body.capabilities,"platformOverride":override_used})),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HandoverInput {
    to_user_id: Option<i64>,
    to_username: Option<String>,
    #[serde(default)]
    revoke_from_user: bool,
    note: Option<String>,
    super_password: Option<String>,
}

async fn handover_branch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<HandoverInput>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.permissions.manage").await?;
    let target_user_id =
        resolve_target_user_id(&state, body.to_user_id, body.to_username.as_deref()).await?;
    let override_used = needs_multi_branch_override(&state, target_user_id, id).await?;
    if override_used {
        confirm_current_super_password(&state, &user, body.super_password.as_deref()).await?;
    }
    let mut tx = state.pools.core.begin().await?;
    for capability in ADMIN_CAPABILITIES {
        sqlx::query("INSERT INTO capability_grants (user_id,capability,scope_type,scope_id,granted_by) VALUES (?,?,'branch',?,?) ON CONFLICT(user_id,capability,scope_type,scope_id) DO UPDATE SET granted_by=excluded.granted_by,updated_at=datetime('now')")
            .bind(target_user_id).bind(capability).bind(id.to_string()).bind(user.id).execute(&mut *tx).await?;
    }
    if body.revoke_from_user && target_user_id != user.id {
        sqlx::query(
            "DELETE FROM capability_grants WHERE user_id=? AND scope_type='branch' AND scope_id=?",
        )
        .bind(user.id)
        .bind(id.to_string())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    sqlx::query("INSERT INTO branch_admin_handovers (branch_id,from_user_id,to_user_id,initiated_by,note,state,completed_at) VALUES (?,?,?,?,?,'completed',datetime('now'))")
        .bind(id).bind(user.id).bind(target_user_id).bind(user.id).bind(body.note).execute(&state.pools.chapter).await?;
    audit(
        &state,
        Some(id),
        user.id,
        "handover.complete",
        "grant",
        target_user_id,
        Some(&json!({"multiBranchOverride":override_used})),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn moderate_branch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize_capability(
        &state.pools.core,
        &user,
        "branch.platform.intervene",
        "platform",
        "chapter",
    )
    .await?;
    let entity_type = required_string(&body, "entityType", "对象类型")?;
    let state_name = required_string(&body, "state", "状态")?;
    match entity_type.as_str() {
        "branch" if ["active", "paused", "archived"].contains(&state_name.as_str()) => {
            sqlx::query("UPDATE branches SET status=?,updated_at=datetime('now') WHERE id=?")
                .bind(&state_name)
                .bind(id)
                .execute(&state.pools.chapter)
                .await?;
        }
        "timeline" | "event" if ["normal", "withheld"].contains(&state_name.as_str()) => {
            let entity_id = body
                .get("entityId")
                .and_then(Value::as_i64)
                .ok_or_else(|| AppError::bad_request("缺少 entityId"))?;
            let table = if entity_type == "timeline" {
                "branch_event_timeline_entries"
            } else {
                "branch_events"
            };
            let sql = format!("UPDATE {table} SET moderation_state=?,updated_at=datetime('now') WHERE id=? AND branch_id=?");
            sqlx::query(&sql)
                .bind(&state_name)
                .bind(entity_id)
                .bind(id)
                .execute(&state.pools.chapter)
                .await?;
        }
        _ => return Err(AppError::bad_request("不支持的监管操作")),
    }
    audit(
        &state,
        Some(id),
        user.id,
        "platform.intervene",
        &entity_type,
        body.get("entityId").and_then(Value::as_i64).unwrap_or(id),
        Some(&body),
    )
    .await;
    Ok(Json(json!({"ok":true})))
}

async fn branch_audit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    allow(&state, &user, id, "branch.audit.read").await?;
    let rows:Vec<(i64,Option<i64>,String,Option<String>,Option<String>,Option<String>,String)>=sqlx::query_as("SELECT id,actor_id,action,entity_type,entity_id,detail_json,created_at FROM branch_audit_log WHERE branch_id=? ORDER BY id DESC LIMIT 500")
        .bind(id).fetch_all(&state.pools.chapter).await?;
    Ok(Json(
        json!({"items":rows.into_iter().map(|(id,actor_id,action,entity_type,entity_id,detail_json,created_at)|json!({"id":id,"actorId":actor_id,"action":action,"entityType":entity_type,"entityId":entity_id,"detail":detail_json.and_then(|v|serde_json::from_str::<Value>(&v).ok()),"createdAt":created_at})).collect::<Vec<_>>() }),
    ))
}

async fn upload_media(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    headers: HeaderMap,
    mut mp: Multipart,
) -> AppResult<Json<Value>> {
    allow_media_upload(&state, &user, id).await?;
    let ip = client_ip(&headers);
    if let Err(seconds) = state.upload_limiter.check_and_record(&ip) {
        return Err(AppError::TooManyRequests(format!(
            "上传过于频繁，请 {seconds} 秒后再试"
        )));
    }
    let mut file = None;
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        if matches!(field.name(), Some("file" | "image")) {
            let name = field.file_name().unwrap_or("image.bin").to_string();
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("读取上传失败: {e}")))?;
            file = Some((name, bytes));
        }
    }
    let (name, bytes) = file.ok_or_else(|| AppError::bad_request("缺少图片文件"))?;
    let ext = haruhi_media::ext_of(&name, "").to_lowercase();
    if !["jpg", "jpeg", "png", "webp", "gif", "avif"].contains(&ext.as_str()) {
        return Err(AppError::bad_request("仅支持 JPG、PNG、WebP、GIF 或 AVIF"));
    }
    haruhi_media::check_image(&ext, bytes.len())
        .map_err(|e| AppError::bad_request(e.to_string()))?;
    let file_name = format!(
        "{}-{}.{}",
        chrono::Utc::now().timestamp_millis(),
        uuid::Uuid::new_v4().simple(),
        ext
    );
    let sub = id.to_string();
    let dir = state.cfg.uploads_subdir("chapter").join(&sub);
    haruhi_media::save_file(&dir, &file_name, &bytes).await?;
    let path = format!("/uploads/chapter/{sub}/{file_name}");
    audit(
        &state,
        Some(id),
        user.id,
        "media.upload",
        "media",
        &file_name,
        None,
    )
    .await;
    Ok(Json(json!({"path":path})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_validation_is_conservative() {
        assert!(valid_slug("hong-kong-01"));
        assert!(!valid_slug("Hong-Kong"));
        assert!(!valid_slug("香港"));
        assert!(!valid_slug("../branch"));
    }

    #[test]
    fn public_links_reject_executable_schemes() {
        assert!(public_url(&json!({"url":"https://example.test/path"}), "url", "链接").is_ok());
        assert!(public_url(&json!({"url":"javascript:alert(1)"}), "url", "链接").is_err());
        assert!(contact_url(&json!({"url":"mailto:test@example.test"}), "url", "链接").is_ok());
        assert!(contact_url(&json!({"url":"data:text/html,boom"}), "url", "链接").is_err());
    }

    #[test]
    fn datetime_local_values_are_comparable() {
        assert_eq!(
            comparable_datetime("2026-07-18T15:30:00"),
            "2026-07-18 15:30:00"
        );
        assert!(comparable_datetime("2026-07-18T15:30:00").as_str() > "2026-07-18 14:30:00");
    }
}
