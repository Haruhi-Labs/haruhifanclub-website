<script setup>
/**
 * ShopAuthLayout —— 商城账号系统（登录 / 资料 / 设置 / 邮件落地）的专门布局。
 * shop 自带的 ShopLayout 页头较重（导航 / 购物车 / 移动菜单），不适合账号独立全页；
 * 这里提供一个精简的专门页头 + 满屏 shop 主题背景：
 *   - .sos-scope[data-sos-site=shop] 绘制 shop 的 bg-page，min-height:100dvh 铺满整屏，
 *     消除此前登录页「下半截露出未主题化 body、颜色不同的底」的问题；
 *   - 精简品牌锁头（春日商城 + 返回商城），补上之前缺失的通用页头。
 */
import { RouterLink, RouterView } from 'vue-router'

const logoUrl = `${import.meta.env.BASE_URL}haruhi-logo-192.png`
</script>

<template>
  <div class="shop-auth sos-scope" data-sos-site="shop">
    <header class="shop-auth__header">
      <RouterLink class="shop-auth__brand" to="/" aria-label="返回春日商城首页">
        <img class="shop-auth__logo" :src="logoUrl" alt="" />
        <span class="shop-auth__brand-text">
          <strong>春日商城</strong>
          <small>凉宫春日应援团 · 商城</small>
        </span>
      </RouterLink>
      <RouterLink class="shop-auth__back" to="/">返回商城 →</RouterLink>
    </header>

    <main class="shop-auth__body">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.shop-auth {
  /* .sos-scope 已绘制 shop 主题背景（bg-page），铺满整屏以消除登录页两色底 */
  min-height: 100dvh;
  display: flex;
  flex-direction: column;
}

.shop-auth__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-4);
  padding: var(--sos-space-4) clamp(1rem, 4vw, 2.5rem);
  background: color-mix(in srgb, var(--sos-bg-surface) 88%, transparent);
  border-bottom: 1px solid var(--sos-border-subtle);
  backdrop-filter: blur(8px);
}

.shop-auth__brand {
  display: inline-flex;
  align-items: center;
  gap: var(--sos-space-3);
  text-decoration: none;
  color: var(--sos-text-primary);
}
.shop-auth__logo {
  width: 2.5rem;
  height: 2.5rem;
  border-radius: var(--sos-radius-full);
  object-fit: cover;
  flex: 0 0 auto;
}
.shop-auth__brand-text {
  display: grid;
  gap: 2px;
  line-height: 1.2;
}
.shop-auth__brand-text > strong {
  font-size: var(--sos-text-lg);
  font-weight: var(--sos-weight-bold);
}
.shop-auth__brand-text > small {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}

.shop-auth__back {
  flex: 0 0 auto;
  font-size: var(--sos-text-sm);
  font-weight: var(--sos-weight-semibold);
  color: var(--sos-link);
  text-decoration: none;
}
.shop-auth__back:hover {
  color: var(--sos-link-hover);
  text-decoration: underline;
}

.shop-auth__body {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

@media (max-width: 560px) {
  .shop-auth__brand-text > small {
    display: none;
  }
}
</style>
