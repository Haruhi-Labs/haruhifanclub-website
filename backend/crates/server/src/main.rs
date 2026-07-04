//! haruhifanclub 统一后端入口（二进制）。模块定义在 `lib.rs`，此处仅装配并启动。

use std::sync::Arc;

use haruhi_core::Config;
use haruhi_db::Pools;
use haruhi_mail::Mailer;
use haruhi_server::ratelimit::RateLimiter;
use haruhi_server::state::AppState;
use haruhi_server::{modules, routes, seed};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let cfg = Arc::new(Config::from_env()?);
    tracing::info!(data_dir = ?cfg.data_dir, uploads_dir = ?cfg.uploads_dir, "配置加载完成");

    let pools = Pools::connect(&cfg).await?;
    pools.migrate().await?;
    seed::seed_superadmin(&cfg, &pools.core).await?;

    // 启动 shop 邮件队列后台 worker（Mailer 为 None 即邮件未启用时内部空转不启）。
    modules::shop::spawn_email_worker(cfg.clone(), pools.shop.clone());

    // 资源站（download）：语雀知识库索引缓存 + 后台定时同步（未配 token 时内部不启）。
    let download_cache = modules::download::new_cache();
    modules::download::spawn_sync(cfg.clone(), download_cache.clone());

    let state = AppState {
        cfg: cfg.clone(),
        pools: pools.clone(),
        // 登录限流：单 IP 10 分钟内最多 10 次尝试
        login_limiter: Arc::new(RateLimiter::new(10, 600)),
        // 匿名上传限流：单 IP 10 分钟内最多 60 次（足够正常创作，封堵刷量）
        upload_limiter: Arc::new(RateLimiter::new(60, 600)),
        // 注册/找回/重发验证：单 IP 1 小时内最多 5 次，防刷邮件
        account_limiter: Arc::new(RateLimiter::new(5, 3600)),
        // 统一邮件发送器（未配置邮件时为 None）
        mailer: Mailer::from_config(&cfg),
        // 资源站索引缓存（与后台同步任务共享同一 Arc）
        download: download_cache,
    };
    let app = routes::router(state);

    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    tracing::info!("haruhi 后端启动: http://{}", cfg.bind);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // 优雅退出：等在途请求结束后，显式 checkpoint + 关闭连接池刷 WAL，保证数据一致。
    tracing::info!("正在关闭连接池并刷盘 WAL…");
    for pool in [
        &pools.core,
        &pools.news,
        &pools.art,
        &pools.exam,
        &pools.novel,
        &pools.shop,
        &pools.fiction,
    ] {
        let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(pool)
            .await;
        pool.close().await;
    }
    tracing::info!("已安全退出");
    Ok(())
}

/// 等待 SIGTERM(systemd 重启) 或 Ctrl-C。
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("收到关闭信号，开始优雅退出…");
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();
}
