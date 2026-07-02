//! 统一 SQLite 连接池管理：每模块一库 + core 库，全部启用 WAL。

use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use haruhi_core::Config;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

/// 所有模块的连接池集合，放进 axum AppState 共享。
#[derive(Clone)]
pub struct Pools {
    pub core: SqlitePool,
    pub news: SqlitePool,
    pub art: SqlitePool,
    pub exam: SqlitePool,
    pub novel: SqlitePool,
    pub shop: SqlitePool,
    pub fiction: SqlitePool,
}

/// 打开（或创建）一个 SQLite 库，启用 WAL + busy_timeout。
pub async fn open_pool(path: &Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(10))
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .acquire_timeout(Duration::from_secs(15))
        .connect_with(opts)
        .await?;
    Ok(pool)
}

impl Pools {
    /// 按配置打开全部连接池。
    pub async fn connect(cfg: &Config) -> anyhow::Result<Self> {
        tokio::fs::create_dir_all(&cfg.data_dir).await.ok();
        let pools = Pools {
            core: open_pool(&cfg.db_path("core")).await?,
            news: open_pool(&cfg.db_path("news")).await?,
            art: open_pool(&cfg.db_path("art")).await?,
            exam: open_pool(&cfg.db_path("exam")).await?,
            novel: open_pool(&cfg.db_path("novel")).await?,
            shop: open_pool(&cfg.db_path("shop")).await?,
            fiction: open_pool(&cfg.db_path("fiction")).await?,
        };
        Ok(pools)
    }

    /// 运行各库迁移。core 库的 RBAC 迁移始终执行；
    /// 模块库的迁移在对应模块接入时再补充（此处先跑 core）。
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("../../migrations/core")
            .run(&self.core)
            .await?;
        sqlx::migrate!("../../migrations/novel")
            .run(&self.novel)
            .await?;
        sqlx::migrate!("../../migrations/art")
            .run(&self.art)
            .await?;
        sqlx::migrate!("../../migrations/news")
            .run(&self.news)
            .await?;
        sqlx::migrate!("../../migrations/exam")
            .run(&self.exam)
            .await?;
        sqlx::migrate!("../../migrations/shop")
            .run(&self.shop)
            .await?;
        sqlx::migrate!("../../migrations/fiction")
            .run(&self.fiction)
            .await?;
        tracing::info!("数据库迁移完成");
        Ok(())
    }
}
