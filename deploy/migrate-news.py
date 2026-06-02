#!/usr/bin/env python3
"""news 数据迁移：把 articles/activities/prizes 的图片引用统一规整为 /uploads/news/<basename>，
并把对应文件汇集到 uploads/news/。处理旧库三种不一致前缀：/uploads/<md5>、/Name.webp、./Name.webp、裸 Name.webp。
用法：python3 deploy/migrate-news.py  （在 monorepo 根执行；data/news.db 须已由 cp 就位）"""
import os, re, shutil, sqlite3, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OLD = "/Volumes/data/harunews"
DB = os.path.join(ROOT, "data", "news.db")
DST = os.path.join(ROOT, "uploads", "news")
SRC_UPLOADS = os.path.join(OLD, "public", "uploads")
SRC_PUBLIC = os.path.join(OLD, "public")
os.makedirs(DST, exist_ok=True)

# 先把 public/uploads 的 md5 图片整体拷过来
if os.path.isdir(SRC_UPLOADS):
    for f in os.listdir(SRC_UPLOADS):
        s = os.path.join(SRC_UPLOADS, f)
        if os.path.isfile(s):
            shutil.copy2(s, os.path.join(DST, f))

missing = []

def ensure_file(basename):
    """确保 uploads/news/<basename> 存在；否则从 public/uploads 或 public 根拷贝。"""
    if not basename:
        return
    dst = os.path.join(DST, basename)
    if os.path.exists(dst):
        return
    for cand in (os.path.join(SRC_UPLOADS, basename), os.path.join(SRC_PUBLIC, basename)):
        if os.path.isfile(cand):
            shutil.copy2(cand, dst)
            return
    missing.append(basename)

def norm(ref):
    """把任意图片引用规整为 /uploads/news/<basename>；http/data/blob 原样保留；空值保留。"""
    if not ref:
        return ref
    r = ref.strip()
    if r == "" or r.startswith(("http://", "https://", "data:", "blob:")):
        return ref
    base = r.split("/")[-1].lstrip(".")  # 去掉路径与开头的 ./
    base = os.path.basename(base)
    if not base:
        return ref
    ensure_file(base)
    return "/uploads/news/" + base

con = sqlite3.connect(DB)
cur = con.cursor()

# articles：image / originalImage 走 norm；content(JSON 文本) 内嵌 /uploads/ → /uploads/news/
for aid, image, orig, content in cur.execute(
    "SELECT id, image, originalImage, content FROM articles"
).fetchall():
    new_image = norm(image)
    new_orig = norm(orig)
    new_content = content
    if content:
        new_content = re.sub(r"/uploads/(?!news/)", "/uploads/news/", content)
    cur.execute(
        "UPDATE articles SET image=?, originalImage=?, content=? WHERE id=?",
        (new_image, new_orig, new_content, aid),
    )

for table in ("activities", "prizes"):
    for rid, image in cur.execute(f"SELECT id, image FROM {table}").fetchall():
        cur.execute(f"UPDATE {table} SET image=? WHERE id=?", (norm(image), rid))

# 规整旧库的松类型数值列（SQLite 动态类型导致部分值以 TEXT 存储，sqlx 严格解码会失败）。
# 统一 CAST 为 INTEGER，与旧 JS 的 Number(x)||0 语义一致。
for sql in (
    "UPDATE activities SET totalPoints = CAST(COALESCE(totalPoints,0) AS INTEGER), "
    "pointsPerAction = CAST(COALESCE(pointsPerAction,0) AS INTEGER)",
    "UPDATE points_history SET change = CAST(COALESCE(change,0) AS INTEGER)",
):
    cur.execute(sql)

con.commit()
con.close()

total = len(os.listdir(DST))
print(f"✓ news 图片规整完成：uploads/news/ 共 {total} 个文件")
if missing:
    print(f"⚠ {len(missing)} 个引用文件未找到（破图，可后台补传）：", ", ".join(sorted(set(missing))[:10]))
