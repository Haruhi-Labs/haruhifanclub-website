// store 特性路由：积分商城
const StoreView = () => import('@/features/store/views/StoreView.vue');

export const storeRoutes = [
    {
        path: '/store',
        name: 'store',
        component: StoreView
    }
];

export default storeRoutes;
