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
import AnnouncementView from '../views/AnnouncementView.vue'
import ExchangeView from '../views/ExchangeView.vue'
import AdventurerProfileView from '../views/AdventurerProfileView.vue'

const AdminView = () => import('../views/AdminView.vue')
const LicenseView = () => import('../views/LicenseView.vue')

const authProps = { site: 'art', title: '应援团画廊', home: '/' }

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/gallery', name: 'gallery', component: GalleryView },
    { path: '/upload', name: 'upload', component: UploadView, meta: { requiresAuth: true } },
    { path: '/admin', name: 'admin', component: AdminView, meta: { requiresAuth: true } },
    { path: '/points', redirect: '/exchange' },
    { path: '/announcements', name: 'announcements', component: AnnouncementView },
    { path: '/exchange', name: 'exchange', component: ExchangeView },
    { path: '/terminal', name: 'terminal', component: AdventurerProfileView, meta: { requiresAuth: true } },
    { path: '/profile/:uid', name: 'adventurer-profile', component: AdventurerProfileView, props: true },
    { path: '/license', name: 'license', component: LicenseView },

    { path: '/login', name: 'login', component: LoginView, props: authProps },
    { path: '/account', name: 'account', component: ProfileView, props: authProps, meta: { requiresAuth: true } },
    {
      path: '/account/settings',
      name: 'account-settings',
      component: SettingsView,
      props: authProps,
      meta: { requiresAuth: true }
    },
    { path: '/verify-email', name: 'verify-email', component: VerifyEmailView, props: authProps },
    { path: '/reset-password', name: 'reset-password', component: ResetPasswordView, props: authProps },

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
