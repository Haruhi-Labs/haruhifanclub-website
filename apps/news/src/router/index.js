import { createRouter, createWebHistory } from 'vue-router';

// 按特性组合各 feature 的路由数组（path/name/meta 与重构前一字不差）。
import { blogRoutes } from '@/features/blog/routes';
import { handbookRoutes } from '@/features/handbook/routes';
import { storeRoutes } from '@/features/store/routes';
import { quizRoutes } from '@/features/quiz/routes';
import { activityRoutes } from '@/features/activity/routes';
import { adminRoutes } from '@/features/admin/routes';

const routes = [
    ...blogRoutes,
    ...handbookRoutes,
    ...storeRoutes,
    ...activityRoutes,
    ...quizRoutes,
    ...adminRoutes
];

const router = createRouter({
    // 部署于子路径 /news/，history base 取自 vite 的 BASE_URL
    history: createWebHistory(import.meta.env.BASE_URL),
    routes,
    scrollBehavior(to, from, savedPosition) {
        if (savedPosition) {
            return savedPosition;
        } else {
            return { top: 0, behavior: 'smooth' };
        }
    }
});

export default router;
