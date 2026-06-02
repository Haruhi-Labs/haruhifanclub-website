// admin 特性路由：后台管理。AdminView 现为“后台壳”——登录 gate + RBAC 子作用域门控 tab + 懒加载各特性 admin 子组件。
const AdminView = () => import('@/features/admin/views/AdminView.vue');

export const adminRoutes = [
    {
        path: '/admin',
        name: 'admin',
        component: AdminView
    }
];

export default adminRoutes;
