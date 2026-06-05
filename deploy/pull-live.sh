#!/usr/bin/env bash
# 从生产服务器拉取最新真实数据到本地 monorepo 的 data/ 与 uploads/。
#
# 适用前提（已确认）：haruyuki.cn 仍由 5 个旧 Express 站提供生产服务，真实数据在
# 服务器 /root/<旧项目> 的库里；新统一后端尚未接管生产。
#
# 机制：把 deploy/migrate-live.py 推到服务器执行——它对线上旧库做 sqlite3 .backup（一致性
# 快照、只读、不扰动 live 服务），对 uploads 做只读 rsync，并把旧 URL 前缀改写成统一 /uploads/<m>/。
# 这里用环境变量 DEST_DATA/DEST_UP 把产物落到服务器 /tmp 暂存目录，**不碰正在运行的 test 部署**，
# 然后 rsync 回本地。
#
# 用法：  HARUHI_DEPLOY_HOST=root@119.23.77.86 bash deploy/pull-live.sh
#
# 安全保证：
#   - 每次运行先把本地 data/ 备份到 ~/haruhi-data-backups/（可回滚）。
#   - 不覆盖本地 core.db（用户 / 角色 / RBAC 是本地体系，不属于旧站生产数据）。
#   - 对服务器只读，唯一的写是 /tmp 暂存目录，结束时清理。
set -euo pipefail
cd "$(dirname "$0")/.."

HOST="${HARUHI_DEPLOY_HOST:?需设置 HARUHI_DEPLOY_HOST，如 root@119.23.77.86}"
REMOTE_STAGE="/tmp/hfc-pull"
MODULES="novel news art exam shop"

main_table() { # 各模块主表（用于行数核对）
  case "$1" in
    novel) echo books ;; news) echo articles ;; art) echo artworks ;;
    exam) echo exams ;; shop) echo products ;;
  esac
}

echo "==> 1/5 备份本地 data/（可回滚）"
BK=~/haruhi-data-backups; mkdir -p "$BK"
ts="$(date +%Y%m%d-%H%M%S)"
tar czf "$BK/data-$ts.tgz" data 2>/dev/null && echo "    → $BK/data-$ts.tgz"

echo "==> 2/5 上传迁移脚本到服务器 /tmp"
scp -q deploy/migrate-live.py "$HOST:/tmp/migrate-live.py"

echo "==> 3/5 在服务器生成最新统一数据（读线上旧库，写入服务器 /tmp 暂存，不动 test 部署）"
ssh "$HOST" "rm -rf '$REMOTE_STAGE' && DEST_DATA='$REMOTE_STAGE/data' DEST_UP='$REMOTE_STAGE/uploads' python3 /tmp/migrate-live.py"

echo "==> 4/5 拉取到本地（保留本地 core.db）"
mkdir -p data uploads
rsync -az --exclude='core.db' --exclude='core.db-*' "$HOST:$REMOTE_STAGE/data/" ./data/
rsync -az "$HOST:$REMOTE_STAGE/uploads/" ./uploads/

echo "==> 5/5 本地收尾：清旧 WAL + 完整性校验 + 行数核对"
for m in $MODULES; do
  rm -f "data/$m.db-wal" "data/$m.db-shm"
  if [ -f "data/$m.db" ]; then
    ic=$(sqlite3 "data/$m.db" "PRAGMA integrity_check;" 2>&1 | head -1)
    t=$(main_table "$m")
    n=$(sqlite3 "data/$m.db" "SELECT COUNT(*) FROM $t;" 2>/dev/null || echo '?')
    printf "    %-6s integrity=%-3s  %s=%s\n" "$m" "$ic" "$t" "$n"
  else
    printf "    %-6s（无 data/%s.db，可能旧库为空或迁移失败）\n" "$m" "$m"
  fi
done

ssh "$HOST" "rm -rf '$REMOTE_STAGE' /tmp/migrate-live.py" || true
echo "完成。data/*.db 与 uploads/<module>/ 已刷新（core.db 未动）。本地后端若在运行请重启以加载新数据。"
