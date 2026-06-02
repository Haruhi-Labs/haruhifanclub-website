//! 从环境变量加载强类型配置。关键密钥缺失时返回错误（fail-fast）。

use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub uploads_dir: PathBuf,

    // 鉴权
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub superadmin_user: Option<String>,
    pub superadmin_password: Option<String>,

    // AI 审核（DashScope / OpenAI 兼容）
    pub dashscope_api_key: Option<String>,
    pub ai_api_url: String,
    pub ai_text_model: String,
    pub ai_image_model: String,

    // art 匿名 Cookie 签名
    pub art_cookie_secret: String,

    // shop
    pub shop_free_shipping_threshold: i64,
    pub mail: MailConfig,
    pub public_site_url: String,

    // CORS 允许的来源（release 生效）；为空则默认仅 public_site_url。逗号分隔 HARUHI_CORS_ORIGINS。
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MailConfig {
    pub enabled: bool,
    pub provider: String, // resend | smtp | auto
    pub from_name: String,
    pub from_address: Option<String>,
    pub reply_to: Option<String>,
    pub resend_api_key: Option<String>,
    pub resend_api_base_url: String,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_secure: bool,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
}

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

fn env_or(key: &str, default: &str) -> String {
    env(key).unwrap_or_else(|| default.to_string())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    env(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn env_bool(key: &str, default: bool) -> bool {
    match env(key).as_deref() {
        Some("true" | "1" | "yes" | "on") => true,
        Some("false" | "0" | "no" | "off") => false,
        _ => default,
    }
}

impl Config {
    /// 从环境变量加载（在此之前应已调用 dotenvy::dotenv）。
    pub fn from_env() -> anyhow::Result<Self> {
        let bind: SocketAddr = env_or("HARUHI_BIND", "127.0.0.1:8080")
            .parse()
            .map_err(|e| anyhow::anyhow!("HARUHI_BIND 解析失败: {e}"))?;

        // 关键密钥：release 必填(fail-fast)；debug 用不安全默认值方便本地 `cargo run` 开箱即用
        let jwt_secret = match env("HARUHI_JWT_SECRET") {
            Some(s) => s,
            None if cfg!(debug_assertions) => {
                tracing::warn!("⚠ 未设置 HARUHI_JWT_SECRET，调试构建用不安全默认值（生产务必设置 ≥32 位随机串）");
                "dev-insecure-jwt-secret-change-me".to_string()
            }
            None => anyhow::bail!("缺少 HARUHI_JWT_SECRET（生产必填）"),
        };
        if jwt_secret.len() < 16 {
            tracing::warn!("⚠ HARUHI_JWT_SECRET 过短(<16 位)，建议 ≥32 位随机串");
        }
        let art_cookie_secret = match env("ART_COOKIE_SECRET") {
            Some(s) => s,
            None if cfg!(debug_assertions) => "dev-art-cookie-secret".to_string(),
            None => anyhow::bail!("缺少 ART_COOKIE_SECRET（生产必填）"),
        };
        // 调试构建默认 seed 一个 admin/admin123 超管，免去本地配置
        let dev = cfg!(debug_assertions);

        let mail = MailConfig {
            enabled: env_bool("MAIL_ENABLED", false),
            provider: env_or("MAIL_PROVIDER", "auto"),
            from_name: env_or("MAIL_FROM_NAME", "春日商城"),
            from_address: env("MAIL_FROM_ADDRESS"),
            reply_to: env("MAIL_REPLY_TO"),
            resend_api_key: env("RESEND_API_KEY"),
            resend_api_base_url: env_or("RESEND_API_BASE_URL", "https://api.resend.com"),
            smtp_host: env("SMTP_HOST"),
            smtp_port: env_parse("SMTP_PORT", 465),
            smtp_secure: env_bool("SMTP_SECURE", true),
            smtp_user: env("SMTP_USER"),
            smtp_pass: env("SMTP_PASS"),
        };

        Ok(Config {
            bind,
            data_dir: PathBuf::from(env_or("HARUHI_DATA_DIR", "./data")),
            uploads_dir: PathBuf::from(env_or("HARUHI_UPLOADS_DIR", "./uploads")),
            jwt_secret,
            jwt_ttl_seconds: env_parse("HARUHI_JWT_TTL_SECONDS", 86_400),
            superadmin_user: env("HARUHI_SUPERADMIN_USER")
                .or_else(|| dev.then(|| "admin".to_string())),
            superadmin_password: env("HARUHI_SUPERADMIN_PASSWORD")
                .or_else(|| dev.then(|| "admin123".to_string())),
            dashscope_api_key: env("DASHSCOPE_API_KEY"),
            ai_api_url: env_or(
                "AI_API_URL",
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
            ),
            ai_text_model: env_or("AI_TEXT_MODEL", "qwen-plus"),
            ai_image_model: env_or("AI_IMAGE_MODEL", "qwen-vl-plus"),
            art_cookie_secret,
            shop_free_shipping_threshold: env_parse("SHOP_FREE_SHIPPING_THRESHOLD", 150),
            mail,
            public_site_url: env_or("PUBLIC_SITE_URL", "https://haruyuki.cn"),
            cors_origins: env("HARUHI_CORS_ORIGINS")
                .map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|x| !x.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    /// 某模块数据库文件路径，如 data/news.db。
    pub fn db_path(&self, name: &str) -> PathBuf {
        self.data_dir.join(format!("{name}.db"))
    }

    /// 某模块上传子目录，如 uploads/novel。
    pub fn uploads_subdir(&self, module: &str) -> PathBuf {
        self.uploads_dir.join(module)
    }
}
