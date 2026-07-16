<script setup>
import { computed, onMounted } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { AccountAvatarLink, AccountMenu, haruhiLogoUrl } from '@haruhi/auth-ui'
import { SosAppbar, SosToastRegion } from '@haruhi/ui'
import { session } from '@/api'
import SiteFooter from '@/components/SiteFooter.vue'

onMounted(() => session.ensureReady())

const route = useRoute()
const navItems = [
  { path: '/', label: '支部目录' },
  { path: '/timeline', label: '活动时间线' },
  { path: '/events', label: '地方活动' },
]

function isActive(path) {
  const section = route.meta.section
  if (path === '/admin') return route.path.startsWith('/admin') || route.path.includes('/manage')
  if (route.path.includes('/manage')) return false
  if (path === '/timeline') return route.path.startsWith('/timeline') || section === 'timeline'
  if (path === '/events') {
    return (
      route.path.startsWith('/events') ||
      section === 'events' ||
      route.meta.contentType === 'events'
    )
  }
  if (route.meta.contentType === 'events') return false
  return (
    route.path === '/' ||
    (route.path.startsWith('/branches/') && !['timeline', 'events'].includes(section))
  )
}

const canManage = computed(() => {
  const user = session.state.user
  return (
    !!user &&
    (user.isSuperAdmin ||
      (user.capabilities || []).some((g) => g.scopeId === 'chapter' || g.scopeType === 'branch'))
  )
})
</script>

<template>
  <div class="chapter-app sos-scope" data-sos-site="chapter">
    <SosAppbar class="chapter-appbar">
      <template #brand>
        <RouterLink to="/" class="sos-brand-lockup" aria-label="返回地方支部首页">
          <span class="sos-brand-lockup__mark"><img :src="haruhiLogoUrl" alt="" /></span>
          <span class="sos-brand-lockup__text">
            <strong>地方支部</strong>
            <small>凉宫春日应援团 · Chapter Network</small>
          </span>
        </RouterLink>
      </template>
      <nav class="sos-navlinks" aria-label="地方支部功能导航">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="sos-navlink"
          :class="{ 'sos-navlink--active': isActive(item.path) }"
          :aria-current="isActive(item.path) ? 'page' : undefined"
        >
          {{ item.label }}
        </RouterLink>
        <RouterLink
          v-if="canManage"
          to="/admin/branches"
          class="sos-navlink"
          :class="{ 'sos-navlink--active': isActive('/admin') }"
          :aria-current="isActive('/admin') ? 'page' : undefined"
        >
          管理
        </RouterLink>
      </nav>
      <template #actions>
        <AccountMenu
          login-path="/login"
          profile-path="/account"
          settings-path="/account/settings"
          home="/"
        />
      </template>
      <template #mobile-lead><AccountAvatarLink profile-path="/account" /></template>
    </SosAppbar>
    <main class="chapter-main"><RouterView /></main>
    <SiteFooter />
    <SosToastRegion />
  </div>
</template>
