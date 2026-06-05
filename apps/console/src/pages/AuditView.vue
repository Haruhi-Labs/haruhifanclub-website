<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api, store, refreshAdmin, flash, errMsg, appName, type AuditItem } from '../api'

const items = ref<AuditItem[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    if (!store.loaded) await refreshAdmin()
    const res = await api.get('/admin/audit')
    items.value = res.items
  } catch (e) {
    flash(errMsg(e), 'err')
  } finally {
    loading.value = false
  }
})

const userName = (id: number | null) => {
  if (id == null) return '系统'
  return store.users.find((u) => u.id === id)?.username || `#${id}`
}

const rows = computed(() => items.value)

function fmt(ts: string): string {
  // 后端是 UTC（CURRENT_TIMESTAMP）；这里直接展示，避免时区库依赖
  return ts.replace('T', ' ').replace('Z', '').slice(0, 19)
}
</script>

<template>
  <section class="card">
    <p class="muted small" style="margin-bottom: 12px">最近 200 条后台关键操作（建/改/删用户、改密、授权等）。</p>
    <p v-if="loading" class="muted">加载中…</p>
    <table v-else class="data">
      <thead>
        <tr>
          <th>时间</th>
          <th>操作者</th>
          <th>动作</th>
          <th>模块</th>
          <th>目标</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="it in rows" :key="it.id">
          <td class="muted small mono">{{ fmt(it.createdAt) }}</td>
          <td>{{ userName(it.userId) }}</td>
          <td><span class="chip">{{ it.action }}</span></td>
          <td>{{ it.app ? appName(it.app) : '—' }}</td>
          <td class="muted">{{ it.target || '—' }}</td>
        </tr>
        <tr v-if="!rows.length">
          <td colspan="5" class="muted">暂无记录</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
.mono {
  font-variant-numeric: tabular-nums;
}
.chip {
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 12px;
}
</style>
