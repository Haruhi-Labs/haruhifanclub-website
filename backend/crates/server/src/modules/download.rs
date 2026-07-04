//! 资源站（download）：语雀知识库「凉宫春日资源站」的服务端索引镜像。
//!
//! 本模块**不落库**：定时（默认 6 小时）从语雀 API 拉取知识库目录（TOC）与文档元数据，
//! 在内存里合并成一棵「分类 + 条目」树并缓存；前端拉 `/api/download/index` 直接取缓存，
//! 点击条目跳转到语雀原文。数据随语雀更新而更新，本站只做索引镜像、不展示正文、不承载下载。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::{AppError, AppResult, Config};
use serde::Serialize;
use serde_json::{json, Value};

use crate::state::AppState;

/// 语雀空间域名（该团队知识库固定托管于此）。
const YUQUE_HOST: &str = "https://haruhifanclub.yuque.com";
/// 单页文档上限（语雀 docs 接口默认/上限 100）。
const PAGE_SIZE: usize = 100;

/// 内存缓存：整棵资源索引，原子替换（读多写少）。
pub type DownloadCache = Arc<RwLock<Arc<DownloadIndex>>>;

/// 建一个空缓存（同步任务首次成功前，前端拿到 synced_at=null 的空索引）。
pub fn new_cache() -> DownloadCache {
    Arc::new(RwLock::new(Arc::new(DownloadIndex::default())))
}

// ============================================================
//  对外数据结构（直接 JSON 序列化给前端）
// ============================================================

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadIndex {
    /// 知识库标题（语雀 repo name）。
    pub title: String,
    /// 知识库简介。
    pub description: String,
    /// 语雀知识库主页 URL。
    pub source_url: String,
    /// 服务端最近一次成功同步时间（RFC3339）；None 表示尚未同步成功。
    pub synced_at: Option<String>,
    /// 语雀内容最近更新时间（所有文档 content_updated_at 的最大值）。
    pub content_updated_at: Option<String>,
    pub stats: Stats,
    /// 顶层「相关链接」（语雀 TOC 里的 LINK 节点，如「提交缺失资源」）。
    pub links: Vec<ExtLink>,
    /// 分类树（顶层分类，递归含子分类与条目，保留语雀 TOC 原始顺序）。
    pub tree: Vec<Node>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    /// 资源条目总数（DOC）。
    pub entries: u32,
    /// 顶层分类数。
    pub top_categories: u32,
    /// 分类总数（含子分类）。
    pub categories: u32,
}

#[derive(Serialize)]
pub struct ExtLink {
    pub title: String,
    pub url: String,
}

/// 树节点：分类或资源条目（保留语雀 TOC 的原始顺序）。
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Node {
    Category(Category),
    Entry(Entry),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    /// 语雀 TOC 节点 uuid（前端锚点/展开态用）。
    pub id: String,
    pub title: String,
    /// 该分类下（含子分类，递归）的资源条目总数。
    pub count: u32,
    pub children: Vec<Node>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// 语雀 TOC 节点 uuid。
    pub id: String,
    pub title: String,
    /// 语雀文档 slug。
    pub slug: String,
    /// 跳转到语雀原文的完整 URL。
    pub url: String,
    /// 文档摘要（语雀正文前若干字，用于列表预览与搜索）。
    pub description: String,
    /// 内容最近更新时间（RFC3339）。
    pub updated_at: Option<String>,
    /// 点赞数（语雀）。
    pub likes: u32,
    /// 评论数（语雀）。语雀公开 API 不提供阅读量（read_count 恒为 0），故以评论数作第二项指标。
    pub comments: u32,
}

// ============================================================
//  路由与 handler
// ============================================================

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/index", get(get_index))
        .route("/admin/refresh", post(admin_refresh))
}

/// 返回整棵资源索引（内存缓存）。尚未同步成功时返回空索引（synced_at=null），前端据此提示。
async fn get_index(State(state): State<AppState>) -> Json<Arc<DownloadIndex>> {
    let snap = state.download.read().unwrap().clone();
    Json(snap)
}

/// 强制立即从语雀重新同步（需超管 / download 管理角色）。
async fn admin_refresh(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "download", Action::Manage).await?;
    let Some(token) = state.cfg.yuque_token.clone() else {
        return Err(AppError::bad_request(
            "未配置语雀 Token（HARUHI_YUQUE_TOKEN），无法同步",
        ));
    };
    let index = fetch_index(&state.cfg.yuque_repo, &token)
        .await
        .map_err(|e| AppError::internal(format!("语雀同步失败：{e}")))?;
    let arc = Arc::new(index);
    *state.download.write().unwrap() = arc.clone();
    Ok(Json(json!({
        "ok": true,
        "entries": arc.stats.entries,
        "categories": arc.stats.categories,
        "syncedAt": arc.synced_at,
    })))
}

// ============================================================
//  后台定时同步
// ============================================================

/// 启动语雀同步后台任务：启动即拉一次（interval 首个 tick 立即触发），此后每 interval 一次。
/// 未配置 token 时不启动（资源站前端会拿到空索引并提示未接入）。
pub fn spawn_sync(cfg: Arc<Config>, cache: DownloadCache) {
    let Some(token) = cfg.yuque_token.clone() else {
        tracing::info!("[资源站] 未配置 HARUHI_YUQUE_TOKEN，语雀同步未启动");
        return;
    };
    let repo = cfg.yuque_repo.clone();
    // 最短 5 分钟，避免误配成 0 把语雀刷爆。
    let interval = Duration::from_secs(cfg.yuque_sync_interval_secs.max(300));
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            match fetch_index(&repo, &token).await {
                Ok(index) => {
                    let n = index.stats.entries;
                    *cache.write().unwrap() = Arc::new(index);
                    tracing::info!("[资源站] 同步完成：{n} 条资源");
                }
                Err(e) => tracing::error!("[资源站] 同步失败：{e}"),
            }
        }
    });
}

// ============================================================
//  语雀抓取与合并
// ============================================================

/// 文档元数据（来自 /repos/:repo/docs 列表），按 doc_id 索引以增强 TOC 里的 DOC 节点。
struct DocMeta {
    description: String,
    updated_at: Option<String>,
    likes: u32,
    comments: u32,
}

/// 拉取 + 合并出一棵完整索引：repo 详情（标题/简介）+ docs 元数据 + TOC 结构。
async fn fetch_index(repo: &str, token: &str) -> anyhow::Result<DownloadIndex> {
    let client = reqwest::Client::builder()
        // 语雀经 Cloudflare，缺 UA 会被拦；固定一个可识别的 UA。
        .user_agent("haruhifanclub-resource-index/1.0")
        .timeout(Duration::from_secs(20))
        .build()?;
    let api = format!("{YUQUE_HOST}/api/v2/repos/{repo}");

    // 1) 知识库详情：标题 / 简介
    let repo_detail = get_json(&client, token, &api).await?;
    let rd = repo_detail.get("data").cloned().unwrap_or(Value::Null);
    let title = rd
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("凉宫春日资源站")
        .to_string();
    let description = rd
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    // 2) 文档元数据（分页），建立 doc_id → meta
    let mut metas: HashMap<i64, DocMeta> = HashMap::new();
    let mut offset = 0usize;
    loop {
        let url = format!("{api}/docs?limit={PAGE_SIZE}&offset={offset}");
        let resp = get_json(&client, token, &url).await?;
        let arr = resp
            .get("data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let n = arr.len();
        for d in &arr {
            let Some(id) = d.get("id").and_then(Value::as_i64) else {
                continue;
            };
            metas.insert(
                id,
                DocMeta {
                    description: d
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    updated_at: d
                        .get("content_updated_at")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    likes: d
                        .get("likes_count")
                        .and_then(Value::as_i64)
                        .unwrap_or(0)
                        .max(0) as u32,
                    comments: d
                        .get("comments_count")
                        .and_then(Value::as_i64)
                        .unwrap_or(0)
                        .max(0) as u32,
                },
            );
        }
        // 拉满一页则可能还有下一页；不足一页即结束。5000 条硬上限兜底防死循环。
        if n < PAGE_SIZE || offset >= 5000 {
            break;
        }
        offset += PAGE_SIZE;
    }

    // 3) TOC 结构
    let toc_resp = get_json(&client, token, &format!("{api}/toc")).await?;
    let items = toc_resp
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let public_base = format!("{YUQUE_HOST}/{repo}");
    Ok(build_index(&items, &metas, title, description, public_base))
}

/// 语雀 GET，带鉴权头；非 2xx 视为错误。
async fn get_json(client: &reqwest::Client, token: &str, url: &str) -> anyhow::Result<Value> {
    let resp = client.get(url).header("X-Auth-Token", token).send().await?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("语雀 API 返回 {status}：{url}");
    }
    Ok(resp.json().await?)
}

/// 把扁平的 TOC 列表 + 文档元数据合并成一棵有序树。
fn build_index(
    items: &[Value],
    metas: &HashMap<i64, DocMeta>,
    title: String,
    description: String,
    public_base: String,
) -> DownloadIndex {
    // parent_uuid → 有序子节点下标（保留 TOC 原始文档顺序）。
    let mut children: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, it) in items.iter().enumerate() {
        let kind = it.get("type").and_then(Value::as_str).unwrap_or("");
        if kind == "META" {
            continue; // META 是知识库自身元信息，不入树
        }
        let parent = it
            .get("parent_uuid")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        children.entry(parent).or_default().push(i);
    }

    let mut links = Vec::new();
    let mut cat_total = 0u32;
    let mut latest: Option<String> = None;
    let tree = build_nodes(
        "",
        items,
        &children,
        metas,
        &public_base,
        &mut links,
        &mut cat_total,
        &mut latest,
    );
    let entries = count_entries(&tree);
    let top_categories = tree
        .iter()
        .filter(|n| matches!(n, Node::Category(_)))
        .count() as u32;

    DownloadIndex {
        title,
        description,
        source_url: public_base,
        synced_at: Some(chrono::Utc::now().to_rfc3339()),
        content_updated_at: latest,
        stats: Stats {
            entries,
            top_categories,
            categories: cat_total,
        },
        links,
        tree,
    }
}

/// 递归构建某个父节点下的子节点列表。
#[allow(clippy::too_many_arguments)]
fn build_nodes(
    parent: &str,
    items: &[Value],
    children: &HashMap<String, Vec<usize>>,
    metas: &HashMap<i64, DocMeta>,
    public_base: &str,
    links: &mut Vec<ExtLink>,
    cat_total: &mut u32,
    latest: &mut Option<String>,
) -> Vec<Node> {
    let mut out = Vec::new();
    let Some(idxs) = children.get(parent) else {
        return out;
    };
    for &i in idxs {
        let it = &items[i];
        let kind = it.get("type").and_then(Value::as_str).unwrap_or("");
        let uuid = it
            .get("uuid")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let title = it
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        match kind {
            "TITLE" => {
                *cat_total += 1;
                let child_nodes = build_nodes(
                    &uuid,
                    items,
                    children,
                    metas,
                    public_base,
                    links,
                    cat_total,
                    latest,
                );
                let count = count_entries(&child_nodes);
                out.push(Node::Category(Category {
                    id: uuid,
                    title,
                    count,
                    children: child_nodes,
                }));
            }
            "DOC" => {
                let slug = it
                    .get("slug")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let meta = it
                    .get("doc_id")
                    .and_then(Value::as_i64)
                    .and_then(|id| metas.get(&id));
                let description = meta.map(|m| m.description.clone()).unwrap_or_default();
                let updated_at = meta.and_then(|m| m.updated_at.clone());
                let likes = meta.map(|m| m.likes).unwrap_or(0);
                let comments = meta.map(|m| m.comments).unwrap_or(0);
                // 记录全站最新更新时间（同格式 RFC3339 UTC，字典序即时间序）。
                if let Some(u) = &updated_at {
                    if latest.as_deref().map_or(true, |cur| u.as_str() > cur) {
                        *latest = Some(u.clone());
                    }
                }
                out.push(Node::Entry(Entry {
                    id: uuid.clone(),
                    title,
                    url: format!("{public_base}/{slug}"),
                    slug,
                    description,
                    updated_at,
                    likes,
                    comments,
                }));
                // 语雀允许 DOC 下挂子 DOC（如某作品的附属绘/衍生条目）。子节点平铺到同级、
                // 紧跟父条目之后，保证不漏条目。
                let mut sub = build_nodes(
                    &uuid,
                    items,
                    children,
                    metas,
                    public_base,
                    links,
                    cat_total,
                    latest,
                );
                out.append(&mut sub);
            }
            "LINK" => {
                // 外链（如「提交缺失资源」表单）不进分类树，收集为「相关链接」单独呈现。
                let url = it
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                links.push(ExtLink { title, url });
            }
            _ => {}
        }
    }
    out
}

/// 统计一组节点下的资源条目总数（分类取其已算好的 count）。
fn count_entries(nodes: &[Node]) -> u32 {
    nodes
        .iter()
        .map(|n| match n {
            Node::Entry(_) => 1,
            Node::Category(c) => c.count,
        })
        .sum()
}
