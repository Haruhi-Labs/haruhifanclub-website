-- 旧逻辑中 visit_count 是同一 Cookie 每次调用 /visitors 都累加，
-- 但公开展示的 total 一直是 COUNT(*)。切换为“10 分钟独立访问”口径前，
-- 将存量基线归一为每个匿名身份 1 次，避免历史刷新次数让上线后的数字突然膨胀。
UPDATE art_visitors
SET visit_count = 1
WHERE visit_count <> 1;
