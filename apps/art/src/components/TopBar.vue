<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { AccountMenu, useSession } from '@haruhi/auth-ui'
import logoUrl from '../assets/logo.webp'

const route = useRoute()
const session = useSession('/api')

const navItems = [
  { path: '/gallery', label: '画廊' },
  { path: '/upload', label: '投稿' },
  { path: '/exchange', label: '公会' },
]
const announcementPath = '/announcements'
const showTerminal = computed(() => !!session.state.user)

const isActive = (path) => {
  if (path === '/gallery' && route.path.startsWith('/profile/')) return true
  if (path === '/exchange' && route.path === '/points') return true
  return route.path === path
}
const linkClass = (path) => ['navlink', isActive(path) ? 'on' : ''].join(' ')
</script>

<template>
  <header class="topbar__inner sos-appbar art-appbar">
    <RouterLink class="brand sos-brand-lockup" to="/" data-sfx="click">
      <span class="brand__mark sos-brand-lockup__mark">
        <img :src="logoUrl" alt="" />
      </span>
      <span class="brand__text sos-brand-lockup__text">
        <strong class="brand__title">应援团绘画部 · 画廊</strong>
        <small class="brand__sub">一起来体验分享的快乐吧！</small>
      </span>
    </RouterLink>

    <div class="topbar-actions">
      <RouterLink
        :class="['notice-orb', isActive(announcementPath) ? 'on' : '']"
        :to="announcementPath"
        aria-label="公告"
        title="公告"
        data-sfx="click"
      >
        <span class="notice-orb__sign" aria-hidden="true">
          <span class="notice-orb__pin"></span>
          <span class="notice-orb__line long"></span>
          <span class="notice-orb__line short"></span>
        </span>
      </RouterLink>

      <nav class="nav" aria-label="画廊功能导航">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :class="linkClass(item.path)"
          :to="item.path"
          data-sfx="click"
        >
          {{ item.label }}
        </RouterLink>
        <RouterLink
          v-if="showTerminal"
          :class="linkClass('/terminal')"
          to="/terminal"
          data-sfx="click"
        >
          终端
        </RouterLink>
      </nav>

      <div class="account-entry">
        <AccountMenu login-path="/login" profile-path="/account" settings-path="/account/settings" />
      </div>
    </div>
  </header>
</template>

<style scoped>
.topbar__inner {
  position: fixed;
  inset: 0 0 auto;
  z-index: 999;
  display: flex;
  height: 72px;
  align-items: center;
  justify-content: flex-start;
  gap: 18px;
  padding: 0 24px;
  background: rgba(255, 255, 255, 0.28);
  border: 0;
  box-shadow: none;
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
}

.brand {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 12px;
  min-width: 0;
  color: #fff;
  text-decoration: none;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.6);
}

.brand__mark {
  display: block;
  width: 44px;
  height: 44px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.25);
  border: 1px solid rgba(255, 255, 255, 0.6);
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.brand__mark img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.brand__text {
  display: flex;
  min-width: 0;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
}

.brand__title {
  overflow: hidden;
  font-size: 15px;
  font-weight: 950;
  letter-spacing: 0;
  line-height: 1.1;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.brand__sub {
  margin-top: 2px;
  overflow: hidden;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0;
  opacity: 0.95;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.topbar-actions {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  margin-left: auto;
}

.notice-orb {
  position: relative;
  display: inline-flex;
  flex: 0 0 auto;
  width: 46px;
  height: 46px;
  align-items: center;
  justify-content: center;
  color: #073b4c;
  text-decoration: none;
  background:
    radial-gradient(circle at 34% 26%, rgba(255, 255, 255, 0.74), rgba(255, 255, 255, 0.18) 56%, rgba(255, 255, 255, 0.08)),
    linear-gradient(135deg, rgba(255, 240, 166, 0.54), rgba(103, 232, 249, 0.2));
  border: 1px solid rgba(255, 255, 255, 0.5);
  border-radius: 999px;
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.15), inset 0 1px 0 rgba(255, 255, 255, 0.58);
  transition: transform 0.24s ease, border-color 0.24s ease, box-shadow 0.24s ease, background 0.24s ease;
}

.notice-orb:hover,
.notice-orb.on {
  transform: translateY(-1px) scale(1.04);
  border-color: rgba(255, 255, 255, 0.78);
  box-shadow: 0 14px 34px rgba(0, 0, 0, 0.18), 0 0 0 4px rgba(254, 240, 138, 0.16);
}

.notice-orb.on {
  background:
    radial-gradient(circle at 36% 24%, rgba(255, 255, 255, 0.9), transparent 34%),
    linear-gradient(135deg, #fef08a 0%, #fb7185 48%, #67e8f9 100%);
}

.notice-orb__sign {
  position: relative;
  display: block;
  width: 25px;
  height: 21px;
  background: rgba(255, 255, 255, 0.52);
  border: 2px solid currentColor;
  border-radius: 7px;
  box-shadow: inset 0 -3px 0 rgba(7, 59, 76, 0.08);
}

.notice-orb__sign::before,
.notice-orb__sign::after {
  position: absolute;
  top: -7px;
  width: 2px;
  height: 7px;
  background: currentColor;
  border-radius: 99px;
  content: "";
}

.notice-orb__sign::before {
  left: 6px;
}

.notice-orb__sign::after {
  right: 6px;
}

.notice-orb__pin {
  position: absolute;
  top: -3px;
  left: 50%;
  width: 6px;
  height: 6px;
  background: #fb7185;
  border-radius: 50%;
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.74);
  transform: translateX(-50%);
}

.notice-orb__line {
  position: absolute;
  left: 6px;
  height: 2px;
  background: currentColor;
  border-radius: 99px;
  opacity: 0.78;
}

.notice-orb__line.long {
  top: 8px;
  right: 6px;
}

.notice-orb__line.short {
  top: 13px;
  right: 11px;
}

.nav {
  display: flex;
  flex: 0 1 auto;
  max-width: min(720px, 62vw);
  min-width: 0;
  align-items: center;
  gap: 4px;
  overflow-x: auto;
  padding: 5px;
  background: rgba(255, 255, 255, 0.28);
  border: 1px solid rgba(255, 255, 255, 0.34);
  border-radius: 999px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.14), inset 0 1px 0 rgba(255, 255, 255, 0.38);
  scrollbar-width: none;
}

.nav::-webkit-scrollbar {
  display: none;
}

.navlink {
  display: inline-flex;
  min-width: 58px;
  align-items: center;
  justify-content: center;
  padding: 8px 13px;
  color: rgba(13, 63, 74, 0.88);
  font-size: 13px;
  font-weight: 900;
  letter-spacing: 0;
  text-decoration: none;
  white-space: nowrap;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 999px;
  transition: all 0.24s ease;
}

.navlink:hover {
  color: #063f4f;
  background: rgba(255, 255, 255, 0.62);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
  transform: translateY(-1px);
}

.navlink.on {
  z-index: 5;
  color: #073b4c;
  background: linear-gradient(135deg, #fef08a 0%, #86efac 50%, #67e8f9 100%);
  border-color: rgba(255, 255, 255, 0.72);
  box-shadow: 0 8px 18px rgba(20, 140, 160, 0.22), inset 0 1px 0 rgba(255, 255, 255, 0.72);
}

.account-entry {
  flex: 0 0 auto;
  color: #073b4c;
}

:global(html.art-lights-out) .topbar__inner,
:global(html.art-home-route.art-home-lights-out) .topbar__inner {
  background:
    linear-gradient(180deg, rgba(3, 10, 24, 0.78), rgba(3, 10, 24, 0.38)),
    radial-gradient(ellipse at 22% 0%, rgba(116, 231, 255, 0.12), transparent 42%);
  border-bottom: 1px solid rgba(126, 227, 255, 0.16);
}

:global(html.art-lights-out) .brand,
:global(html.art-home-route.art-home-lights-out) .brand {
  color: rgba(241, 248, 255, 0.96);
}

:global(html.art-lights-out) .nav,
:global(html.art-home-route.art-home-lights-out) .nav {
  background: rgba(5, 13, 28, 0.58);
  border-color: rgba(126, 227, 255, 0.2);
}

:global(html.art-lights-out) .navlink,
:global(html.art-home-route.art-home-lights-out) .navlink {
  color: rgba(220, 238, 255, 0.88);
}

:global(html.art-lights-out) .notice-orb,
:global(html.art-home-route.art-home-lights-out) .notice-orb {
  color: #bae6fd;
  background:
    radial-gradient(circle at 34% 24%, rgba(186, 230, 253, 0.28), rgba(15, 23, 42, 0.4) 58%),
    linear-gradient(135deg, rgba(15, 23, 42, 0.72), rgba(30, 27, 75, 0.62));
  border-color: rgba(125, 211, 252, 0.28);
}

@media (max-width: 768px) {
  .topbar__inner {
    height: 76px;
    gap: 8px;
    padding: 0 12px;
  }

  .brand {
    flex: 1;
    gap: 8px;
  }

  .brand__mark {
    width: 36px;
    height: 36px;
  }

  .brand__title {
    font-size: 14px;
  }

  .brand__sub {
    font-size: 10px;
  }

  .notice-orb {
    width: 42px;
    height: 42px;
  }

  .nav {
    max-width: 44vw;
    gap: 3px;
    padding: 4px;
  }

  .navlink {
    min-width: 52px;
    padding: 7px 11px;
  }
}

@media (max-width: 560px) {
  .brand__text {
    display: none;
  }

  .nav {
    max-width: calc(100vw - 178px);
  }
}
</style>
