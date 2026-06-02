// 统一 JWT 单点登录：本模块不再维护自研 token 存储，
// 全部委托给 @haruhi/api-client（统一 localStorage key + /api/auth）。
// 保留原有导出函数名（getAdminToken / setAdminToken / clearAdminToken /
// hasValidAdminToken / buildAdminAuthHeaders），以最小改动复用既有调用点。
import { getToken, setToken, clearToken, createAuth } from '@haruhi/api-client';

// 统一鉴权：登录 / 会话恢复 / 登出走 /api/auth
const auth = createAuth('/api');

const decodePayload = (token) => {
    if (!token || typeof token !== 'string') return null;
    const parts = token.split('.');
    if (parts.length !== 3) return null;

    try {
        const base64 = parts[1].replace(/-/g, '+').replace(/_/g, '/');
        const padded = base64 + '='.repeat((4 - (base64.length % 4)) % 4);
        const json = atob(padded);
        return JSON.parse(json);
    } catch {
        return null;
    }
};

export const getAdminToken = () => getToken();

export const setAdminToken = (token) => {
    if (!token) return;
    setToken(token);
};

export const clearAdminToken = () => {
    clearToken();
};

// 同步的本地有效性判断（仅校验 JWT 是否存在且未过期），供路由守卫使用。
// 真正的「是否具备 shop 管理权限」由 verifyShopAccess() 异步走 /api/auth/me 校验。
export const hasValidAdminToken = () => {
    const token = getAdminToken();
    if (!token) return false;

    const payload = decodePayload(token);
    if (!payload) {
        clearAdminToken();
        return false;
    }

    if (payload.exp && Number(payload.exp) <= Math.floor(Date.now() / 1000)) {
        clearAdminToken();
        return false;
    }

    return true;
};

export const buildAdminAuthHeaders = (headers = {}) => {
    const token = getAdminToken();
    if (!token) return { ...headers };
    return {
        ...headers,
        Authorization: `Bearer ${token}`
    };
};

// 是否具备春日商城（shop）管理权限：超管或被授予 shop 角色
export const hasShopPerm = (user) => !!user && (user.isSuperAdmin || (user.apps && user.apps.shop));

// 异步校验当前会话是否具备 shop 管理权限；无效则清除 token。
// 返回 true 表示通过；false 表示未登录 / 无权限 / token 失效。
export const verifyShopAccess = async () => {
    if (!hasValidAdminToken()) return false;
    try {
        const user = await auth.me();
        if (hasShopPerm(user)) return true;
        clearAdminToken();
        return false;
    } catch {
        clearAdminToken();
        return false;
    }
};

// 统一 JWT 登录：用户名 + 密码 → /api/auth/login。
// 成功且具备 shop 权限返回 { ok: true }；否则清除 token 并返回 { ok: false, error }。
export const loginShopAdmin = async (username, password) => {
    try {
        const user = await auth.login(username, password);
        if (!hasShopPerm(user)) {
            clearAdminToken();
            return { ok: false, error: '该账号无春日商城管理权限' };
        }
        return { ok: true, user };
    } catch (e) {
        clearAdminToken();
        const error = e?.status === 401 ? '用户名或密码错误' : (e?.message || '登录失败');
        return { ok: false, error };
    }
};
