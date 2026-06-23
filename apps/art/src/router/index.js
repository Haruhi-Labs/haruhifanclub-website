import { createRouter, createWebHistory } from 'vue-router'

// 首屏首页保持同步加载（落地即用）；其余路由懒加载按需分包，
// 尤其 AdminView 较大，避免拖累首屏体积。
import HomeView from '../views/HomeView.vue'

const GalleryView = () => import('../views/GalleryView.vue')
const UploadView = () => import('../views/UploadView.vue')
const AdminView = () => import('../views/AdminView.vue')
const LicenseView = () => import('../views/LicenseView.vue')
const PointsView = () => import('../views/PointsView.vue')
const AnnouncementView = () => import('../views/AnnouncementView.vue')
const ExchangeView = () => import('../views/ExchangeView.vue')

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/gallery', name: 'gallery', component: GalleryView },
    { path: '/upload', name: 'upload', component: UploadView },
    { path: '/admin', name: 'admin', component: AdminView },
    { path: '/points', name: 'points', component: PointsView },
    { path: '/announcements', name: 'announcements', component: AnnouncementView },
    { path: '/exchange', name: 'exchange', component: ExchangeView },
    // 新增：授权查询页
    { path: '/license', name: 'license', component: LicenseView },

    // fallback
    { path: '/:pathMatch(.*)*', redirect: '/' }
  ],
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      return savedPosition
    }
    // 如果只是 query 变化 (例如打开/关闭详情弹窗)，不重置滚动条
    if (to.path === from.path) {
      return false
    }
    return { top: 0 }
  }
})

export default router
