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
  <div class="min-h-screen bg-[#FAF9DE] text-[#4A3B32] flex flex-col">
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
.account-corner {
  position: fixed;
  top: 12px;
  right: 16px;
  z-index: 1000;
}
</style>
