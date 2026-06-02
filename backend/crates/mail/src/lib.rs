//! haruhi-mail：统一邮件发送（Resend API + SMTP 双驱动），替代旧 shop 的 nodemailer/Resend。
//! 只负责"把一封邮件发出去"；邮件队列(email_jobs)的重试/触发由 shop 模块在其库上实现。

use haruhi_core::{Config, MailConfig};
use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Clone)]
pub struct Mailer {
    cfg: MailConfig,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Provider {
    Resend,
    Smtp,
}

impl Mailer {
    /// 邮件未启用时返回 None；业务层据此跳过发送。
    pub fn from_config(cfg: &Config) -> Option<Self> {
        if !cfg.mail.enabled {
            return None;
        }
        Some(Self {
            cfg: cfg.mail.clone(),
            http: reqwest::Client::new(),
        })
    }

    fn resolve_provider(&self) -> anyhow::Result<Provider> {
        match self.cfg.provider.as_str() {
            "resend" => Ok(Provider::Resend),
            "smtp" => Ok(Provider::Smtp),
            _ => {
                // auto：有 Resend key 优先 Resend，否则 SMTP
                if self.cfg.resend_api_key.is_some() {
                    Ok(Provider::Resend)
                } else if self.cfg.smtp_host.is_some() {
                    Ok(Provider::Smtp)
                } else {
                    anyhow::bail!("未配置 Resend 或 SMTP")
                }
            }
        }
    }

    fn from_mailbox(&self) -> anyhow::Result<Mailbox> {
        let addr = self
            .cfg
            .from_address
            .clone()
            .ok_or_else(|| anyhow::anyhow!("缺少 MAIL_FROM_ADDRESS"))?;
        format!("{} <{}>", self.cfg.from_name, addr)
            .parse()
            .map_err(|e| anyhow::anyhow!("发件人地址非法: {e}"))
    }

    /// 发送一封邮件（同时含纯文本与 HTML）。
    pub async fn send(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: &str,
    ) -> anyhow::Result<()> {
        match self.resolve_provider()? {
            Provider::Resend => self.send_resend(to, subject, html, text).await,
            Provider::Smtp => self.send_smtp(to, subject, html, text).await,
        }
    }

    async fn send_resend(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: &str,
    ) -> anyhow::Result<()> {
        let key = self
            .cfg
            .resend_api_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("缺少 RESEND_API_KEY"))?;
        let from = self.from_mailbox()?.to_string();
        let url = format!("{}/emails", self.cfg.resend_api_base_url.trim_end_matches('/'));
        let mut body = serde_json::json!({
            "from": from,
            "to": [to],
            "subject": subject,
            "html": html,
            "text": text,
        });
        if let Some(reply) = &self.cfg.reply_to {
            body["reply_to"] = serde_json::json!(reply);
        }
        let resp = self
            .http
            .post(&url)
            .bearer_auth(key)
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            anyhow::bail!("Resend 发送失败 {status}: {txt}");
        }
        Ok(())
    }

    async fn send_smtp(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: &str,
    ) -> anyhow::Result<()> {
        let host = self
            .cfg
            .smtp_host
            .clone()
            .ok_or_else(|| anyhow::anyhow!("缺少 SMTP_HOST"))?;
        let email = Message::builder()
            .from(self.from_mailbox()?)
            .to(to.parse().map_err(|e| anyhow::anyhow!("收件人非法: {e}"))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html.to_string()),
                    ),
            )?;

        // 465 隐式 TLS 用 relay；其它端口用 starttls。
        let mut builder = if self.cfg.smtp_secure {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)?
        };
        builder = builder.port(self.cfg.smtp_port);
        if let (Some(user), Some(pass)) = (&self.cfg.smtp_user, &self.cfg.smtp_pass) {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }
        let transport = builder.build();
        transport.send(email).await?;
        Ok(())
    }
}
