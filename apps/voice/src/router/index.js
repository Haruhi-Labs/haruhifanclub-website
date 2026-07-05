import { createRouter, createWebHistory } from 'vue-router'
import {
  LoginView,
  ResetPasswordView,
  VerifyEmailView,
  ProfileView,
  SettingsView,
  UserConsoleLayout,
  OverviewView,
  MyArtworksView,
  MyArticlesView,
  MyCommentsView,
  PointsView as AccountPointsView,
} from '@haruhi/auth-ui'
import { session } from '@/api'

const HomeView = () => import('@/views/HomeView.vue')
const TtsView = () => import('@/views/TtsView.vue')
const RvcView = () => import('@/views/RvcView.vue')
const NotFoundView = () => import('@/views/NotFoundView.vue')

const SITE = 'voice'
// 全站统一个人控制台的子页（语音工坊无本站专属内容，用通用跨 app 分区）
const ACCOUNT_SECTIONS = [
  'overview',
  'artworks',
  'articles',
  'comments',
  'points',
  'profile',
  'settings',
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  scrollBehavior(to, from, saved) {
    if (saved) return saved
    return { top: 0 }
  },
  routes: [
    { path: '/', name: 'home', component: HomeView },
    // 浏览公开；发起合成/转换在页面内检查登录（未登录引导去 /login）
    { path: '/tts', name: 'tts', component: TtsView, meta: { title: '语音合成' } },
    { path: '/rvc', name: 'rvc', component: RvcView, meta: { title: '声线转换' } },

    // 账号（全站统一账号体系，自托管登录 + 个人控制台）
    {
      path: '/login',
      name: 'login',
      component: LoginView,
      props: { site: SITE },
      meta: { title: '登录', noindex: true },
    },
    {
      path: '/reset-password',
      name: 'reset-password',
      component: ResetPasswordView,
      props: { site: SITE },
      meta: { title: '重置密码', noindex: true },
    },
    {
      path: '/verify-email',
      name: 'verify-email',
      component: VerifyEmailView,
      props: { site: SITE },
      meta: { title: '验证邮箱', noindex: true },
    },
    {
      path: '/account',
      component: UserConsoleLayout,
      meta: { requiresAuth: true, title: '个人中心', noindex: true },
      props: { site: SITE, basePath: '/account', home: '/', sections: ACCOUNT_SECTIONS },
      children: [
        { path: '', name: 'account', component: OverviewView },
        { path: 'artworks', name: 'account-artworks', component: MyArtworksView },
        { path: 'articles', name: 'account-articles', component: MyArticlesView },
        { path: 'comments', name: 'account-comments', component: MyCommentsView },
        { path: 'points', name: 'account-points', component: AccountPointsView },
        {
          path: 'profile',
          name: 'account-profile',
          component: ProfileView,
          props: { site: SITE, embedded: true },
        },
        {
          path: 'settings',
          name: 'account-settings',
          component: SettingsView,
          props: { site: SITE, embedded: true },
        },
      ],
    },

    { path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundView, meta: { noindex: true } },
  ],
})

router.beforeEach(async (to) => {
  if (!to.matched.some((r) => r.meta.requiresAuth)) return true
  await session.ensureReady()
  if (!session.state.user) return { name: 'login', query: { redirect: to.fullPath } }
  return true
})

export default router
