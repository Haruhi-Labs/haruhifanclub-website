<script setup>
import { computed, onMounted } from 'vue'
import { RouterView, RouterLink, useRoute } from 'vue-router'
import { SosAppbar, SosToastRegion } from '@haruhi/ui'
import { AccountMenu, AccountAvatarLink } from '@haruhi/auth-ui'
import SiteFooter from '@/components/SiteFooter.vue'
import { session } from '@/api'

const route = useRoute()
// 阅读页沉浸全屏：隐藏页头页脚
const immersive = computed(() => route.name === 'read')
const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

onMounted(() => session.ensureReady())
</script>

<template>
  <div class="fiction-app sos-scope" data-sos-site="library">
    <SosAppbar v-if="!immersive">
      <template #brand>
        <RouterLink to="/" class="sos-brand-lockup">
          <span class="sos-brand-lockup__mark"><img :src="logoSrc" alt="" /></span>
          <span class="sos-brand-lockup__text">
            <strong>同人文库</strong>
            <small>凉宫春日应援团 · 同人小说</small>
          </span>
        </RouterLink>
      </template>

      <nav class="sos-navlinks">
        <RouterLink to="/" class="sos-navlink">首页</RouterLink>
        <RouterLink to="/library" class="sos-navlink">书库</RouterLink>
        <RouterLink to="/write" class="sos-navlink">创作</RouterLink>
      </nav>

      <template #actions>
        <RouterLink to="/write" class="sos-btn sos-btn--primary sos-btn--sm fiction-write-cta">
          ✎ 写小说
        </RouterLink>
        <AccountMenu login-path="/login" profile-path="/account" settings-path="/account/settings" home="/" />
      </template>

      <template #mobile-lead>
        <AccountAvatarLink profile-path="/account" />
      </template>
    </SosAppbar>

    <main class="fiction-main">
      <RouterView />
    </main>

    <SiteFooter v-if="!immersive" />
    <SosToastRegion />
  </div>
</template>
