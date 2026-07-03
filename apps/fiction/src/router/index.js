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
    { path: '/library', name: 'library', component: LibraryView },
    { path: '/story/:id', name: 'story', component: StoryView },
    { path: '/story/:id/chapter/:cid', name: 'read', component: ReadView },

    {
      path: '/bookmarks',
      name: 'bookmarks',
      component: BookmarksView,
      meta: { requiresAuth: true },
    },

    // 创作（需登录）
    { path: '/write', name: 'write', component: WriteDashboardView, meta: { requiresAuth: true } },
    {
      path: '/write/new',
      name: 'write-new',
      component: StoryEditorView,
      meta: { requiresAuth: true },
    },
    {
      path: '/write/:id',
      name: 'write-story',
      component: StoryEditorView,
      meta: { requiresAuth: true },
    },
    {
      path: '/write/:id/chapter/:cid',
      name: 'write-chapter',
      component: ChapterEditorView,
      meta: { requiresAuth: true },
    },

    // 账号
    { path: '/login', name: 'login', component: LoginView, props: { site: SITE } },
    {
      path: '/reset-password',
      name: 'reset-password',
      component: ResetPasswordView,
      props: { site: SITE },
    },
    {
      path: '/verify-email',
      name: 'verify-email',
      component: VerifyEmailView,
      props: { site: SITE },
    },
    {
      path: '/account',
      component: UserConsoleLayout,
      meta: { requiresAuth: true },
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

    { path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundView },
  ],
})

router.beforeEach(async (to) => {
  if (!to.matched.some((r) => r.meta.requiresAuth)) return true
  await session.ensureReady()
  if (session.state.user) return true
  return { name: 'login', query: { redirect: to.fullPath } }
})

export default router
