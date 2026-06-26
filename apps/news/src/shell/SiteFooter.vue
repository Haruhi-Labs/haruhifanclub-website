<script setup>
// 设计系统统一页脚 .sos-footer（news 墨色 / 纯白纸主题）
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const show = computed(() => !route.name?.includes('editor') && !route.meta.hideNavbar)
const logoUrl = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

const scrollToTop = () => {
  const appContainer = document.getElementById('app')
  if (appContainer) appContainer.scrollTo({ top: 0, behavior: 'smooth' })
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

const groups = [
  {
    title: '主要站点',
    links: [
      { label: '春日团报', href: 'https://haruyuki.cn/news/', brand: '#3b82f6' },
      { label: '春日画廊', href: 'https://haruyuki.cn/art/', brand: '#14b8a6' },
      { label: '春日商城', href: 'https://haruyuki.cn/shop/', brand: '#0ea5e9' },
      { label: '长门有希的书架', href: 'https://haruyuki.cn/library/', brand: '#10b981' },
      { label: '凉宫春日语音合成', href: 'https://tts.haruyuki.cn/', brand: '#3b82f6' },
    ],
  },
  {
    title: '探索',
    links: [
      { label: '凉宫春日资源站', href: 'https://haruhifanclub.yuque.com/staff-sqlmik/phgf5z', brand: '#eab308' },
      { label: '凉宫春日声线转换', href: 'https://rvc.haruyuki.cn/', brand: '#06b6d4' },
      { label: '京阿尼台词检索', href: 'https://anitool.haruyuki.cn/', brand: '#10b981' },
      { label: '春日试卷中心', href: 'https://haruyuki.cn/exam/', brand: '#ef4444' },
      { label: '求学路', href: 'https://study.haruyuki.cn/', brand: '#f59e0b' },
    ],
  },
  {
    title: '游戏',
    links: [
      { label: '漫无止境的五月', href: 'https://crt.haruyuki.cn/', brand: '#a855f7' },
      { label: '无尽八月模拟器', href: 'https://loop.haruyuki.cn/', brand: '#0ea5e9' },
      { label: '射手座之日', href: 'https://star.haruyuki.cn/', brand: '#eab308' },
    ],
  },
  {
    title: '加入我们',
    links: [
      { label: '团员手册', href: 'https://haruyuki.cn/news/handbook', brand: '#3b82f6' },
      { label: '超能力者群', href: 'https://qm.qq.com/q/CVssyL3Pj2', brand: '#10b981' },
      { label: '异世界人群', href: 'https://qm.qq.com/q/JcS7yXYoU2', brand: '#a855f7' },
      { label: '未来人群', href: 'https://qm.qq.com/q/8nYNs7rFwA', brand: '#f97316' },
    ],
  },
]
</script>

<template>
  <footer v-if="show" class="sos-footer">
    <button type="button" class="sos-footer__totop" aria-label="回到顶部" @click="scrollToTop">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <polyline points="18 15 12 9 6 15" />
      </svg>
    </button>

    <div class="sos-footer__inner">
      <div class="sos-footer__top">
        <div class="sos-footer__brand">
          <span class="sos-brand-lockup">
            <span class="sos-brand-lockup__mark"><img :src="logoUrl" alt="" /></span>
            <span class="sos-brand-lockup__text">
              <strong>春日团报</strong>
              <small>Haruhi Fan Club</small>
            </span>
          </span>
          <p class="sos-footer__tagline">
            让世界变得更加热闹！《凉宫春日系列》的非营利粉丝团体。
          </p>
          <div class="sos-footer__social">
            <a class="sos-footer__social-link" style="--brand: #fb7299" href="https://space.bilibili.com/201296348" target="_blank" rel="noopener">哔哩哔哩</a>
            <a class="sos-footer__social-link" style="--brand: #ff2442" href="https://xhslink.com/m/8u0JGkC5KUE" target="_blank" rel="noopener">小红书</a>
          </div>
        </div>

        <div class="sos-footer__groups">
          <div v-for="g in groups" :key="g.title" class="sos-footer__group">
            <p class="sos-footer__group-title">{{ g.title }}</p>
            <div class="sos-footer__links">
              <a
                v-for="l in g.links"
                :key="l.href"
                class="sos-footer__link"
                :style="{ '--brand': l.brand }"
                :href="l.href"
                target="_blank"
                rel="noopener"
              >
                <span class="sos-footer__dot" aria-hidden="true"></span>{{ l.label }}
              </a>
            </div>
          </div>
        </div>
      </div>

      <div class="sos-footer__bottom">
        <span>© 2026 Haruhifanclub 凉宫春日应援团开发组</span>
        <div class="sos-footer__bottom-meta">
          <a class="sos-footer__link" href="https://beian.miit.gov.cn/#/Integrated/index" target="_blank" rel="noopener noreferrer">皖ICP备2025089290号-1</a>
        </div>
      </div>
    </div>
  </footer>
</template>
