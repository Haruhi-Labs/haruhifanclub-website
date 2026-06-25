import { createRouter, createWebHistory } from 'vue-router'
import ShopLayout from '../layouts/ShopLayout.vue'
import ShopAuthLayout from '../layouts/ShopAuthLayout.vue'
import AdminLayout from '../layouts/AdminLayout.vue'
import { trackEvent } from '@/utils/analytics'
import { hasValidAdminToken, verifyShopAccess } from '@/utils/adminAuth'
// 统一账号 UI（终端用户登录/资料/设置/邮件链接落地）
import {
  LoginView,
  ProfileView,
  SettingsView,
  VerifyEmailView,
  ResetPasswordView,
  UserConsoleLayout,
  OverviewView as AccountOverviewView,
  MyArtworksView,
  MyArticlesView,
  MyCommentsView,
  PointsView as AccountPointsView,
} from '@haruhi/auth-ui'

// 个人控制台导航分区：全集（全站可用——从任一 app 进入都能管理全站内容）。
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
  routes: [
    {
      path: '/admin/login',
      name: 'admin-login',
      component: () => import('../views/admin/AdminLoginView.vue'),
    },
    // 终端用户账号系统：套专门的 ShopAuthLayout（精简页头 + 满屏 shop 主题背景），
    // 既补上之前缺失的页头，也消除登录页「下半截颜色不同的底」。
    // 用「哑父路径 + 绝对路径子路由」包裹布局，避免与 ShopLayout 的 '/' 首页冲突。
    {
      path: '/account-portal',
      component: ShopAuthLayout,
      redirect: '/login',
      children: [
        { path: '/login', name: 'login', component: LoginView, props: { site: 'shop' } },
        {
          path: '/account',
          component: UserConsoleLayout,
          props: { site: 'shop', basePath: '/account', sections: ACCOUNT_SECTIONS },
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
              props: { site: 'shop', embedded: true },
            },
            {
              path: 'settings',
              name: 'account-settings',
              component: SettingsView,
              props: { site: 'shop', embedded: true },
            },
          ],
        },
        {
          path: '/verify-email',
          name: 'verify-email',
          component: VerifyEmailView,
          props: { site: 'shop' },
        },
        {
          path: '/reset-password',
          name: 'reset-password',
          component: ResetPasswordView,
          props: { site: 'shop' },
        },
      ],
    },
    // 前台商城路由
    {
      path: '/',
      component: ShopLayout,
      children: [
        { path: '', name: 'home', component: () => import('../views/shop/HomeView.vue') },
        {
          path: 'product/:id',
          name: 'product',
          component: () => import('../views/shop/ProductDetailView.vue'),
        },
        { path: 'cart', name: 'cart', component: () => import('../views/shop/CartView.vue') },
        {
          path: 'checkout',
          name: 'checkout',
          component: () => import('../views/shop/CheckoutView.vue'),
        },
        {
          path: 'payment',
          name: 'payment',
          component: () => import('../views/shop/PaymentView.vue'),
        },
        {
          path: 'success',
          name: 'success',
          component: () => import('../views/shop/SuccessView.vue'),
        },
        {
          path: 'query',
          name: 'query',
          component: () => import('../views/shop/OrderQueryView.vue'),
        },
        {
          path: 'contact',
          name: 'contact',
          component: () => import('../views/shop/ContactView.vue'),
        },
      ],
    },
    // 后台管理路由
    {
      path: '/admin',
      component: AdminLayout,
      children: [
        { path: '', redirect: '/admin/dashboard' },
        {
          path: 'dashboard',
          name: 'admin-dashboard',
          component: () => import('../views/admin/DashboardView.vue'),
        },
        {
          path: 'orders',
          name: 'admin-orders',
          component: () => import('../views/admin/OrdersView.vue'),
        },
        {
          path: 'messages',
          name: 'admin-messages',
          component: () => import('../views/admin/MessagesView.vue'),
        },
        {
          path: 'products',
          name: 'admin-products',
          component: () => import('../views/admin/ProductsView.vue'),
        }, // 新增路由
        {
          path: 'coupons',
          name: 'admin-coupons',
          component: () => import('../views/admin/CouponsView.vue'),
        },
        {
          path: 'stats',
          name: 'admin-stats',
          component: () => import('../views/admin/StatsView.vue'),
        },
        {
          path: 'settings',
          name: 'admin-settings',
          component: () => import('../views/admin/SettingsView.vue'),
        },
      ],
    },
  ],
  scrollBehavior(to, from, savedPosition) {
    return { top: 0 }
  },
})

router.beforeEach(async (to) => {
  const isAdminRoute = to.path.startsWith('/admin')
  if (!isAdminRoute) return true

  if (to.name === 'admin-login') {
    // 已登录且具备 shop 权限则跳过登录页
    if (hasValidAdminToken() && (await verifyShopAccess())) return { path: '/admin/dashboard' }
    return true
  }

  // 统一 JWT 会话恢复 + 权限校验：本地有有效 token 且 /api/auth/me 确认具备 shop 权限
  if (!hasValidAdminToken() || !(await verifyShopAccess())) {
    return { path: '/admin/login', query: { redirect: to.fullPath } }
  }

  return true
})

const routeEventMap = {
  home: 'home_view',
  product: 'product_view',
  cart: 'cart_view',
  checkout: 'checkout_view',
  payment: 'payment_view',
  success: 'success_view',
  query: 'order_query_view',
  contact: 'contact_view',
}

router.afterEach((to) => {
  if (to.path.startsWith('/admin')) return
  const eventKey = routeEventMap[to.name]
  if (!eventKey) return
  trackEvent(eventKey, { routeName: to.name })
})

export default router
