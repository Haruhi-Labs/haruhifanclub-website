<script setup>
import { onMounted } from 'vue'
import { RouterView, RouterLink } from 'vue-router'
import { SosAppbar } from '@haruhi/ui'
import { AccountMenu, AccountAvatarLink } from '@haruhi/auth-ui'
import SiteFooter from '@/components/SiteFooter.vue'
import { session } from '@/api'
import { ensureIndex } from '@/lib/store'

const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

onMounted(() => {
  ensureIndex()
  session.ensureReady()
})
</script>

<template>
  <div class="dl-app sos-scope" data-sos-site="download">
    <SosAppbar>
      <template #brand>
        <RouterLink to="/" class="sos-brand-lockup">
          <span class="sos-brand-lockup__mark"><img :src="logoSrc" alt="" /></span>
          <span class="sos-brand-lockup__text">
            <strong>凉宫春日资源站</strong>
            <small>凉宫春日应援团 · 资源索引</small>
          </span>
        </RouterLink>
      </template>

      <nav class="sos-navlinks">
        <RouterLink to="/" class="sos-navlink">索引</RouterLink>
      </nav>

      <template #actions>
        <AccountMenu
          login-path="/login"
          profile-path="/account"
          settings-path="/account/settings"
          home="/"
        />
      </template>

      <template #mobile-lead>
        <AccountAvatarLink profile-path="/account" />
      </template>
    </SosAppbar>

    <main class="dl-main">
      <RouterView />
    </main>

    <SiteFooter />
  </div>
</template>
