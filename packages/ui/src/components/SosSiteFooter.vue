<script setup lang="ts">
interface FooterLink {
  label: string
  href: string
  brand?: string
}

interface FooterGroup {
  title: string
  links: FooterLink[]
}

withDefaults(
  defineProps<{
    logoSrc?: string
    logoAlt?: string
    brandTitle?: string
    brandSubtitle?: string
    tagline?: string
    groups?: FooterGroup[]
    socialLinks?: FooterLink[]
    bottomLinks?: FooterLink[]
  }>(),
  {
    logoSrc: undefined,
    logoAlt: '',
    brandTitle: '凉宫春日应援团',
    brandSubtitle: 'Haruhifanclub',
    tagline: '让世界变得更加热闹！《凉宫春日系列》的非营利粉丝团体。',
    groups: () => [],
    socialLinks: () => [
      { label: '哔哩哔哩', href: 'https://space.bilibili.com/201296348', brand: '#fb7299' },
      { label: '小红书', href: 'https://xhslink.com/m/8u0JGkC5KUE', brand: '#ff2442' },
    ],
    bottomLinks: () => [],
  }
)

const year = new Date().getFullYear()

function scrollToTop() {
  const appContainer = document.getElementById('app')
  if (appContainer) appContainer.scrollTo({ top: 0, behavior: 'smooth' })
  window.scrollTo({ top: 0, behavior: 'smooth' })
}
</script>

<template>
  <footer class="sos-footer">
    <button type="button" class="sos-footer__totop" aria-label="回到顶部" @click="scrollToTop">
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.4"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <polyline points="18 15 12 9 6 15" />
      </svg>
    </button>

    <div class="sos-footer__inner">
      <div class="sos-footer__top">
        <div class="sos-footer__brand">
          <span class="sos-brand-lockup">
            <span v-if="logoSrc" class="sos-brand-lockup__mark">
              <img :src="logoSrc" :alt="logoAlt" />
            </span>
            <span class="sos-brand-lockup__text">
              <strong>{{ brandTitle }}</strong>
              <small>{{ brandSubtitle }}</small>
            </span>
          </span>
          <p class="sos-footer__tagline">{{ tagline }}</p>
          <div v-if="socialLinks.length" class="sos-footer__social">
            <a
              v-for="link in socialLinks"
              :key="link.href"
              class="sos-footer__social-link"
              :style="{ '--brand': link.brand }"
              :href="link.href"
              target="_blank"
              rel="noopener"
            >
              {{ link.label }}
            </a>
          </div>
        </div>

        <div class="sos-footer__groups">
          <div v-for="group in groups" :key="group.title" class="sos-footer__group">
            <p class="sos-footer__group-title">{{ group.title }}</p>
            <div class="sos-footer__links">
              <a
                v-for="link in group.links"
                :key="link.href"
                class="sos-footer__link"
                :style="{ '--brand': link.brand }"
                :href="link.href"
                target="_blank"
                rel="noopener"
              >
                <span class="sos-footer__dot" aria-hidden="true"></span>{{ link.label }}
              </a>
            </div>
          </div>
        </div>
      </div>

      <div class="sos-footer__bottom">
        <span>
          © {{ year }}
          <a
            class="sos-footer__link"
            href="https://github.com/Haruhi-Labs"
            target="_blank"
            rel="noopener"
          >
            Haruhi-Labs
          </a>
          · 凉宫春日应援团开发组
        </span>
        <div v-if="bottomLinks.length" class="sos-footer__bottom-meta">
          <template v-for="(link, index) in bottomLinks" :key="link.href">
            <span v-if="index" class="sos-footer__bottom-sep" aria-hidden="true"></span>
            <a class="sos-footer__link" :href="link.href" target="_blank" rel="noopener noreferrer">
              {{ link.label }}
            </a>
          </template>
        </div>
      </div>
    </div>
  </footer>
</template>
