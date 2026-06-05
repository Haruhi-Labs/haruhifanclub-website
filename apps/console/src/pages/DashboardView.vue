<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { store, refreshAdmin, flash, errMsg, session } from '../api'

onMounted(() => {
  if (!store.loaded) refreshAdmin().catch((e) => flash(errMsg(e), 'err'))
})

const stats = computed(() => {
  const u = store.users
  return {
    total: u.length,
    active: u.filter((x) => x.status === 'active').length,
    supers: u.filter((x) => x.isSuperAdmin).length,
    withEmail: u.filter((x) => (x.email || '').trim()).length,
  }
})
</script>

<template>
  <p class="muted welcome">
    欢迎，<strong>{{ session.me?.displayName || session.me?.username }}</strong>。这里统一管理用户、权限与通知。
  </p>

  <div class="cards">
    <div class="stat">
      <div class="num">{{ stats.total }}</div>
      <div class="lbl">用户总数</div>
    </div>
    <div class="stat">
      <div class="num ok">{{ stats.active }}</div>
      <div class="lbl">启用中</div>
    </div>
    <div class="stat">
      <div class="num accent">{{ stats.supers }}</div>
      <div class="lbl">超级管理员</div>
    </div>
    <div class="stat">
      <div class="num" :class="{ warn: stats.withEmail === 0 }">{{ stats.withEmail }}</div>
      <div class="lbl">已配置邮箱</div>
    </div>
  </div>

  <section class="card">
    <h3>快速入口</h3>
    <div class="links">
      <RouterLink to="/users" class="link-card">
        <strong>用户与权限 →</strong>
        <span class="muted small">新建管理员、分配模块角色、设置通知邮箱</span>
      </RouterLink>
      <RouterLink to="/notify" class="link-card">
        <strong>通知设置 →</strong>
        <span class="muted small">查看 AI 审核拦截邮件会发给谁</span>
      </RouterLink>
      <RouterLink to="/audit" class="link-card">
        <strong>审计日志 →</strong>
        <span class="muted small">查看后台关键操作记录</span>
      </RouterLink>
    </div>
  </section>
</template>

<style scoped>
.welcome {
  margin: 0 0 18px;
}
.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 14px;
  margin-bottom: 20px;
}
.stat {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 20px;
}
.num {
  font-size: 30px;
  font-weight: 700;
}
.num.ok {
  color: var(--ok);
}
.num.accent {
  color: var(--accent);
}
.num.warn {
  color: var(--danger);
}
.lbl {
  color: var(--muted);
  font-size: 13px;
  margin-top: 4px;
}
.links {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}
.link-card {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  text-decoration: none;
  color: var(--text);
  background: var(--panel-2);
}
.link-card:hover {
  border-color: var(--accent);
}
</style>
