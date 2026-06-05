<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import {
  api,
  store,
  refreshAdmin,
  flash,
  errMsg,
  appName,
  type AdminUser,
} from '../api'

const q = ref('')
const showCreate = ref(false)
const busy = ref(false)
const selected = ref<AdminUser | null>(null)

const newUser = reactive({
  username: '',
  password: '',
  display_name: '',
  email: '',
  is_super_admin: false,
})

const filtered = computed(() => {
  const k = q.value.trim().toLowerCase()
  if (!k) return store.users
  return store.users.filter(
    (u) =>
      u.username.toLowerCase().includes(k) ||
      (u.displayName || '').toLowerCase().includes(k) ||
      (u.email || '').toLowerCase().includes(k),
  )
})

// 角色可分配的 app（超管/console 不在表里逐个分配）
const roleApps = computed(() => store.apps.filter((a) => a !== 'console'))

onMounted(() => {
  if (!store.loaded) refreshAdmin().catch((e) => flash(errMsg(e), 'err'))
})

async function createUser() {
  if (!newUser.username.trim() || newUser.password.length < 6) {
    flash('用户名必填且密码至少 6 位', 'err')
    return
  }
  busy.value = true
  try {
    await api.post('/admin/users', { ...newUser, username: newUser.username.trim() })
    Object.assign(newUser, {
      username: '',
      password: '',
      display_name: '',
      email: '',
      is_super_admin: false,
    })
    showCreate.value = false
    await refreshAdmin()
    flash('已创建用户')
  } catch (e) {
    flash(errMsg(e), 'err')
  } finally {
    busy.value = false
  }
}

async function saveEmail(u: AdminUser, ev: Event) {
  const val = (ev.target as HTMLInputElement).value.trim()
  if (val === (u.email || '')) return
  try {
    await api.patch(`/admin/users/${u.id}`, { email: val })
    u.email = val || null
    flash(`已更新 ${u.username} 的邮箱`)
  } catch (e) {
    flash(errMsg(e), 'err')
  }
}

async function setRole(u: AdminUser, app: string, roleKey: string) {
  const next = { ...u.roles }
  if (roleKey) next[app] = roleKey
  else delete next[app]
  try {
    await api.put(`/admin/users/${u.id}/roles`, { roles: next })
    u.roles = next
    flash(`已更新 ${u.username} 在「${appName(app)}」的角色`)
  } catch (e) {
    flash(errMsg(e), 'err')
  }
}

async function toggleStatus(u: AdminUser) {
  const status = u.status === 'active' ? 'disabled' : 'active'
  try {
    await api.patch(`/admin/users/${u.id}`, { status })
    u.status = status
    flash(status === 'active' ? '已启用' : '已停用')
  } catch (e) {
    flash(errMsg(e), 'err')
  }
}

async function toggleSuper(u: AdminUser) {
  const next = !u.isSuperAdmin
  if (!confirm(next ? `把 ${u.username} 设为超级管理员？` : `取消 ${u.username} 的超管身份？`)) return
  try {
    await api.patch(`/admin/users/${u.id}`, { is_super_admin: next })
    u.isSuperAdmin = next
    flash('已更新超管身份')
  } catch (e) {
    flash(errMsg(e), 'err')
  }
}

async function resetPassword(u: AdminUser) {
  const pw = prompt(`为 ${u.username} 设置新密码（至少 6 位）`)
  if (!pw) return
  try {
    await api.post(`/admin/users/${u.id}/password`, { password: pw })
    flash('密码已重置')
  } catch (e) {
    flash(errMsg(e), 'err')
  }
}

async function removeUser(u: AdminUser) {
  if (!confirm(`确认删除用户 ${u.username}？此操作不可恢复。`)) return
  try {
    await api.del(`/admin/users/${u.id}`)
    if (selected.value?.id === u.id) selected.value = null
    await refreshAdmin()
    flash('已删除')
  } catch (e) {
    flash(errMsg(e), 'err')
  }
}

function roleChips(u: AdminUser): string[] {
  if (u.isSuperAdmin) return ['全部模块']
  return Object.entries(u.roles).map(([app, key]) => `${appName(app)}:${key}`)
}
</script>

<template>
  <div class="toolbar">
    <input v-model="q" class="search" placeholder="搜索用户名 / 显示名 / 邮箱" />
    <span class="muted small">共 {{ store.users.length }} 人</span>
    <span class="spacer"></span>
    <button class="primary" @click="showCreate = !showCreate">＋ 新建用户</button>
  </div>

  <section v-if="showCreate" class="card create">
    <div class="grid">
      <label>用户名<input v-model="newUser.username" placeholder="必填" /></label>
      <label>密码<input v-model="newUser.password" type="password" placeholder="≥6 位" /></label>
      <label>显示名<input v-model="newUser.display_name" placeholder="可选" /></label>
      <label>邮箱<input v-model="newUser.email" placeholder="用于 AI 审核通知，可选" /></label>
    </div>
    <div class="row">
      <label class="check"><input v-model="newUser.is_super_admin" type="checkbox" /> 设为超级管理员</label>
      <span class="spacer"></span>
      <button @click="showCreate = false">取消</button>
      <button class="primary" :disabled="busy" @click="createUser">创建</button>
    </div>
  </section>

  <section class="card">
    <table class="data">
      <thead>
        <tr>
          <th>用户</th>
          <th>邮箱（AI 通知收件）</th>
          <th>角色</th>
          <th>状态</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="u in filtered" :key="u.id">
          <td>
            <strong>{{ u.username }}</strong>
            <span v-if="u.isSuperAdmin" class="badge">超管</span>
            <div v-if="u.displayName" class="muted small">{{ u.displayName }}</div>
          </td>
          <td>
            <input
              class="email-input"
              :value="u.email || ''"
              placeholder="未设置"
              @change="saveEmail(u, $event)"
            />
          </td>
          <td>
            <div class="chips">
              <span v-for="c in roleChips(u)" :key="c" class="chip">{{ c }}</span>
              <span v-if="roleChips(u).length === 0" class="muted small">—</span>
            </div>
          </td>
          <td>
            <span class="dot" :class="u.status"></span>{{ u.status === 'active' ? '启用' : '停用' }}
          </td>
          <td class="actions">
            <button @click="selected = u">角色</button>
            <button @click="toggleStatus(u)">{{ u.status === 'active' ? '停用' : '启用' }}</button>
            <button @click="resetPassword(u)">改密</button>
            <button @click="toggleSuper(u)">{{ u.isSuperAdmin ? '降级' : '升超管' }}</button>
            <button class="danger" @click="removeUser(u)">删除</button>
          </td>
        </tr>
      </tbody>
    </table>
  </section>

  <!-- 角色编辑抽屉 -->
  <transition name="slide">
    <div v-if="selected" class="drawer-mask" @click.self="selected = null">
      <aside class="drawer">
        <header class="drawer-head">
          <div>
            <strong>{{ selected.username }}</strong>
            <span v-if="selected.isSuperAdmin" class="badge">超管</span>
          </div>
          <button class="ghost" @click="selected = null">✕</button>
        </header>
        <p v-if="selected.isSuperAdmin" class="muted">超级管理员对所有模块拥有全部权限，无需逐个分配。</p>
        <div v-else class="role-grid">
          <div v-for="app in roleApps" :key="app" class="role-row">
            <span class="role-app">{{ appName(app) }}</span>
            <select
              :value="selected.roles[app] || ''"
              @change="setRole(selected, app, ($event.target as HTMLSelectElement).value)"
            >
              <option value="">— 无 —</option>
              <option v-for="r in store.roles" :key="r.key" :value="r.key">{{ r.name }}</option>
            </select>
          </div>
        </div>
      </aside>
    </div>
  </transition>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.search {
  width: 280px;
}
.spacer {
  flex: 1;
}
.create .grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}
.create label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 12px;
  color: var(--muted);
}
.create .row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 14px;
}
.check {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text);
}
.email-input {
  width: 100%;
  min-width: 180px;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}
.chip {
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 12px;
}
.actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.drawer-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: flex-end;
  z-index: 50;
}
.drawer {
  width: 380px;
  max-width: 90vw;
  height: 100%;
  background: var(--panel);
  border-left: 1px solid var(--border);
  padding: 22px;
  overflow-y: auto;
}
.drawer-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 18px;
}
.role-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.role-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.role-app {
  font-size: 14px;
}
.role-row select {
  width: 160px;
}
.slide-enter-active,
.slide-leave-active {
  transition: opacity 0.2s;
}
.slide-enter-from,
.slide-leave-to {
  opacity: 0;
}
</style>
