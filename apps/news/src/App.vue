<template>
  <!-- [修改] 根据路由 meta 决定是否显示 NavBar -->
  <NavBar v-if="!$route.meta.hideNavbar" :overlay="isBlogDetail" />

  <main :class="mainClass">
    <router-view />

    <footer
      v-if="!$route.name?.includes('editor') && !$route.meta.hideNavbar"
      class="app-footer"
    >
      <div
        :class="[
          'footer-links-row',
          isBlogDetail ? 'footer-blog-container' : ''
        ]"
      >
        <a
          v-for="link in footerLinks"
          :key="link.url"
          :href="link.url"
          target="_blank"
          rel="noopener noreferrer"
          class="footer-link"
        >
          {{ link.name }}
        </a>
      </div>
      <div
        :class="[
          'footer-icp-row',
          isBlogDetail ? 'footer-blog-container' : ''
        ]"
      >
        <a
          href="https://beian.miit.gov.cn/#/Integrated/index"
          target="_blank"
          rel="noopener noreferrer"
          class="footer-link"
        >
          皖ICP备2025089290号-1
        </a>
      </div>
    </footer>
  </main>

  <SearchOverlay />
  <DetailModal />
</template>

<script setup>
import { computed } from 'vue';
import { useRoute } from 'vue-router';

import NavBar from '@/components/NavBar.vue';
import SearchOverlay from '@/components/SearchOverlay.vue';
import DetailModal from '@/components/DetailModal.vue';

const route = useRoute();

const isBlogDetail = computed(() => route.path.startsWith('/blog/'));

const mainClass = computed(() => {
  // 如果是隐藏导航栏的页面（如 Quiz），则不应用任何布局限制，由页面自己全权接管
  if (route.meta.hideNavbar) {
    return '';
  }

  const base = 'main-base';
  if (isBlogDetail.value) {
    return base;
  }
  return `${base} main-default`;
});

const footerLinks = [
  { name: '长门有希的书架', url: 'https://haruyuki.cn/library/' },
  { name: '凉宫春日AI语音合成', url: 'https://tts.haruyuki.cn/' },
  { name: '凉宫春日AI声线转换', url: 'https://rvc.haruyuki.cn/' },
  { name: '圣地巡礼照片墙', url: 'https://haruhifanclub.notion.site/anitabi' },
  { name: '京阿尼台词语义化检索工具', url: 'https://anitool.haruyuki.cn/ ' },
  { name: '凉宫春日资源站', url: 'https://haruhifanclub.yuque.com/staff-sqlmik/phgf5z' },
  { name: '应援团活动', url: 'https://haruhifanclub.yuque.com/staff-sqlmik/cm5mug' },
];
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

/* Footer */
.app-footer {
  margin-top: auto;
  padding-top: 5rem;
  padding-bottom: 3rem;
  font-size: 0.75rem;
  line-height: 1rem;
  color: #6b7280;
  font-family: "Noto Sans SC", sans-serif;
}

/* Footer links row */
.footer-links-row {
  display: flex;
  flex-wrap: wrap;
  column-gap: 1.5rem;
  row-gap: 0.75rem;
  justify-content: center;
}

@media (min-width: 768px) {
  .footer-links-row {
    justify-content: flex-start;
  }
}

/* Footer ICP row */
.footer-icp-row {
  margin-top: 1rem;
  display: flex;
  justify-content: center;
}

@media (min-width: 768px) {
  .footer-icp-row {
    justify-content: flex-start;
  }
}

/* Container for footer rows when inside a blog detail page */
.footer-blog-container {
  max-width: 1000px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1rem;
  padding-right: 1rem;
}

@media (min-width: 768px) {
  .footer-blog-container {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

/* Footer links */
.footer-link {
  transition-property: color, background-color, border-color;
  transition-duration: 150ms;
}

.footer-link:hover {
  color: #000000;
}
</style>
