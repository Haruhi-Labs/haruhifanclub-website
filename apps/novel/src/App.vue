<!-- src/App.vue -->
<script setup>
import { RouterView } from 'vue-router'
import FooterBar from '@/components/FooterBar.vue'
</script>

<template>
  <div class="min-h-screen bg-[#FAF9DE] text-[#4A3B32] flex flex-col">
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
</style>
