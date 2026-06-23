<template>
  <!-- [修改] 根据路由 meta 决定是否显示 NavBar -->
  <NavBar v-if="!$route.meta.hideNavbar" :overlay="isBlogDetail" />

  <SosPage v-if="!$route.meta.hideNavbar" as="main" site="news" contained="none" :class="mainClass">
    <router-view />

    <SiteFooter />
  </SosPage>

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
import { SosPage } from '@haruhi/ui'

import NavBar from '@/shell/NavBar.vue'
import SiteFooter from '@/shell/SiteFooter.vue'
import SearchOverlay from '@/features/blog/components/SearchOverlay.vue'
import DetailModal from '@/features/blog/components/DetailModal.vue'

const route = useRoute()

const isBlogDetail = computed(() => route.path.startsWith('/blog/'))

const mainClass = computed(() => {
  // 如果是隐藏导航栏的页面（如 Quiz），则不应用任何布局限制，由页面自己全权接管
  if (route.meta.hideNavbar) {
    return ''
  }

  const base = 'main-base'
  if (isBlogDetail.value) {
    return base
  }
  return `${base} main-default`
})
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

/* Default (non-blog-detail) page additions */
.main-default {
  max-width: 1200px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1rem;
  padding-right: 1rem;
  padding-top: 2rem;
  padding-bottom: 2rem;
}
</style>
