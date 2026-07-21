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

const HomeView = () => import('../views/HomeView.vue')
const GalleryView = () => import('../views/GalleryView.vue')
const GallerySearchView = () => import('../views/GallerySearchView.vue')
const UploadView = () => import('../views/UploadView.vue')
const AnnouncementView = () => import('../views/AnnouncementView.vue')
const ExchangeView = () => import('../views/ExchangeView.vue')
const AdventurerProfileView = () => import('../views/AdventurerProfileView.vue')
const AdminView = () => import('../views/AdminView.vue')
const LicenseView = () => import('../views/LicenseView.vue')
const ArtworkDetailView = () => import('../views/ArtworkDetailView.vue')
const CreatorsView = () => import('../views/CreatorsView.vue')

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
    { path: '/gallery', name: 'gallery', component: GalleryView, meta: { title: '画廊' } },
    { path: '/creators', name: 'creators', component: CreatorsView, meta: { title: '创作者' } },
    {
      path: '/categories',
      name: 'gallery-categories',
      redirect: to => ({ name: 'gallery', query: to.query, hash: '#gallery-catalog' })
    },
    {
      path: '/gallery/search',
      name: 'gallery-search',
      component: GallerySearchView,
      meta: { title: '搜索作品' }
    },
    {
      path: '/artwork/:id',
      name: 'artwork-detail',
      component: ArtworkDetailView,
      props: true,
      meta: { title: '作品详情' }
    },
    { path: '/upload', name: 'upload', component: UploadView, meta: { title: '投稿', noindex: true } },
    {
      path: '/admin',
      name: 'admin',
      component: AdminView,
      meta: { requiresAuth: true, title: '审核后台', noindex: true }
    },
    { path: '/points', redirect: '/exchange' },
    {
      path: '/announcements',
      name: 'announcements',
      component: AnnouncementView,
      meta: { title: '公告' }
    },
    { path: '/exchange', name: 'exchange', component: ExchangeView, meta: { title: '积分兑换' } },
    {
      path: '/terminal',
      name: 'terminal',
      component: AdventurerProfileView,
      meta: { requiresAuth: true, title: '创作者终端', noindex: true }
    },
    // 冒险者档案为动态内容，本期不做视图级 meta
    { path: '/profile/:uid', name: 'adventurer-profile', component: AdventurerProfileView, props: true },
    { path: '/license', name: 'license', component: LicenseView, meta: { title: '访问许可' } },

    {
      path: '/login',
      name: 'login',
      component: LoginView,
      props: authProps,
      meta: { title: '登录', noindex: true }
    },
    {
      path: '/account',
      component: UserConsoleLayout,
      props: { site: 'art', basePath: '/account', home: '/', sections: accountSections },
      meta: { requiresAuth: true, title: '个人中心', noindex: true },
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
    {
      path: '/verify-email',
      name: 'verify-email',
      component: VerifyEmailView,
      props: authProps,
      meta: { title: '验证邮箱', noindex: true }
    },
    {
      path: '/reset-password',
      name: 'reset-password',
      component: ResetPasswordView,
      props: authProps,
      meta: { title: '重置密码', noindex: true }
    },

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
  if (['gallery', 'gallery-search'].includes(to.name) && to.query.artwork) {
    return { name: 'artwork-detail', params: { id: to.query.artwork } }
  }

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
