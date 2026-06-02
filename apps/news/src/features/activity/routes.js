// activity 特性路由：活动中心
const ActivityView = () => import('@/features/activity/views/ActivityView.vue');

export const activityRoutes = [
    {
        path: '/activity',
        name: 'activity',
        component: ActivityView
    }
];

export default activityRoutes;
