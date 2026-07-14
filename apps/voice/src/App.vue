<script setup>
import { onMounted } from 'vue'
import { RouterView, RouterLink } from 'vue-router'
import { SosAppbar } from '@haruhi/ui'
import { AccountMenu, AccountAvatarLink } from '@haruhi/auth-ui'
import SiteFooter from '@/components/SiteFooter.vue'
import { session } from '@/api'
import { ensureRoles, startStatusPolling } from '@/lib/store'

const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

onMounted(() => {
  ensureRoles()
  startStatusPolling()
  session.ensureReady()
})
</script>

<template>
  <div class="vo-app sos-scope" data-sos-site="voice">
    <SosAppbar>
      <template #brand>
        <RouterLink to="/" class="sos-brand-lockup">
          <span class="sos-brand-lockup__mark"><img :src="logoSrc" alt="" /></span>
          <span class="sos-brand-lockup__text">
            <strong>春日语音工坊</strong>
            <small>凉宫春日应援团 · 声音实验室</small>
          </span>
        </RouterLink>
      </template>

      <nav class="sos-navlinks">
        <RouterLink to="/" class="sos-navlink" exact-active-class="router-link-active">工坊</RouterLink>
        <RouterLink to="/tts" class="sos-navlink">语音合成</RouterLink>
        <RouterLink to="/rvc" class="sos-navlink">声线转换</RouterLink>
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

    <main class="vo-main">
      <RouterView />
    </main>

    <SiteFooter />
  </div>
</template>
