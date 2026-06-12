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
  # 旧 art 库可能为空且 schema 滞后（缺运行期 ensureColumn 加的 ai_reason/images_json 等列）。
  # 空库时跳过拷贝，由后端迁移建全新完整 schema；有数据时拷贝并补齐缺列（幂等）。
  local rows
  rows=$(sqlite3 "$ART_SRC/server/data.sqlite" "SELECT COUNT(*) FROM artworks" 2>/dev/null || echo 0)
  if [ "${rows:-0}" -gt 0 ]; then
    copy_db "$ART_SRC/server/data.sqlite" art
    for c in "ai_reason TEXT" "images_json TEXT" "licenses_json TEXT" "file_path_original TEXT" \
             "reviewed_at TEXT" "like_total INTEGER DEFAULT 0"; do
      n=${c%% *}
      sqlite3 data/art.db "PRAGMA table_info(artworks)" | grep -q "|$n|" || \
        sqlite3 data/art.db "ALTER TABLE artworks ADD COLUMN $c"
    done
    sqlite3 data/art.db "UPDATE artworks SET file_path='art/'||substr(file_path,9)                   WHERE file_path LIKE 'uploads/%'; \
                         UPDATE artworks SET file_path_original='art/'||substr(file_path_original,9) WHERE file_path_original LIKE 'uploads/%';" 2>/dev/null || true
  else
    echo "  (旧 art 库为空，跳过拷贝，由后端迁移建全新 schema)"
    rm -f data/art.db data/art.db-wal data/art.db-shm
  fi
  mkdir -p uploads/art
  [ -d "$ART_SRC/server/uploads" ] && rsync -a "$ART_SRC/server/uploads/" uploads/art/ --exclude='.gitkeep' 2>/dev/null || true
}

migrate_news() {
  echo "[news] 团内新闻（春日团报）"
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
  # 若生产 exam 数据的 config/questions JSON 内嵌旧媒体前缀 /exam/api/uploads/，统一改为 /uploads/exam/
  sqlite3 data/exam.db "UPDATE exams SET questions=replace(questions,'/exam/api/uploads/','/uploads/exam/'), config=replace(config,'/exam/api/uploads/','/uploads/exam/') WHERE questions LIKE '%/exam/api/uploads/%' OR config LIKE '%/exam/api/uploads/%';"
}

migrate_shop() {
  echo "[shop] 春日商城"
  copy_db "$SHOP_SRC/server/shop.db" shop
  mkdir -p uploads/shop
  rsync -a "$SHOP_SRC/server/uploads/" uploads/shop/ 2>/dev/null || true
  # 图片前缀 /shop-api/uploads/ -> /uploads/shop/（商品多图、站点配置、订单内嵌商品快照）
  sqlite3 data/shop.db "
  UPDATE products SET image=replace(image,'/shop-api/uploads/','/uploads/shop/'),
    imageOriginal=replace(imageOriginal,'/shop-api/uploads/','/uploads/shop/'),
    imageMobile=replace(imageMobile,'/shop-api/uploads/','/uploads/shop/'),
    detailImages=replace(detailImages,'/shop-api/uploads/','/uploads/shop/');
  UPDATE site_settings SET value=replace(value,'/shop-api/uploads/','/uploads/shop/');
  UPDATE orders SET items=replace(items,'/shop-api/uploads/','/uploads/shop/') WHERE items LIKE '%/shop-api/uploads/%';
  UPDATE sub_orders SET items=replace(items,'/shop-api/uploads/','/uploads/shop/') WHERE items LIKE '%/shop-api/uploads/%';"
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
