-- 文章型地方动态改为活动现场时间线，保留既有管理员授权语义。
UPDATE capability_grants
SET capability = 'branch.timeline.write', updated_at = datetime('now')
WHERE capability = 'branch.posts.write'
  AND NOT EXISTS (
      SELECT 1 FROM capability_grants existing
      WHERE existing.user_id = capability_grants.user_id
        AND existing.capability = 'branch.timeline.write'
        AND existing.scope_type = capability_grants.scope_type
        AND existing.scope_id = capability_grants.scope_id
  );

DELETE FROM capability_grants WHERE capability = 'branch.posts.write';

UPDATE capability_grants
SET capability = 'branch.timeline.publish', updated_at = datetime('now')
WHERE capability = 'branch.posts.publish'
  AND NOT EXISTS (
      SELECT 1 FROM capability_grants existing
      WHERE existing.user_id = capability_grants.user_id
        AND existing.capability = 'branch.timeline.publish'
        AND existing.scope_type = capability_grants.scope_type
        AND existing.scope_id = capability_grants.scope_id
  );

DELETE FROM capability_grants WHERE capability = 'branch.posts.publish';
