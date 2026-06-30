import { createRouter, createWebHistory } from 'vue-router'
import {
  LoginView,
  MyArticlesView,
  MyArtworksView,
  MyCommentsView,
  OverviewView as AccountOverviewView,
  PointsView as AccountPointsView,
  ProfileView,
  ResetPasswordView,
  SettingsView,
  UserConsoleLayout,
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

const authProps = { site: 'art', title: '凉宫春日应援团', home: '/' }
const accountSections = [
  'overview',
  'artworks',
  'articles',
  'comments',
  'points',
  'profile',
  'settings'
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/gallery', name: 'gallery', component: GalleryView },
    { path: '/upload', name: 'upload', component: UploadView },
    { path: '/admin', name: 'admin', component: AdminView, meta: { requiresAuth: true } },
    { path: '/points', redirect: '/exchange' },
    { path: '/announcements', name: 'announcements', component: AnnouncementView },
    { path: '/exchange', name: 'exchange', component: ExchangeView },
    { path: '/terminal', name: 'terminal', component: AdventurerProfileView, meta: { requiresAuth: true } },
    { path: '/profile/:uid', name: 'adventurer-profile', component: AdventurerProfileView, props: true },
    { path: '/license', name: 'license', component: LicenseView },

    { path: '/login', name: 'login', component: LoginView, props: authProps },
    {
      path: '/account',
      component: UserConsoleLayout,
      props: { site: 'art', basePath: '/account', home: '/', sections: accountSections },
      meta: { requiresAuth: true },
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
          props: { site: 'art', embedded: true }
        },
        {
          path: 'settings',
          name: 'account-settings',
          component: SettingsView,
          props: { site: 'art', embedded: true }
        }
      ]
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

  await session.ensureReady()

  if (!session.state.user) {
    return {
      name: 'login',
      query: { redirect: to.fullPath }
    }
  }

  return true
})

export default router
