//! 首次启动 seed 超级管理员（仅当 core.db 尚无任何用户时）。

use haruhi_auth::hash_password;
use haruhi_core::Config;
use sqlx::SqlitePool;

pub async fn seed_superadmin(cfg: &Config, core: &SqlitePool) -> anyhow::Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(core)
        .await?;
    if count > 0 {
        return Ok(());
    }

    let (Some(user), Some(pass)) = (
        cfg.superadmin_user.clone(),
        cfg.superadmin_password.clone(),
    ) else {
        tracing::warn!(
            "core.db 无用户且未配置 HARUHI_SUPERADMIN_USER/PASSWORD，跳过超管 seed（请稍后手动创建）"
        );
        return Ok(());
    };

    let hash = hash_password(&pass)?;
    sqlx::query(
        "INSERT INTO users (username, password_hash, display_name, is_super_admin, status) \
         VALUES (?, ?, ?, 1, 'active')",
    )
    .bind(&user)
    .bind(&hash)
    .bind("超级管理员")
    .execute(core)
    .await?;

    tracing::info!("已创建初始超级管理员账号: {user}");
    Ok(())
}
