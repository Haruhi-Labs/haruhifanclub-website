<script setup>
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { SosCard, SosButton, SosNotice, SosSpinner } from '@haruhi/ui'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  home: { type: String, default: '/' },
  title: { type: String, default: '凉宫春日应援团' },
  site: { type: String, default: undefined },
})

const session = useSession(props.apiBase)
const route = useRoute()

const state = ref('loading') // loading | ok | error
const error = ref('')

onMounted(async () => {
  const token = String((route.query && route.query.token) || '')
  if (!token) {
    state.value = 'error'
    error.value = '链接无效：缺少令牌。'
    return
  }
  try {
    await session.verifyEmail(token)
    await session.refresh()
    state.value = 'ok'
  } catch (e) {
    state.value = 'error'
    error.value = e?.message || '验证失败，链接可能已过期。'
  }
})
</script>

<template>
  <div class="hauth-root sos-scope hauth-shell" :data-sos-site="site">
    <SosCard class="hauth-card" as="section">
      <header class="hauth-brand">
        <span class="hauth-brand__mark" aria-hidden="true">{{ title.slice(0, 1) }}</span>
        <h1 class="sos-title" style="font-size: var(--sos-text-2xl)">邮箱验证</h1>
      </header>

      <div
        v-if="state === 'loading'"
        class="sos-inline"
        style="justify-content: center; margin-top: var(--sos-space-4)"
      >
        <SosSpinner />
        <span class="sos-copy sos-copy--small">正在验证…</span>
      </div>

      <template v-else-if="state === 'ok'">
        <SosNotice tone="success" title="验证成功" style="margin-top: var(--sos-space-4)">
          你的邮箱已验证，现在可以发布内容了。
        </SosNotice>
        <SosButton
          as="a"
          :href="home"
          class="sos-button--block"
          style="margin-top: var(--sos-space-5)"
        >
          返回首页
        </SosButton>
      </template>

      <template v-else>
        <SosNotice tone="danger" title="验证失败" style="margin-top: var(--sos-space-4)">
          {{ error }}
        </SosNotice>
        <SosButton
          as="a"
          :href="home"
          variant="secondary"
          class="sos-button--block"
          style="margin-top: var(--sos-space-5)"
        >
          返回首页
        </SosButton>
      </template>
    </SosCard>
  </div>
</template>
