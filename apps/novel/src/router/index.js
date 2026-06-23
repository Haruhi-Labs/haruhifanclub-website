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
} from '@haruhi/auth-ui'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'Shelf', component: Shelf },
    { path: '/read/:id', name: 'Reader', component: Reader, props: true },
    { path: '/admin', name: 'Admin', component: Admin },
    { path: '/feedback', name: 'Feedback', component: FeedbackView }, // 顺便大小写统一一下
    { path: '/login', name: 'login', component: LoginView, props: { site: 'library' } },
    { path: '/account', name: 'account', component: ProfileView, props: { site: 'library' } },
    {
      path: '/account/settings',
      name: 'account-settings',
      component: SettingsView,
      props: { site: 'library' },
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
