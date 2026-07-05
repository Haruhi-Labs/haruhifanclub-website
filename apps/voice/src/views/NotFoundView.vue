<script setup>
import { RouterLink } from 'vue-router'
</script>

<template>
  <div class="vo-page vo-page--narrow vo-nf">
    <p class="vo-nf__code">404</p>
    <h1 class="vo-nf__title">页面走丢了</h1>
    <p class="vo-nf__sub">这里没有对应的页面。回工坊首页看看，或直接去合成一句台词。</p>
    <div class="vo-nf__actions">
      <RouterLink to="/" class="sos-button sos-button--primary">返回工坊</RouterLink>
      <RouterLink to="/tts" class="sos-button sos-button--secondary">语音合成</RouterLink>
    </div>
  </div>
</template>

<style scoped>
.vo-nf {
  text-align: center;
  padding-block: var(--sos-space-12, 6rem);
}
.vo-nf__code {
  margin: 0;
  font-size: clamp(3rem, 12vw, 6rem);
  font-weight: 900;
  line-height: 1;
  color: var(--sos-accent);
}
.vo-nf__title {
  margin: var(--sos-space-3) 0 0;
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-2xl);
  color: var(--sos-text-primary);
}
.vo-nf__sub {
  margin: var(--sos-space-3) auto 0;
  max-width: 40ch;
  color: var(--sos-text-secondary);
}
.vo-nf__actions {
  margin-top: var(--sos-space-6);
  display: flex;
  gap: var(--sos-space-3);
  justify-content: center;
  flex-wrap: wrap;
}
.vo-nf__actions a {
  text-decoration: none;
}
</style>
