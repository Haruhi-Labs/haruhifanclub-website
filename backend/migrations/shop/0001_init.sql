-- shop.db：春日商城（忠实照搬旧 haruhishop/server/db.cjs，幂等）。
-- 金额单位统一为分(INTEGER)；JSON 字段存 TEXT。
-- 注意：旧 db.cjs 用 CREATE TABLE + 多次 ensureColumn(ALTER TABLE ADD COLUMN) 增量补列，
-- 这里把所有最终列直接写进 CREATE TABLE（合并两者的最终形态）。

-- ============================================================
-- 1. 商品表 products
-- ============================================================
CREATE TABLE IF NOT EXISTS products (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    name                  TEXT,
    price                 INTEGER,
    discountPrice         INTEGER,
    category              TEXT,
    typeId                TEXT,
    stock                 INTEGER,
    image                 TEXT,
    imageOriginal         TEXT,
    desc                  TEXT,
    specs                 TEXT,
    detailText            TEXT,
    detailImages          TEXT,
    shippingTag           TEXT,
    shippingCost          INTEGER,
    presaleMode           TEXT DEFAULT 'none',
    presaleGoalTarget     INTEGER DEFAULT 0,
    presaleFixedDateType  TEXT,
    presaleFixedDateValue TEXT,
    presalePaidOffset     INTEGER DEFAULT 0,
    -- ensureColumn 增量补列：
    imageMobile           TEXT,
    sortOrder             INTEGER DEFAULT 0
);

-- ============================================================
-- 2. 订单表 orders
-- status: 0=已取消, 1=待付款, 2=待发货(已付款), 3=已发货, 4=已完成, 5=待确认(用户已付款)
-- ============================================================
CREATE TABLE IF NOT EXISTS orders (
    id                       TEXT PRIMARY KEY,
    total                    INTEGER,
    originalTotal            REAL DEFAULT 0,
    discountAmount           REAL DEFAULT 0,
    couponCode               TEXT,
    mergeMeta                TEXT,
    items                    TEXT,        -- JSON: 商品列表
    contactName              TEXT,        -- 联系人
    contactPhone             TEXT,        -- 电话
    contactEmail             TEXT,        -- 邮箱
    province                 TEXT,        -- 省
    city                     TEXT,        -- 市
    district                 TEXT,        -- 区
    addressDetail            TEXT,        -- 详细地址
    trackingCompany          TEXT,        -- 快递公司
    trackingNo               TEXT,        -- 快递单号
    status                   INTEGER DEFAULT 1,
    created_at               DATETIME DEFAULT CURRENT_TIMESTAMP,
    -- ensureColumn 增量补列：
    exported                 INTEGER DEFAULT 0,
    hasPresaleItems          INTEGER DEFAULT 0,
    hasSpotItems             INTEGER DEFAULT 1,
    spotExported             INTEGER DEFAULT 0,
    presaleExportedProducts  TEXT
);

-- ============================================================
-- 3. 埋点事件表 analytics_events
-- ============================================================
CREATE TABLE IF NOT EXISTS analytics_events (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    sessionId  TEXT,
    eventKey   TEXT,
    page       TEXT,
    meta       TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_analytics_event_key_time ON analytics_events(eventKey, created_at);
CREATE INDEX IF NOT EXISTS idx_analytics_session ON analytics_events(sessionId);

-- ============================================================
-- 4. 站点设置表 site_settings
-- ============================================================
CREATE TABLE IF NOT EXISTS site_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- 5. 优惠券表 coupons
-- status: 0=禁用, 1=未用, 2=已用
-- ============================================================
CREATE TABLE IF NOT EXISTS coupons (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    code          TEXT UNIQUE,
    name          TEXT,
    batchNo       TEXT,
    minSpend      REAL DEFAULT 0,
    discountType  TEXT DEFAULT 'amount',
    discountValue REAL DEFAULT 0,
    maxDiscount   REAL,
    status        INTEGER DEFAULT 1,
    expiresAt     DATETIME,
    usedOrderId   TEXT,
    used_at       DATETIME,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_coupons_status_batch ON coupons(status, batchNo);
CREATE INDEX IF NOT EXISTS idx_coupons_code ON coupons(code);

-- ============================================================
-- 6. 联系/留言表 contact_messages
-- status: 0=未处理, 1=已处理
-- ============================================================
CREATE TABLE IF NOT EXISTS contact_messages (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT,
    contact    TEXT,
    orderId    TEXT,
    content    TEXT,
    status     INTEGER DEFAULT 0,
    handled_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_contact_messages_status_time ON contact_messages(status, created_at);

-- ============================================================
-- 7. 邮件任务队列 email_jobs
-- status: pending / sent / failed
-- ============================================================
CREATE TABLE IF NOT EXISTS email_jobs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    orderId     TEXT NOT NULL,
    toEmail     TEXT NOT NULL,
    eventKey    TEXT NOT NULL,
    subject     TEXT NOT NULL,
    html        TEXT NOT NULL,
    text        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',
    attempts    INTEGER NOT NULL DEFAULT 0,
    maxAttempts INTEGER NOT NULL DEFAULT 5,
    nextRunAt   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    lastError   TEXT,
    sentAt      DATETIME,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_email_jobs_status_next ON email_jobs(status, nextRunAt);
CREATE INDEX IF NOT EXISTS idx_email_jobs_order_event_time ON email_jobs(orderId, eventKey, created_at);

-- ============================================================
-- 8. 子订单表 sub_orders（混合订单拆分发货）
-- subKey: presale / spot / group
-- ============================================================
CREATE TABLE IF NOT EXISTS sub_orders (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    orderId         TEXT NOT NULL,
    subKey          TEXT NOT NULL,
    label           TEXT NOT NULL,
    items           TEXT NOT NULL,
    trackingCompany TEXT,
    trackingNo      TEXT,
    shipped         INTEGER DEFAULT 0,
    shipped_at      DATETIME,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(orderId, subKey)
);
CREATE INDEX IF NOT EXISTS idx_sub_orders_order ON sub_orders(orderId);
