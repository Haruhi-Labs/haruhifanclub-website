#!/usr/bin/env bash
# 每日备份：对各 SQLite 库做一致性快照(.backup，WAL 下安全) + 打包 uploads，按时间戳归档并轮转。
# 用法：HARUHI_ROOT=/var/www/haruhifanclub-test bash backup.sh
# 由 systemd timer 调度（见 haruhifanclub-backup.{service,timer}）。
set -euo pipefail

ROOT="${HARUHI_ROOT:-/var/www/haruhifanclub-test}"      # 部署根（含 data/ 与 uploads/）
DEST="${HARUHI_BACKUP_DIR:-/var/backups/haruhifanclub}" # 备份输出目录
KEEP_DAYS="${HARUHI_BACKUP_KEEP:-14}"                   # 保留天数
STAMP="$(date +%Y%m%d-%H%M%S)"
OUT="$DEST/$STAMP"
mkdir -p "$OUT"

# 1) 各库一致性快照（.backup 不会因 WAL 写入而损坏）
for db in "$ROOT"/data/*.db; do
  [ -f "$db" ] || continue
  sqlite3 "$db" ".backup '$OUT/$(basename "$db")'"
done

# 2) uploads 打包
if [ -d "$ROOT/uploads" ]; then
  tar -czf "$OUT/uploads.tar.gz" -C "$ROOT" uploads
fi

# 3) 轮转：删除超过保留天数的旧快照
find "$DEST" -maxdepth 1 -type d -name '20*' -mtime +"$KEEP_DAYS" -exec rm -rf {} + 2>/dev/null || true

echo "[backup] $STAMP 完成 → $OUT （$(du -sh "$OUT" | cut -f1)）"
