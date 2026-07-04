// handbook 特性路由
const HandbookView = () => import('@/features/handbook/views/HandbookView.vue');

export const handbookRoutes = [
    {
        path: '/handbook',
        name: 'handbook',
        component: HandbookView,
        meta: { title: '团员手册' }
    }
];

export default handbookRoutes;
