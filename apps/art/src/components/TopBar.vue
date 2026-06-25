<script setup>
import { useRoute, useRouter } from 'vue-router'
import { AccountMenu } from '@haruhi/auth-ui'

const route = useRoute()
const router = useRouter()

const navItems = [
  { path: '/gallery', label: '画廊' },
  { path: '/upload', label: '投稿' },
  { path: '/points', label: '积分' },
  { path: '/exchange', label: '兑换' },
]
const announcementPath = '/announcements'

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

    <div class="topbar-actions">
      <RouterLink
        :class="['notice-orb', isActive(announcementPath) ? 'on' : '']"
        :to="announcementPath"
        aria-label="公告"
        title="公告"
        data-sfx="click"
      >
        <span class="notice-orb__sign" aria-hidden="true">
          <span class="notice-orb__pin"></span>
          <span class="notice-orb__line long"></span>
          <span class="notice-orb__line short"></span>
        </span>
      </RouterLink>

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

      <div class="account-entry">
        <AccountMenu login-path="/login" profile-path="/account" settings-path="/account/settings" />
      </div>
    </div>
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
  justify-content: flex-start;
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

.topbar-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  min-width: 0;
  margin-left: auto;
}

.notice-orb {
  position: relative;
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 46px;
  border: 1px solid rgba(255, 255, 255, 0.5);
  border-radius: 999px;
  background:
    radial-gradient(circle at 34% 26%, rgba(255, 255, 255, 0.74), rgba(255, 255, 255, 0.18) 56%, rgba(255, 255, 255, 0.08)),
    linear-gradient(135deg, rgba(255, 240, 166, 0.54), rgba(103, 232, 249, 0.2));
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.15), inset 0 1px 0 rgba(255, 255, 255, 0.58);
  color: #073b4c;
  text-decoration: none;
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  transition: transform 0.24s ease, border-color 0.24s ease, box-shadow 0.24s ease, background 0.24s ease;
}

.notice-orb:hover {
  transform: translateY(-1px) scale(1.04);
  border-color: rgba(255, 255, 255, 0.78);
  background:
    radial-gradient(circle at 34% 26%, rgba(255, 255, 255, 0.86), rgba(255, 255, 255, 0.28) 58%, rgba(255, 255, 255, 0.12)),
    linear-gradient(135deg, rgba(254, 240, 138, 0.72), rgba(103, 232, 249, 0.3));
  box-shadow: 0 14px 34px rgba(0, 0, 0, 0.18), 0 0 0 4px rgba(254, 240, 138, 0.16);
}

.notice-orb.on {
  border-color: rgba(255, 255, 255, 0.84);
  background:
    radial-gradient(circle at 36% 24%, rgba(255, 255, 255, 0.9), transparent 34%),
    linear-gradient(135deg, #fef08a 0%, #fb7185 48%, #67e8f9 100%);
  box-shadow: 0 14px 32px rgba(244, 63, 94, 0.22), 0 0 0 4px rgba(103, 232, 249, 0.14);
}

.notice-orb__sign {
  position: relative;
  display: block;
  width: 25px;
  height: 21px;
  border: 2px solid currentColor;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.52);
  box-shadow: inset 0 -3px 0 rgba(7, 59, 76, 0.08);
}

.notice-orb__sign::before,
.notice-orb__sign::after {
  content: "";
  position: absolute;
  top: -7px;
  width: 2px;
  height: 7px;
  border-radius: 99px;
  background: currentColor;
}

.notice-orb__sign::before {
  left: 6px;
}

.notice-orb__sign::after {
  right: 6px;
}

.notice-orb__pin {
  position: absolute;
  top: -3px;
  left: 50%;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #fb7185;
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.74);
  transform: translateX(-50%);
}

.notice-orb__line {
  position: absolute;
  left: 6px;
  height: 2px;
  border-radius: 99px;
  background: currentColor;
  opacity: 0.78;
}

.notice-orb__line.long {
  right: 6px;
  top: 8px;
}

.notice-orb__line.short {
  right: 11px;
  top: 13px;
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
  flex: 0 1 auto;
  min-width: 0;
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

.account-entry {
  flex: 0 0 auto;
  color: #073b4c;
}

.account-entry :deep(.hauth-root) {
  --hauth-accent: #f43f5e;
  --hauth-accent-hover: #e11d48;
  --hauth-bg: rgba(255, 255, 255, 0.92);
  --hauth-field-bg: rgba(248, 250, 252, 0.9);
  --hauth-border: rgba(255, 255, 255, 0.55);
  --hauth-fg: #073b4c;
  --hauth-muted: rgba(7, 59, 76, 0.62);
}

.account-entry :deep(.hauth-btn--sm),
.account-entry :deep(.hauth-trigger) {
  min-height: 42px;
  border: 1px solid rgba(255, 255, 255, 0.56);
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.68), rgba(255, 255, 255, 0.34)),
    linear-gradient(135deg, rgba(244, 63, 94, 0.12), rgba(103, 232, 249, 0.16));
  color: #073b4c;
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.16), inset 0 1px 0 rgba(255, 255, 255, 0.58);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
}

.account-entry :deep(.hauth-trigger:hover),
.account-entry :deep(.hauth-trigger[aria-expanded="true"]),
.account-entry :deep(.hauth-btn--sm:hover) {
  border-color: rgba(255, 255, 255, 0.78);
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.78), rgba(255, 255, 255, 0.42)),
    linear-gradient(135deg, rgba(244, 63, 94, 0.18), rgba(103, 232, 249, 0.22));
}

.account-entry :deep(.hauth-avatar) {
  width: 32px;
  height: 32px;
}

.account-entry :deep(.hauth-dropdown) {
  right: 0;
  left: auto;
  color: #073b4c;
  border-color: rgba(255, 255, 255, 0.7);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.96), rgba(242, 251, 255, 0.9)),
    radial-gradient(circle at top right, rgba(103, 232, 249, 0.2), transparent 42%);
}

:global(html.art-lights-out) .notice-orb {
  border-color: rgba(125, 211, 252, 0.28);
  background:
    radial-gradient(circle at 34% 24%, rgba(186, 230, 253, 0.28), rgba(15, 23, 42, 0.4) 58%),
    linear-gradient(135deg, rgba(15, 23, 42, 0.72), rgba(30, 27, 75, 0.62));
  color: #bae6fd;
  box-shadow: 0 14px 30px rgba(0, 0, 0, 0.26), inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

:global(html.art-lights-out) .notice-orb:hover,
:global(html.art-lights-out) .notice-orb.on {
  border-color: rgba(125, 211, 252, 0.52);
  background:
    radial-gradient(circle at 36% 24%, rgba(186, 230, 253, 0.38), transparent 36%),
    linear-gradient(135deg, rgba(30, 41, 59, 0.78), rgba(88, 28, 135, 0.58));
  box-shadow: 0 14px 34px rgba(0, 0, 0, 0.32), 0 0 0 4px rgba(125, 211, 252, 0.1);
}

:global(html.art-lights-out) .notice-orb__sign {
  background: rgba(15, 23, 42, 0.46);
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
    max-width: 44vw;
    gap: 3px;
    padding: 4px;
  }

  .notice-orb {
    width: 42px;
    height: 42px;
  }

  .notice-orb__sign {
    width: 23px;
    height: 19px;
  }

  .navlink {
    min-width: 52px;
    font-size: 13px;
    padding: 7px 11px;
  }

  .account-entry :deep(.hauth-trigger-main),
  .account-entry :deep(.hauth-trigger-caret) {
    display: none;
  }

  .account-entry :deep(.hauth-trigger) {
    padding: 4px 5px;
  }
}

@media (max-width: 560px) {
  .brand__text {
    display: none;
  }

  .nav {
    max-width: calc(100vw - 178px);
  }
}
</style>
