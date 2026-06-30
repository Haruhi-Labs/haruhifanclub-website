-- 积分流水补充「来源类型」，用于把消耗类记录从「历史累计获得积分」中区分出来。
-- 排行榜的历史获得积分 = SUM(points) 但排除 source_type='redemption'（兑换等正常消耗），
-- 撤稿扣回(source_type='withdraw', 负值)仍计入，因而被正确扣减。
ALTER TABLE points_ledger ADD COLUMN source_type TEXT;

-- 回填历史兑换扣分记录：此前兑换扣分仅以 note 文本「兑换「…」扣除金币」标识，
-- 统一标注为 redemption，使其从历史获得积分中排除（系统此前不存在其他负向记录）。
UPDATE points_ledger
   SET source_type = 'redemption'
 WHERE source_type IS NULL
   AND points < 0
   AND note LIKE '兑换%扣除金币%';
