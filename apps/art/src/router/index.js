import { createRouter, createWebHistory } from 'vue-router'

// 首屏画廊保持同步加载（落地即用）；其余路由懒加载按需分包，
// 尤其 AdminView 较大，避免拖累首屏体积。
import GalleryView from '../views/GalleryView.vue'
// 统一账号 UI（登录/资料/设置/邮件链接落地）
import { LoginView, ProfileView, SettingsView, VerifyEmailView, ResetPasswordView } from '@haruhi/auth-ui'

const UploadView = () => import('../views/UploadView.vue')
const AdminView = () => import('../views/AdminView.vue')
const LicenseView = () => import('../views/LicenseView.vue')
const PointsView = () => import('../views/PointsView.vue')

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'gallery', component: GalleryView },
    { path: '/upload', name: 'upload', component: UploadView },
    { path: '/admin', name: 'admin', component: AdminView },
    { path: '/points', name: 'points', component: PointsView },
    // 新增：授权查询页
    { path: '/license', name: 'license', component: LicenseView },

    // 统一账号系统
    { path: '/login', name: 'login', component: LoginView },
    { path: '/account', name: 'account', component: ProfileView },
    { path: '/account/settings', name: 'account-settings', component: SettingsView },
    { path: '/verify-email', name: 'verify-email', component: VerifyEmailView },
    { path: '/reset-password', name: 'reset-password', component: ResetPasswordView },

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