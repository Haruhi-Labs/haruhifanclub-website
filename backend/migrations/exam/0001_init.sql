-- exam.db：考试平台（忠实照搬旧 haruhi-exam-platform/server/db.js，幂等）

-- 1. 试卷主表
CREATE TABLE IF NOT EXISTS exams (
    id          TEXT PRIMARY KEY,
    title       TEXT,
    subtitle    TEXT,
    config      TEXT,                       -- JSON
    questions   TEXT,                       -- JSON
    levels      TEXT,                       -- JSON
    status      TEXT DEFAULT 'pending',     -- pending(审核中)/published(已发布)/locked(已锁定)
    edit_token  TEXT,                       -- 私密编辑密钥
    visit_count INTEGER DEFAULT 0,
    ai_reason   TEXT,                       -- 审核理由
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 2. 站点统计表
CREATE TABLE IF NOT EXISTS site_stats (
    key   TEXT PRIMARY KEY,
    value INTEGER DEFAULT 0
);
-- 初始化总访问量
INSERT OR IGNORE INTO site_stats (key, value) VALUES ('total_visits', 0);

-- 3. 答题记录表
CREATE TABLE IF NOT EXISTS submissions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    exam_id    TEXT,
    user_name  TEXT,
    score      INTEGER,
    answers    TEXT,                        -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(exam_id) REFERENCES exams(id)
);
