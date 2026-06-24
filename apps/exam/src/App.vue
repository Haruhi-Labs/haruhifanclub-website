<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import TheFooter from '@/components/TheFooter.vue';
import { AccountMenu } from '@haruhi/auth-ui';

const route = useRoute();

/**
 * 决定是否显示 Footer
 * 规则：排除试卷页 ('exam') 和官方试卷页 ('haruhi')
 */
const showFooter = computed(() => {
  const hiddenRoutes = ['exam', 'haruhi'];
  return !hiddenRoutes.includes(route.name as string);
});
</script>

<template>
  <div class="app-wrapper">
    <!-- 右上角统一账号入口（全屏试卷页隐藏，避免遮挡） -->
    <div v-if="showFooter" class="account-corner">
      <AccountMenu />
    </div>
    <!-- 主内容区：自动伸展 -->
    <div class="app-main">
      <router-view></router-view>
    </div>
    
    <!-- Footer：仅在非试卷页显示 -->
    <TheFooter v-if="showFooter" />
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

.account-corner {
  position: fixed;
  top: 12px;
  right: 16px;
  z-index: 1000;
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