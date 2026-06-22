<script setup>
import { useRoute, useRouter } from 'vue-router'
import { AccountMenu } from '@haruhi/auth-ui'

const route = useRoute()
const router = useRouter()

const isActive = (path) => {
  if (path === '/' && route.path.startsWith('/creator')) return true
  return route.path === path
}
const linkClass = (path) => ['navlink', isActive(path) ? 'on' : ''].join(' ')

function enterAdmin() {
  // 管理后台登录已统一为 JWT 单点登录，直接进入后台由 AdminView 处理登录
  router.push('/admin')
}
</script>

<template>
  <div class="topbar__inner">
    <div class="brand" role="button" tabindex="0" @click="router.push('/').then(() => { if(route.path === '/') router.go(0) })" @keydown.enter="router.push('/')">
      <div class="brand__mark" aria-hidden="true"></div>
      <div class="brand__text">
        <div class="brand__title">应援团绘画部  ·  画廊</div>
        <div class="brand__sub">一起来体验分享的快乐吧！</div>
      </div>
    </div>

    <nav class="nav">
      <RouterLink :class="linkClass('/')" to="/" data-sfx="click">画廊</RouterLink>
      <RouterLink :class="linkClass('/upload')" to="/upload" data-sfx="click">投稿</RouterLink>
      <RouterLink :class="linkClass('/points')" to="/points" data-sfx="click">积分</RouterLink>
      <AccountMenu />
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
  gap: 16px;
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
  gap: 32px;
  flex-shrink: 0; /* 导航区不许被压缩，保证按钮完整 */
}

.navlink {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 24px;
  font-weight: 900;
  font-size: 14px;
  letter-spacing: 1px;
  border-radius: 2px 20px 2px 20px;
  background: linear-gradient(135deg, #d9f99d 0%, #86efac 100%);
  color: #14532d;
  border: 1px solid rgba(255,255,255,0.5);
  box-shadow: 0 4px 8px rgba(0,0,0,0.2);
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  text-decoration: none; /* 确保链接无下划线 */
}

.navlink:hover { transform: translateY(-2px) scale(1.05); box-shadow: 0 8px 16px rgba(0,0,0,0.25); }
.navlink.on {
  background: linear-gradient(135deg, #bef264 0%, #4ade80 100%);
  color: #064e3b;
  text-shadow: 0 1px 0 rgba(255,255,255,0.3);
  transform: scale(1.1) rotate(-3deg);
  box-shadow: 0 6px 14px rgba(20, 200, 100, 0.4);
  z-index: 5;
}

.adminBtn {
  border: none; cursor: pointer; padding: 8px 20px; font-weight: 900; font-size: 13px;
  border-radius: 20px 2px 20px 2px;
  background: linear-gradient(135deg, #fdba74 0%, #fb923c 100%);
  color: #7c2d12;
  box-shadow: 0 4px 8px rgba(0,0,0,0.2);
  transition: transform 0.2s ease;
}
.adminBtn:hover { transform: rotate(5deg) scale(1.1); }
.adminBtn:active { transform: scale(0.95); }

/* =========================================
   手机端自适应优化 (宽度 <= 768px)
   ========================================= */
@media (max-width: 768px) {
  .topbar__inner {
    /* 1. 减小两边内边距，给内容更多空间 */
    padding: 0 12px;
    /* 2. 减小中间间距 */
    gap: 8px;
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
    /* 8. 减小导航按钮之间的间距 */
    gap: 8px;
  }

  .navlink {
    /* 9. 以前是 18px 太大了，导致挤压左边。改为 14px 比较平衡 */
    font-size: 14px;
    padding: 6px 12px; /* 减小按钮内边距 */
  }

  .adminBtn {
    display: none;
  }
}
</style>