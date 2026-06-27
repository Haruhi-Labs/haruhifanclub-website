<script setup>
import { useRouter } from 'vue-router'
import { useMainStore } from '@/stores/main'
import { SosAppbar } from '@haruhi/ui'
import { AccountMenu } from '@haruhi/auth-ui'

defineProps({
  // 博客详情页等深色 hero 之上时，页头转透明叠加态
  overlay: { type: Boolean, default: false },
})

const store = useMainStore()
const router = useRouter()
const logoUrl = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

const goHome = () => {
  store.searchQuery = ''
  router.push('/')
}

// 主导航（站内 + 跨站）。exam 是独立 app，必须用原生 a 整页跳转。
const navLinks = [
  { label: '活动中心', to: '/activity' },
  { label: '奖品兑换', to: '/store' },
  { label: '我要投稿', to: '/submit' },
  { label: '团员手册', to: '/handbook' },
]
</script>

<template>
  <!-- 统一页头：SosAppbar 提供移动端汉堡 + 右侧抽屉，搜索/账号收进 actions（窄屏入抽屉） -->
  <SosAppbar class="news-appbar" :class="{ 'news-appbar--overlay': overlay }">
    <template #brand>
      <button class="sos-brand-lockup news-brand" type="button" @click="goHome" aria-label="返回春日团报首页">
        <span class="sos-brand-lockup__mark"><img :src="logoUrl" alt="" /></span>
        <span class="sos-brand-lockup__text">
          <strong>春日团报</strong>
          <small>凉宫春日应援团 · 文章中心</small>
        </span>
      </button>
    </template>

    <nav class="sos-navlinks">
      <RouterLink v-for="n in navLinks" :key="n.to" :to="n.to" class="sos-navlink">{{ n.label }}</RouterLink>
      <RouterLink :to="{ name: 'quiz' }" target="_blank" rel="noopener noreferrer" class="sos-navlink">凉宫入坑测试</RouterLink>
      <a href="/exam/" target="_blank" rel="noopener noreferrer" class="sos-navlink">SOS团期末考试</a>
    </nav>

    <template #actions>
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
    </template>
  </SosAppbar>
</template>

<style scoped>
/* 在统一 .sos-appbar 基础上补 news 特有部分 */
.news-brand {
  border: 0;
  background: transparent;
  cursor: pointer;
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

/* 博客详情等深色 hero 上：页头透明浮层 + 浅色文字（不占位，叠在 hero 上） */
.news-appbar--overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  border-bottom-color: transparent;
}
/* 磨砂背景在 ::before 上，叠加态需隐藏它以透出深色 hero */
.news-appbar--overlay::before {
  display: none;
}
.news-appbar--overlay .sos-brand-lockup__text > strong,
.news-appbar--overlay .sos-brand-lockup__text > small,
.news-appbar--overlay .sos-navlink,
.news-appbar--overlay .news-appbar__search,
.news-appbar--overlay :deep(.sos-appbar__burger) {
  color: var(--sos-white);
}
/* 深色 hero 上搜索框 / 汉堡需要可见的浅色描边 */
.news-appbar--overlay .news-appbar__search,
.news-appbar--overlay :deep(.sos-appbar__burger) {
  border-color: color-mix(in srgb, var(--sos-white) 45%, transparent);
  background: transparent;
}
</style>
