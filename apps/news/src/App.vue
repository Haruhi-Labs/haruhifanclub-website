<template>
  <!-- [修改] 根据路由 meta 决定是否显示 NavBar -->
  <NavBar v-if="!$route.meta.hideNavbar" :overlay="isBlogDetail" />

  <!-- 设计系统浅层接入：仅保留 data-sos-site 提供 token 表达，不再套 SosPage（DS 基础皮）。 -->
  <main v-if="!$route.meta.hideNavbar" class="main-base" data-sos-site="news">
    <!-- 限宽只作用于内容区；SiteFooter 留在 main-base 内但在内容容器之外，保持全宽通栏，
         与 art/shop/exam/novel 各站 footer 一致（此前 footer 被裹进 1200px 容器导致变窄）。 -->
    <div :class="['news-content', { 'news-content--default': !isBlogDetail }]">
      <router-view />
    </div>

    <SiteFooter />
  </main>

  <main v-else>
    <router-view />

    <SiteFooter />
  </main>

  <SearchOverlay />
  <DetailModal />
</template>

<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'

import NavBar from '@/shell/NavBar.vue'
import SiteFooter from '@/shell/SiteFooter.vue'
import SearchOverlay from '@/features/blog/components/SearchOverlay.vue'
import DetailModal from '@/features/blog/components/DetailModal.vue'

const route = useRoute()

const isBlogDetail = computed(() => route.path.startsWith('/blog/'))
</script>

<style scoped>
/* Main layout base: shared by blog detail and default pages */
.main-base {
  flex-grow: 1;
  width: 100%;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  position: relative;
}

/* 内容区：撑满剩余高度把 footer 顶到底部（吸底）；footer 在此容器之外，保持全宽。 */
.news-content {
  flex: 1 0 auto;
  width: 100%;
}

/* Default (non-blog-detail) page additions：仅限内容区居中限宽，不再波及 footer。 */
.news-content--default {
  max-width: 1200px;
  margin-left: auto;
  margin-right: auto;
  padding: 2rem 1rem;
}
</style>
