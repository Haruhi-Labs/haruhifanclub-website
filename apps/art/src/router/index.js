import { createRouter, createWebHistory } from 'vue-router'
import {
  LoginView,
  ProfileView,
  ResetPasswordView,
  SettingsView,
  VerifyEmailView,
  useSession
} from '@haruhi/auth-ui'

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
    { path: '/upload', name: 'upload', component: UploadView, meta: { requiresAuth: true } },
    { path: '/admin', name: 'admin', component: AdminView, meta: { requiresAuth: true } },
    { path: '/points', name: 'points', component: PointsView },
    { path: '/announcements', name: 'announcements', component: AnnouncementView },
    { path: '/exchange', name: 'exchange', component: ExchangeView, meta: { requiresAuth: true } },
    { path: '/license', name: 'license', component: LicenseView },

    { path: '/login', name: 'login', component: LoginView, props: { title: '应援团画廊', home: '/' }, meta: { public: true } },
    { path: '/account', name: 'account', component: ProfileView, meta: { requiresAuth: true } },
    { path: '/account/settings', name: 'account-settings', component: SettingsView, meta: { requiresAuth: true } },
    { path: '/verify-email', name: 'verify-email', component: VerifyEmailView, meta: { public: true } },
    { path: '/reset-password', name: 'reset-password', component: ResetPasswordView, meta: { public: true } },

    { path: '/:pathMatch(.*)*', redirect: '/' }
  ],
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) return savedPosition
    if (to.path === from.path) return false
    return { top: 0 }
  }
})

const session = useSession('/api')

router.beforeEach(async (to) => {
  if (to.meta.public) return true
  if (!to.matched.some((record) => record.meta.requiresAuth)) return true

  if (!session.state.ready) {
    await session.refresh()
  }

  if (!session.state.user) {
    return {
      name: 'login',
      query: { redirect: to.fullPath }
    }
  }

  return true
})

export default router
