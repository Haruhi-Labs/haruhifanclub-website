<script setup>
import { RouterLink } from 'vue-router'
const year = new Date().getFullYear()
</script>

<template>
  <footer class="fiction-footer">
    <div class="fiction-footer__inner">
      <div class="fiction-footer__brand">
        <strong>同人文库</strong>
        <p>凉宫春日应援团出品 · 用文字延续那个夏天</p>
      </div>
      <nav class="fiction-footer__links">
        <RouterLink to="/library">书库</RouterLink>
        <RouterLink to="/write">开始创作</RouterLink>
        <RouterLink to="/account">个人中心</RouterLink>
        <a href="/news/">团报主站</a>
      </nav>
    </div>
    <div class="fiction-footer__legal">
      <span>© {{ year }} 凉宫春日应援团</span>
      <span>同人创作 · 仅供爱好者交流，请勿商用</span>
    </div>
  </footer>
</template>
