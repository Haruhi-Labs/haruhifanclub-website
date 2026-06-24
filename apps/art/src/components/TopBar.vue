<script setup>
import { useRoute } from 'vue-router'
import { AccountMenu } from '@haruhi/auth-ui'
import logoUrl from '../assets/logo.webp'

const route = useRoute()
const isActive = (path) => {
  if (path === '/' && route.path.startsWith('/creator')) return true
  return route.path === path
}
</script>

<template>
  <header class="sos-appbar art-appbar">
    <div class="sos-appbar__inner">
      <RouterLink to="/" class="sos-brand-lockup">
        <span class="sos-brand-lockup__mark">
          <img :src="logoUrl" alt="" />
        </span>
        <span class="sos-brand-lockup__text">
          <strong>绘画部画廊</strong>
          <small>凉宫春日应援团 · 美术</small>
        </span>
      </RouterLink>

      <div class="art-appbar__right">
        <nav class="sos-navlinks">
          <RouterLink class="sos-navlink" :class="{ 'sos-navlink--active': isActive('/') }" to="/">画廊</RouterLink>
          <RouterLink class="sos-navlink" :class="{ 'sos-navlink--active': isActive('/upload') }" to="/upload">投稿</RouterLink>
          <RouterLink class="sos-navlink" :class="{ 'sos-navlink--active': isActive('/points') }" to="/points">积分</RouterLink>
        </nav>
        <AccountMenu />
      </div>
    </div>
  </header>
</template>

<style scoped>
.art-appbar__right {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}
</style>
