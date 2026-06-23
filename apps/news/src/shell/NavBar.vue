<template>
  <header>
    <div v-if="!overlay" class="nav-spacer"></div>

    <nav
      :class="[
        'navbar',
        isHidden ? 'navbar--hidden' : 'navbar--visible',
        isScrolled ? 'navbar--scrolled' : 'navbar--top',
        overlay ? 'navbar--overlay' : 'navbar--default',
      ]"
    >
      <div class="nav-container">
        <!-- Logo 区域 -->
        <div class="nav-left">
          <button class="logo-group" type="button" @click="goHome" aria-label="返回春日团报首页">
            <SosHeaderBrand
              as="span"
              logo-src="/haruhi-logo-192.png"
              title="春日团报"
              subtitle="Haruhi Fan Club"
              :compact="isScrolled || overlay"
            />
          </button>

          <div :class="['nav-links', overlay ? 'nav-links--overlay' : 'nav-links--default']">
            <router-link to="/submit" class="nav-link">我要投稿</router-link>
            <!-- [新增] 活动列表入口 -->
            <router-link to="/activity" class="nav-link nav-link--with-icon">
              <span>活动中心</span>
            </router-link>
            <router-link to="/store" class="nav-link nav-link--with-icon">
              <span>奖品兑换</span>
            </router-link>
            <router-link to="/admin" class="nav-link">管理后台</router-link>
          </div>
        </div>

        <!-- 右侧功能区 -->
        <div class="nav-right">
          <!-- 新增：SOS团期末考试 -->
          <!-- exam 是独立 app（/exam/），必须用原生 a 整页跳转；router-link 会被解析进 /news/ base 导致跳错 -->
          <a
            href="/exam/"
            target="_blank"
            class="nav-right-link"
            :class="overlay ? 'nav-right-link--overlay' : 'nav-right-link--default'"
          >
            SOS团期末考试
          </a>

          <!-- 修改：去掉了 emoji -->
          <router-link
            :to="{ name: 'quiz' }"
            target="_blank"
            class="nav-right-link"
            :class="overlay ? 'nav-right-link--overlay' : 'nav-right-link--default'"
          >
            凉宫入坑测试
          </router-link>

          <router-link
            to="/handbook"
            class="nav-right-link nav-right-link--no-margin"
            :class="overlay ? 'nav-right-link--overlay' : 'nav-right-link--default'"
          >
            团员手册
          </router-link>

          <div
            :class="['nav-divider', overlay ? 'nav-divider--overlay' : 'nav-divider--default']"
          ></div>

          <button
            @click="store.toggleSearch"
            aria-label="搜索团报"
            :class="[
              'search-button',
              overlay ? 'search-button--overlay' : 'search-button--default',
            ]"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="1.5"
              stroke="currentColor"
              class="icon-sm"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
              />
            </svg>
            <span class="search-button-label">搜索团报</span>
          </button>

          <AccountMenu />

          <button
            @click="toggleMobileMenu"
            class="mobile-menu-button"
            :class="overlay ? 'mobile-menu-button--overlay' : 'mobile-menu-button--default'"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="1.5"
              stroke="currentColor"
              class="icon-md"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
              />
            </svg>
          </button>
        </div>
      </div>
    </nav>

    <Transition name="fade">
      <div v-if="isMobileMenuOpen" class="mobile-overlay">
        <button @click="toggleMobileMenu" class="mobile-close-button">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="icon-lg"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>

        <nav class="mobile-nav serif-font">
          <router-link to="/" @click="toggleMobileMenu" class="mobile-nav-link">首页</router-link>

          <!-- 移动端菜单也添加了考试入口 -->
          <a href="/exam/" target="_blank" @click="toggleMobileMenu" class="mobile-nav-link"
            >SOS团期末考试</a
          >

          <router-link to="/activity" @click="toggleMobileMenu" class="mobile-nav-link"
            >活动中心</router-link
          >
          <router-link to="/store" @click="toggleMobileMenu" class="mobile-nav-link"
            >奖品兑换</router-link
          >

          <router-link
            :to="{ name: 'quiz' }"
            target="_blank"
            @click="toggleMobileMenu"
            class="mobile-nav-link mobile-nav-link--with-icon"
          >
            <span>凉宫入坑测试</span>
          </router-link>

          <router-link to="/handbook" @click="toggleMobileMenu" class="mobile-nav-link"
            >团员手册</router-link
          >
          <router-link to="/submit" @click="toggleMobileMenu" class="mobile-nav-link"
            >我要投稿</router-link
          >
          <router-link to="/admin" @click="toggleMobileMenu" class="mobile-nav-link"
            >管理后台</router-link
          >
        </nav>

        <div class="mobile-footer">- 凉宫春日应援团 -</div>
      </div>
    </Transition>
  </header>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useMainStore } from '@/stores/main'
import { AccountMenu } from '@haruhi/auth-ui'
import { SosHeaderBrand } from '@haruhi/ui'

const props = defineProps({
  overlay: {
    type: Boolean,
    default: false,
  },
})

const store = useMainStore()
const router = useRouter()

const isScrolled = ref(false)
const isHidden = ref(false)
const isMobileMenuOpen = ref(false)

let lastScrollY = 0

const goHome = () => {
  store.searchQuery = ''
  router.push('/')
}

const toggleMobileMenu = () => {
  isMobileMenuOpen.value = !isMobileMenuOpen.value
  if (isMobileMenuOpen.value) {
    document.body.style.overflow = 'hidden'
  } else {
    document.body.style.overflow = ''
  }
}

const handleScroll = () => {
  const currentScrollY = window.scrollY
  isScrolled.value = currentScrollY > 20
  if (currentScrollY > lastScrollY && currentScrollY > 100) {
    isHidden.value = true
  } else {
    isHidden.value = false
  }
  lastScrollY = currentScrollY
}

onMounted(() => window.addEventListener('scroll', handleScroll))
onUnmounted(() => window.removeEventListener('scroll', handleScroll))
</script>

<style scoped>
.serif-font {
  font-family: 'Noto Serif SC', 'Songti SC', serif;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* ===== Nav Spacer ===== */
.nav-spacer {
  width: 100%;
  height: 5rem;
  background-color: transparent;
}

@media (min-width: 768px) {
  .nav-spacer {
    height: 6rem;
  }
}

/* ===== Navbar ===== */
.navbar {
  position: fixed;
  z-index: 40;
  width: 100%;
  transition: all 500ms ease-in-out;
}

.navbar--hidden {
  transform: translateY(-100%);
}

.navbar--visible {
  transform: translateY(0);
}

.navbar--scrolled {
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.navbar--top {
  padding-top: 1.25rem;
  padding-bottom: 1.25rem;
}

.navbar--overlay {
  top: 0;
  left: 0;
  background-color: transparent;
  border-bottom: 1px solid transparent;
  color: #ffffff;
}

.navbar--default {
  top: 0;
  background-color: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid #f3f4f6;
  color: #111827;
}

/* ===== Nav Container ===== */
.nav-container {
  max-width: 1600px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1rem;
  padding-right: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

@media (min-width: 768px) {
  .nav-container {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

/* ===== Nav Left ===== */
.nav-left {
  display: flex;
  align-items: center;
  gap: 1rem;
}

@media (min-width: 768px) {
  .nav-left {
    gap: 2rem;
  }
}

/* ===== Logo Group ===== */
.logo-group {
  display: inline-flex;
  align-items: center;
  border: 0;
  background: transparent;
  padding: 0;
  cursor: pointer;
}

.logo-group:hover {
  opacity: 0.8;
}

.navbar--overlay :deep(.sos-brand-lockup__text > strong),
.navbar--overlay :deep(.sos-brand-lockup__text > small) {
  color: #ffffff;
}

.navbar--overlay :deep(.sos-brand-lockup__mark) {
  background: #ffffff;
}

/* ===== Nav Links (Desktop) ===== */
.nav-links {
  display: none;
  gap: 1.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  transition: color 150ms;
}

@media (min-width: 768px) {
  .nav-links {
    display: flex;
  }
}

.nav-links--overlay {
  color: rgba(255, 255, 255, 0.9);
}

.nav-links--default {
  color: #4b5563;
}

/* ===== Nav Link ===== */
.nav-link {
  opacity: 0.8;
  transition: opacity 150ms;
  padding-top: 0.25rem;
  padding-bottom: 0.25rem;
}

.nav-link:hover {
  opacity: 1;
}

/* ===== Nav Link with Icon ===== */
.nav-link--with-icon {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

/* ===== Nav Right ===== */
.nav-right {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

@media (min-width: 768px) {
  .nav-right {
    gap: 1.25rem;
  }
}

/* ===== Nav Right Link (Desktop) ===== */
.nav-right-link {
  display: none;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  transition: opacity 150ms;
  margin-right: 0.5rem;
}

@media (min-width: 768px) {
  .nav-right-link {
    display: flex;
  }
}

.nav-right-link:hover {
  opacity: 0.8;
}

.nav-right-link--overlay {
  color: #ffffff;
}

.nav-right-link--default {
  color: #4b5563;
}

.nav-right-link--no-margin {
  margin-right: 0;
}

/* ===== Nav Divider ===== */
.nav-divider {
  display: none;
  height: 1rem;
  width: 1px;
}

@media (min-width: 768px) {
  .nav-divider {
    display: block;
  }
}

.nav-divider--overlay {
  background-color: rgba(255, 255, 255, 0.3);
}

.nav-divider--default {
  background-color: #d1d5db;
}

/* ===== Search Button ===== */
.search-button {
  display: flex;
  align-items: center;
  transition: all 300ms;
  width: 2.25rem;
  height: 2.25rem;
  justify-content: center;
  border-radius: 9999px;
  border-width: 1px;
  border-style: solid;
}

.search-button-label {
  display: none;
  font-size: 0.875rem;
  font-weight: 700;
}

@media (min-width: 768px) {
  .search-button {
    width: 16rem;
    padding-left: 1rem;
    padding-right: 1rem;
    padding-top: 0.5rem;
    padding-bottom: 0.5rem;
    justify-content: space-between;
  }

  .search-button-label {
    display: inline;
  }
}

.search-button--overlay {
  background-color: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.3);
  color: #ffffff;
}

.search-button--default {
  background-color: #f9fafb;
  border-color: #e5e7eb;
  color: #6b7280;
}

/* ===== Mobile Menu Button ===== */
.mobile-menu-button {
  width: 2.25rem;
  height: 2.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  transition: background-color 150ms;
}

@media (min-width: 768px) {
  .mobile-menu-button {
    display: none;
  }
}

.mobile-menu-button--overlay {
  color: #ffffff;
}

.mobile-menu-button--overlay:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.mobile-menu-button--default {
  color: #000000;
}

.mobile-menu-button--default:hover {
  background-color: #f3f4f6;
}

/* ===== Icons ===== */
.icon-sm {
  width: 1rem;
  height: 1rem;
}

.icon-md {
  width: 1.5rem;
  height: 1.5rem;
}

.icon-lg {
  width: 2rem;
  height: 2rem;
}

/* ===== Mobile Overlay ===== */
.mobile-overlay {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  z-index: 50;
  background-color: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(24px);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

/* ===== Mobile Close Button ===== */
.mobile-close-button {
  position: absolute;
  top: 1.5rem;
  right: 1rem;
  padding: 0.5rem;
  color: #6b7280;
}

.mobile-close-button:hover {
  color: #000000;
}

/* ===== Mobile Nav ===== */
.mobile-nav {
  display: flex;
  flex-direction: column;
  gap: 2rem;
  font-size: 1.5rem;
  font-weight: 900;
}

/* ===== Mobile Nav Link ===== */
.mobile-nav-link {
  transition: color 150ms;
}

.mobile-nav-link:hover {
  color: #111111;
  text-decoration: underline;
  text-decoration-color: #ffc83d;
  text-decoration-thickness: 0.2em;
  text-underline-offset: 0.2em;
}

.mobile-nav-link--with-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

/* ===== Mobile Footer ===== */
.mobile-footer {
  margin-top: 3rem;
  font-size: 0.875rem;
  color: #9ca3af;
  font-family: 'Noto Sans SC', sans-serif;
}
</style>
