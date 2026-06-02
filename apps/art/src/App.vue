<script setup>
import TopBar from './components/TopBar.vue'
</script>

<template>
  <div class="bg-layer gallery-bg"></div>
  <div class="bg-layer gallery-mask"></div>
  
  <div class="app-shell">
    <header class="topbar">
      <TopBar />
    </header>

    <main class="main">
      <router-view v-slot="{ Component }">
        <transition name="page" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
  </div>
</template>

<style>
/* =========================================
   全局页面布局约束
   ========================================= */
.app-shell {
  /* ⚠️ 重要：因为导航栏是 fixed 定位，必须给内容区一个顶部内边距。
     数值 = 导航栏高度 (约72px) + 间距 (24px) = 96px */
  padding-top: 96px;
  /* Ensure the shell takes full height so short pages don't abruptly end */
  min-height: 100dvh;
}

/* =========================================
   全局页面切换动画：磨砂浮动 + 缩放效果
   ========================================= */

/* 1. 进场和离场的激活状态 */
.page-enter-active,
.page-leave-active {
  /* 使用贝塞尔曲线模拟物理惯性，比 linear 更自然 */
  transition: 
    opacity 0.5s cubic-bezier(0.2, 0.8, 0.2, 1),
    transform 0.5s cubic-bezier(0.2, 0.8, 0.2, 1),
    filter 0.5s ease;
}

/* 2. 进场开始状态 (页面刚要出来时) */
.page-enter-from {
  opacity: 0;
  /* 稍微向下偏移 15px，有一种浮上来的感觉 */
  transform: translateY(15px) scale(0.98); 
  /* 加上模糊，模拟对焦过程，增加高级感 */
  filter: blur(8px);
}

/* 3. 离场结束状态 (旧页面离开后) */
.page-leave-to {
  opacity: 0;
  /* 稍微向上偏移 -15px，有一种飘走的感觉 */
  transform: translateY(-15px) scale(0.98);
  /* 离开时也变模糊 */
  filter: blur(8px);
}
</style>