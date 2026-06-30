<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { SosAppbar } from '@haruhi/ui'
import { AccountMenu, AccountAvatarLink, useSession } from '@haruhi/auth-ui'
import logoUrl from '../assets/logo.webp'

const route = useRoute()
const session = useSession('/api')

// 导航项（含 PR 新增的公会/公告）。终端登录后才显示。
const navItems = [
  { path: '/gallery', label: '画廊' },
  { path: '/upload', label: '投稿' },
  { path: '/exchange', label: '公会' },
  { path: '/announcements', label: '公告' },
]
const showTerminal = computed(() => !!session.state.user)

const isActive = (path) => {
  if (path === '/gallery' && route.path.startsWith('/profile/')) return true
  if (path === '/exchange' && route.path === '/points') return true
  return route.path === path
}
</script>

<template>
  <!-- 统一页头：复用设计系统 SosAppbar（#21 移动端汉堡+右侧抽屉+遮罩+滚动锁+Esc 全部内置），
       art 站配色经 data-sos-site="art" 的 bridges 提供，仅以 art-appbar 类做局部点缀。 -->
  <SosAppbar class="art-appbar">
    <template #brand>
      <RouterLink to="/" class="sos-brand-lockup" data-sfx="click">
        <span class="sos-brand-lockup__mark">
          <img :src="logoUrl" alt="" />
        </span>
        <span class="sos-brand-lockup__text">
          <strong>春日画廊</strong>
          <small>凉宫春日应援团 · 绘画部</small>
        </span>
      </RouterLink>
    </template>

    <nav class="sos-navlinks" aria-label="画廊功能导航">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        class="sos-navlink"
        :class="{ 'sos-navlink--active': isActive(item.path) }"
        :to="item.path"
        :aria-current="isActive(item.path) ? 'page' : undefined"
        data-sfx="click"
      >{{ item.label }}</RouterLink>
      <RouterLink
        v-if="showTerminal"
        class="sos-navlink"
        :class="{ 'sos-navlink--active': isActive('/terminal') }"
        to="/terminal"
        :aria-current="isActive('/terminal') ? 'page' : undefined"
        data-sfx="click"
      >终端</RouterLink>
    </nav>

    <template #actions>
      <AccountMenu login-path="/login" profile-path="/account" settings-path="/account/settings" />
    </template>

    <!-- 移动端：头像快捷入口提到汉堡左侧，点头像直达个人中心 -->
    <template #mobile-lead>
      <AccountAvatarLink profile-path="/account" />
    </template>
  </SosAppbar>
</template>
