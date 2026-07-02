-- fiction.db：凉宫春日同人小说创作站（作品 + 章节 + 标签 + 评论 + 点赞收藏 + 阅读进度）。
-- 全站统一账号下的 UGC：author_user_id 逻辑引用 core.users.id（跨库无 DB 级 FK）；
-- author_name 为署名昵称快照，author_uid 取 "u{id}" 沿用全站 uid 命名空间。
-- 时间统一存 UTC RFC3339 文本（如 2026-07-01T08:00:00Z），排序用 datetime() 包裹更稳。

-- 作品（一部小说 = 一条 stories + 若干 chapters）
CREATE TABLE IF NOT EXISTS stories (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    summary         TEXT NOT NULL DEFAULT '',          -- 简介
    cover_path      TEXT,                              -- 封面相对路径，如 fiction/covers/x.webp
    category        TEXT NOT NULL DEFAULT 'other',     -- 分类 slug（后端 allowlist 校验）
    content_rating  TEXT NOT NULL DEFAULT 'general',   -- general / teen / mature
    status          TEXT NOT NULL DEFAULT 'draft',     -- draft / published / hidden
    is_completed    INTEGER NOT NULL DEFAULT 0,        -- 0 连载中 / 1 已完结
    featured        INTEGER NOT NULL DEFAULT 0,        -- 编辑精选
    author_user_id  INTEGER,                           -- 逻辑引用 core.users.id
    author_uid      TEXT,                              -- "u{id}"
    author_name     TEXT NOT NULL DEFAULT '',          -- 署名昵称快照
    word_count      INTEGER NOT NULL DEFAULT 0,        -- 全书字数（章节汇总）
    chapter_count   INTEGER NOT NULL DEFAULT 0,        -- 已发布章节数
    view_count      INTEGER NOT NULL DEFAULT 0,
    like_count      INTEGER NOT NULL DEFAULT 0,
    bookmark_count  INTEGER NOT NULL DEFAULT 0,
    comment_count   INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    published_at    TEXT,                              -- 首次发布时间
    last_chapter_at TEXT                               -- 最近一次有章节发布的时间（“最近更新”排序）
);
CREATE INDEX IF NOT EXISTS idx_stories_status ON stories(status);
CREATE INDEX IF NOT EXISTS idx_stories_author_user ON stories(author_user_id);
CREATE INDEX IF NOT EXISTS idx_stories_category ON stories(category);
CREATE INDEX IF NOT EXISTS idx_stories_published ON stories(datetime(published_at));
CREATE INDEX IF NOT EXISTS idx_stories_updated ON stories(datetime(last_chapter_at));

-- 章节（正文以编辑器输出的 HTML 存，服务端 ammonia 白名单清洗后落库；text_plain 为纯文本派生，供字数/摘要）
CREATE TABLE IF NOT EXISTS chapters (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    story_id     INTEGER NOT NULL,
    title        TEXT NOT NULL DEFAULT '',
    content_html TEXT NOT NULL DEFAULT '',
    text_plain   TEXT NOT NULL DEFAULT '',
    author_note  TEXT NOT NULL DEFAULT '',          -- 作者的话
    word_count   INTEGER NOT NULL DEFAULT 0,
    position     INTEGER NOT NULL DEFAULT 0,        -- 卷内顺序（从 1 递增）
    status       TEXT NOT NULL DEFAULT 'draft',     -- draft / published
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    published_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_chapters_story ON chapters(story_id, position);

-- 标签（自由词，多对多）
CREATE TABLE IF NOT EXISTS tags (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS story_tags (
    story_id INTEGER NOT NULL,
    tag_id   INTEGER NOT NULL,
    PRIMARY KEY (story_id, tag_id)
);
CREATE INDEX IF NOT EXISTS idx_story_tags_tag ON story_tags(tag_id);

-- 评论（chapter_id 为 NULL = 作品级评论，否则为章节评论；parent_id 支持楼中楼）
CREATE TABLE IF NOT EXISTS comments (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    story_id       INTEGER NOT NULL,
    chapter_id     INTEGER,
    parent_id      INTEGER,
    author_user_id INTEGER NOT NULL,
    author_uid     TEXT,
    author_name    TEXT NOT NULL DEFAULT '',
    body           TEXT NOT NULL,
    status         TEXT NOT NULL DEFAULT 'visible',  -- visible / hidden
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_comments_story ON comments(story_id, datetime(created_at));
CREATE INDEX IF NOT EXISTS idx_comments_chapter ON comments(chapter_id);
CREATE INDEX IF NOT EXISTS idx_comments_author_user ON comments(author_user_id);

-- 点赞 / 收藏（按登录用户去重；user_id 即 core.users.id）
CREATE TABLE IF NOT EXISTS reactions (
    user_id    INTEGER NOT NULL,
    story_id   INTEGER NOT NULL,
    kind       TEXT NOT NULL,                         -- like / bookmark
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (user_id, story_id, kind)
);
CREATE INDEX IF NOT EXISTS idx_reactions_story ON reactions(story_id, kind);

-- 阅读进度（每人每作品一行，记录读到的章节与章内滚动比例）
CREATE TABLE IF NOT EXISTS reading_progress (
    user_id    INTEGER NOT NULL,
    story_id   INTEGER NOT NULL,
    chapter_id INTEGER,
    progress   REAL NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (user_id, story_id)
);
