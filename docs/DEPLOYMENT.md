# 部署指南（从零到生产）

> 目标：单台 Linux 服务器上跑起 `haruhifanclub` —— 一个 Rust 后端二进制（systemd 守护，监听
> `127.0.0.1:17777`）+ 各前端 `apps/*/dist` 静态产物，由 Nginx 反代 `/api`、`/uploads` 并托管子路径。
> 所有部署资产在 `deploy/`。
>
> 本仓已对齐单版本工具链基线（Vite 7 + Pinia 3 全仓单版本，pnpm 单 lockfile），构建命令对所有 app 一致；
> 上传 URL 拼接抽到 `@haruhi/api-client` 的 `resolveUploadUrl`、数值解析抽到 `core::parse`，
> 部署流程本身不受影响。协作/发布基建见根 `CONTRIBUTING.md`、`docs/COLLABORATION.md`、`cliff.toml`（changelog）。

## 0. 现状速览

| 站点 | 域名 | 部署根 | 后端端口 | systemd 单元 | nginx 配置 |
|---|---|---|---|---|---|
| 测试站（当前在用） | `test.haruyuki.cn` | `/var/www/haruhifanclub-test` | `127.0.0.1:17777` | `haruhifanclub-test.service` | `deploy/test.haruyuki.cn.nginx.conf` |
| 生产（模板） | `haruyuki.cn` | `/var/www/haruhifanclub` | `127.0.0.1:17777` | `haruhifanclub.service` | `deploy/nginx.conf` |

> 测试站 `test.haruyuki.cn` 是目前实际跑的环境：端口 `17777`、部署根 `/var/www/haruhifanclub-test`、
> 后端 `EnvironmentFile=.env`（在部署根下）、HTTPS 由 `test.haruyuki.cn` 的 Let's Encrypt 证书提供。
> 备份 timer（§7）也默认指向 `-test` 根。生产单元用 `/etc/haruhifanclub.env`（权限 600）。

## 1. 系统依赖

```bash
# Node 20 + pnpm 10（前端构建）
nvm install 20 && nvm use 20            # .nvmrc 锁定 20
corepack enable && corepack prepare pnpm@10.11.0 --activate

# Rust stable（本机直接编 native 时需要；用 Docker 交叉编译则服务器免装）
curl https://sh.rustup.rs -sSf | sh    # rust-toolchain.toml 锁定 stable + rustfmt/clippy

# 运行/构建期工具
apt-get install -y ffmpeg sqlite3 nginx   # ffmpeg: 音频处理；sqlite3: 迁移/备份；nginx: 反代
# certbot（HTTPS，§8）
apt-get install -y certbot
```

版本约束：`package.json` 要求 Node ≥20、`packageManager: pnpm@10.11.0`、`.nvmrc=20`；
Rust 见 `rust-toolchain.toml`（stable，含 rustfmt/clippy），`Cargo.toml` 的最低 `rust-version = 1.80`。

## 2. 构建

### 前端（所有 app）

```bash
pnpm install --frozen-lockfile
pnpm -r --filter "./apps/*" build      # = 根脚本 build:apps；产物 → apps/<app>/dist（base 已指向各自子路径）
```

> 全仓 6 个 app 统一 Vite 7、其中 news/art 用 Pinia 3。`exam`、`console` 是 TS app，其
> `build` 含 `vue-tsc --noEmit`（类型检查）后再 `vite build`，构建较慢但不影响产物形态。

### 后端二进制

本机原生编译：

```bash
cargo build --release -p haruhi-server  # → target/release/haruhi-server
```

**生产服务器是 linux/amd64，推荐用 Docker 交叉编译**出 Linux 二进制（无需在服务器装 Rust，也避免本机
是 macOS/arm 的架构不匹配）。我们实际用的镜像是 `rust:1.87-bookworm`（满足 `Cargo.toml` 的
`rust-version = 1.80` 下限，对应 `rust-toolchain.toml` 的 stable 通道）：

```bash
# 在 monorepo 根执行；输出到独立 target-linux/（与本机 target/ 隔离，复用缓存）
docker run --rm --platform linux/amd64 \
  -v "$PWD":/app -w /app \
  -e CARGO_TARGET_DIR=/app/target-linux \
  rust:1.87-bookworm \
  cargo build --release -p haruhi-server
# 产物：target-linux/release/haruhi-server
```

把二进制放到部署根的 `bin/`（systemd `ExecStart` 指向 `bin/haruhi-server`）：

```bash
install -D target-linux/release/haruhi-server /var/www/haruhifanclub-test/bin/haruhi-server
# 同步前端产物（含目录结构 apps/<app>/dist）：
rsync -a --delete apps/ /var/www/haruhifanclub-test/apps/  # 仅同步含 dist 的 app 目录
```

## 3. 环境变量

模板：`deploy/env.sample`。生产环境复制为后端读取的文件（测试站放部署根 `.env`，生产放
`/etc/haruhifanclub.env` 权限 600）：

```bash
cp deploy/env.sample /var/www/haruhifanclub-test/.env   # 测试站
# 或：cp deploy/env.sample /etc/haruhifanclub.env && chmod 600 /etc/haruhifanclub.env  # 生产
```

**release 必填，否则启动即失败（fail-fast，见 `backend/crates/core/src/config.rs`；debug 构建用不安全默认值方便本地）：**

| 变量 | 说明 |
|---|---|
| `HARUHI_JWT_SECRET` | JWT 签名密钥，**≥32 位随机串**。缺失 → release 拒绝启动（<16 位仅告警） |
| `ART_COOKIE_SECRET` | art 匿名 Cookie 签名。缺失 → release 拒绝启动 |

常用其它项：

| 变量 | 默认/建议 |
|---|---|
| `HARUHI_BIND` | `127.0.0.1:17777`（与 Nginx upstream / service 对齐） |
| `HARUHI_DATA_DIR` / `HARUHI_UPLOADS_DIR` | `./data` / `./uploads`（相对 `WorkingDirectory`） |
| `HARUHI_JWT_TTL_SECONDS` | `86400` |
| `HARUHI_CORS_ORIGINS` | 逗号分隔来源；同源部署可留空（默认仅 `PUBLIC_SITE_URL`，release 生效，debug 放宽为 Any） |
| `HARUHI_SUPERADMIN_USER` / `_PASSWORD` | 首启 seed 超管（仅 `core.db` 无用户时生效，见 `seed.rs`；未配置则跳过并告警） |
| `PUBLIC_SITE_URL` | `https://haruyuki.cn`（CORS 默认来源 + 邮件链接） |
| `DASHSCOPE_API_KEY` | 空则 AI 审核离线放行；可配 `AI_API_URL` / `AI_TEXT_MODEL` / `AI_IMAGE_MODEL` |
| `MAIL_*` / `RESEND_*` / `SMTP_*` | shop 下单邮件；`MAIL_ENABLED=false` 时不启 worker，`MAIL_PROVIDER=auto\|resend\|smtp` |
| `SHOP_FREE_SHIPPING_THRESHOLD` | `150` |
| `RUST_LOG` | `info,haruhi=debug` |

> 首次启动会自动跑迁移（`Pools::migrate`）并 seed 超管；之后改 `HARUHI_SUPERADMIN_*` 不再生效，
> 改密码走 `/console/`。

## 4. systemd 守护

生产单元 `deploy/haruhifanclub.service`：

```bash
cp deploy/haruhifanclub.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now haruhifanclub
journalctl -u haruhifanclub -f          # 看日志
```

关键点：

- `ExecStart=/var/www/haruhifanclub/bin/haruhi-server`，`EnvironmentFile=/etc/haruhifanclub.env`，`User=www-data`，`WorkingDirectory=/var/www/haruhifanclub`。
- `KillSignal=SIGTERM` + **`TimeoutStopSec=30`**：配合后端优雅关闭——收到 SIGTERM（或本地 Ctrl-C）后停收新请求、
  等在途请求结束、对每个库做 `PRAGMA wal_checkpoint(TRUNCATE)` 刷盘再退出（见 `main.rs`）。30s 内退完才不会被 SIGKILL，
  保证不丢数据/不损坏 WAL。
- `ReadWritePaths=.../data .../uploads`：仅这两目录可写。
- `Restart=on-failure` / `RestartSec=3`。

测试站用 `deploy/haruhifanclub-test.service`（`WorkingDirectory=/var/www/haruhifanclub-test`，
`EnvironmentFile=.env`，无 `User=`、无 `ReadWritePaths=`），逻辑同上。

## 5. Nginx 子路径反代

生产配置 `deploy/nginx.conf`（测试站对应 `deploy/test.haruyuki.cn.nginx.conf`，按 `server_name` 路由、互不影响）：

```bash
cp deploy/nginx.conf /etc/nginx/sites-available/haruyuki.cn
ln -s /etc/nginx/sites-available/haruyuki.cn /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx
```

要点：

- `upstream haruhi_backend { server 127.0.0.1:17777; }`，`location /api/` 反代并注入
  `X-Real-IP` / `X-Forwarded-For`（限流取 IP 依赖它，见 `ratelimit.rs`）/ `X-Forwarded-Proto`，`proxy_read_timeout 120s`。
- `location /uploads/` 用 `alias` 直接服务磁盘文件（`expires 7d`、`access_log off`、`try_files $uri =404`，绕过后端提速）。
- 各前端 app 子路径 `alias` 到 `apps/<app>/dist` + SPA history 回退（`try_files $uri $uri/ /<base>/index.html`）：
  `/news/ /art/ /exam/ /library/(=novel) /shop/ /console/`；`/` 302 跳 `/news/`。
- `client_max_body_size 256m`（与后端 `DefaultBodyLimit` 的 `256 * 1024 * 1024` 对齐，见 `routes.rs`）。
- 安全响应头（纵深防御）：`Strict-Transport-Security` / `X-Content-Type-Options: nosniff` /
  `X-Frame-Options: SAMEORIGIN` / `Referrer-Policy: strict-origin-when-cross-origin`。

## 6. 数据迁移（从旧 5 个站迁入）

统一约定：每个旧站的 sqlite 拷进 `data/<module>.db` 并启 WAL，uploads 拷进 `uploads/<module>/`，
同时把数据库里残留的旧 URL 前缀（`uploads/...`、各旧站自带前缀等）重写到统一的
`/uploads/<module>/`。两种用法：

### 本地/已有旧项目目录 → `deploy/migrate-data.sh`

```bash
bash deploy/migrate-data.sh            # 全部模块
bash deploy/migrate-data.sh novel art  # 指定模块
```

读取脚本顶部写死的旧项目路径（`NEWS_SRC=/Volumes/data/harunews`、`ART_SRC=/Volumes/data/haruhi-art-club`、
`EXAM_SRC=/Volumes/data/haruhi-exam-platform/...`、`NOVEL_SRC=/Volumes/data/haruhi-novel-reader`、
`SHOP_SRC=/Users/haruhi/haruhishop`），逐模块拷库 + `rsync` uploads + `sqlite3` 重写前缀。注意：

- **art** 空库时跳过拷贝（让后端迁移建全新完整 schema），有数据时补齐运行期才加的列（`ai_reason`/`images_json`/…）。
- **news** 图片前缀不一致，单独交给 `deploy/migrate-news.py` 规整（汇集散落图片到 `uploads/news/`）。

### 服务器上从“正在跑的旧生产”抓最新数据 → `deploy/migrate-live.py`

```bash
# 在旧项目所在服务器执行（读 /root/<旧项目>/...）；可用 DEST_DATA/DEST_UP 覆盖目标
python3 deploy/migrate-live.py
```

用 `sqlite3 .backup`（一致性快照，不扰动 live 服务）+ `rsync` 只读拷贝 uploads，再做同样的前缀重写。
默认目标是 `-test` 部署根（`DEST_DATA=/var/www/haruhifanclub-test/data`、`DEST_UP=.../uploads`）。

> 迁移后重启后端（`systemctl restart haruhifanclub[-test]`），`migrate()` 会对各库补跑结构迁移（幂等）。

## 7. 备份

`deploy/backup.sh`：对 `data/*.db` 做 `sqlite3 .backup`（WAL 下安全的一致性快照）+ 打包 `uploads/`，
按时间戳归档到 `HARUHI_BACKUP_DIR`，保留 `HARUHI_BACKUP_KEEP`（默认 14）天后轮转删除。

由 systemd timer 每日 04:00 调度（错过开机补跑）：

```bash
cp deploy/haruhifanclub-backup.service deploy/haruhifanclub-backup.timer /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now haruhifanclub-backup.timer
systemctl start haruhifanclub-backup.service   # 手动跑一次验证
```

> `.service` 里写死 `HARUHI_ROOT=/var/www/haruhifanclub-test`、输出 `/var/backups/haruhifanclub`、保留 14 份——
> 切到生产记得改 `HARUHI_ROOT`。

## 8. HTTPS（certbot webroot）

测试站 nginx 已留 ACME 验证位：`location ^~ /.well-known/acme-challenge/ { root /var/www/haruhifanclub-test; }`。

```bash
certbot certonly --webroot -w /var/www/haruhifanclub-test -d test.haruyuki.cn
# 证书签发后，nginx 的 ssl_certificate 指向 /etc/letsencrypt/live/test.haruyuki.cn/...（已在配置中）
systemctl reload nginx
# certbot 自带 systemd timer 自动续期；续期后 reload nginx
```

生产 `deploy/nginx.conf` 里 `ssl_certificate` / `ssl_certificate_key` 两行默认注释，签好 `haruyuki.cn` 证书后取消注释。

## 9. 上线后自检

```bash
# liveness：进程在跑（返回 {"status":"ok","service":"haruhifanclub"}）
curl -s http://127.0.0.1:17777/api/health
# readiness：核心库连通（SELECT 1 成功才 200；不通则 503），供巡检/负载判断是否真可服务
curl -s http://127.0.0.1:17777/api/health/ready        # {"status":"ready"}
curl -s https://test.haruyuki.cn/api/health
journalctl -u haruhifanclub-test -n 50                 # 看“数据库迁移完成 / 后端启动”
# 用 seed 的超管登录 /console/ → 给管理员按 app 分配角色
```
