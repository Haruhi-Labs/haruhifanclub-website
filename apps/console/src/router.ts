import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/dashboard' },
  {
    path: '/dashboard',
    name: 'dashboard',
    component: () => import('./pages/DashboardView.vue'),
    meta: { title: '概览' },
  },
  {
    path: '/users',
    name: 'users',
    component: () => import('./pages/UsersView.vue'),
    meta: { title: '用户与权限' },
  },
  {
    path: '/migration',
    name: 'migration',
    component: () => import('./pages/MigrationView.vue'),
    meta: { title: '内容归属迁移' },
  },
  {
    path: '/notify',
    name: 'notify',
    component: () => import('./pages/NotifyView.vue'),
    meta: { title: '通知设置' },
  },
  {
    path: '/audit',
    name: 'audit',
    component: () => import('./pages/AuditView.vue'),
    meta: { title: '审计日志' },
  },
  { path: '/:pathMatch(.*)*', redirect: '/dashboard' },
]

export const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
})
