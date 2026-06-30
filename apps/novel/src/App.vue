<!-- src/App.vue -->
<script setup>
import { computed } from 'vue'
import { RouterView, RouterLink, useRoute } from 'vue-router'
import { SosAppbar } from '@haruhi/ui'
import FooterBar from '@/components/FooterBar.vue'
import { AccountMenu, AccountAvatarLink } from '@haruhi/auth-ui'

const route = useRoute()
// 阅读页全屏沉浸，隐藏全局页头
const showHeader = computed(() => route.name !== 'Reader')
const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`
</script>

<template>
  <div class="library-app sos-scope min-h-screen flex flex-col" data-sos-site="library">
    <!-- 全局统一页头：SosAppbar 规范（含移动端汉堡 + 右侧抽屉），library 主题 -->
    <SosAppbar v-if="showHeader">
      <template #brand>
        <RouterLink to="/" class="sos-brand-lockup">
          <span class="sos-brand-lockup__mark">
            <img :src="logoSrc" alt="" />
          </span>
          <span class="sos-brand-lockup__text">
            <strong>长门有希的书架</strong>
            <small>凉宫春日应援团 · 书库</small>
          </span>
        </RouterLink>
      </template>
      <nav class="sos-navlinks">
        <RouterLink to="/feedback" class="sos-navlink">同人投稿 &amp; 问题反馈</RouterLink>
      </nav>
      <template #actions>
        <AccountMenu />
      </template>

      <!-- 移动端：头像快捷入口提到汉堡左侧，点头像直达个人中心 -->
      <template #mobile-lead>
        <AccountAvatarLink />
      </template>
    </SosAppbar>

    <!-- 中间是当前路由对应的页面 -->
    <div class="flex-1">
      <RouterView />
    </div>

    <!-- 底部统一 Footer -->
    <FooterBar />
  </div>
</template>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
}
/* 设计系统接入：library 表达提供配色/几何，但保留长门书架原有的
   Nunito 界面字体与暖纸行距，遵守「接入不得同化既有站点字体」铁律。
   标题与正文衬线沿用 library 的 --sos-font-reading（Noto Serif SC）。 */
.library-app.sos-scope {
  --sos-scope-font: 'Nunito', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --sos-scope-leading: 1.6;
}
</style>
