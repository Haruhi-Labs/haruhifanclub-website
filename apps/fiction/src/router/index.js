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
import { canModerate } from '@/lib/admin'

import HomeView from '@/views/HomeView.vue'
const LibraryView = () => import('@/views/LibraryView.vue')
const StoryView = () => import('@/views/StoryView.vue')
const ReadView = () => import('@/views/ReadView.vue')
const BookmarksView = () => import('@/views/BookmarksView.vue')
const WriteDashboardView = () => import('@/views/WriteDashboardView.vue')
const StoryEditorView = () => import('@/views/StoryEditorView.vue')
const ChapterEditorView = () => import('@/views/ChapterEditorView.vue')
const AccountStoriesView = () => import('@/views/AccountStoriesView.vue')
const NotFoundView = () => import('@/views/NotFoundView.vue')

// 隐藏后台（/novel/admin）：无导航入口，仅 fiction 管理员可达
const AdminLayout = () => import('@/views/admin/AdminLayout.vue')
const AdminOverview = () => import('@/views/admin/AdminOverview.vue')
const AdminWorks = () => import('@/views/admin/AdminWorks.vue')
const AdminComments = () => import('@/views/admin/AdminComments.vue')

const SITE = 'shop'
// 「我的同人文」为 fiction 专属分区：导航项在 auth-ui 通用，子页用本地视图（跳创作中心管理）
const ACCOUNT_SECTIONS = [
  'overview',
  'artworks',
  'articles',
  'stories',
  'comments',
  'points',
  'profile',
  'settings',
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  scrollBehavior(to, from, saved) {
    if (saved) return saved
    if (to.hash) return { el: to.hash }
    return { top: 0 }
  },
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/library', name: 'library', component: LibraryView, meta: { title: '书库' } },
    // 详情页 title/meta 由视图内 usePageMeta 在数据加载后设置
    { path: '/story/:id', name: 'story', component: StoryView },
    { path: '/story/:id/chapter/:cid', name: 'read', component: ReadView },

    {
      path: '/bookmarks',
      name: 'bookmarks',
      component: BookmarksView,
      meta: { requiresAuth: true, title: '我的书架', noindex: true },
    },

    // 创作（需登录）
    {
      path: '/write',
      name: 'write',
      component: WriteDashboardView,
      meta: { requiresAuth: true, title: '创作中心', noindex: true },
    },
    {
      path: '/write/new',
      name: 'write-new',
      component: StoryEditorView,
      meta: { requiresAuth: true, title: '新建作品', noindex: true },
    },
    {
      path: '/write/:id',
      name: 'write-story',
      component: StoryEditorView,
      meta: { requiresAuth: true, title: '管理作品', noindex: true },
    },
    {
      path: '/write/:id/chapter/:cid',
      name: 'write-chapter',
      component: ChapterEditorView,
      meta: { requiresAuth: true, title: '编辑章节', noindex: true },
    },

    // 账号
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
        { path: 'stories', name: 'account-stories', component: AccountStoriesView },
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

    // 隐藏后台
    {
      path: '/admin',
      component: AdminLayout,
      meta: { requiresAuth: true, requiresAdmin: true, noindex: true },
      children: [
        { path: '', name: 'admin', component: AdminOverview },
        { path: 'works', name: 'admin-works', component: AdminWorks },
        { path: 'comments', name: 'admin-comments', component: AdminComments },
      ],
    },

    { path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundView, meta: { noindex: true } },
  ],
})

router.beforeEach(async (to) => {
  const needsAuth = to.matched.some((r) => r.meta.requiresAuth)
  const needsAdmin = to.matched.some((r) => r.meta.requiresAdmin)
  if (!needsAuth && !needsAdmin) return true
  await session.ensureReady()
  const user = session.state.user
  if (!user) return { name: 'login', query: { redirect: to.fullPath } }
  // 隐藏后台：非 fiction 管理员一律当作不存在，静默回首页（不暴露入口）
  if (needsAdmin && !canModerate(user)) return { name: 'home' }
  return true
})

export default router
