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

const linkClass = (path) => [
  'navlink',
  'sos-navlink',
  isActive(path) ? 'on sos-navlink--active' : '',
].filter(Boolean).join(' ')
</script>

<template>
  <header class="topbar__inner sos-appbar art-appbar" role="banner">
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
        :class="['notice-orb', isActive(announcementPath) ? 'on sos-navlink--active' : '']"
        :to="announcementPath"
        :aria-current="isActive(announcementPath) ? 'page' : undefined"
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

      <nav class="nav sos-navlinks" aria-label="画廊功能导航">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :class="linkClass(item.path)"
          :to="item.path"
          :aria-current="isActive(item.path) ? 'page' : undefined"
          data-sfx="click"
        >
          {{ item.label }}
        </RouterLink>
        <RouterLink
          v-if="showTerminal"
          :class="linkClass('/terminal')"
          to="/terminal"
          :aria-current="isActive('/terminal') ? 'page' : undefined"
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
  z-index: var(--sos-z-sticky);
  display: flex;
  min-height: 4.5rem;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-5);
  padding-block: var(--sos-space-3);
  padding-inline: var(--sos-page-gutter);
  background: color-mix(in srgb, var(--sos-bg-page) 84%, transparent);
  border: 0;
  border-bottom: 1px solid var(--sos-border-subtle);
  box-shadow: var(--sos-shadow-hairline);
  backdrop-filter: saturate(1.35) blur(14px);
  -webkit-backdrop-filter: saturate(1.35) blur(14px);
}

.brand {
  flex: 0 1 auto;
  min-width: 0;
  max-width: min(32rem, 42vw);
  color: var(--sos-text-primary);
  text-shadow: none;
  transition:
    color var(--sos-duration-base) var(--sos-ease-standard),
    transform var(--sos-duration-fast) var(--sos-ease-out);
}

.brand:hover {
  color: var(--sos-link);
  transform: translateY(-1px);
}

.brand__mark {
  border: 1px solid var(--sos-border-subtle);
  background: color-mix(in srgb, var(--sos-bg-surface) 84%, transparent);
  box-shadow: var(--sos-shadow-xs);
  transition:
    border-color var(--sos-duration-base) var(--sos-ease-standard),
    box-shadow var(--sos-duration-base) var(--sos-ease-standard);
}

.brand:hover .brand__mark {
  border-color: color-mix(in srgb, var(--sos-accent) 42%, var(--sos-border-default));
  box-shadow: var(--sos-shadow-sm);
}

.brand__text {
  min-width: 0;
  overflow: hidden;
}

.brand__title,
.brand__sub {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.brand__title {
  color: currentColor;
  font-weight: var(--sos-weight-heavy);
}

.brand__sub {
  color: var(--sos-text-secondary);
}

.topbar-actions {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: flex-end;
  gap: var(--sos-space-3);
  margin-left: auto;
}

.notice-orb {
  position: relative;
  display: inline-grid;
  flex: 0 0 auto;
  width: var(--sos-control-lg);
  height: var(--sos-control-lg);
  place-items: center;
  color: var(--sos-link);
  text-decoration: none;
  background: color-mix(in srgb, var(--sos-bg-surface) 82%, transparent);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-full);
  box-shadow: var(--sos-shadow-xs);
  backdrop-filter: saturate(1.2) blur(10px);
  -webkit-backdrop-filter: saturate(1.2) blur(10px);
  transition:
    color var(--sos-duration-base) var(--sos-ease-standard),
    background-color var(--sos-duration-base) var(--sos-ease-standard),
    border-color var(--sos-duration-base) var(--sos-ease-standard),
    box-shadow var(--sos-duration-base) var(--sos-ease-standard),
    transform var(--sos-duration-fast) var(--sos-ease-out);
}

.notice-orb:hover,
.notice-orb.on {
  color: var(--sos-link-hover);
  background: color-mix(in srgb, var(--sos-accent) 16%, var(--sos-bg-surface));
  border-color: color-mix(in srgb, var(--sos-accent) 44%, var(--sos-border-default));
  box-shadow: var(--sos-shadow-sm);
  transform: translateY(-1px);
}

.notice-orb.on {
  color: var(--sos-accent-contrast);
  background: var(--sos-accent);
}

.notice-orb__sign {
  position: relative;
  display: block;
  width: 1.55rem;
  height: 1.3rem;
  background: color-mix(in srgb, var(--sos-bg-surface) 38%, transparent);
  border: 2px solid currentColor;
  border-radius: var(--sos-radius-sm);
}

.notice-orb__sign::before,
.notice-orb__sign::after {
  position: absolute;
  top: -0.45rem;
  width: 2px;
  height: 0.45rem;
  background: currentColor;
  border-radius: var(--sos-radius-full);
  content: "";
}

.notice-orb__sign::before {
  left: 0.38rem;
}

.notice-orb__sign::after {
  right: 0.38rem;
}

.notice-orb__pin {
  position: absolute;
  top: -0.25rem;
  left: 50%;
  width: 0.42rem;
  height: 0.42rem;
  background: var(--sos-signal);
  border-radius: var(--sos-radius-full);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--sos-bg-surface) 86%, transparent);
  transform: translateX(-50%);
}

.notice-orb__line {
  position: absolute;
  left: 0.36rem;
  height: 2px;
  background: currentColor;
  border-radius: var(--sos-radius-full);
  opacity: 0.72;
}

.notice-orb__line.long {
  top: 0.48rem;
  right: 0.36rem;
}

.notice-orb__line.short {
  top: 0.78rem;
  right: 0.68rem;
}

.nav {
  flex: 0 1 auto;
  max-width: min(46rem, 58vw);
  min-width: 0;
  overflow-x: auto;
  padding: var(--sos-space-1);
  background: color-mix(in srgb, var(--sos-bg-surface) 76%, transparent);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-full);
  box-shadow: var(--sos-shadow-xs);
  scrollbar-width: none;
  backdrop-filter: saturate(1.2) blur(12px);
  -webkit-backdrop-filter: saturate(1.2) blur(12px);
}

.nav::-webkit-scrollbar {
  display: none;
}

.navlink {
  min-width: 3.65rem;
  min-height: var(--sos-control-sm);
  justify-content: center;
  border: 1px solid transparent;
  border-radius: var(--sos-radius-full);
  color: var(--sos-text-secondary);
  font-weight: var(--sos-weight-bold);
  white-space: nowrap;
  transition:
    color var(--sos-duration-base) var(--sos-ease-standard),
    background-color var(--sos-duration-base) var(--sos-ease-standard),
    border-color var(--sos-duration-base) var(--sos-ease-standard),
    box-shadow var(--sos-duration-base) var(--sos-ease-standard),
    transform var(--sos-duration-fast) var(--sos-ease-out);
}

.navlink:hover {
  color: var(--sos-text-primary);
  background: var(--sos-bg-subtle);
  border-color: var(--sos-border-subtle);
  box-shadow: var(--sos-shadow-xs);
  transform: translateY(-1px);
}

.navlink.on,
.navlink.sos-navlink--active {
  color: var(--sos-link);
  background: color-mix(in srgb, var(--sos-accent) 16%, transparent);
  border-color: color-mix(in srgb, var(--sos-accent) 34%, var(--sos-border-default));
  box-shadow: var(--sos-shadow-hairline);
}

.account-entry {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  color: var(--sos-text-primary);
}

.account-entry :deep(.hauth-trigger),
.account-entry :deep(.sos-button.sos-button--secondary) {
  min-height: var(--sos-control-sm);
  background: color-mix(in srgb, var(--sos-bg-surface) 82%, transparent);
  border-color: var(--sos-border-subtle);
  box-shadow: var(--sos-shadow-xs);
}

.account-entry :deep(.hauth-trigger:hover),
.account-entry :deep(.sos-button.sos-button--secondary:hover) {
  border-color: color-mix(in srgb, var(--sos-accent) 40%, var(--sos-border-default));
  box-shadow: var(--sos-shadow-sm);
}

:global(html.art-home-route:not(.art-home-lights-out)) .topbar__inner,
:global(html.art-home-route.art-home-lights-out) .topbar__inner,
:global(html.art-lights-out) .topbar__inner {
  background: color-mix(in srgb, var(--sos-bg-page) 78%, transparent);
  border-bottom-color: var(--sos-border-subtle);
  box-shadow: var(--sos-shadow-hairline);
}

:global(html.art-home-route:not(.art-home-lights-out)) .brand,
:global(html.art-home-route.art-home-lights-out) .brand,
:global(html.art-lights-out) .brand {
  color: var(--sos-text-primary);
  text-shadow: none;
}

:global(html.art-home-route:not(.art-home-lights-out)) .brand__mark,
:global(html.art-home-route.art-home-lights-out) .brand__mark,
:global(html.art-lights-out) .brand__mark {
  background: color-mix(in srgb, var(--sos-bg-surface) 82%, transparent);
  border-color: var(--sos-border-subtle);
  box-shadow: var(--sos-shadow-xs);
}

:global(html.art-home-route:not(.art-home-lights-out)) .nav,
:global(html.art-home-route.art-home-lights-out) .nav,
:global(html.art-lights-out) .nav {
  background: color-mix(in srgb, var(--sos-bg-surface) 72%, transparent);
  border-color: var(--sos-border-subtle);
  box-shadow: var(--sos-shadow-xs);
}

:global(html.art-home-route:not(.art-home-lights-out)) .navlink,
:global(html.art-home-route.art-home-lights-out) .navlink,
:global(html.art-lights-out) .navlink {
  color: var(--sos-text-secondary);
}

:global(html.art-home-route:not(.art-home-lights-out)) .navlink:hover,
:global(html.art-home-route.art-home-lights-out) .navlink:hover,
:global(html.art-lights-out) .navlink:hover {
  color: var(--sos-text-primary);
  background: var(--sos-bg-subtle);
  box-shadow: var(--sos-shadow-xs);
}

:global(html.art-home-route:not(.art-home-lights-out)) .navlink.on,
:global(html.art-home-route.art-home-lights-out) .navlink.on,
:global(html.art-lights-out) .navlink.on {
  color: var(--sos-link);
  background: color-mix(in srgb, var(--sos-accent) 16%, transparent);
  border-color: color-mix(in srgb, var(--sos-accent) 34%, var(--sos-border-default));
  box-shadow: var(--sos-shadow-hairline);
}

@media (prefers-reduced-motion: reduce) {
  .brand,
  .brand__mark,
  .notice-orb,
  .navlink {
    transition-duration: 1ms;
  }

  .brand:hover,
  .notice-orb:hover,
  .notice-orb.on,
  .navlink:hover {
    transform: none;
  }
}

@media (max-width: 768px) {
  .topbar__inner {
    min-height: 4.75rem;
    gap: var(--sos-space-2);
    padding-inline: var(--sos-space-3);
  }

  .brand {
    flex: 1 1 auto;
    max-width: none;
    gap: var(--sos-space-2);
  }

  .brand__mark {
    width: 2.25rem;
    height: 2.25rem;
  }

  .brand__title {
    font-size: var(--sos-text-sm);
  }

  .brand__sub {
    font-size: var(--sos-text-2xs);
  }

  .topbar-actions {
    gap: var(--sos-space-2);
  }

  .notice-orb {
    width: var(--sos-control-md);
    height: var(--sos-control-md);
  }

  .notice-orb__sign {
    transform: scale(0.9);
  }

  .nav {
    max-width: 44vw;
  }

  .navlink {
    min-width: 3.25rem;
    padding-inline: var(--sos-space-3);
  }
}

@media (max-width: 560px) {
  .brand {
    flex: 0 0 auto;
  }

  .brand__text {
    display: none;
  }

  .nav {
    max-width: calc(100vw - 11.75rem);
  }

  .account-entry :deep(.hauth-trigger__name) {
    display: none;
  }
}
</style>
