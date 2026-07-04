// fiction 后台权限判定：与后端 authorize(…, "fiction", Action::X) 严格对齐。
//   Moderate=3（审核）：进后台、管理作品/评论、设精选、上下架
//   Manage=4（管理）：以上 + 硬删除作品
// 超管（isSuperAdmin）视为最高级。前端判定仅用于显隐，真正的权限由后端强制。
export function fictionRoleLevel(user) {
  if (!user) return 0
  if (user.isSuperAdmin) return 99
  return user.apps?.fiction?.level ?? 0
}

export const canModerate = (user) => fictionRoleLevel(user) >= 3
export const canManage = (user) => fictionRoleLevel(user) >= 4
