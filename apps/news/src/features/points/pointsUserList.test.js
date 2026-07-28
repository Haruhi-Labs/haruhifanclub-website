import { describe, expect, it, vi } from 'vitest';
import { loadAllPointsUsers, sortPointsUsers } from './pointsUserList';

describe('积分用户列表', () => {
    it('读取全部分页、按 UID 去重并做全局积分排序', async () => {
        const pages = [
            [
                { id: 'u1', nickname: 'C', total: 2 },
                { id: 'u2', nickname: 'B', total: 10 }
            ],
            [
                { id: 'u3', nickname: 'A', total: 10 },
                { id: 'u1', nickname: 'C', total: 5 }
            ],
            []
        ];
        const fetchPage = vi.fn(async (page) => ({ data: pages[page - 1] }));

        const users = await loadAllPointsUsers(fetchPage, 2);

        expect(fetchPage.mock.calls).toEqual([
            [1, 2],
            [2, 2],
            [3, 2]
        ]);
        expect(users.map((user) => [user.id, user.total])).toEqual([
            ['u3', 10],
            ['u2', 10],
            ['u1', 5]
        ]);
    });

    it('把无效积分按零处理，并用公开名称稳定排序', () => {
        const users = sortPointsUsers([
            { id: 'u2', username: 'B', total: 'invalid' },
            { id: 'u1', nickname: 'A', total: null }
        ]);

        expect(users.map((user) => user.id)).toEqual(['u1', 'u2']);
    });
});
