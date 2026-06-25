// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'
// 首屏书架保持同步加载；Reader（含 epubjs/dompurify/OpenCC）与后台懒加载分包。
import Shelf from '../views/Shelf.vue'

const Reader = () => import('../views/Reader.vue')
const Admin = () => import('../views/Admin.vue')
const FeedbackView = () => import('../views/FeedbackView.vue')
// 统一账号 UI
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
  MyExamsView,
  MyCommentsView,
  PointsView as AccountPointsView,
} from '@haruhi/auth-ui'

// 个人控制台导航分区：全集（全站可用——从任一 app 进入都能管理全站内容）。
const ACCOUNT_SECTIONS = [
  'overview',
  'artworks',
  'articles',
  'exams',
  'comments',
  'points',
  'profile',
  'settings',
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'Shelf', component: Shelf },
    { path: '/read/:id', name: 'Reader', component: Reader, props: true },
    { path: '/admin', name: 'Admin', component: Admin },
    { path: '/feedback', name: 'Feedback', component: FeedbackView }, // 顺便大小写统一一下
    { path: '/login', name: 'login', component: LoginView, props: { site: 'library' } },
    {
      path: '/account',
      component: UserConsoleLayout,
      props: { site: 'library', basePath: '/account', sections: ACCOUNT_SECTIONS },
      children: [
        { path: '', name: 'account', component: AccountOverviewView },
        { path: 'artworks', name: 'account-artworks', component: MyArtworksView },
        { path: 'articles', name: 'account-articles', component: MyArticlesView },
        { path: 'exams', name: 'account-exams', component: MyExamsView },
        { path: 'comments', name: 'account-comments', component: MyCommentsView },
        { path: 'points', name: 'account-points', component: AccountPointsView },
        {
          path: 'profile',
          name: 'account-profile',
          component: ProfileView,
          props: { site: 'library', embedded: true },
        },
        {
          path: 'settings',
          name: 'account-settings',
          component: SettingsView,
          props: { site: 'library', embedded: true },
        },
      ],
    },
    {
      path: '/verify-email',
      name: 'verify-email',
      component: VerifyEmailView,
      props: { site: 'library' },
    },
    {
      path: '/reset-password',
      name: 'reset-password',
      component: ResetPasswordView,
      props: { site: 'library' },
    },
  ],
})

export default router
