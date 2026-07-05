//! 从环境变量加载强类型配置。关键密钥缺失时返回错误（fail-fast）。

use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub uploads_dir: PathBuf,
    /// 前端构建产物根目录（apps/<app>/dist）：SEO 模块读取 index.html 作 meta 注入模板。
    /// 生产 systemd WorkingDirectory=/var/www/haruhifanclub、开发从仓库根 cargo run，
    /// 默认相对路径 ./apps 两端通用。
    pub apps_dir: PathBuf,

    // 鉴权
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    /// 会话 cookie 有效期（秒）；登录写 sessions 行并下发 httpOnly cookie。
    pub session_ttl_seconds: i64,
    /// 会话/CSRF cookie 是否带 Secure 属性；默认按 public_site_url 是否 https 推导，可被 env 覆盖。
    pub cookie_secure: bool,
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
    /// 账号邮件链接（邮箱验证/找回密码）指向的前端基址。各前端 app 在各自子路径下，
    /// 默认指向主站 news（`{public_site_url}/news`）；可用 HARUHI_ACCOUNT_WEB_BASE 覆盖。
    pub account_web_base: String,

    // CORS 允许的来源（release 生效）；为空则默认仅 public_site_url。逗号分隔 HARUHI_CORS_ORIGINS。
    pub cors_origins: Vec<String>,

    // 资源站（download）：语雀知识库索引镜像
    /// 语雀 API Token（X-Auth-Token）；未配置则资源站同步不启动、前端拿到空索引。
    pub yuque_token: Option<String>,
    /// 语雀知识库 namespace（login/slug），默认「凉宫春日资源站」。
    pub yuque_repo: String,
    /// 资源站后台同步间隔（秒），默认 6 小时（最短 5 分钟，见 download 模块）。
    pub yuque_sync_interval_secs: u64,

    // 语音工坊（voice）：转发本地 TTS/RVC 服务（跑在团员本地、经 frp 上服务器）
    /// TTS（GPT-SoVITS integrated_server）基址；生产走 frp 域名 https://tts.haruyuki.cn。
    pub voice_tts_base: String,
    /// RVC（integrated_server）基址；生产走 frp 域名 https://rvc.haruyuki.cn。
    pub voice_rvc_base: String,
    /// 与两个本地服务约定的共享密钥（X-HFC-Voice-Key）；未配置则不带头（本地开发）。
    pub voice_shared_key: Option<String>,
    /// 探活间隔（秒），默认 60。
    pub voice_probe_interval_secs: u64,
    /// TTS 合成上游超时（秒），默认 180。
    pub voice_tts_timeout_secs: u64,
    /// RVC 转换上游超时（秒），默认 600（整曲可能数分钟）。
    pub voice_rvc_timeout_secs: u64,
    /// 同一用户两次提交之间的冷却（秒），默认 30。
    pub voice_user_cooldown_secs: u64,
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

        // 不安全的本地默认值（JWT/cookie 密钥、admin/admin123 超管）仅在
        // 「调试构建 且 绑定回环地址」时启用：防止误把 debug 二进制暴露到公网时
        // 仍带默认凭证。正常本地 `cargo run`（默认 127.0.0.1）行为不变。
        let dev = cfg!(debug_assertions) && bind.ip().is_loopback();
        if cfg!(debug_assertions) && !dev {
            tracing::warn!(
                "⚠ 调试构建绑定到非回环地址（{bind}），已禁用所有不安全默认值；\
                 JWT/ART_COOKIE 密钥与超管账号须显式配置"
            );
        }

        // 关键密钥：非 dev（release 或暴露的 debug）必填(fail-fast)；dev 用不安全默认值方便本地开箱即用
        let jwt_secret = match env("HARUHI_JWT_SECRET") {
            Some(s) => s,
            None if dev => {
                tracing::warn!("⚠ 未设置 HARUHI_JWT_SECRET，本地调试用不安全默认值（生产务必设置 ≥32 位随机串）");
                "dev-insecure-jwt-secret-change-me".to_string()
            }
            None => anyhow::bail!("缺少 HARUHI_JWT_SECRET（生产必填）"),
        };
        if jwt_secret.len() < 16 {
            tracing::warn!("⚠ HARUHI_JWT_SECRET 过短(<16 位)，建议 ≥32 位随机串");
        }
        let art_cookie_secret = match env("ART_COOKIE_SECRET") {
            Some(s) => s,
            None if dev => "dev-art-cookie-secret".to_string(),
            None => anyhow::bail!("缺少 ART_COOKIE_SECRET（生产必填）"),
        };
        // dev 下若未显式配置超管，则默认 seed admin/admin123——提示勿用于公网
        if dev
            && (env("HARUHI_SUPERADMIN_USER").is_none()
                || env("HARUHI_SUPERADMIN_PASSWORD").is_none())
        {
            tracing::warn!("⚠ 本地调试使用默认超管 admin/admin123，请勿用于公网环境");
        }

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

        let public_site_url = env_or("PUBLIC_SITE_URL", "https://haruyuki.cn");
        // cookie Secure 默认随站点协议；本地 http 调试自动关闭，生产 https 自动开启
        let cookie_secure = env_bool(
            "HARUHI_COOKIE_SECURE",
            public_site_url.starts_with("https://"),
        );
        // 账号邮件链接基址：默认主站 news 子路径
        let account_web_base = env("HARUHI_ACCOUNT_WEB_BASE")
            .unwrap_or_else(|| format!("{}/news", public_site_url.trim_end_matches('/')));

        Ok(Config {
            bind,
            data_dir: PathBuf::from(env_or("HARUHI_DATA_DIR", "./data")),
            uploads_dir: PathBuf::from(env_or("HARUHI_UPLOADS_DIR", "./uploads")),
            apps_dir: PathBuf::from(env_or("HARUHI_APPS_DIR", "./apps")),
            jwt_secret,
            jwt_ttl_seconds: env_parse("HARUHI_JWT_TTL_SECONDS", 86_400),
            // 会话默认 30 天（cookie 模式，可吊销，故可比 JWT 更长）
            session_ttl_seconds: env_parse("HARUHI_SESSION_TTL_SECONDS", 2_592_000),
            cookie_secure,
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
            public_site_url,
            account_web_base,
            cors_origins: env("HARUHI_CORS_ORIGINS")
                .map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|x| !x.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            yuque_token: env("HARUHI_YUQUE_TOKEN"),
            yuque_repo: env_or("HARUHI_YUQUE_REPO", "staff-sqlmik/phgf5z"),
            yuque_sync_interval_secs: env_parse("HARUHI_YUQUE_SYNC_INTERVAL_SECS", 21_600),
            voice_tts_base: env_or("HARUHI_VOICE_TTS_BASE", "http://127.0.0.1:9872"),
            voice_rvc_base: env_or("HARUHI_VOICE_RVC_BASE", "http://127.0.0.1:7865"),
            voice_shared_key: env("HARUHI_VOICE_KEY"),
            voice_probe_interval_secs: env_parse("HARUHI_VOICE_PROBE_INTERVAL_SECS", 60),
            voice_tts_timeout_secs: env_parse("HARUHI_VOICE_TTS_TIMEOUT_SECS", 180),
            voice_rvc_timeout_secs: env_parse("HARUHI_VOICE_RVC_TIMEOUT_SECS", 600),
            voice_user_cooldown_secs: env_parse("HARUHI_VOICE_USER_COOLDOWN_SECS", 30),
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
