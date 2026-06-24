import { createRouter, createWebHistory } from 'vue-router'

import HomeView from '../views/HomeView.vue'
import GalleryView from '../views/GalleryView.vue'
import UploadView from '../views/UploadView.vue'
import PointsView from '../views/PointsView.vue'
import AnnouncementView from '../views/AnnouncementView.vue'
import ExchangeView from '../views/ExchangeView.vue'

const AdminView = () => import('../views/AdminView.vue')
const LicenseView = () => import('../views/LicenseView.vue')
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
