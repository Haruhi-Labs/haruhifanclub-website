// blog 特性路由：首页（含 tag/参与者/作者/搜索别名）、投稿编辑器、博客详情
// path/name 与重构前保持一字不差；组件懒加载按 feature 分包；meta.title/noindex 供 @haruhi/seo 消费。
const HomeView = () => import('@/features/blog/views/HomeView.vue');
const EditorView = () => import('@/features/blog/views/EditorView.vue');
const BlogDetailView = () => import('@/features/blog/views/BlogDetailView.vue');

export const blogRoutes = [
    {
        // 首页不设 title，回落 defaultTitle（春日团报 · 凉宫春日应援团）
        path: '/',
        name: 'home',
        component: HomeView
    },
    {
        path: '/tag/:tag',
        name: 'tag',
        component: HomeView,
        props: true,
        meta: { title: (to) => '#' + to.params.tag }
    },
    {
        path: '/participant/:name',
        name: 'participant',
        component: HomeView,
        props: true,
        meta: { title: (to) => to.params.name + ' 参与的活动' }
    },
    {
        path: '/author/:author',
        name: 'author',
        component: HomeView,
        props: true,
        meta: { title: (to) => to.params.author + ' 的文章' }
    },
    {
        path: '/search',
        name: 'search',
        component: HomeView,
        meta: { title: '搜索', noindex: true }
    },
    {
        path: '/submit',
        name: 'editor',
        component: EditorView,
        meta: { title: '投稿', noindex: true }
    },
    {
        // 详情页 title/meta 由视图内 usePageMeta 在数据加载后设置
        path: '/blog/:id',
        name: 'blog',
        component: BlogDetailView,
        props: true
    }
];

export default blogRoutes;
