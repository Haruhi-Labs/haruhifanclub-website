-- news.db：团内新闻（春日团报）/ 新闻站（忠实照搬旧 harunews/server/db.js，幂等）
-- 只移植 sqlite(database.sqlite) 相关逻辑，lowdb / *.json 备份均为死代码，已忽略。

CREATE TABLE IF NOT EXISTS articles (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    title         TEXT,
    subtitle      TEXT,
    date          TEXT,
    type          TEXT,
    author        TEXT,
    tags          TEXT,          -- JSON 数组（字符串）
    image         TEXT,
    originalImage TEXT,
    coverFocalX   REAL,
    coverFocalY   REAL,
    content       TEXT,          -- JSON（富文本块数组，含内嵌图）
    isPinned      INTEGER DEFAULT 0,
    pinOrder      INTEGER DEFAULT 0,
    participants  TEXT,          -- JSON 数组（字符串）
    status        TEXT,
    created_at    TEXT,
    summary       TEXT
);

CREATE TABLE IF NOT EXISTS activities (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT,
    intro           TEXT,
    detail          TEXT,
    image           TEXT,
    totalPoints     INTEGER,
    actionName      TEXT,
    pointsPerAction INTEGER,
    status          TEXT,
    type            TEXT,
    -- 旧 db.js 的 CREATE TABLE 不含 displayOrder，但 index.js 在 reorder/插入时使用之；
    -- 旧 sqlite3 对缺列会运行期报错，实际部署库已有该列。这里建表即补齐，保持忠实行为。
    displayOrder    INTEGER
);

CREATE TABLE IF NOT EXISTS prizes (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT,
    description  TEXT,
    points       INTEGER,
    stock        INTEGER,
    category     TEXT,
    rarity       TEXT,
    color        TEXT,
    size         TEXT,
    image        TEXT,
    -- 同 activities：旧 index.js 依赖 displayOrder，建表补齐。
    displayOrder INTEGER
);

CREATE TABLE IF NOT EXISTS users (
    id    TEXT PRIMARY KEY,
    total INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS points_history (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id   TEXT,
    date      TEXT,
    change    TEXT,
    reason    TEXT,
    timestamp INTEGER,
    FOREIGN KEY(user_id) REFERENCES users(id)
);
