<!-- src/App.vue -->
<script setup>
import { computed } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import FooterBar from '@/components/FooterBar.vue'
import { AccountMenu } from '@haruhi/auth-ui'

const route = useRoute()
// 阅读页全屏沉浸，隐藏账号入口
const showAccount = computed(() => route.name !== 'Reader')
</script>

<template>
  <div class="library-app sos-scope min-h-screen flex flex-col" data-sos-site="library">
    <div v-if="showAccount" class="account-corner">
      <AccountMenu />
    </div>
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
.account-corner {
  position: fixed;
  top: 12px;
  right: 16px;
  z-index: var(--sos-z-sticky);
}
</style>
