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

        let jwt_secret = env("HARUHI_JWT_SECRET")
            .ok_or_else(|| anyhow::anyhow!("缺少 HARUHI_JWT_SECRET（必填）"))?;

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
            superadmin_user: env("HARUHI_SUPERADMIN_USER"),
            superadmin_password: env("HARUHI_SUPERADMIN_PASSWORD"),
            dashscope_api_key: env("DASHSCOPE_API_KEY"),
            ai_api_url: env_or(
                "AI_API_URL",
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
            ),
            ai_text_model: env_or("AI_TEXT_MODEL", "qwen-plus"),
            ai_image_model: env_or("AI_IMAGE_MODEL", "qwen-vl-plus"),
            art_cookie_secret: env_or("ART_COOKIE_SECRET", "dev-art-cookie-secret"),
            shop_free_shipping_threshold: env_parse("SHOP_FREE_SHIPPING_THRESHOLD", 150),
            mail,
            public_site_url: env_or("PUBLIC_SITE_URL", "https://haruyuki.cn"),
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
