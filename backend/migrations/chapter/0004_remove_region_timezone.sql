-- 地方支部不维护地区代码或独立时区；地点只保留面向用户的 locality_name。
DROP INDEX IF EXISTS idx_branches_public;

ALTER TABLE branches DROP COLUMN region_code;
ALTER TABLE branches DROP COLUMN timezone;

CREATE INDEX IF NOT EXISTS idx_branches_public
    ON branches(status, locality_name);
