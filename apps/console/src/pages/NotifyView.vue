<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { store, refreshAdmin, flash, errMsg, appName, AI_MODULES, type AdminUser } from '../api'

onMounted(() => {
  if (!store.loaded) refreshAdmin().catch((e) => flash(errMsg(e), 'err'))
})

function hasEmail(u: AdminUser): boolean {
  return !!(u.email || '').trim()
}

// 某模块 AI 拦截时会收到通知的人：有邮箱的（超管 ∪ 在该模块有角色的管理员）
function recipients(app: string): AdminUser[] {
  return store.users.filter(
    (u) =>
      u.status === 'active' &&
      hasEmail(u) &&
      (u.isSuperAdmin || !!u.roles[app]),
  )
}

// 配了邮箱但被停用的超管/管理员（提醒：停用后收不到）
const supersNoEmail = computed(() =>
  store.users.filter((u) => u.isSuperAdmin && u.status === 'active' && !hasEmail(u)),
)
</script>

<template>
  <section class="card">
    <h3>AI 审核拦截邮件通知</h3>
    <p class="muted">
      当 <strong>画廊（作品 / 评论）</strong> 或 <strong>考试（试卷）</strong> 的内容被 AI 审核拦截时，
      系统会自动发邮件通知管理员。收件人规则：
    </p>
    <ul class="rules">
      <li><strong>超级管理员</strong>默认收到所有模块的通知（前提：给自己的账号配了邮箱）。</li>
      <li>某模块的<strong>管理员</strong>（在该模块有角色的用户）配了邮箱后，也会收到该模块的通知。</li>
      <li>仅 <strong>active</strong> 状态、且填了邮箱的账号会收到；其余自动跳过。</li>
    </ul>
    <p class="muted small">
      去 <RouterLink to="/users">用户与权限</RouterLink> 给管理员配置邮箱 / 分配模块角色即可调整收件人。
    </p>
  </section>

  <section v-if="supersNoEmail.length" class="card warnbox">
    ⚠ 有 {{ supersNoEmail.length }} 位超级管理员尚未配置邮箱，将收不到任何通知：
    <span v-for="u in supersNoEmail" :key="u.id" class="chip">{{ u.username }}</span>
  </section>

  <section v-for="app in AI_MODULES" :key="app" class="card">
    <h3>{{ appName(app) }}（{{ app }}）—— 当前会通知</h3>
    <div v-if="recipients(app).length" class="rcpt">
      <div v-for="u in recipients(app)" :key="u.id" class="rcpt-item">
        <span class="dot active"></span>
        <strong>{{ u.username }}</strong>
        <span class="muted">{{ u.email }}</span>
        <span class="badge" :class="u.isSuperAdmin ? '' : 'soft'">
          {{ u.isSuperAdmin ? '超管' : appName(app) + ' 管理员' }}
        </span>
      </div>
    </div>
    <p v-else class="muted">暂无收件人。给超管或该模块管理员配置邮箱后这里就会出现。</p>
  </section>
</template>

<style scoped>
.rules {
  margin: 10px 0;
  padding-left: 20px;
  line-height: 1.9;
  font-size: 14px;
}
.warnbox {
  border-color: var(--danger);
  color: var(--text);
}
.rcpt {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.rcpt-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
}
.chip {
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 12px;
  margin-left: 6px;
}
.badge.soft {
  background: var(--panel-2);
  color: var(--muted);
  border: 1px solid var(--border);
}
</style>
