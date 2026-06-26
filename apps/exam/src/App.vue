<script setup lang="ts">
import { computed } from 'vue';
import { RouterLink, useRoute } from 'vue-router';
import TheFooter from '@/components/TheFooter.vue';

const route = useRoute();

/**
 * 全屏试卷页（'exam' / 'haruhi'）隐藏统一页头与页脚，保持沉浸答题
 */
const showChrome = computed(() => {
  const hiddenRoutes = ['exam', 'haruhi'];
  return !hiddenRoutes.includes(route.name as string);
});
const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`;
</script>

<template>
  <div class="app-wrapper sos-scope" data-sos-site="exam">
    <!-- 全局统一页头：SosAppbar 规范 + exam 主题，账号收进右侧 actions -->
    <header v-if="showChrome" class="sos-appbar">
      <div class="sos-appbar__inner">
        <RouterLink to="/" class="sos-brand-lockup">
          <span class="sos-brand-lockup__mark">
            <img :src="logoSrc" alt="" />
          </span>
          <span class="sos-brand-lockup__text">
            <strong>春日试卷中心</strong>
            <small>凉宫春日应援团 · 开放试卷系统</small>
          </span>
        </RouterLink>
        <div class="exam-appbar__right">
          <nav class="sos-navlinks">
            <RouterLink to="/create" class="sos-navlink">出题</RouterLink>
          </nav>
        </div>
      </div>
    </header>

    <!-- 主内容区：自动伸展 -->
    <div class="app-main">
      <router-view></router-view>
    </div>

    <!-- Footer：仅在非试卷页显示 -->
    <TheFooter v-if="showChrome" />
  </div>
</template>

<style>
/* 重构 App 布局以支持 Sticky Footer
  使用 flex-col 和 min-height: 100vh 
*/
#app {
  height: 100%;
  width: 100%;
  /* 移除原先的 height: 100% 锁定，改用 flex 布局让内容自然撑开 */
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-overflow-scrolling: touch;
  
  /* * 新增：
   * 确保 App 根节点的背景是透明的，否则会遮挡住 main.css 或 HomeView 中的背景设计 
   */
  background: transparent; 
}

.app-wrapper {
  display: flex;
  flex-direction: column;
  min-height: 100vh; /* 确保至少占满一屏 */
  position: relative; /* 建立层级上下文 */
}

/* 设计系统接入：exam 表达提供配色/几何，但保留考场原有的系统界面字体
   与行距，遵守「接入不得同化既有站点字体」铁律。标题衬线由 exam 表达的
   --sos-display-family 决定；试卷页 .desk 自带木纹背景与配色不受影响。 */
.app-wrapper.sos-scope {
  --sos-scope-font: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC',
    'Hiragino Sans GB', 'Microsoft YaHei', system-ui, sans-serif;
  --sos-scope-leading: 1.5;
  background: transparent; /* 让 HomeView 固定背景层/试卷木纹透出 */
}

.exam-appbar__right {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}

.app-main {
  flex: 1; /* 占据剩余空间，将 footer 推到底部 */
  display: flex;
  flex-direction: column;
  width: 100%;
}

/* 针对试卷页的特殊处理：恢复全屏锁定 */
/* 当路由是 exam/haruhi 时，router-view 内部的 .desk 应该使用 fixed 定位覆盖这些布局 */
</style>