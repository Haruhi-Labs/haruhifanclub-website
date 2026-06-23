<script setup>
/**
 * TheFooter.vue
 * 经典米色卡片风格页脚 - 重构版
 */

// 使用项目中实际存在的 Logo 图片。
// JS 字符串不会被 Vite 按 base 重写（模板/CSS 才会），app 部署在 /library/ 子路径下，
// 必须显式拼 BASE_URL，否则打到域名根 404。
const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`;

const scrollToTop = () => {
  // 优先尝试滚动 #app 容器（移动端/主布局可能锁定在 #app 上）
  const appContainer = document.getElementById('app');
  if (appContainer) {
    appContainer.scrollTo({ top: 0, behavior: 'smooth' });
  }
  // 兜底滚动 window
  window.scrollTo({ top: 0, behavior: 'smooth' });
};
</script>

<template>
  <footer class="foot">
    <!-- 顶部很轻的上扬阴影，让 footer 独立出来但不抢视线 -->
    <div class="foot__lift" aria-hidden="true"></div>

    <div class="foot__inner">
      <!-- 上半部分：身份 & 链接 -->
      <div class="foot__top">
        <!-- 左侧：项目身份 -->
        <div class="foot__brand">
          <div class="foot__logo">
            <img
              :src="logoSrc"
              alt="凉宫春日应援团"
              onerror="this.style.display='none'"
            />
          </div>
          <div>
            <div class="foot__name">凉宫春日应援团</div>
            <div class="foot__handle">Haruhifanclub</div>
            <div class="foot__tagline">非营利的凉宫春日粉丝团体</div>
          </div>
        </div>

        <!-- 右侧：链接组 -->
        <div class="foot__groups">
          <!-- 社交平台 -->
          <div class="foot__group">
            <span class="foot__group-label">社交平台</span>
            <div class="foot__links">
              <a
                href="https://space.bilibili.com/201296348"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #fb7299"
              >
                哔哩哔哩
              </a>
              <a
                href="https://xhslink.com/m/8u0JGkC5KUE"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #ff2442"
              >
                小红书
              </a>
            </div>
          </div>

          <!-- 主要站点 -->
          <div class="foot__group">
            <span class="foot__group-label">主要站点</span>
            <div class="foot__links">
              <a
                href="https://tts.haruyuki.cn"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #3b82f6"
              >
                AI语音
              </a>
              <a
                href="https://rvc.haruyuki.cn"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #06b6d4"
              >
                AI变声
              </a>
              <a
                href="https://haruhifanclub.notion.site/anitabi"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #eab308"
              >
                圣地巡礼
              </a>
              <a
                href="https://haruyuki.cn/library"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #10b981"
              >
                长门书架
              </a>
            </div>
          </div>

          <!-- 加入我们 -->
          <div class="foot__group">
            <span class="foot__group-label">加入我们</span>
            <div class="foot__links">
              <a
                href="https://qm.qq.com/q/CVssyL3Pj2"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #10b981"
              >
                超能力者
              </a>
              <a
                href="https://qm.qq.com/q/JcS7yXYoU2"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #a855f7"
              >
                异世界人
              </a>
              <a
                href="https://qm.qq.com/q/8nYNs7rFwA"
                target="_blank"
                rel="noopener"
                class="foot-link"
                style="--brand: #f97316"
              >
                未来人
              </a>
            </div>
          </div>
        </div>
      </div>

      <!-- 下半部分：版权 & 说明 -->
      <div class="foot__bottom">
        <div class="foot__copy">© 2025 HARUHIFANCLUB · Non-profit fan project</div>
        <div class="foot__notes">
          <span>非营利项目</span>
          <span class="foot__dot" aria-hidden="true"></span>
          <span>仅供交流</span>
        </div>
      </div>
    </div>

    <!-- 回到顶部按钮 -->
    <button type="button" class="foot__top-btn" aria-label="回到顶部" @click="scrollToTop">
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <polyline points="18 15 12 9 6 15" />
      </svg>
    </button>
  </footer>
</template>

<style scoped>
/* 页脚：吃 library 表面/文本 token；社交/站点链接保留各自品牌悬浮色（--brand）。 */
.foot {
  position: relative;
  margin-top: var(--sos-space-8);
  border-top: 1px solid var(--sos-border-subtle);
  background: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
}
.foot__lift {
  pointer-events: none;
  position: absolute;
  inset-inline: var(--sos-space-6);
  top: -3px;
  height: 3px;
  border-radius: var(--sos-radius-full);
  background: color-mix(in srgb, var(--sos-ink-950) 4%, transparent);
  filter: blur(6px);
  opacity: 0.8;
}
.foot__inner {
  margin-inline: auto;
  max-width: var(--sos-container-content);
  padding-inline: var(--sos-page-gutter);
}

.foot__top {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-8);
  padding-block: var(--sos-space-10);
}
@media (min-width: 768px) {
  .foot__top {
    flex-direction: row;
    align-items: flex-start;
    justify-content: space-between;
  }
}

.foot__brand {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}
.foot__logo {
  width: 3rem;
  height: 3rem;
  flex-shrink: 0;
  display: grid;
  place-items: center;
  overflow: hidden;
  border-radius: var(--sos-radius-full);
  box-shadow: var(--sos-shadow-xs);
  /* SOS 三色柔光：暖橙→粉→天蓝，保留品牌识别 */
  background: linear-gradient(135deg, #ffe6c7, #fad5e5 55%, #d9e8ff);
}
.foot__logo > img {
  width: 2.5rem;
  height: 2.5rem;
  border-radius: var(--sos-radius-full);
  object-fit: cover;
  opacity: 0.9;
  transition: opacity var(--sos-duration-fast) var(--sos-ease-out);
}
.foot__logo > img:hover {
  opacity: 1;
}
.foot__name {
  font-size: var(--sos-text-sm);
  font-weight: var(--sos-weight-heavy);
  letter-spacing: var(--sos-tracking-wide);
  color: var(--sos-text-primary);
}
.foot__handle {
  margin-top: 0.1rem;
  font-family: var(--sos-font-mono);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.foot__tagline {
  margin-top: 0.2rem;
  font-size: var(--sos-text-2xs);
  color: var(--sos-text-tertiary);
}

.foot__groups {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-8);
}
@media (min-width: 768px) {
  .foot__groups {
    flex-direction: row;
    align-items: flex-start;
    gap: var(--sos-space-12);
  }
}
.foot__group {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3);
}
.foot__group-label {
  font-size: var(--sos-text-2xs);
  font-weight: var(--sos-weight-heavy);
  letter-spacing: var(--sos-tracking-wider);
  text-transform: uppercase;
  color: var(--sos-text-tertiary);
  user-select: none;
}
.foot__links {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
  max-width: 22rem;
}

.foot-link {
  display: inline-flex;
  align-items: center;
  padding: var(--sos-space-1) var(--sos-space-3);
  border-radius: var(--sos-radius-full);
  border: 1px solid var(--sos-border-subtle);
  background: color-mix(in srgb, var(--sos-bg-page) 60%, var(--sos-bg-surface));
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-xs);
  text-decoration: none;
  box-shadow: var(--sos-shadow-xs);
  transition:
    color var(--sos-duration-fast) var(--sos-ease-out),
    border-color var(--sos-duration-fast) var(--sos-ease-out),
    background-color var(--sos-duration-fast) var(--sos-ease-out);
}
.foot-link:hover {
  color: var(--brand, var(--sos-link));
  border-color: color-mix(in srgb, var(--brand, var(--sos-link)) 40%, transparent);
  background: var(--sos-bg-surface);
}

.foot__bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-3);
  padding-block: var(--sos-space-6);
  border-top: 1px solid var(--sos-border-subtle);
  font-size: var(--sos-text-2xs);
  color: var(--sos-text-tertiary);
}
@media (min-width: 640px) {
  .foot__bottom {
    flex-direction: row;
  }
}
.foot__copy {
  text-align: center;
  font-weight: var(--sos-weight-medium);
}
.foot__notes {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
  opacity: 0.85;
}
.foot__dot {
  width: 1px;
  height: 0.65rem;
  background: var(--sos-border-default);
}

/* 回到顶部 */
.foot__top-btn {
  position: absolute;
  top: -1rem;
  left: 50%;
  transform: translateX(-50%);
  display: grid;
  place-items: center;
  width: 2rem;
  height: 2rem;
  border-radius: var(--sos-radius-full);
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  box-shadow: var(--sos-shadow-sm);
  color: var(--sos-accent);
  z-index: var(--sos-z-raised);
  transition:
    transform var(--sos-duration-base) var(--sos-ease-out),
    box-shadow var(--sos-duration-base),
    color var(--sos-duration-fast);
}
.foot__top-btn:hover {
  transform: translateX(-50%) translateY(-3px);
  box-shadow: var(--sos-shadow-card);
  color: var(--sos-accent-hover);
}
.foot__top-btn > svg {
  width: 1rem;
  height: 1rem;
}
</style>
