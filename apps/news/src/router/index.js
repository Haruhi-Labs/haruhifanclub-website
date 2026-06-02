import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '../views/HomeView.vue';
import EditorView from '../views/EditorView.vue';
import BlogDetailView from '../views/BlogDetailView.vue';
import AdminView from '../views/AdminView.vue';
import HandbookView from '../views/HandbookView.vue';
import QuizView from '../views/QuizView.vue';
import StoreView from '../views/StoreView.vue';
import ActivityView from '../views/ActivityView.vue'; // [新增] 引入

const routes = [
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
        path: '/admin',
        name: 'admin',
        component: AdminView
    },
    {
        path: '/blog/:id',
        name: 'blog',
        component: BlogDetailView,
        props: true
    },
    {
        path: '/handbook',
        name: 'handbook',
        component: HandbookView
    },
    {
        path: '/store',
        name: 'store',
        component: StoreView
    },
    {
        // [新增] 活动中心路由
        path: '/activity',
        name: 'activity',
        component: ActivityView
    },
    {
        path: '/quiz-game',
        name: 'quiz',
        component: QuizView,
        meta: { hideNavbar: true }
    }
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