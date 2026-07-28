export const POINTS_USERS_PAGE_SIZE = 100;

const pointsTotal = (user) => {
    const total = Number(user?.total);
    return Number.isFinite(total) ? total : 0;
};

const pointsUserName = (user) =>
    String(user?.nickname || user?.username || user?.id || '');

export const sortPointsUsers = (users) =>
    [...users].sort((left, right) => {
        const totalDifference = pointsTotal(right) - pointsTotal(left);
        if (totalDifference !== 0) return totalDifference;
        return pointsUserName(left).localeCompare(pointsUserName(right), 'zh-CN');
    });

/**
 * 逐页读取全部积分用户，避免单次全表响应，同时在最终渲染前完成去重和全局排序。
 */
export const loadAllPointsUsers = async (
    fetchPage,
    pageSize = POINTS_USERS_PAGE_SIZE
) => {
    const usersById = new Map();
    let page = 1;

    while (true) {
        const result = await fetchPage(page, pageSize);
        const batch = Array.isArray(result?.data) ? result.data : [];

        for (const user of batch) {
            if (user?.id) usersById.set(user.id, user);
        }

        if (batch.length < pageSize) break;
        page += 1;
    }

    return sortPointsUsers(usersById.values());
};
