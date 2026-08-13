<script setup>
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { SosCard, SosNotice } from '@haruhi/ui'

const route = useRoute()
const error = ref('')

onMounted(() => {
  const keys = ['client_id', 'redirect_uri', 'code_challenge', 'code_challenge_method', 'state']
  const missing = keys.filter((key) => typeof route.query[key] !== 'string' || !route.query[key])
  if (missing.length > 0) {
    error.value = '登录请求缺少必要参数，请返回游戏后重试。'
    return
  }

  // 这里只把受信任客户端参数交给后端；回调白名单、PKCE 和一次性授权码均由后端校验。
  const authorize = new URL('/api/auth/game/authorize', window.location.origin)
  for (const key of keys) authorize.searchParams.set(key, String(route.query[key]))
  window.location.replace(authorize.toString())
})
</script>

<template>
  <main class="game-authorize sos-scope">
    <SosCard class="game-authorize__card" as="section">
      <p class="game-authorize__eyebrow">统一身份认证</p>
      <h1>正在返回凉宫春日游戏站</h1>
      <p>将只向游戏提供账号 ID、昵称和头像，不会提供邮箱或主站会话。</p>
      <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
      <p v-else class="game-authorize__loading" role="status">正在生成一次性授权码…</p>
    </SosCard>
  </main>
</template>

<style scoped>
.game-authorize {
  min-height: 72vh;
  display: grid;
  place-items: center;
  padding: 2rem 1rem;
  background: radial-gradient(circle at top, #f5f8ff 0, #fff 58%);
}

.game-authorize__card {
  width: min(32rem, 100%);
  display: grid;
  gap: 1rem;
  text-align: center;
  padding: clamp(1.5rem, 5vw, 2.5rem);
}

.game-authorize__eyebrow {
  color: #315ec9;
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: 0.12em;
}

h1 {
  margin: 0;
  font-size: clamp(1.45rem, 4vw, 2rem);
  font-weight: 800;
}

.game-authorize__card > p:not(.game-authorize__eyebrow) {
  color: #596273;
}

.game-authorize__loading::after {
  content: '';
  display: inline-block;
  width: 0.8em;
  height: 0.8em;
  margin-left: 0.55em;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  vertical-align: -0.08em;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
