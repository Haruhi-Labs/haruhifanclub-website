# 部署

目标环境是一台 Linux 服务器：

- Nginx 托管各前端 app 的 `dist/`。
- Nginx 反代 `/api/` 到 `haruhi-server`。
- Nginx 直接服务 `/uploads/`。
- `haruhi-server` 由 systemd 守护。
- `data/` 和 `uploads/` 留在服务器持久化并备份。

## 默认路径

| 环境 | 域名               | 部署根                        | systemd 单元                 | env 文件                           | Nginx 配置                           |
| ---- | ------------------ | ----------------------------- | ---------------------------- | ---------------------------------- | ------------------------------------ |
| 生产 | `haruyuki.cn`      | `/var/www/haruhifanclub`      | `haruhifanclub.service`      | `/etc/haruhifanclub.env`           | `deploy/nginx.conf`                  |
| 测试 | `test.haruyuki.cn` | `/var/www/haruhifanclub-test` | `haruhifanclub-test.service` | `/var/www/haruhifanclub-test/.env` | `deploy/test.haruyuki.cn.nginx.conf` |

后端监听地址：生产固定 `127.0.0.1:17777`；测试站与生产同机共存，固定 `127.0.0.1:17778`（`deploy/deploy.sh` 部署测试站时传 `HARUHI_BACKEND_PORT=17778` 使健康门禁探测正确端口）。

## 服务器依赖

```bash
apt-get update
apt-get install -y nginx sqlite3 ffmpeg libvips-tools rsync certbot
```

`libvips-tools` 提供 `vips` 命令，art 画廊缩略图由后端用它流式生成（shrink-on-load、内存有界），替代进程内全解码——避免巨图把小内存机器打爆。缺失时缩略图端点会回退原图（不裂图但失去压缩收益）。

如果只用 `deploy/deploy.sh` 从本机构建并推送，服务器不需要 Node、pnpm 或 Rust。构建机需要：

- pnpm `10.11.0`
- Docker
- ssh/rsync 能访问服务器

## 环境变量

推荐用脚本生成：

```bash
# 生产
bash deploy/gen-secrets.sh /etc/haruhifanclub.env
chmod 600 /etc/haruhifanclub.env

# 测试站或本地
bash deploy/gen-secrets.sh /var/www/haruhifanclub-test/.env
```

也可以复制模板后手动改：

```bash
cp deploy/env.sample /etc/haruhifanclub.env
chmod 600 /etc/haruhifanclub.env
```

关键变量：

| 变量                                                    | 说明                                                          |
| ------------------------------------------------------- | ------------------------------------------------------------- |
| `HARUHI_BIND`                                           | 后端监听地址，部署/Vite 代理按 `127.0.0.1:17777`              |
| `HARUHI_DATA_DIR`                                       | SQLite 目录，通常是 `./data`，相对 systemd `WorkingDirectory` |
| `HARUHI_UPLOADS_DIR`                                    | 上传目录，通常是 `./uploads`                                  |
| `HARUHI_JWT_SECRET`                                     | JWT 签名密钥；release 构建缺失会启动失败                      |
| `ART_COOKIE_SECRET`                                     | art 匿名 Cookie 签名密钥；release 构建缺失会启动失败          |
| `HARUHI_SUPERADMIN_USER` / `HARUHI_SUPERADMIN_PASSWORD` | 首次启动 seed 超管，仅 `core.db` 无用户时生效                 |
| `HARUHI_CORS_ORIGINS`                                   | release 下允许的来源，逗号分隔；留空时用 `PUBLIC_SITE_URL`    |
| `PUBLIC_SITE_URL`                                       | 站点公开地址，也用于邮件链接                                  |
| `DASHSCOPE_API_KEY`                                     | 空值时 AI 审核放行                                            |
| `MAIL_ENABLED`                                          | `true` 时启用 shop 邮件发送                                   |
| `MAIL_PROVIDER`                                         | `auto`、`resend` 或 `smtp`                                    |

`EnvironmentFile` 的格式要保持简单：不要写 `export`，不要给值加 shell 引号，不要在同一行末尾写注释。`deploy/env.sample` 已按这个规则编写。

## 初次部署目录

```bash
mkdir -p /var/www/haruhifanclub/{bin,apps,data,uploads}
chown -R www-data:www-data /var/www/haruhifanclub/data /var/www/haruhifanclub/uploads
```

测试站对应把路径换成 `/var/www/haruhifanclub-test`。测试单元现已与生产对等：带 `ReadWritePaths` 与沙箱加固指令（仅 `User=www-data` 按需自行决定），按文件里的注释使用。

## 构建与推送

常规部署使用脚本：

```bash
HARUHI_DEPLOY_HOST=root@<server> \
HARUHI_DEPLOY_ROOT=/var/www/haruhifanclub \
HARUHI_DEPLOY_SERVICE=haruhifanclub \
bash deploy/deploy.sh
```

脚本会执行：

1. `pnpm install --frozen-lockfile`
2. `pnpm -r --filter "./apps/*" build`
3. Docker 以 `linux/amd64` 编译 `haruhi-server`
4. rsync 前端 `dist/` 到服务器
5. 上传后端二进制，备份旧二进制为 `haruhi-server.bak`
6. `systemctl restart <service>` 并执行**健康门禁**：等待服务 active 且 `/api/health/ready` 返回 200（最多约 24 秒），未通过则输出状态与日志、给出回滚命令并以非零码退出——服务起不来时绝不会打印"部署完成"

可选变量：

| 变量                                   | 作用                                                       |
| -------------------------------------- | ---------------------------------------------------------- |
| `HARUHI_SKIP_FRONTEND=1`               | 只发后端                                                   |
| `HARUHI_SKIP_BACKEND=1`                | 只发前端                                                   |
| `HARUHI_RUST_IMAGE=rust:1.87-bookworm` | 修改交叉编译镜像                                           |
| `HARUHI_BACKEND_PORT=17778`            | 健康门禁探测端口，默认 17777（生产）；部署测试站时传 17778 |

本机手动构建命令：

```bash
pnpm build:apps
cargo build --release -p haruhi-server
```

注意：这会生成构建机平台的二进制。macOS/arm 构建产物不能直接放到 Linux/amd64 服务器上运行。

## systemd

生产：

```bash
cp deploy/haruhifanclub.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now haruhifanclub
journalctl -u haruhifanclub -f
```

服务文件要点：

- `WorkingDirectory=/var/www/haruhifanclub`
- `EnvironmentFile=/etc/haruhifanclub.env`
- `ExecStart=/var/www/haruhifanclub/bin/haruhi-server`
- `User=www-data`
- `ReadWritePaths=/var/www/haruhifanclub/data /var/www/haruhifanclub/uploads`
- `KillSignal=SIGTERM`
- `TimeoutStopSec=30`

`SIGTERM` 会触发后端关停流程：停止接收新请求，等待在途请求，对各 SQLite 库做 WAL checkpoint，再关闭连接池。

单元还启用了一组沙箱加固指令（纵深防御）：`NoNewPrivileges`、`ProtectSystem=strict`、`ProtectHome`、`PrivateTmp`、`RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX`、`SystemCallFilter=@system-service`、`UMask=0077` 等。可写目录只有 `ReadWritePaths` 列出的 `data/`、`uploads/`。`@system-service` 已覆盖音频转码所需的 ffmpeg `fork`/`exec`；若转码异常可临时放宽该项排查。

测试站：

```bash
cp deploy/haruhifanclub-test.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now haruhifanclub-test
journalctl -u haruhifanclub-test -f
```

## Nginx

生产：

```bash
cp deploy/nginx.conf /etc/nginx/sites-available/haruyuki.cn
ln -s /etc/nginx/sites-available/haruyuki.cn /etc/nginx/sites-enabled/haruyuki.cn
nginx -t
systemctl reload nginx
```

配置要点：

- `/api/` 反代 `127.0.0.1:17777`。
- `/uploads/art/.thumbs/` 静态直出画廊缩略图；未命中走 `@thumb_fallback` 回源 `/api/art/thumb` 现场生成并落盘，下次即静态命中（`^~` 前缀保证优先级高于 `/uploads/` 与任何正则 location）。
- `/uploads/` alias 到部署根的 `uploads/`。
- `/news/`、`/art/`、`/exam/`、`/library/`、`/shop/`、`/console/`、`/design-system/` alias 到各 app 的 `dist/`。
- SPA 子路径使用 `try_files ... /<base>/index.html`。
- `client_max_body_size 256m` 与后端 body 上限一致。
- `server_tokens off` 不暴露 nginx 版本。
- 安全响应头：HSTS（含 `includeSubDomains`）、`X-Content-Type-Options`、`X-Frame-Options`、`Referrer-Policy`。
- `/uploads/` 额外加 `Content-Security-Policy: default-src 'none'; ... ; sandbox`，杜绝用户上传的恶意 SVG 被直链打开时执行脚本。
- `/` 跳转到 `/news/`。

签发 HTTPS 证书后，把 `deploy/nginx.conf` 里的 `ssl_certificate` 和 `ssl_certificate_key` 两行改成实际证书路径并取消注释。配置已预置 TLS 加固（`ssl_protocols TLSv1.2 TLSv1.3`、现代 `ssl_ciphers`、`ssl_session_cache`），证书启用后即生效。

> 全站 CSP 未默认开启：各前端用到的外部源（OpenCC、Live2D、字体等）需逐一允许，贸然加 `default-src 'self'` 会破坏页面；`deploy/nginx.conf` 给了注释示例，联调后按需启用。

### 画廊缩略图预热（backfill）

新部署或导入存量图后，一次性预生成缩略图，使 nginx 全程静态命中、后端零回源：

```bash
HARUHI_ROOT=/var/www/haruhifanclub bash deploy/backfill-thumbs.sh
```

幂等（已存在的跳过）、串行执行、用 `vips` 流式生成（内存有界），可对任意大图安全运行。缓存落在 `uploads/art/.thumbs/<w>/`，属可再生数据，备份可排除、可整目录删除重建。新上传的作品由后端自动预热 640 档，无需手动跑。

测试站配置已经包含 `test.haruyuki.cn` 的 ACME webroot 和 HTTPS 路径：

```bash
cp deploy/test.haruyuki.cn.nginx.conf /etc/nginx/sites-available/test.haruyuki.cn
ln -s /etc/nginx/sites-available/test.haruyuki.cn /etc/nginx/sites-enabled/test.haruyuki.cn
nginx -t
systemctl reload nginx
```

## 备份

`deploy/backup.sh` 对 `data/*.db` 执行 `sqlite3 .backup`，并打包 `uploads/`：

```bash
HARUHI_ROOT=/var/www/haruhifanclub \
HARUHI_BACKUP_DIR=/var/backups/haruhifanclub \
HARUHI_BACKUP_KEEP=30 \
HARUHI_BACKUP_REMOTE=user@nas:/backups/haruhifanclub \
bash deploy/backup.sh
```

`HARUHI_BACKUP_KEEP` 默认 30 天（用户内容站建议 ≥30）。设置 `HARUHI_BACKUP_REMOTE`（`user@host:/path`）后会 rsync 一份到异地，防本机磁盘损坏/勒索连本地备份一起丢；不设则跳过。

启用 systemd timer：

```bash
cp deploy/haruhifanclub-backup.service deploy/haruhifanclub-backup.timer /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now haruhifanclub-backup.timer
systemctl start haruhifanclub-backup.service
```

切到生产前检查 `deploy/haruhifanclub-backup.service` 里的 `HARUHI_ROOT` 是否仍指向测试站。

## 上线自检

在服务器上：

```bash
curl -fsS http://127.0.0.1:17777/api/health
curl -fsS http://127.0.0.1:17777/api/health/ready
journalctl -u haruhifanclub -n 80 --no-pager
```

从公网：

```bash
curl -fsS https://haruyuki.cn/api/health
curl -I https://haruyuki.cn/news/
curl -I https://haruyuki.cn/uploads/
```

再登录 `/console/`，确认超管账号可用，并给普通管理员分配对应 app 角色。
