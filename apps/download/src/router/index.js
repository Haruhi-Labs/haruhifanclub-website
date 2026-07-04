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

const CatalogView = () => import('@/views/CatalogView.vue')
const NotFoundView = () => import('@/views/NotFoundView.vue')

const SITE = 'download'
// 全站统一个人控制台的子页（资源站无本站专属内容，用通用跨 app 分区）
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
    // 仅 query 变化（检索/切分类）不滚动，避免输入时页面跳动
    if (to.path === from.path) return false
    if (saved) return saved
    return { top: 0 }
  },
  routes: [
    { path: '/', name: 'home', component: CatalogView },

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
