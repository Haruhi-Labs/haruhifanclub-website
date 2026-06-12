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
  <footer
    class="relative mt-8 bg-[#FDFBF5] border-t border-[#E6E0D2] text-[#5B5048]"
  >
    <!-- 顶部很轻的上扬阴影，让 footer 独立出来但不抢视线 -->
    <div
      class="pointer-events-none absolute inset-x-6 -top-3 h-3 rounded-full bg-black/[0.04] blur-md opacity-80"
    ></div>

    <div class="mx-auto max-w-6xl px-4 sm:px-6">
      <!-- 上半部分：身份 & 链接 -->
      <div
        class="flex flex-col gap-8 py-10 md:flex-row md:items-start md:justify-between"
      >
        <!-- 左侧：项目身份 -->
        <div class="flex items-center gap-3 md:mt-1">
          <div
            class="h-12 w-12 flex items-center justify-center rounded-full bg-gradient-to-br from-[#FFE6C7] via-[#FAD5E5] to-[#D9E8FF] shadow-sm overflow-hidden shrink-0"
          >
            <img
              :src="logoSrc"
              alt="凉宫春日应援团"
              class="h-10 w-10 rounded-full object-cover opacity-90 hover:opacity-100 transition-opacity"
              onerror="this.style.display='none'"
            />
          </div>
          <div class="leading-tight">
            <div class="text-sm font-bold tracking-wide text-[#4A403A]">
              凉宫春日应援团
            </div>
            <div class="mt-0.5 text-xs text-[#9B8F80] font-mono tracking-tight">
              Haruhifanclub
            </div>
            <div class="mt-1 text-[10px] text-[#B8AC9A]">
              非营利的凉宫春日粉丝团体
            </div>
          </div>
        </div>

        <!-- 右侧：链接组 -->
        <div
          class="flex flex-col gap-8 text-xs md:flex-row md:items-start md:gap-12"
        >
          <!-- 社交平台 -->
          <div class="flex flex-col gap-3">
            <span class="text-[10px] font-bold tracking-[0.15em] text-[#C2B4A3] uppercase select-none">
              社交平台
            </span>
            <div class="flex flex-wrap gap-2.5">
              <a
                href="https://space.bilibili.com/201296348"
                target="_blank"
                class="inline-flex items-center gap-1.5 rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#FB7299] transition-all text-[#6B6058]"
              >
                <span>哔哩哔哩</span>
              </a>
              <a
                href="https://xhslink.com/m/8u0JGkC5KUE"
                target="_blank"
                class="inline-flex items-center gap-1.5 rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#FF2442] transition-all text-[#6B6058]"
              >
                <span>小红书</span>
              </a>
            </div>
          </div>

          <!-- 主要站点 -->
          <div class="flex flex-col gap-3">
            <span class="text-[10px] font-bold tracking-[0.15em] text-[#C2B4A3] uppercase select-none">
              主要站点
            </span>
            <div class="flex flex-wrap gap-2.5 max-w-xs">
              <a
                href="https://tts.haruyuki.cn"
                target="_blank"
                class="inline-flex items-center rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#3B82F6] transition-all text-[#6B6058]"
              >
                AI语音
              </a>
              <a
                href="https://rvc.haruyuki.cn"
                target="_blank"
                class="inline-flex items-center rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#06B6D4] transition-all text-[#6B6058]"
              >
                AI变声
              </a>
              <a
                href="https://haruhifanclub.notion.site/anitabi"
                target="_blank"
                class="inline-flex items-center rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#EAB308] transition-all text-[#6B6058]"
              >
                圣地巡礼
              </a>
              <a
                href="https://haruyuki.cn/library"
                target="_blank"
                class="inline-flex items-center rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#10B981] transition-all text-[#6B6058]"
              >
                长门书架
              </a>
            </div>
          </div>

          <!-- 加入我们 -->
          <div class="flex flex-col gap-3">
            <span class="text-[10px] font-bold tracking-[0.15em] text-[#C2B4A3] uppercase select-none">
              加入我们
            </span>
            <div class="flex flex-wrap gap-2.5">
              <a
                href="https://qm.qq.com/q/CVssyL3Pj2"
                target="_blank"
                class="inline-flex items-center rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#10B981] transition-all text-[#6B6058]"
              >
                超能力者
              </a>
              <a
                href="https://qm.qq.com/q/JcS7yXYoU2"
                target="_blank"
                class="inline-flex items-center rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#A855F7] transition-all text-[#6B6058]"
              >
                异世界人
              </a>
              <a
                href="https://qm.qq.com/q/8nYNs7rFwA"
                target="_blank"
                class="inline-flex items-center rounded-full bg-white/60 px-3 py-1.5 shadow-sm ring-1 ring-black/[0.03] hover:bg-white hover:ring-black/[0.08] hover:text-[#F97316] transition-all text-[#6B6058]"
              >
                未来人
              </a>
            </div>
          </div>
        </div>
      </div>

      <!-- 下半部分：版权 & 说明 -->
      <div
        class="flex flex-col items-center justify-between gap-3 border-t border-[#EAE0D0] py-6 text-[10px] text-[#A89A8A] sm:flex-row"
      >
        <div class="text-center sm:text-left font-medium">
          © 2025 HARUHIFANCLUB · Non-profit fan project
        </div>
        <div class="flex items-center gap-3 opacity-80">
           <span>非营利项目</span>
           <span class="w-px h-2.5 bg-[#D4C8B8]"></span>
           <span>仅供交流</span>
         </div>
      </div>
    </div>

    <!-- 回到顶部按钮 -->
    <button
      type="button"
      @click="scrollToTop"
      class="group absolute -top-4 left-1/2 flex h-8 w-8 -translate-x-1/2 items-center justify-center rounded-full bg-white shadow-md ring-1 ring-black/[0.06] hover:-translate-y-1 hover:bg-[#FFFBF5] hover:shadow-lg transition-all duration-300 z-10"
      aria-label="回到顶部"
    >
      <svg
        class="h-4 w-4 text-[#C26B57] group-hover:text-[#B4543E] transition-colors"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="18 15 12 9 6 15" />
      </svg>
    </button>
  </footer>
</template>

<style scoped>
/* 此处无需额外样式，完全依赖 Tailwind Utility 类。
  背景色 #FDFBF5 是一种非常淡的米黄色，营造温暖氛围。
  文字颜色 #5B5048 是深褐灰色，保证可读性且不刺眼。
*/
</style>
