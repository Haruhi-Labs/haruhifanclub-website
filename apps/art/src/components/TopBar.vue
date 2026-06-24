<script setup>
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const navItems = [
  { path: '/', label: '首页' },
  { path: '/gallery', label: '画廊' },
  { path: '/announcements', label: '公告' },
  { path: '/upload', label: '投稿' },
  { path: '/points', label: '积分' },
  { path: '/exchange', label: '兑换' },
]

const isActive = (path) => {
  if (path === '/gallery' && route.path.startsWith('/creator')) return true
  return route.path === path
}
const linkClass = (path) => ['navlink', isActive(path) ? 'on' : ''].join(' ')
</script>

<template>
  <div class="topbar__inner">
    <div class="brand" role="button" tabindex="0" @click="router.push('/')" @keydown.enter="router.push('/')">
      <div class="brand__mark" aria-hidden="true"></div>
      <div class="brand__text">
        <div class="brand__title">应援团绘画部  ·  画廊</div>
        <div class="brand__sub">一起来体验分享的快乐吧！</div>
      </div>
    </div>

    <nav class="nav" aria-label="画廊功能导航">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :class="linkClass(item.path)"
        :to="item.path"
        data-sfx="click"
      >
        {{ item.label }}
      </RouterLink>
    </nav>
  </div>
</template>

<style scoped>
.topbar__inner {
  /* --- 布局定位 --- */
  position: fixed; 
  top: 0;
  left: 0;
  right: 0; 
  height: 72px; 
  z-index: 999; 
  
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 0 24px;
  
  /* --- 视觉风格：全透明 --- */
  background: transparent;
  border: none;
  box-shadow: none;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  user-select: none;
  color: #fff;
  text-shadow: 0 2px 4px rgba(0,0,0,0.6);
  
  /* 关键：防止 Logo 被压缩 */
  flex-shrink: 0; 
}

.brand__mark {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background-image: url('../assets/logo.webp');
  background-size: cover;
  background-position: center;
  border: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  background-color: rgba(255, 255, 255, 0.25);
  flex-shrink: 0;
}

.brand__text {
  display: flex;
  flex-direction: column;
  justify-content: center;
  /* 默认情况下内容不压缩 */
  overflow: hidden;
}

.brand__title {
  font-weight: 950;
  letter-spacing: .5px;
  line-height: 1.1;
  font-size: 15px;
  white-space: nowrap;
  /* 如果实在太长，允许省略号 */
  text-overflow: ellipsis;
  overflow: hidden;
}

.brand__sub {
  margin-top: 2px;
  font-size: 11px;
  font-weight: 800;
  opacity: .95;
  letter-spacing: 0.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}

.nav {
  display: flex;
  align-items: center;
  gap: 4px;
  max-width: min(720px, 62vw);
  padding: 5px;
  border: 1px solid rgba(255, 255, 255, 0.34);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.28);
  box-shadow: 0 10px 30px rgba(0,0,0,0.14), inset 0 1px 0 rgba(255,255,255,0.38);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  flex-shrink: 0; /* 导航区不许被压缩，保证按钮完整 */
  overflow-x: auto;
  scrollbar-width: none;
}

.nav::-webkit-scrollbar {
  display: none;
}

.navlink {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 58px;
  padding: 8px 13px;
  font-weight: 900;
  font-size: 13px;
  letter-spacing: 0;
  border-radius: 999px;
  background: transparent;
  color: rgba(13, 63, 74, 0.88);
  border: 1px solid transparent;
  box-shadow: none;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  text-decoration: none; /* 确保链接无下划线 */
  white-space: nowrap;
}

.navlink:hover {
  transform: translateY(-1px);
  background: rgba(255,255,255,0.62);
  color: #063f4f;
  box-shadow: 0 8px 16px rgba(0,0,0,0.1);
}

.navlink.on {
  background: linear-gradient(135deg, #fef08a 0%, #86efac 50%, #67e8f9 100%);
  color: #073b4c;
  text-shadow: 0 1px 0 rgba(255,255,255,0.3);
  transform: scale(1.03);
  border-color: rgba(255,255,255,0.72);
  box-shadow: 0 8px 18px rgba(20, 140, 160, 0.22), inset 0 1px 0 rgba(255,255,255,0.72);
  z-index: 5;
}

/* =========================================
   手机端自适应优化 (宽度 <= 768px)
   ========================================= */
@media (max-width: 768px) {
  .topbar__inner {
    /* 1. 减小两边内边距，给内容更多空间 */
    padding: 0 12px;
    /* 2. 减小中间间距 */
    gap: 8px;
    height: 76px;
  }
  
  .brand {
    /* 3. 核心修改：允许 brand 区域占据剩余所有空间 */
    flex: 1;
    /* 4. 核心修改：允许 flex 子元素内部收缩 (否则文字会将容器撑开导致溢出) */
    min-width: 0;
    gap: 8px; /* 减小 logo 和文字的间距 */
  }

  .brand__mark {
    /* 5. 稍微缩小 logo，省出空间给文字 */
    width: 36px;
    height: 36px;
  }

  .brand__title {
    /* 6. 稍微调小标题字号 */
    font-size: 14px;
    line-height: 1.2;
  }

  .brand__sub {
    /* 7. 调小副标题字号，确保能放下更多字 */
    font-size: 10px;
    margin-top: 0;
    opacity: 0.9;
  }

  .nav {
    max-width: 52vw;
    gap: 3px;
    padding: 4px;
  }

  .navlink {
    min-width: 52px;
    font-size: 13px;
    padding: 7px 11px;
  }
}

@media (max-width: 560px) {
  .brand__text {
    display: none;
  }

  .nav {
    max-width: calc(100vw - 74px);
  }
}
</style>
