import { createRouter, createWebHistory } from 'vue-router'
import {
  LoginView,
  MyArticlesView,
  MyArtworksView,
  MyCommentsView,
  OverviewView,
  PointsView,
  ProfileView,
  ResetPasswordView,
  SettingsView,
  UserConsoleLayout,
  VerifyEmailView,
} from '@haruhi/auth-ui'
import { session } from '@/api'

const HomeView = () => import('@/views/HomeView.vue')
const CollectionView = () => import('@/views/CollectionView.vue')
const BranchView = () => import('@/views/BranchView.vue')
const DetailView = () => import('@/views/DetailView.vue')
const ManageView = () => import('@/views/ManageView.vue')
const PlatformAdminView = () => import('@/views/PlatformAdminView.vue')
const NotFoundView = () => import('@/views/NotFoundView.vue')

const routes = [
  { path: '/', name: 'home', component: HomeView, meta: { title: '地方支部' } },
  {
    path: '/timeline',
    name: 'timeline',
    component: CollectionView,
    meta: { title: '活动时间线', collection: 'timeline' },
  },
  { path: '/posts', redirect: '/timeline' },
  {
    path: '/events',
    name: 'events',
    component: CollectionView,
    meta: { title: '地方活动', collection: 'events' },
  },
  {
    path: '/branches/:branchSlug',
    name: 'branch',
    component: BranchView,
    meta: { section: 'home' },
  },
  ...['organization', 'members', 'events', 'merchandise', 'join'].map((section) => ({
    path: `/branches/:branchSlug/${section}`,
    name: `branch-${section}`,
    component: BranchView,
    meta: { section },
  })),
  ...['about', 'timeline', 'posts'].map((section) => ({
    path: `/branches/:branchSlug/${section}`,
    redirect: (to) => ({
      name: 'branch',
      params: { branchSlug: to.params.branchSlug },
      query: to.query,
      hash: to.hash,
    }),
  })),
  {
    path: '/branches/:branchSlug/contact',
    redirect: (to) => ({
      name: 'branch-join',
      params: { branchSlug: to.params.branchSlug },
      query: to.query,
      hash: to.hash,
    }),
  },
  {
    path: '/branches/:branchSlug/events/:contentSlug',
    name: 'branch-event',
    component: DetailView,
    meta: { contentType: 'events' },
  },
  {
    path: '/branches/:branchSlug/manage/events/new/:section?',
    name: 'branch-manage-event-new',
    component: ManageView,
    meta: { requiresAuth: true, noindex: true },
  },
  {
    path: '/branches/:branchSlug/manage/events/invitations',
    name: 'branch-manage-event-invitations',
    component: ManageView,
    meta: { requiresAuth: true, noindex: true },
  },
  {
    path: '/branches/:branchSlug/manage/events/:eventId/:section?',
    name: 'branch-manage-event-edit',
    component: ManageView,
    meta: { requiresAuth: true, noindex: true },
  },
  {
    path: '/branches/:branchSlug/manage/timeline',
    redirect: (to) => `/branches/${to.params.branchSlug}/manage/albums`,
  },
  {
    path: '/branches/:branchSlug/manage/:panel?/:subpanel?',
    name: 'branch-manage',
    component: ManageView,
    meta: { requiresAuth: true, noindex: true },
  },
  {
    path: '/admin/branches',
    name: 'platform-admin',
    component: PlatformAdminView,
    meta: { requiresAuth: true, noindex: true },
  },
  {
    path: '/login',
    name: 'login',
    component: LoginView,
    props: { site: 'chapter', title: '地方支部', home: '/' },
    meta: { noindex: true },
  },
  {
    path: '/account',
    component: UserConsoleLayout,
    props: {
      site: 'chapter',
      basePath: '/account',
      home: '/',
      sections: ['overview', 'artworks', 'articles', 'comments', 'points', 'profile', 'settings'],
    },
    meta: { requiresAuth: true, noindex: true },
    children: [
      { path: '', component: OverviewView },
      { path: 'artworks', component: MyArtworksView },
      { path: 'articles', component: MyArticlesView },
      { path: 'comments', component: MyCommentsView },
      { path: 'points', component: PointsView },
      { path: 'profile', component: ProfileView, props: { site: 'chapter', embedded: true } },
      { path: 'settings', component: SettingsView, props: { site: 'chapter', embedded: true } },
    ],
  },
  {
    path: '/verify-email',
    component: VerifyEmailView,
    props: { site: 'chapter' },
    meta: { noindex: true },
  },
  {
    path: '/reset-password',
    component: ResetPasswordView,
    props: { site: 'chapter' },
    meta: { noindex: true },
  },
  {
    path: '/:pathMatch(.*)*',
    component: NotFoundView,
    meta: { title: '页面不存在', noindex: true },
  },
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
  scrollBehavior: (_to, _from, savedPosition) => savedPosition || { top: 0 },
})

router.beforeEach(async (to) => {
  if (!to.matched.some((record) => record.meta.requiresAuth)) return true
  await session.ensureReady()
  if (session.state.user) return true
  return { name: 'login', query: { redirect: to.fullPath } }
})

export default router
