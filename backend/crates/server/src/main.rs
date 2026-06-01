//! haruhifanclub 统一后端入口。

mod admin_routes;
mod auth_routes;
mod modules;
mod routes;
mod seed;
mod state;

use std::sync::Arc;

use haruhi_core::Config;
use haruhi_db::Pools;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let cfg = Arc::new(Config::from_env()?);
    tracing::info!(data_dir = ?cfg.data_dir, uploads_dir = ?cfg.uploads_dir, "配置加载完成");

    let pools = Pools::connect(&cfg).await?;
    pools.migrate().await?;
    seed::seed_superadmin(&cfg, &pools.core).await?;

    let state = AppState {
        cfg: cfg.clone(),
        pools,
    };
    let app = routes::router(state);

    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    tracing::info!("haruhi 后端启动: http://{}", cfg.bind);
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();
}
