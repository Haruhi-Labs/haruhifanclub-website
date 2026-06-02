// blog 特性路由：首页（含 tag/参与者/作者/搜索别名）、投稿编辑器、博客详情
// 路由 path/name/meta 与重构前保持一字不差；组件改为懒加载以按 feature 分包。
const HomeView = () => import('@/features/blog/views/HomeView.vue');
const EditorView = () => import('@/features/blog/views/EditorView.vue');
const BlogDetailView = () => import('@/features/blog/views/BlogDetailView.vue');

export const blogRoutes = [
    {
        path: '/',
        name: 'home',
        component: HomeView
    },
    {
        path: '/tag/:tag',
        name: 'tag',
        component: HomeView,
        props: true
    },
    {
        path: '/participant/:name',
        name: 'participant',
        component: HomeView,
        props: true
    },
    {
        path: '/author/:author',
        name: 'author',
        component: HomeView,
        props: true
    },
    {
        path: '/search',
        name: 'search',
        component: HomeView
    },
    {
        path: '/submit',
        name: 'editor',
        component: EditorView
    },
    {
        path: '/blog/:id',
        name: 'blog',
        component: BlogDetailView,
        props: true
    }
];

export default blogRoutes;
