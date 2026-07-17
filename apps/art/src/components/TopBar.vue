<script setup>
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Search, X } from 'lucide-vue-next'
import { SosAppbar } from '@haruhi/ui'
import { AccountMenu, AccountAvatarLink, useSession } from '@haruhi/auth-ui'
import logoUrl from '../assets/logo.webp'

const route = useRoute()
const router = useRouter()
const session = useSession('/api')
const searchText = ref(route.name === 'gallery-search' ? String(route.query.q || '') : '')
const mobileSearchOpen = ref(false)
const mobileSearchInput = ref(null)

// 导航项（含 PR 新增的公会/公告）。终端登录后才显示。
const navItems = [
  { path: '/gallery', label: '首页' },
  { path: '/upload', label: '投稿' },
  { path: '/exchange', label: '公会' },
  { path: '/announcements', label: '公告' },
]
const showTerminal = computed(() => !!session.state.user)

const isActive = (path) => {
  if (path === '/gallery' && (route.path.startsWith('/profile/') || route.path.startsWith('/gallery'))) return true
  if (path === '/exchange' && route.path === '/points') return true
  return route.path === path
}

function submitSearch(closeMenu) {
  closeMenu?.()
  mobileSearchOpen.value = false
  const q = searchText.value.trim()
  if (!q) {
    router.push({ name: 'gallery' })
    return
  }
  router.push({ name: 'gallery-search', query: { q } })
}

function toggleMobileSearch() {
  mobileSearchOpen.value = !mobileSearchOpen.value
  if (mobileSearchOpen.value) {
    nextTick(() => mobileSearchInput.value?.focus())
  }
}

function closeMobileSearch() {
  mobileSearchOpen.value = false
}

watch([() => route.name, () => route.query.q], () => {
  searchText.value = route.name === 'gallery-search' ? String(route.query.q || '') : ''
  mobileSearchOpen.value = false
})
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
      >
        {{ item.label }}
      </RouterLink>
      <RouterLink
        v-if="showTerminal"
        class="sos-navlink"
        :class="{ 'sos-navlink--active': isActive('/terminal') }"
        to="/terminal"
        :aria-current="isActive('/terminal') ? 'page' : undefined"
        data-sfx="click"
      >
        终端
      </RouterLink>
    </nav>

    <template #actions="{ close }">
      <form class="header-search" role="search" @submit.prevent="submitSearch(close)">
        <Search class="header-search__icon" :size="17" :stroke-width="2.2" aria-hidden="true" />
        <input
          v-model="searchText"
          type="search"
          placeholder="搜索画廊作品"
          aria-label="搜索画廊作品"
        />
        <button
          v-if="searchText"
          class="header-search__clear"
          type="button"
          aria-label="清空搜索"
          title="清空搜索"
          @click="searchText = ''"
        >
          <X :size="15" :stroke-width="2.3" aria-hidden="true" />
        </button>
        <button class="header-search__submit" type="submit" aria-label="搜索" title="搜索">
          <Search :size="16" :stroke-width="2.3" aria-hidden="true" />
        </button>
      </form>
      <AccountMenu login-path="/login" profile-path="/account" settings-path="/account/settings" />
    </template>

    <!-- 移动端：头像快捷入口提到汉堡左侧，点头像直达个人中心 -->
    <template #mobile-lead>
      <AccountAvatarLink profile-path="/account" />
      <button
        class="mobile-search-trigger"
        type="button"
        :aria-expanded="mobileSearchOpen"
        aria-label="搜索画廊作品"
        title="搜索"
        @click="toggleMobileSearch"
      >
        <Search :size="19" :stroke-width="2.3" aria-hidden="true" />
      </button>
    </template>
  </SosAppbar>

  <div v-if="mobileSearchOpen" class="mobile-search-scrim" @click="closeMobileSearch"></div>
  <Transition name="mobile-search">
    <form
      v-if="mobileSearchOpen"
      class="mobile-search-popover"
      role="search"
      @submit.prevent="submitSearch()"
      @keydown.esc.prevent="closeMobileSearch"
    >
      <Search class="mobile-search-popover__icon" :size="18" :stroke-width="2.2" aria-hidden="true" />
      <input
        ref="mobileSearchInput"
        v-model="searchText"
        type="search"
        placeholder="搜索画廊作品"
        aria-label="搜索画廊作品"
      />
      <button
        v-if="searchText"
        class="mobile-search-popover__clear"
        type="button"
        aria-label="清空搜索"
        @click="searchText = ''"
      >
        <X :size="16" :stroke-width="2.3" aria-hidden="true" />
      </button>
      <button class="mobile-search-popover__submit" type="submit" aria-label="搜索">
        <Search :size="18" :stroke-width="2.3" aria-hidden="true" />
      </button>
    </form>
  </Transition>
</template>

<style scoped>
.header-search {
  display: flex;
  align-items: center;
  width: clamp(210px, 23vw, 340px);
  height: 38px;
  padding-left: 11px;
  overflow: hidden;
  color: var(--sos-text-tertiary);
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid rgba(95, 215, 226, 0.68);
  border-radius: 999px;
  box-shadow: 0 3px 12px rgba(36, 111, 126, 0.05);
}

.header-search:focus-within {
  color: var(--sos-text-secondary);
  background: rgba(255, 255, 255, 0.94);
  border-color: rgb(59, 190, 204);
  box-shadow: 0 0 0 3px rgba(59, 190, 204, 0.12);
}

.header-search__icon { flex: 0 0 auto; }

.header-search input {
  min-width: 0;
  flex: 1;
  height: 100%;
  padding: 0 8px;
  color: var(--sos-text-primary);
  font: inherit;
  font-size: 13px;
  background: transparent;
  border: 0;
  outline: none;
}

.header-search input::-webkit-search-cancel-button { display: none; }

.header-search__clear,
.header-search__submit {
  display: inline-grid;
  flex: 0 0 auto;
  width: 32px;
  height: 32px;
  place-items: center;
  padding: 0;
  color: var(--sos-text-secondary);
  cursor: pointer;
  background: transparent;
  border: 0;
  border-radius: 50%;
}

.header-search__submit {
  width: 36px;
  height: 36px;
  margin-right: 1px;
  color: white;
  background: var(--sos-accent);
}

.header-search__clear:hover { color: var(--sos-text-primary); }
.header-search__submit:hover { filter: brightness(0.94); }

.mobile-search-trigger,
.mobile-search-popover,
.mobile-search-scrim { display: none; }

@media (max-width: 1100px) {
  .header-search { width: 210px; }
}

@media (max-width: 768px) {
  .header-search { display: none; }

  :deep(.sos-appbar__burger) { border-radius: 50%; }

  .mobile-search-trigger {
    display: inline-grid;
    flex: 0 0 auto;
    width: var(--sos-control-md);
    height: var(--sos-control-md);
    margin-left: 8px;
    place-items: center;
    padding: 0;
    color: var(--sos-text-primary);
    cursor: pointer;
    background: var(--sos-bg-surface);
    border: 1px solid var(--sos-border-default);
    border-radius: 50%;
  }

  .mobile-search-trigger[aria-expanded='true'] {
    color: white;
    background: var(--sos-accent);
    border-color: var(--sos-accent);
  }

  .mobile-search-scrim {
    position: fixed;
    inset: var(--sos-header-height) 0 0;
    z-index: 890;
    display: block;
    background: rgba(18, 35, 40, 0.18);
  }

  .mobile-search-popover {
    position: fixed;
    top: calc(var(--sos-header-height) + 10px);
    right: 14px;
    left: 14px;
    z-index: 900;
    display: flex;
    align-items: center;
    height: 46px;
    padding-left: 13px;
    overflow: hidden;
    color: var(--sos-text-tertiary);
    background: rgba(255, 255, 255, 0.96);
    border: 1px solid rgb(95, 215, 226);
    border-radius: 999px;
    box-shadow: 0 14px 34px rgba(30, 74, 82, 0.2);
  }

  .mobile-search-popover input {
    min-width: 0;
    flex: 1;
    height: 100%;
    padding: 0 9px;
    color: var(--sos-text-primary);
    font: inherit;
    font-size: 14px;
    background: transparent;
    border: 0;
    outline: none;
  }

  .mobile-search-popover input::-webkit-search-cancel-button { display: none; }

  .mobile-search-popover__clear,
  .mobile-search-popover__submit {
    display: inline-grid;
    flex: 0 0 auto;
    width: 36px;
    height: 36px;
    place-items: center;
    padding: 0;
    color: var(--sos-text-secondary);
    cursor: pointer;
    background: transparent;
    border: 0;
    border-radius: 50%;
  }

  .mobile-search-popover__submit {
    width: 42px;
    height: 42px;
    margin-right: 1px;
    color: white;
    background: var(--sos-accent);
  }

  .mobile-search-enter-active,
  .mobile-search-leave-active { transition: opacity 0.18s ease, transform 0.18s ease; }
  .mobile-search-enter-from,
  .mobile-search-leave-to { opacity: 0; transform: translateY(-6px); }

}

@media (max-width: 360px) {
  :deep(.sos-appbar__inner) { gap: 8px; }
  .sos-brand-lockup__text small { display: none; }
  .mobile-search-trigger { margin-left: 0; }
}

@media (prefers-reduced-motion: reduce) {
  .mobile-search-enter-active,
  .mobile-search-leave-active { transition: none; }
}
</style>
