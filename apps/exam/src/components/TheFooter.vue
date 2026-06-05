<script setup lang="ts">
/**
 * TheFooter.vue
 * 旗舰版白色风格页脚 - 融合胶囊卡片与紧凑布局
 * 设计灵感：Stripe / Vercel
 */

// 使用项目中实际存在的 Logo 图片
const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

const scrollToTop = () => {
  const appContainer = document.getElementById('app');
  if (appContainer) {
    appContainer.scrollTo({ top: 0, behavior: 'smooth' });
  }
  window.scrollTo({ top: 0, behavior: 'smooth' });
};
</script>

<template>
  <footer class="site-footer">
    <div class="footer-inner">
      
      <!-- 上半部分：网格布局 -->
      <div class="footer-grid">
        
        <!-- 左侧：品牌与社交 (占据较宽空间) -->
        <div class="brand-col">
          <div class="brand-header">
            <div class="logo-circle">
              <img :src="logoSrc" alt="Logo" class="logo-img" onerror="this.style.display='none'" />
            </div>
            <div>
              <h3 class="brand-name">凉宫春日应援团</h3>
              <p class="brand-slug">Haruhifanclub</p>
            </div>
          </div>
          <p class="brand-desc">
            让世界变得更加热闹！<br>
            《凉宫春日系列》的非营利粉丝团体。
          </p>
          
          <!-- 社交按钮组 -->
          <div class="social-row">
            <a href="https://space.bilibili.com/201296348" target="_blank" class="social-btn bilibili" aria-label="Bilibili">
              <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18"><path d="M17.813 4.653h.854c1.51.054 2.769.578 3.737 1.548.967.966 1.494 2.227 1.549 3.735v6.526c-.055 1.508-.582 2.768-1.549 3.735-.968.967-2.227 1.494-3.737 1.548h-11.334c-1.51-.054-2.769-.581-3.737-1.548-.967-.967-1.494-2.227-1.549-3.735v-6.526c.055-1.508.582-2.769 1.549-3.735.968-.97 2.227-1.494 3.737-1.548h.854l-2.094-2.197.662-.681 2.825 2.898h3.94l2.822-2.898.665.681-2.093 2.197zm-10.42 5.094c-.818 0-1.482.663-1.482 1.482s.664 1.481 1.482 1.481c.819 0 1.482-.662 1.482-1.481s-.663-1.482-1.482-1.482zm7.214 0c-.818 0-1.482.663-1.482 1.482s.664 1.481 1.482 1.481c.819 0 1.482-.662 1.482-1.481s-.663-1.482-1.482-1.482z"/></svg>
              <span>哔哩哔哩</span>
            </a>
            <a href="https://xhslink.com/m/8u0JGkC5KUE" target="_blank" class="social-btn xiaohongshu" aria-label="Xiaohongshu">
              <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18"><path d="M19 2H6C4.9 2 4 2.9 4 4V20C4 21.1 4.9 22 6 22H19C20.1 22 21 21.1 21 20V4C21 2.9 20.1 2 19 2ZM19 20H6V4H19V20ZM10 9L12.5 7.5L15 9V4H10V9Z" /></svg>
              <span>小红书</span>
            </a>
          </div>
        </div>

        <!-- 右侧：导航组 -->
        <div class="nav-cols-wrapper">
          
          <!-- 列：主要站点 (卡片式 - 双列) -->
          <div class="nav-group wide-group">
            <h4 class="group-header">主要站点</h4>
            <div class="card-list">
              <a href="https://haruyuki.cn" target="_blank" class="nav-card">
                <span class="dot blue"></span>
                <span class="nav-text">凉宫春日应援团主站</span>
              </a>

              <a href="https://tts.haruyuki.cn" target="_blank" class="nav-card">
                <span class="dot blue"></span>
                <span class="nav-text">凉宫春日AI语音合成</span>
              </a>
              <a href="https://rvc.haruyuki.cn" target="_blank" class="nav-card">
                <span class="dot cyan"></span>
                <span class="nav-text">凉宫春日AI声线转换</span>
              </a>
              <a href="https://haruhifanclub.notion.site/anitabi" target="_blank" class="nav-card">
                <span class="dot yellow"></span>
                <span class="nav-text">圣地巡礼照片墙</span>
              </a>
              <a href="https://haruyuki.cn/library" target="_blank" class="nav-card">
                <span class="dot green"></span>
                <span class="nav-text">长门有希的书架</span>
              </a>
              <a href="https://anitool.haruyuki.cn/" target="_blank" class="nav-card">
                <span class="dot green"></span>
                <span class="nav-text">京阿尼语义化台词检索</span>
              </a>
            </div>
          </div>

          <!-- 列：加入我们 (胶囊式) -->
          <div class="nav-group">
            <h4 class="group-header">加入组织</h4>
            <div class="capsule-list">
              <a href="https://haruyuki.cn/news/handbook" target="_blank" class="capsule-btn blue">
                <span>团员手册</span>
                <span class="arrow">→</span>
              </a>
              <a href="https://qm.qq.com/q/CVssyL3Pj2" target="_blank" class="capsule-btn green">
                <span>超能力者群</span>
                <span class="arrow">→</span>
              </a>
              <a href="https://qm.qq.com/q/JcS7yXYoU2" target="_blank" class="capsule-btn purple">
                <span>异世界人群</span>
                <span class="arrow">→</span>
              </a>
              <a href="https://qm.qq.com/q/8nYNs7rFwA" target="_blank" class="capsule-btn orange">
                <span>宇宙人群</span>
                <span class="arrow">→</span>
              </a>
            </div>
          </div>

        </div>
      </div>

      <!-- 底部条 -->
      <div class="footer-bottom">
        <div class="bottom-left">
          <span class="copyright">© 2025 Haruhifanclub</span>
          <span class="divider"></span>
          <span class="slogan-mini">凉宫春日应援团开发组</span>
        </div>
        <div class="bottom-right">
          <span>此网站中的试卷由用户上传，内容不代表网站运营方的观点，网站仅供学习交流使用</span>
        </div>
      </div>
    </div>

    <!-- 悬浮回到顶部 -->
    <button type="button" @click="scrollToTop" class="elevator-btn" aria-label="回到顶部">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
        <line x1="12" y1="19" x2="12" y2="5"></line>
        <polyline points="5 12 12 5 19 12"></polyline>
      </svg>
    </button>
  </footer>
</template>

<style scoped>
/* =========================================
   Base Layout
   ========================================= */
.site-footer {
  width: 100%;
  margin-top: 60px;
  background-color: #ffffff;
  border-top: 1px solid #eaeaea;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  position: relative;
  z-index: 10;
}

.footer-inner {
  max-width: 1280px; /* 更宽的布局 */
  margin: 0 auto;
  padding: 0 32px;
}

/* =========================================
   Grid System
   ========================================= */
.footer-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 48px;
  padding: 64px 0 48px;
}

@media (min-width: 768px) {
  .footer-grid {
    grid-template-columns: 320px 1fr; /* 左侧固定宽，右侧自适应 */
    gap: 80px;
  }
}

/* =========================================
   Brand Column (Left)
   ========================================= */
.brand-col {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.brand-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-circle {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: #f5f5f5;
  border: 1px solid #e5e5e5;
  overflow: hidden;
  flex-shrink: 0;
}
.logo-img { width: 100%; height: 100%; object-fit: cover; }

.brand-name { font-size: 15px; font-weight: 700; color: #111; margin: 0; line-height: 1.2; }
.brand-slug { font-size: 11px; color: #888; margin: 0; font-weight: 500; letter-spacing: 0.5px; }

.brand-desc {
  font-size: 13px;
  color: #666;
  line-height: 1.6;
  margin: 8px 0;
}

/* Social Buttons */
.social-row { display: flex; gap: 10px; margin-top: 8px; }

.social-btn {
  /* 改为自适应宽度的胶囊样式 */
  padding: 8px 12px;
  border-radius: 99px;
  background: #f9f9f9;
  border: 1px solid #e5e5e5;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: #666;
  transition: all 0.2s ease;
  text-decoration: none;
  font-size: 13px;
  font-weight: 600;
}
.social-text-icon { font-size: 13px; font-weight: 800; }

.social-btn:hover {
  border-color: #d4d4d4;
  transform: translateY(-2px);
  color: #111;
  background: #fff;
  box-shadow: 0 4px 12px rgba(0,0,0,0.06);
}
.social-btn.bilibili:hover { color: #FB7299; border-color: #fbcce0; background: #fff5f8; }
.social-btn.xiaohongshu:hover { color: #FF2442; border-color: #ffccd4; background: #fff0f2; }


/* =========================================
   Navigation Columns (Right)
   ========================================= */
.nav-cols-wrapper {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 40px;
}

.nav-group {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 占据更多宽度的组，适合放双列卡片 */
.nav-group.wide-group {
  flex: 2; 
  min-width: 280px; /* 保证双列在窄屏不拥挤 */
}

.group-header {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  color: #999;
  letter-spacing: 0.8px;
  margin: 0 0 4px 0;
}

/* Style A: Nav Cards (Main Sites) - Two Columns */
.card-list {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.nav-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: #fbfbfb;
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  text-decoration: none;
  transition: all 0.2s ease;
  min-width: 0; /* 防止Flex子元素溢出 */
}

.nav-card:hover {
  background: #fff;
  border-color: #e0e0e0;
  box-shadow: 0 2px 8px rgba(0,0,0,0.03);
  transform: translateX(2px);
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot.blue { background: #3B82F6; }
.dot.cyan { background: #06B6D4; }
.dot.yellow { background: #EAB308; }
.dot.green { background: #10B981; }

.nav-text {
  font-size: 13px;
  font-weight: 500;
  color: #444;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.nav-card:hover .nav-text { color: #111; }

/* Style B: Capsule Buttons (Community) */
.capsule-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.capsule-btn {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-radius: 99px;
  font-size: 13px;
  font-weight: 500;
  text-decoration: none;
  border: 1px solid transparent;
  transition: all 0.2s ease;
}

.capsule-btn.green { background: #ecfdf5; color: #059669; }
.capsule-btn.purple { background: #f3e8ff; color: #7e22ce; }
.capsule-btn.orange { background: #ffedd5; color: #c2410c; }
.capsule-btn.blue { background: #eff6ff; color: #1d4ed8; }

.capsule-btn:hover {
  filter: brightness(0.95);
  transform: scale(1.02);
}

.arrow {
  font-size: 14px;
  opacity: 0;
  transform: translateX(-5px);
  transition: all 0.2s ease;
}
.capsule-btn:hover .arrow {
  opacity: 1;
  transform: translateX(0);
}


/* =========================================
   Footer Bottom
   ========================================= */
.footer-bottom {
  border-top: 1px solid #f0f0f0;
  padding: 24px 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
  font-size: 12px;
  color: #888;
}

@media (min-width: 640px) {
  .footer-bottom {
    flex-direction: row;
    justify-content: space-between;
  }
}

.bottom-left, .bottom-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.copyright { font-weight: 600; color: #333; }
.divider { width: 1px; height: 12px; background: #ddd; }
.dot-sep { font-weight: bold; }

/* =========================================
   Elevator Button
   ========================================= */
.elevator-btn {
  position: absolute;
  top: -24px;
  right: 32px;
  width: 48px;
  height: 48px;
  background: #fff;
  border: 1px solid #eaeaea;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #888;
  box-shadow: 0 4px 12px rgba(0,0,0,0.05);
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.elevator-btn:hover {
  transform: translateY(-6px);
  color: #111;
  border-color: #ccc;
  box-shadow: 0 10px 20px rgba(0,0,0,0.1);
}
</style>