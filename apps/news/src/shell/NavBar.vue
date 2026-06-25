<script setup>
import { ref, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useMainStore } from '@/stores/main'
import { AccountMenu } from '@haruhi/auth-ui'

defineProps({
  // 博客详情页等深色 hero 之上时，页头转透明叠加态
  overlay: { type: Boolean, default: false },
})

const store = useMainStore()
const router = useRouter()
const isMobileMenuOpen = ref(false)
const logoUrl = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

const goHome = () => {
  store.searchQuery = ''
  router.push('/')
}
const toggleMobileMenu = () => {
  isMobileMenuOpen.value = !isMobileMenuOpen.value
  document.body.style.overflow = isMobileMenuOpen.value ? 'hidden' : ''
}
const closeMobile = () => {
  isMobileMenuOpen.value = false
  document.body.style.overflow = ''
}

// 卸载兜底：菜单打开时若 NavBar 被移除（路由切换/条件渲染），恢复 body 滚动
onBeforeUnmount(() => {
  document.body.style.overflow = ''
})

// 主导航（站内 + 跨站）。exam 是独立 app，必须用原生 a 整页跳转。
const navLinks = [
  { label: '活动中心', to: '/activity' },
  { label: '奖品兑换', to: '/store' },
  { label: '我要投稿', to: '/submit' },
  { label: '团员手册', to: '/handbook' },
]
</script>

<template>
  <header class="sos-appbar news-appbar" :class="{ 'news-appbar--overlay': overlay }">
    <div class="sos-appbar__inner">
      <button class="sos-brand-lockup news-brand" type="button" @click="goHome" aria-label="返回春日团报首页">
        <span class="sos-brand-lockup__mark"><img :src="logoUrl" alt="" /></span>
        <span class="sos-brand-lockup__text">
          <strong>春日团报</strong>
          <small>凉宫春日应援团 · 编辑部</small>
        </span>
      </button>

      <nav class="sos-navlinks news-appbar__nav">
        <RouterLink v-for="n in navLinks" :key="n.to" :to="n.to" class="sos-navlink">{{ n.label }}</RouterLink>
        <RouterLink :to="{ name: 'quiz' }" target="_blank" rel="noopener noreferrer" class="sos-navlink">凉宫入坑测试</RouterLink>
        <a href="/exam/" target="_blank" rel="noopener noreferrer" class="sos-navlink">SOS团期末考试</a>
      </nav>

      <div class="news-appbar__right">
        <!-- 搜索触发：采用统一搜索规范 .sos-search 外观，点击打开全屏搜索 -->
        <button class="sos-search news-appbar__search" type="button" aria-label="搜索团报" @click="store.toggleSearch">
          <span class="sos-search__icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="7" /><path d="M21 21l-4.3-4.3" />
            </svg>
          </span>
          <span class="news-appbar__search-label">搜索团报</span>
        </button>

        <AccountMenu />

        <button class="news-appbar__menu" type="button" aria-label="菜单" @click="toggleMobileMenu">
          <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
            <path d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 移动端菜单 -->
    <Transition name="fade">
      <div v-if="isMobileMenuOpen" class="news-mobile">
        <button class="news-mobile__close" type="button" aria-label="关闭菜单" @click="closeMobile">
          <svg viewBox="0 0 24 24" width="26" height="26" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
            <path d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
        <nav class="news-mobile__nav">
          <RouterLink to="/" class="news-mobile__link" @click="closeMobile">首页</RouterLink>
          <RouterLink v-for="n in navLinks" :key="n.to" :to="n.to" class="news-mobile__link" @click="closeMobile">{{ n.label }}</RouterLink>
          <a href="/exam/" target="_blank" rel="noopener noreferrer" class="news-mobile__link" @click="closeMobile">SOS团期末考试</a>
          <RouterLink :to="{ name: 'quiz' }" target="_blank" rel="noopener noreferrer" class="news-mobile__link" @click="closeMobile">凉宫入坑测试</RouterLink>
          <RouterLink to="/admin" class="news-mobile__link" @click="closeMobile">管理后台</RouterLink>
        </nav>
        <div class="news-mobile__foot">- 凉宫春日应援团 -</div>
      </div>
    </Transition>
  </header>
</template>

<style scoped>
/* 在统一 .sos-appbar 基础上补 news 特有部分 */
.news-brand {
  border: 0;
  background: transparent;
  cursor: pointer;
}

.news-appbar__nav {
  flex-wrap: wrap;
}
@media (max-width: 1023px) {
  .news-appbar__nav {
    display: none;
  }
}

.news-appbar__right {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}

/* 搜索触发器：复用 .sos-search 外观，桌面显示占位文案 */
.news-appbar__search {
  width: auto;
  cursor: pointer;
  color: var(--sos-text-tertiary);
}
.news-appbar__search:hover {
  border-color: var(--sos-text-secondary);
}
.news-appbar__search-label {
  font-size: var(--sos-text-sm);
}
@media (max-width: 767px) {
  .news-appbar__search-label {
    display: none;
  }
}

.news-appbar__menu {
  display: none;
  place-items: center;
  width: 2.5rem;
  height: 2.5rem;
  border: 0;
  border-radius: var(--sos-radius-sm);
  background: transparent;
  color: var(--sos-text-primary);
  cursor: pointer;
}
@media (max-width: 1023px) {
  .news-appbar__menu {
    display: inline-grid;
  }
}

/* 博客详情等深色 hero 上：页头透明浮层 + 浅色文字（不占位，叠在 hero 上） */
.news-appbar--overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  background: transparent;
  border-bottom-color: transparent;
  backdrop-filter: none;
}
.news-appbar--overlay .sos-brand-lockup__text > strong,
.news-appbar--overlay .sos-brand-lockup__text > small,
.news-appbar--overlay .sos-navlink,
.news-appbar--overlay .news-appbar__search,
.news-appbar--overlay .news-appbar__menu {
  color: var(--sos-white);
}
/* 深色 hero 上搜索框需要可见的浅色描边，否则边界融进背景 */
.news-appbar--overlay .news-appbar__search {
  border-color: color-mix(in srgb, var(--sos-white) 45%, transparent);
}

/* 移动端全屏菜单 */
.news-mobile {
  position: fixed;
  inset: 0;
  z-index: var(--sos-z-overlay);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--sos-space-8);
  background: color-mix(in srgb, var(--sos-bg-page) 96%, transparent);
  backdrop-filter: blur(20px);
}
.news-mobile__close {
  position: absolute;
  top: var(--sos-space-5);
  right: var(--sos-space-4);
  border: 0;
  background: transparent;
  color: var(--sos-text-secondary);
  cursor: pointer;
}
.news-mobile__nav {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sos-space-6);
  font-family: var(--sos-font-reading);
  font-size: var(--sos-text-2xl);
  font-weight: var(--sos-weight-heavy);
}
.news-mobile__link {
  color: var(--sos-text-primary);
  text-decoration: none;
}
.news-mobile__link:hover {
  text-decoration: underline;
  text-decoration-color: var(--sos-signal);
  text-decoration-thickness: 0.15em;
  text-underline-offset: 0.2em;
}
.news-mobile__foot {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-sm);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--sos-duration-base) ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
