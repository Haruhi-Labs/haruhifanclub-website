<script setup>
import { useRoute } from 'vue-router'
import { SosAppbar } from '@haruhi/ui'
import { AccountMenu } from '@haruhi/auth-ui'
import logoUrl from '../assets/logo.webp'

const route = useRoute()
const isActive = (path) => {
  if (path === '/' && route.path.startsWith('/creator')) return true
  return route.path === path
}
</script>

<template>
  <SosAppbar class="art-appbar">
    <template #brand>
      <RouterLink to="/" class="sos-brand-lockup">
        <span class="sos-brand-lockup__mark">
          <img :src="logoUrl" alt="" />
        </span>
        <span class="sos-brand-lockup__text">
          <strong>春日画廊</strong>
          <small>凉宫春日应援团 · 绘画部</small>
        </span>
      </RouterLink>
    </template>

    <nav class="sos-navlinks">
      <RouterLink class="sos-navlink" :class="{ 'sos-navlink--active': isActive('/') }" to="/">画廊</RouterLink>
      <RouterLink class="sos-navlink" :class="{ 'sos-navlink--active': isActive('/upload') }" to="/upload">投稿</RouterLink>
      <RouterLink class="sos-navlink" :class="{ 'sos-navlink--active': isActive('/points') }" to="/points">积分</RouterLink>
    </nav>

    <template #actions>
      <AccountMenu />
    </template>
  </SosAppbar>
</template>
