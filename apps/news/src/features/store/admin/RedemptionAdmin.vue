<template>
  <div class="tab-content">
    <div class="tab-header">
      <h3 class="tab-title">兑换发放</h3>
      <div class="ra-filters">
        <button
          v-for="f in FILTERS"
          :key="f.key"
          class="ra-filter"
          :class="{ active: filter === f.key }"
          @click="setFilter(f.key)"
        >
          {{ f.label }}
        </button>
      </div>
    </div>

    <p v-if="msg" class="ra-msg" :class="msgKind">{{ msg }}</p>

    <div class="table-wrapper">
      <table class="data-table">
        <thead class="table-head">
          <tr>
            <th class="th-cell">单号</th>
            <th class="th-cell">兑换用户</th>
            <th class="th-cell">奖品</th>
            <th class="th-cell">消耗积分</th>
            <th class="th-cell">状态</th>
            <th class="th-cell">时间</th>
            <th class="th-cell th-right">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="r in items" :key="r.id" class="table-row">
            <td class="td-cell td-mono">#{{ r.id }}</td>
            <td class="td-cell td-mono">{{ r.user_id }}</td>
            <td class="td-cell td-bold">{{ r.prize_name || '—' }}</td>
            <td class="td-cell td-mono">{{ r.points_cost }}</td>
            <td class="td-cell">
              <span class="ra-badge" :class="'ra-badge--' + r.status">{{ statusLabel(r.status) }}</span>
            </td>
            <td class="td-cell td-muted">{{ (r.created_at || '').slice(0, 10) }}</td>
            <td class="td-cell td-actions">
              <template v-if="r.status === 'pending'">
                <button class="link-edit" @click="setStatus(r, 'fulfilled')">标记已发放</button>
                <button class="link-delete" @click="setStatus(r, 'cancelled')">撤销退款</button>
              </template>
              <button
                v-else-if="r.status === 'fulfilled'"
                class="link-edit"
                @click="setStatus(r, 'pending')"
              >
                退回待发放
              </button>
              <span v-else class="td-muted">—</span>
            </td>
          </tr>
          <tr v-if="!loading && !items.length">
            <td colspan="7" class="ra-empty">暂无兑换记录</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { createApiClient } from '@haruhi/api-client'

const api = createApiClient('/api/news')

const FILTERS = [
  { key: 'pending', label: '待发放' },
  { key: 'fulfilled', label: '已发放' },
  { key: 'cancelled', label: '已撤销' },
  { key: 'all', label: '全部' },
]
const filter = ref('pending')
const items = ref([])
const loading = ref(false)
const msg = ref('')
const msgKind = ref('ok')

function flash(m, kind = 'ok') {
  msg.value = m
  msgKind.value = kind
  setTimeout(() => (msg.value = ''), 2800)
}
function statusLabel(s) {
  return { pending: '待发放', fulfilled: '已发放', cancelled: '已撤销' }[s] || s
}

async function load() {
  loading.value = true
  try {
    const r = await api.get('/admin/redemptions?status=' + filter.value)
    items.value = r.data || []
  } catch (e) {
    flash(e?.message || '加载失败', 'err')
  } finally {
    loading.value = false
  }
}
function setFilter(k) {
  if (filter.value === k) return
  filter.value = k
  load()
}
async function setStatus(r, status) {
  const labels = {
    fulfilled: '标记为「已发放」',
    cancelled: '撤销兑换并退还积分 / 库存',
    pending: '退回「待发放」',
  }
  if (!confirm(`确定将兑换 #${r.id}「${r.prize_name || ''}」${labels[status]}？`)) return
  try {
    const res = await api.post(`/admin/redemptions/${r.id}/status`, { status })
    flash(res.refunded != null ? `已撤销并退还 ${res.refunded} 积分` : '已更新发放状态')
    await load()
  } catch (e) {
    flash(e?.message || '操作失败', 'err')
  }
}

onMounted(load)
</script>

<style scoped>
.tab-content {
  padding: 1.5rem;
}
.tab-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
  margin-bottom: 1rem;
}
.tab-title {
  font-size: 1.125rem;
  font-weight: 800;
  color: var(--sos-text-primary);
}
.ra-filters {
  display: flex;
  gap: 0.5rem;
}
.ra-filter {
  padding: 0.35rem 0.85rem;
  font-size: 0.8rem;
  font-weight: 700;
  border-radius: 9999px;
  border: 1px solid var(--sos-border-default);
  background: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
  cursor: pointer;
  transition: all 150ms;
}
.ra-filter.active {
  background: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  border-color: var(--sos-text-primary);
}
.ra-msg {
  margin-bottom: 0.75rem;
  font-size: 0.85rem;
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
}
.ra-msg.ok {
  background: var(--sos-success-soft);
  color: var(--sos-success);
}
.ra-msg.err {
  background: var(--sos-danger-soft);
  color: var(--sos-danger);
}
.table-wrapper {
  overflow-x: auto;
}
.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}
.table-head th {
  text-align: left;
  padding: 0.6rem 0.75rem;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--sos-text-tertiary);
  border-bottom: 1px solid var(--sos-bg-muted);
  white-space: nowrap;
}
.th-right {
  text-align: right;
}
.table-row {
  border-bottom: 1px solid var(--sos-bg-muted);
}
.td-cell {
  padding: 0.6rem 0.75rem;
  vertical-align: middle;
}
.td-mono {
  font-variant-numeric: tabular-nums;
  color: var(--sos-text-secondary);
}
.td-bold {
  font-weight: 700;
  color: var(--sos-text-primary);
}
.td-muted {
  color: var(--sos-text-tertiary);
}
.td-actions {
  text-align: right;
  white-space: nowrap;
}
.ra-badge {
  display: inline-block;
  padding: 0.1rem 0.55rem;
  border-radius: 9999px;
  font-size: 0.72rem;
  font-weight: 700;
}
.ra-badge--pending {
  background: var(--sos-warning-soft);
  color: var(--sos-warning);
}
.ra-badge--fulfilled {
  background: var(--sos-success-soft);
  color: var(--sos-success);
}
.ra-badge--cancelled {
  background: var(--sos-bg-muted);
  color: var(--sos-text-tertiary);
}
.link-edit,
.link-delete {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 0.8rem;
  font-weight: 700;
  padding: 0.2rem 0.4rem;
}
.link-edit {
  color: var(--sos-link);
}
.link-delete {
  color: var(--sos-danger);
}
.ra-empty {
  padding: 2rem;
  text-align: center;
  color: var(--sos-text-tertiary);
}
</style>
