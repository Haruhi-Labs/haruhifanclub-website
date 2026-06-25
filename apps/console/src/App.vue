<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { auth, session, toast, appName } from './api'

const booting = ref(true)
const busy = ref(false)
const loginForm = reactive({ username: '', password: '' })
const loginError = ref('')

const isSuper = computed(() => session.me?.isSuperAdmin === true)

const nav = [
  { to: '/dashboard', label: '概览', icon: '◈' },
  { to: '/users', label: '用户与权限', icon: '☻' },
  { to: '/notify', label: '通知设置', icon: '✉' },
  { to: '/audit', label: '审计日志', icon: '◷' },
]

onMounted(async () => {
  if (auth.isLoggedIn()) {
    try {
      session.me = await auth.me()
    } catch {
      auth.logout()
    }
  }
  booting.value = false
})

async function doLogin() {
  loginError.value = ''
  busy.value = true
  try {
    session.me = await auth.login(loginForm.username.trim(), loginForm.password)
  } catch (e: unknown) {
    loginError.value = (e as { message?: string })?.message || '登录失败'
  } finally {
    busy.value = false
  }
}

function logout() {
  auth.logout()
  session.me = null
}
</script>

<template>
  <div v-if="booting" class="center muted">加载中…</div>

  <!-- 登录 -->
  <div v-else-if="!session.me" class="login-screen">
    <section class="login card">
      <h1 class="brand">凉宫春日应援团 · 控制台</h1>
      <input v-model="loginForm.username" placeholder="用户名" @keyup.enter="doLogin" />
      <input
        v-model="loginForm.password"
        type="password"
        placeholder="密码"
        @keyup.enter="doLogin"
      />
      <button class="primary" :disabled="busy" @click="doLogin">登录</button>
      <p v-if="loginError" class="err">{{ loginError }}</p>
    </section>
  </div>

  <!-- 非超管 -->
  <div v-else-if="!isSuper" class="center">
    <section class="card narrow">
      <h2>仅限超级管理员</h2>
      <p class="muted">你已登录为 <strong>{{ session.me.displayName || session.me.username }}</strong>，但控制台仅超级管理员可用。</p>
      <p class="muted" style="margin-top: 12px">你被授权的应用：</p>
      <ul class="plain">
        <li v-for="(r, app) in session.me.apps" :key="app">{{ appName(String(app)) }} — {{ r.roleName }}</li>
      </ul>
      <button style="margin-top: 16px" @click="logout">登出</button>
    </section>
  </div>

  <!-- 控制台主体 -->
  <div v-else class="shell">
    <aside class="sidebar">
      <div class="logo">春日 · 控制台</div>
      <nav class="nav">
        <RouterLink v-for="n in nav" :key="n.to" :to="n.to" class="nav-item" active-class="active">
          <span class="ic">{{ n.icon }}</span>{{ n.label }}
        </RouterLink>
      </nav>
      <div class="side-foot">
        <div class="who">
          <strong>{{ session.me.displayName || session.me.username }}</strong>
          <span class="badge">超管</span>
        </div>
        <button class="ghost" @click="logout">登出</button>
      </div>
    </aside>

    <main class="content">
      <header class="page-head">
        <h1>{{ ($route.meta.title as string) || '控制台' }}</h1>
      </header>
      <RouterView />

      <!-- 精简统一页脚（暗色），保留侧栏招牌 -->
      <footer class="sos-footer console-footer">
        <div class="sos-footer__inner">
          <div class="sos-footer__bottom">
            <span>© 2026 HARUHIFANCLUB · 凉宫春日应援团控制台</span>
            <div class="sos-footer__bottom-meta">
              <a class="sos-footer__link" href="https://haruyuki.cn" target="_blank" rel="noopener noreferrer">应援团主站</a>
              <span class="sos-footer__bottom-sep" aria-hidden="true"></span>
              <a class="sos-footer__link" href="https://beian.miit.gov.cn/#/Integrated/index" target="_blank" rel="noopener noreferrer">皖ICP备2025089290号-1</a>
            </div>
          </div>
        </div>
      </footer>
    </main>
  </div>

  <transition name="fade">
    <div v-if="toast.msg" class="toast" :class="toast.kind">{{ toast.msg }}</div>
  </transition>
</template>
