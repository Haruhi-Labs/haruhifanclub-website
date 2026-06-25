import { createRouter, createWebHistory } from 'vue-router'

// 首屏画廊保持同步加载（落地即用）；其余路由懒加载按需分包，
// 尤其 AdminView 较大，避免拖累首屏体积。
import GalleryView from '../views/GalleryView.vue'
// 统一账号 UI（登录/资料/设置/邮件链接落地）
import {
  LoginView,
  ProfileView,
  SettingsView,
  VerifyEmailView,
  ResetPasswordView,
  UserConsoleLayout,
  OverviewView as AccountOverviewView,
  MyArtworksView,
  MyArticlesView,
  MyCommentsView,
  PointsView as AccountPointsView,
} from '@haruhi/auth-ui'

// 个人控制台导航分区：全集（全站可用——从任一 app 进入都能管理全站内容）。
const ACCOUNT_SECTIONS = [
  'overview',
  'artworks',
  'articles',
  'comments',
  'points',
  'profile',
  'settings',
]

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
    { path: '/login', name: 'login', component: LoginView, props: { site: 'art' } },
    {
      path: '/account',
      component: UserConsoleLayout,
      props: { site: 'art', basePath: '/account', sections: ACCOUNT_SECTIONS },
      children: [
        { path: '', name: 'account', component: AccountOverviewView },
        { path: 'artworks', name: 'account-artworks', component: MyArtworksView },
        { path: 'articles', name: 'account-articles', component: MyArticlesView },
        { path: 'comments', name: 'account-comments', component: MyCommentsView },
        { path: 'points', name: 'account-points', component: AccountPointsView },
        {
          path: 'profile',
          name: 'account-profile',
          component: ProfileView,
          props: { site: 'art', embedded: true },
        },
        {
          path: 'settings',
          name: 'account-settings',
          component: SettingsView,
          props: { site: 'art', embedded: true },
        },
      ],
    },
    {
      path: '/verify-email',
      name: 'verify-email',
      component: VerifyEmailView,
      props: { site: 'art' },
    },
    {
      path: '/reset-password',
      name: 'reset-password',
      component: ResetPasswordView,
      props: { site: 'art' },
    },

    // fallback
    { path: '/:pathMatch(.*)*', redirect: '/' },
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
  },
})

export default router
