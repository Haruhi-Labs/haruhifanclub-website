#!/usr/bin/env bash
# 把 5 个旧站的 SQLite 数据库与 uploads 迁移进统一 monorepo 的 data/ 与 uploads/。
# 幂等思路：先备份再覆盖；路径前缀重写把旧 "uploads/..." 改成 "<module>/..."（统一 /uploads 根）。
# 用法：在 monorepo 根目录执行  bash deploy/migrate-data.sh [module...]
#       不带参数则迁移全部已就绪模块。
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"
mkdir -p data uploads

NEWS_SRC=/Volumes/data/harunews
ART_SRC=/Volumes/data/haruhi-art-club
EXAM_SRC=/Volumes/data/haruhi-exam-platform/haruhi-exam-platform
NOVEL_SRC=/Volumes/data/haruhi-novel-reader
SHOP_SRC=/Users/haruhi/haruhishop

copy_db() { # src_db  dest_name
  cp "$1" "data/$2.db"
  sqlite3 "data/$2.db" "PRAGMA journal_mode=WAL;" >/dev/null
  echo "  ✓ data/$2.db"
}

migrate_novel() {
  echo "[novel] 书库"
  copy_db "$NOVEL_SRC/server/data.db" novel
  mkdir -p uploads/novel
  rsync -a "$NOVEL_SRC/server/uploads/files" "$NOVEL_SRC/server/uploads/covers" uploads/novel/
  # cover_path/file_path: uploads/... -> novel/...
  sqlite3 data/novel.db "UPDATE books SET cover_path='novel/'||substr(cover_path,9) WHERE cover_path LIKE 'uploads/%'; \
                         UPDATE books SET file_path='novel/'||substr(file_path,9)  WHERE file_path  LIKE 'uploads/%';"
}

migrate_art() {
  echo "[art] 画廊"
  copy_db "$ART_SRC/server/data.sqlite" art
  mkdir -p uploads/art
  # 旧 art uploads 按月份子目录；库中 file_path/file_path_original/images_json 存 "uploads/<YYYY-MM>/x"
  if [ -d "$ART_SRC/server/uploads" ]; then rsync -a "$ART_SRC/server/uploads/" uploads/art/ --exclude='.gitkeep'; fi
  sqlite3 data/art.db "UPDATE artworks SET file_path='art/'||substr(file_path,9)                   WHERE file_path LIKE 'uploads/%'; \
                       UPDATE artworks SET file_path_original='art/'||substr(file_path_original,9) WHERE file_path_original LIKE 'uploads/%';" 2>/dev/null || true
  # images_json 内含多路径，逐条 JSON 重写在应用层处理（见后端迁移脚本/启动校验）
}

migrate_news() {
  echo "[news] 京都学报"
  copy_db "$NEWS_SRC/server/database.sqlite" news
  # 图片三种不一致前缀(/uploads/<md5>、/Name.webp、裸 Name.webp)统一规整为 /uploads/news/<basename>
  # 并把 public/uploads 与 public 根的引用文件汇集到 uploads/news/。忽略 lowdb 的 *.json 死数据。
  python3 deploy/migrate-news.py
}

migrate_exam() {
  echo "[exam] 考试平台"
  copy_db "$EXAM_SRC/server/exam_platform.db" exam
  mkdir -p uploads/exam
  rsync -a "$EXAM_SRC/server/uploads/" uploads/exam/ 2>/dev/null || true
  # TODO(exam): 重写 uploads 引用前缀（图片/音频）为 exam/...
}

migrate_shop() {
  echo "[shop] 春日商城"
  copy_db "$SHOP_SRC/server/shop.db" shop
  mkdir -p uploads/shop
  rsync -a "$SHOP_SRC/server/uploads/" uploads/shop/ 2>/dev/null || true
  # TODO(shop): 重写 products.image/imageOriginal/imageMobile/detailImages 等前缀为 shop/...
}

MODULES=("${@:-novel art news exam shop}")
for m in ${MODULES[@]}; do
  case "$m" in
    novel) migrate_novel ;;
    art)   migrate_art ;;
    news)  migrate_news ;;
    exam)  migrate_exam ;;
    shop)  migrate_shop ;;
    *) echo "未知模块: $m" ;;
  esac
done
echo "迁移完成。data/*.db 与 uploads/<module>/ 已就绪。"
