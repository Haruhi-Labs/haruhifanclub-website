#!/usr/bin/env python3
"""从服务器上正在运行的旧项目抓取**最新真实数据**迁入统一后端的 data/ 与 uploads/。
读取用 sqlite3 .backup（一致性快照，不扰动 live 服务）；uploads 用 rsync（只读拷贝）。
在服务器上执行：python3 migrate-live.py
"""
import json, os, re, shutil, sqlite3, subprocess, sys

DEST_DATA = os.environ.get("DEST_DATA", "/var/www/haruhifanclub-test/data")
DEST_UP = os.environ.get("DEST_UP", "/var/www/haruhifanclub-test/uploads")
S3 = "/usr/bin/sqlite3"
os.makedirs(DEST_DATA, exist_ok=True)
os.makedirs(DEST_UP, exist_ok=True)


def snapshot(src, dst):
    subprocess.run([S3, src, f".backup '{dst}'"], check=True)
    subprocess.run([S3, dst, "PRAGMA journal_mode=WAL;"], stdout=subprocess.DEVNULL, check=True)


def rsync(src, dst):
    os.makedirs(dst, exist_ok=True)
    subprocess.run(["rsync", "-a", src.rstrip("/") + "/", dst.rstrip("/") + "/"], check=True)


def migrate_novel():
    data = json.load(open("/root/haruhi-novel-reader/server/db.json"))
    books = data.get("books", [])
    db = os.path.join(DEST_DATA, "novel.db")
    if os.path.exists(db):
        os.remove(db)
    con = sqlite3.connect(db)
    con.execute("CREATE TABLE IF NOT EXISTS books (id TEXT PRIMARY KEY, title TEXT NOT NULL, "
                "author TEXT DEFAULT '佚名', cover_path TEXT, file_path TEXT, upload_date TEXT, "
                "category TEXT, sort_order REAL)")

    def rw(p):
        return ("novel/" + p[len("uploads/"):]) if p and p.startswith("uploads/") else p

    for b in books:
        con.execute("INSERT OR REPLACE INTO books VALUES (?,?,?,?,?,?,?,?)",
                    (str(b.get("id")), b.get("title") or "未命名", b.get("author") or "佚名",
                     rw(b.get("cover_path")), rw(b.get("file_path")), b.get("upload_date"),
                     b.get("category"), b.get("order")))
    con.commit(); con.close()
    rsync("/root/haruhi-novel-reader/server/uploads", os.path.join(DEST_UP, "novel"))
    return f"{len(books)} 本书"


def migrate_news():
    db = os.path.join(DEST_DATA, "news.db")
    snapshot("/root/harunews/server/database.sqlite", db)
    DST = os.path.join(DEST_UP, "news")
    rsync("/root/harunews/dist/uploads", DST)
    SRC_UP, SRC_PUB = "/root/harunews/dist/uploads", "/root/harunews/dist"
    missing = []

    def ensure(base):
        if not base:
            return
        d = os.path.join(DST, base)
        if os.path.exists(d):
            return
        for c in (os.path.join(SRC_UP, base), os.path.join(SRC_PUB, base)):
            if os.path.isfile(c):
                shutil.copy2(c, d); return
        missing.append(base)

    def norm(ref):
        if not ref:
            return ref
        r = ref.strip()
        if r == "" or r.startswith(("http://", "https://", "data:", "blob:")):
            return ref
        base = os.path.basename(r.lstrip("."))
        if not base:
            return ref
        ensure(base)
        return "/uploads/news/" + base

    con = sqlite3.connect(db); cur = con.cursor()
    for aid, image, orig, content in cur.execute("SELECT id,image,originalImage,content FROM articles").fetchall():
        nc = re.sub(r"/uploads/(?!news/)", "/uploads/news/", content) if content else content
        cur.execute("UPDATE articles SET image=?,originalImage=?,content=? WHERE id=?",
                    (norm(image), norm(orig), nc, aid))
    for t in ("activities", "prizes"):
        for rid, image in cur.execute(f"SELECT id,image FROM {t}").fetchall():
            cur.execute(f"UPDATE {t} SET image=? WHERE id=?", (norm(image), rid))
    cur.execute("UPDATE activities SET totalPoints=CAST(COALESCE(totalPoints,0) AS INTEGER), "
                "pointsPerAction=CAST(COALESCE(pointsPerAction,0) AS INTEGER)")
    con.commit(); con.close()
    return f"news 迁移完成，缺失图 {len(missing)}：{missing[:5]}"


def migrate_art():
    db = os.path.join(DEST_DATA, "art.db")
    snapshot("/root/haruhi-art-club/server/data/app.db", db)
    rsync("/root/haruhi-art-club/server/uploads", os.path.join(DEST_UP, "art"))
    con = sqlite3.connect(db); cur = con.cursor()
    # 补齐运行期 ensureColumn 列（幂等）
    have = {r[1] for r in cur.execute("PRAGMA table_info(artworks)").fetchall()}
    for col, ddl in [("ai_reason", "ai_reason TEXT"), ("images_json", "images_json TEXT"),
                     ("file_path_original", "file_path_original TEXT"),
                     ("like_total", "like_total INTEGER DEFAULT 0")]:
        if col not in have:
            cur.execute(f"ALTER TABLE artworks ADD COLUMN {ddl}")

    def rw(p):
        if not p or p.startswith(("art/", "http", "/uploads/")):
            return p
        return "art/" + p.lstrip("/")

    for rid, fp, fpo, imj in cur.execute("SELECT id,file_path,file_path_original,images_json FROM artworks").fetchall():
        nimj = imj
        if imj:
            try:
                arr = json.loads(imj)
                for it in arr:
                    if isinstance(it, dict):
                        if it.get("path"):
                            it["path"] = rw(it["path"])
                        if it.get("original"):
                            it["original"] = rw(it["original"])
                nimj = json.dumps(arr, ensure_ascii=False)
            except Exception:
                pass
        cur.execute("UPDATE artworks SET file_path=?,file_path_original=?,images_json=? WHERE id=?",
                    (rw(fp), rw(fpo), nimj, rid))
    con.commit(); con.close()
    return "art 迁移完成"


def migrate_exam():
    db = os.path.join(DEST_DATA, "exam.db")
    snapshot("/root/haruhi-exam-platform/server/exam_platform.db", db)
    rsync("/root/haruhi-exam-platform/server/uploads", os.path.join(DEST_UP, "exam"))
    con = sqlite3.connect(db)
    con.execute("UPDATE exams SET questions=replace(questions,'/exam/api/uploads/','/uploads/exam/'), "
                "config=replace(config,'/exam/api/uploads/','/uploads/exam/') "
                "WHERE questions LIKE '%/exam/api/uploads/%' OR config LIKE '%/exam/api/uploads/%'")
    con.commit(); con.close()
    return "exam 迁移完成"


def migrate_shop():
    db = os.path.join(DEST_DATA, "shop.db")
    snapshot("/root/haruhishop/server/shop.db", db)
    rsync("/root/haruhishop/server/uploads", os.path.join(DEST_UP, "shop"))
    con = sqlite3.connect(db)
    for sql in [
        "UPDATE products SET image=replace(image,'/shop-api/uploads/','/uploads/shop/'), "
        "imageOriginal=replace(imageOriginal,'/shop-api/uploads/','/uploads/shop/'), "
        "imageMobile=replace(imageMobile,'/shop-api/uploads/','/uploads/shop/'), "
        "detailImages=replace(detailImages,'/shop-api/uploads/','/uploads/shop/')",
        "UPDATE site_settings SET value=replace(value,'/shop-api/uploads/','/uploads/shop/')",
        "UPDATE orders SET items=replace(items,'/shop-api/uploads/','/uploads/shop/') WHERE items LIKE '%/shop-api/uploads/%'",
        "UPDATE sub_orders SET items=replace(items,'/shop-api/uploads/','/uploads/shop/') WHERE items LIKE '%/shop-api/uploads/%'",
    ]:
        try:
            con.execute(sql)
        except Exception as e:
            print("  shop sql 警告:", e)
    con.commit(); con.close()
    return "shop 迁移完成"


for name, fn in [("novel", migrate_novel), ("news", migrate_news), ("art", migrate_art),
                 ("exam", migrate_exam), ("shop", migrate_shop)]:
    try:
        print(f"[{name}]", fn())
    except Exception as e:
        print(f"[{name}] 失败: {e}", file=sys.stderr)
print("全部完成。")
