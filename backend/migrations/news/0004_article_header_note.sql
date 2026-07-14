-- 文章角标（headerNote）：卡片/详情页头部展示的自定义标签。
-- 留空时前端回退到按 type 生成的 NEWS / POST 分类标签；填写后以此文本为准。
-- 历史文章该列为 NULL，等价于「未填写」，行为与旧版一致。
ALTER TABLE articles ADD COLUMN headerNote TEXT;
