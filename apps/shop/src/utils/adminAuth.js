// 统一后台管理员鉴权：本模块不再自研实现，全部委托给 @haruhi/api-client 的
// createAdminAuth('shop')（单点 JWT + shop 权限校验 + JWT exp 同步校验）。
// 这里仅做薄封装，保留原有导出函数名（getAdminToken / setAdminToken /
// clearAdminToken / hasValidAdminToken / buildAdminAuthHeaders / hasShopPerm /
// verifyShopAccess / loginShopAdmin），以零改动复用既有调用点。
import { setToken, clearToken, createAdminAuth } from '@haruhi/api-client';

const admin = createAdminAuth('shop');

export const getAdminToken = () => admin.getToken();

export const setAdminToken = (token) => {
    if (!token) return;
    setToken(token);
};

export const clearAdminToken = () => {
    clearToken();
};

// 同步的本地有效性判断（仅校验 JWT 是否存在且未过期），供路由守卫使用。
export const hasValidAdminToken = () => admin.hasValidToken();

export const buildAdminAuthHeaders = (headers = {}) => admin.buildHeaders(headers);

// 是否具备春日商城（shop）管理权限：超管或被授予 shop 角色
export const hasShopPerm = (user) => admin.hasPerm(user);

// 异步校验当前会话是否具备 shop 管理权限；无效则清除 token。
// 返回 true 表示通过；false 表示未登录 / 无权限 / token 失效。
export const verifyShopAccess = async () => !!(await admin.restore());

// 统一 JWT 登录：用户名 + 密码 → /api/auth/login。
// 成功且具备 shop 权限返回 { ok: true, user }；否则返回 { ok: false, error }。
export const loginShopAdmin = async (username, password) => admin.login(username, password);
