<script setup>
import { RouterLink } from 'vue-router'
</script>

<template>
  <div class="dl-page dl-page--narrow dl-nf">
    <p class="dl-nf__code">404</p>
    <h1 class="dl-nf__title">页面走丢了</h1>
    <p class="dl-nf__sub">这里没有对应的资源页面。回首页看看，或到目录里浏览全部资源。</p>
    <div class="dl-nf__actions">
      <RouterLink to="/" class="sos-button sos-button--primary">返回首页</RouterLink>
      <RouterLink to="/browse" class="sos-button sos-button--secondary">浏览目录</RouterLink>
    </div>
  </div>
</template>

<style scoped>
.dl-nf {
  text-align: center;
  padding-block: var(--sos-space-12, 6rem);
}
.dl-nf__code {
  margin: 0;
  font-size: clamp(3rem, 12vw, 6rem);
  font-weight: 900;
  line-height: 1;
  color: var(--sos-accent);
}
.dl-nf__title {
  margin: var(--sos-space-3) 0 0;
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-2xl);
  color: var(--sos-text-primary);
}
.dl-nf__sub {
  margin: var(--sos-space-3) auto 0;
  max-width: 40ch;
  color: var(--sos-text-secondary);
}
.dl-nf__actions {
  margin-top: var(--sos-space-6);
  display: flex;
  gap: var(--sos-space-3);
  justify-content: center;
  flex-wrap: wrap;
}
.dl-nf__actions a {
  text-decoration: none;
}
</style>
