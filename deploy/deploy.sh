#!/usr/bin/env bash
# ============================================================
# 生产部署脚本：本机构建前端 + Zig 交叉编译后端 → 推送到服务器 → 重启 systemd。
# 把 docs/DEPLOYMENT.md 的步骤固化成一条命令，便于重复、可回滚（旧二进制先备份）。
#
# 用法（在仓库根执行）：
#   HARUHI_DEPLOY_HOST=root@119.23.77.86 \
#   HARUHI_DEPLOY_ROOT=/var/www/haruhifanclub \
#   bash deploy/deploy.sh
#
# 环境变量：
#   HARUHI_DEPLOY_HOST     必填，ssh 目标，如 root@119.23.77.86
#   HARUHI_DEPLOY_ROOT     部署根，默认 /var/www/haruhifanclub
#   HARUHI_DEPLOY_SERVICE  systemd 单元名，默认 haruhifanclub
#   HARUHI_RUST_TARGET     Zig 目标，默认 x86_64-unknown-linux-gnu.2.17
#   HARUHI_TARGET_DIR      Rust 交叉编译目录，默认 target-zig
#   HARUHI_BACKEND_PORT    健康门禁探测端口，默认 17777（生产）；测试站传 17778
#   HARUHI_SKIP_FRONTEND=1 跳过前端构建（只发后端）
#   HARUHI_SKIP_BACKEND=1  跳过后端构建（只发前端）
#
# 前置：本机装 rustup、Zig、cargo-zigbuild、pnpm，ssh/rsync 可达服务器；
#       服务器已按 docs/DEPLOYMENT.md 配好 systemd 单元、nginx、env 文件、data/uploads。
# macOS 下必须使用 Zig 生成 Linux/amd64 ELF，不能上传本机 `cargo build --release` 的 Mach-O。
# ============================================================
set -euo pipefail

HOST="${HARUHI_DEPLOY_HOST:?需设置 HARUHI_DEPLOY_HOST，如 root@119.23.77.86}"
ROOT="${HARUHI_DEPLOY_ROOT:-/var/www/haruhifanclub}"
SERVICE="${HARUHI_DEPLOY_SERVICE:-haruhifanclub}"
RUST_TARGET="${HARUHI_RUST_TARGET:-x86_64-unknown-linux-gnu.2.17}"
TARGET_DIR="${HARUHI_TARGET_DIR:-target-zig}"
RUST_STD_TARGET="${RUST_TARGET%%.*}"

here="$(cd "$(dirname "$0")/.." && pwd)" # 仓库根
cd "$here"

if [ "${HARUHI_SKIP_FRONTEND:-0}" != "1" ]; then
  echo "==> [1/5] 构建前端（所有 app）"
  pnpm install --frozen-lockfile
  pnpm -r --filter "./apps/*" build
else
  echo "==> [1/5] 跳过前端构建（HARUHI_SKIP_FRONTEND=1）"
fi

if [ "${HARUHI_SKIP_BACKEND:-0}" != "1" ]; then
  echo "==> [2/5] Zig 交叉编译后端（${RUST_TARGET}）"
  for tool in rustup zig cargo-zigbuild; do
    command -v "$tool" >/dev/null 2>&1 || {
      echo "✗ 未找到 ${tool}，请按 docs/DEPLOYMENT.md 安装构建依赖" >&2
      exit 1
    }
  done
  rustup target list --installed | grep -qx "$RUST_STD_TARGET" || {
    echo "✗ Rust 缺少 ${RUST_STD_TARGET} 标准库，请执行：rustup target add ${RUST_STD_TARGET}" >&2
    exit 1
  }

  # 某些 macOS 环境把 Homebrew cargo 放在 rustup 前面；显式使用仓库锁定工具链，
  # 否则目标标准库虽已安装，Homebrew rustc 仍会误报 can't find crate for core。
  rustup_cargo="$(rustup which cargo)"
  rustup_bin="$(dirname "$rustup_cargo")"
  PATH="$rustup_bin:$PATH" "$rustup_cargo" zigbuild \
    --locked \
    --release \
    -p haruhi-server \
    --features deploy-vendored-openssl \
    --target "$RUST_TARGET" \
    --target-dir "$TARGET_DIR"

  backend_binary="$TARGET_DIR/$RUST_STD_TARGET/release/haruhi-server"
  test -f "$backend_binary" || {
    echo "✗ 未找到 ${backend_binary}" >&2
    exit 1
  }

  echo "==> [3/5] 推送后端二进制 → ${HOST}:${ROOT}/bin/"
  ssh "$HOST" "mkdir -p '$ROOT/bin'"
  # 原子替换：先传临时名 + 备份旧二进制 + mv，避免覆盖正在运行的进程映像导致异常
  rsync -az "$backend_binary" "$HOST:$ROOT/bin/haruhi-server.new"
  ssh "$HOST" "cd '$ROOT/bin' && { [ -f haruhi-server ] && cp -f haruhi-server haruhi-server.bak || true; } && mv -f haruhi-server.new haruhi-server && chmod +x haruhi-server"
else
  echo "==> [2-3/5] 跳过后端构建/推送（HARUHI_SKIP_BACKEND=1）"
fi

if [ "${HARUHI_SKIP_FRONTEND:-0}" != "1" ]; then
  echo "==> [4/5] 推送前端产物（各 app 的 dist）→ ${HOST}:${ROOT}/apps/"
  for d in apps/*/dist; do
    [ -d "$d" ] || continue
    app="$(basename "$(dirname "$d")")"
    ssh "$HOST" "mkdir -p '$ROOT/apps/$app/dist'"
    rsync -az --delete "$d/" "$HOST:$ROOT/apps/$app/dist/"
  done
else
  echo "==> [4/5] 跳过前端推送"
fi

echo "==> [5/5] 重启服务并执行健康门禁"
# 数据库迁移在服务启动时自动执行（Pools::migrate）；env 取自 systemd EnvironmentFile。
ssh "$HOST" "systemctl restart '$SERVICE'"

# 健康门禁：服务必须 active 且 /api/health/ready 200，否则部署判定失败、退出非零。
# 此前这里用 `|| true` 吞掉重启失败，服务崩溃循环时仍打印"✓ 部署完成"
# （2026-06-12 迁移校验事故即因此漏过），门禁不可再弱化。
PORT="${HARUHI_BACKEND_PORT:-17777}"
echo "    等待 ${SERVICE} 就绪（127.0.0.1:${PORT}/api/health/ready，最多约 24s）"
healthy=0
for _ in $(seq 1 12); do
  sleep 2
  if ssh "$HOST" "systemctl is-active --quiet '$SERVICE' && curl -fsS -m 3 http://127.0.0.1:${PORT}/api/health/ready >/dev/null"; then
    healthy=1
    break
  fi
done
if [ "$healthy" != "1" ]; then
  echo "✗ 部署失败：${SERVICE} 未通过健康检查，最近状态与日志：" >&2
  ssh "$HOST" "systemctl --no-pager --lines=8 status '$SERVICE'; journalctl -u '$SERVICE' -n 20 --no-pager" >&2 || true
  echo "  回滚后端：ssh ${HOST} \"cd ${ROOT}/bin && mv -f haruhi-server.bak haruhi-server && systemctl restart ${SERVICE}\"" >&2
  exit 1
fi

echo "✓ 部署完成并通过健康检查：${SERVICE} @ ${HOST}:${ROOT}（127.0.0.1:${PORT}）"
echo "  回滚后端：ssh ${HOST} \"cd ${ROOT}/bin && mv -f haruhi-server.bak haruhi-server && systemctl restart ${SERVICE}\""
