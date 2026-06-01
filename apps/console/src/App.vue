<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { createApiClient, createAuth, type CurrentUser } from '@haruhi/api-client'

const api = createApiClient('/api')
const auth = createAuth('/api')

const me = ref<CurrentUser | null>(null)
const booting = ref(true)
const loginForm = reactive({ username: '', password: '' })
const loginError = ref('')
const busy = ref(false)

// 用户管理数据
interface AdminUser {
  id: number
  username: string
  displayName: string | null
  isSuperAdmin: boolean
  status: string
  createdAt: string
  lastLoginAt: string | null
  roles: Record<string, string>
}
const users = ref<AdminUser[]>([])
const apps = ref<string[]>([])
const roles = ref<{ key: string; name: string; level: number }[]>([])
const appNames: Record<string, string> = {
  news: '新闻',
  art: '画廊',
  exam: '考试',
  novel: '书库',
  shop: '商城',
  console: '控制台',
}

const newUser = reactive({ username: '', password: '', display_name: '' })
const message = ref('')

const isSuper = computed(() => me.value?.isSuperAdmin === true)

onMounted(async () => {
  if (auth.isLoggedIn()) {
    try {
      me.value = await auth.me()
      if (isSuper.value) await loadAll()
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
    me.value = await auth.login(loginForm.username.trim(), loginForm.password)
    if (isSuper.value) await loadAll()
  } catch (e: any) {
    loginError.value = e.message || '登录失败'
  } finally {
    busy.value = false
  }
}

function logout() {
  auth.logout()
  me.value = null
}

async function loadAll() {
  const [u, r] = await Promise.all([api.get('/admin/users'), api.get('/admin/roles')])
  users.value = u.users
  apps.value = r.apps
  roles.value = r.roles
}

function flash(msg: string) {
  message.value = msg
  setTimeout(() => (message.value = ''), 2500)
}

async function createUser() {
  if (!newUser.username || newUser.password.length < 6) {
    flash('用户名必填且密码至少 6 位')
    return
  }
  busy.value = true
  try {
    await api.post('/admin/users', { ...newUser })
    newUser.username = ''
    newUser.password = ''
    newUser.display_name = ''
    await loadAll()
    flash('已创建用户')
  } catch (e: any) {
    flash(e.message)
  } finally {
    busy.value = false
  }
}

async function setRole(u: AdminUser, app: string, roleKey: string) {
  const next = { ...u.roles }
  if (roleKey) next[app] = roleKey
  else delete next[app]
  try {
    await api.put(`/admin/users/${u.id}/roles`, { roles: next })
    u.roles = next
    flash(`已更新 ${u.username} 的 ${appNames[app] || app} 角色`)
  } catch (e: any) {
    flash(e.message)
  }
}

async function toggleStatus(u: AdminUser) {
  const status = u.status === 'active' ? 'disabled' : 'active'
  try {
    await api.patch(`/admin/users/${u.id}`, { status })
    u.status = status
  } catch (e: any) {
    flash(e.message)
  }
}

async function resetPassword(u: AdminUser) {
  const pw = prompt(`为 ${u.username} 设置新密码（至少 6 位）`)
  if (!pw) return
  try {
    await api.post(`/admin/users/${u.id}/password`, { password: pw })
    flash('密码已重置')
  } catch (e: any) {
    flash(e.message)
  }
}

async function removeUser(u: AdminUser) {
  if (!confirm(`确认删除用户 ${u.username}？此操作不可恢复。`)) return
  try {
    await api.del(`/admin/users/${u.id}`)
    await loadAll()
    flash('已删除')
  } catch (e: any) {
    flash(e.message)
  }
}
</script>

<template>
  <div class="wrap">
    <header class="topbar">
      <h1>春日应援团 · 控制台</h1>
      <div v-if="me" class="who">
        {{ me.displayName || me.username }}
        <span v-if="me.isSuperAdmin" class="badge">超级管理员</span>
        <button @click="logout">登出</button>
      </div>
    </header>

    <main class="main">
      <p v-if="booting" class="muted">加载中…</p>

      <!-- 登录 -->
      <section v-else-if="!me" class="login card">
        <h2>登录</h2>
        <input v-model="loginForm.username" placeholder="用户名" @keyup.enter="doLogin" />
        <input v-model="loginForm.password" type="password" placeholder="密码" @keyup.enter="doLogin" />
        <button class="primary" :disabled="busy" @click="doLogin">登录</button>
        <p v-if="loginError" class="err">{{ loginError }}</p>
      </section>

      <!-- 非超管 -->
      <section v-else-if="!isSuper" class="card">
        <p>你已登录，但控制台仅限超级管理员访问。</p>
        <p class="muted">你被授权的应用：</p>
        <ul>
          <li v-for="(r, app) in me.apps" :key="app">{{ appNames[app] || app }} — {{ r.roleName }}</li>
        </ul>
      </section>

      <!-- 用户管理 -->
      <template v-else>
        <section class="card">
          <h2>新建管理员</h2>
          <div class="newuser">
            <input v-model="newUser.username" placeholder="用户名" />
            <input v-model="newUser.password" type="password" placeholder="密码（≥6位）" />
            <input v-model="newUser.display_name" placeholder="显示名（可选）" />
            <button class="primary" :disabled="busy" @click="createUser">创建</button>
          </div>
        </section>

        <section class="card">
          <h2>用户与权限（{{ users.length }}）</h2>
          <table class="users">
            <thead>
              <tr>
                <th>用户</th>
                <th v-for="app in apps" :key="app">{{ appNames[app] || app }}</th>
                <th>状态</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="u in users" :key="u.id">
                <td>
                  <strong>{{ u.username }}</strong>
                  <span v-if="u.isSuperAdmin" class="badge">超管</span>
                  <div class="muted small">{{ u.displayName }}</div>
                </td>
                <td v-for="app in apps" :key="app">
                  <select
                    v-if="!u.isSuperAdmin && app !== 'console'"
                    :value="u.roles[app] || ''"
                    @change="setRole(u, app, ($event.target as HTMLSelectElement).value)"
                  >
                    <option value="">—</option>
                    <option v-for="r in roles" :key="r.key" :value="r.key">{{ r.name }}</option>
                  </select>
                  <span v-else class="muted">{{ u.isSuperAdmin ? '全部' : '—' }}</span>
                </td>
                <td>
                  <span :class="['dot', u.status]"></span>{{ u.status === 'active' ? '启用' : '停用' }}
                </td>
                <td class="actions">
                  <button @click="toggleStatus(u)">{{ u.status === 'active' ? '停用' : '启用' }}</button>
                  <button @click="resetPassword(u)">改密</button>
                  <button class="danger" @click="removeUser(u)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </section>
      </template>
    </main>

    <transition name="fade">
      <div v-if="message" class="toast">{{ message }}</div>
    </transition>
  </div>
</template>

<style scoped>
.wrap {
  max-width: 1100px;
  margin: 0 auto;
  padding: 0 16px 60px;
}
.topbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 0;
  border-bottom: 1px solid var(--border);
}
.topbar h1 {
  font-size: 18px;
  margin: 0;
}
.who {
  display: flex;
  align-items: center;
  gap: 10px;
}
.main {
  margin-top: 22px;
}
.card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
  margin-bottom: 18px;
}
.card h2 {
  margin: 0 0 14px;
  font-size: 15px;
}
.login {
  max-width: 320px;
  margin: 60px auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.newuser {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.newuser input {
  flex: 1;
  min-width: 150px;
}
.users {
  width: 100%;
  border-collapse: collapse;
}
.users th,
.users td {
  text-align: left;
  padding: 9px 8px;
  border-bottom: 1px solid var(--border);
  vertical-align: middle;
}
.users th {
  color: var(--muted);
  font-weight: 600;
  font-size: 12px;
}
.actions {
  display: flex;
  gap: 6px;
  white-space: nowrap;
}
.badge {
  background: var(--accent-2);
  color: #fff;
  font-size: 11px;
  padding: 2px 7px;
  border-radius: 6px;
  margin-left: 6px;
}
.muted {
  color: var(--muted);
}
.small {
  font-size: 12px;
}
.err {
  color: var(--danger);
}
.dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 6px;
  background: var(--muted);
}
.dot.active {
  background: var(--ok);
}
.dot.disabled {
  background: var(--danger);
}
.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--panel-2);
  border: 1px solid var(--accent);
  padding: 10px 18px;
  border-radius: 10px;
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
