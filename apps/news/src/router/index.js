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
  UserConsoleLayout,
  OverviewView,
  MyArtworksView,
  MyArticlesView,
  MyExamsView,
  MyCommentsView,
  PointsView,
} from '@haruhi/auth-ui'

// 个人控制台：/account 为带侧边导航的「个人空间」外壳，子页嵌套渲染。
// sections 控制导航显示的分区（随子页接入逐步扩展）。
const accountRoutes = [
  { path: '/login', name: 'login', component: LoginView, props: { site: 'news' } },
  {
    path: '/account',
    component: UserConsoleLayout,
    props: {
      site: 'news',
      basePath: '/account',
      sections: [
        'overview',
        'artworks',
        'articles',
        'exams',
        'comments',
        'points',
        'profile',
        'settings',
      ],
    },
    children: [
      { path: '', name: 'account', component: OverviewView },
      { path: 'artworks', name: 'account-artworks', component: MyArtworksView },
      { path: 'articles', name: 'account-articles', component: MyArticlesView },
      { path: 'exams', name: 'account-exams', component: MyExamsView },
      { path: 'comments', name: 'account-comments', component: MyCommentsView },
      { path: 'points', name: 'account-points', component: PointsView },
      {
        path: 'profile',
        name: 'account-profile',
        component: ProfileView,
        props: { site: 'news' },
      },
      {
        path: 'settings',
        name: 'account-settings',
        component: SettingsView,
        props: { site: 'news' },
      },
    ],
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
