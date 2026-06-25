import { createRouter, createWebHistory } from 'vue-router'

// 按特性组合各 feature 的路由数组（path/name/meta 与重构前一字不差）。
import { blogRoutes } from '@/features/blog/routes'
import { handbookRoutes } from '@/features/handbook/routes'
import { storeRoutes } from '@/features/store/routes'
import { quizRoutes } from '@/features/quiz/routes'
import { activityRoutes } from '@/features/activity/routes'
import { adminRoutes } from '@/features/admin/routes'
// 统一账号 UI（news 作主站，承载邮箱验证/找回密码邮件链接落地页）
import {
  LoginView,
  ProfileView,
  SettingsView,
  VerifyEmailView,
  ResetPasswordView,
} from '@haruhi/auth-ui'

const accountRoutes = [
  { path: '/login', name: 'login', component: LoginView, props: { site: 'news' } },
  { path: '/account', name: 'account', component: ProfileView, props: { site: 'news' } },
  {
    path: '/account/settings',
    name: 'account-settings',
    component: SettingsView,
    props: { site: 'news' },
  },
  {
    path: '/verify-email',
    name: 'verify-email',
    component: VerifyEmailView,
    props: { site: 'news' },
  },
  {
    path: '/reset-password',
    name: 'reset-password',
    component: ResetPasswordView,
    props: { site: 'news' },
  },
]

const routes = [
  ...blogRoutes,
  ...handbookRoutes,
  ...storeRoutes,
  ...activityRoutes,
  ...quizRoutes,
  ...adminRoutes,
  ...accountRoutes,
]

const router = createRouter({
  // 部署于子路径 /news/，history base 取自 vite 的 BASE_URL
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      return savedPosition
    } else {
      return { top: 0, behavior: 'smooth' }
    }
  },
})

export default router
