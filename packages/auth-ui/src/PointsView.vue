<script setup>
import { ref, onMounted } from 'vue'
import {
  SosCard,
  SosButton,
  SosBadge,
  SosEyebrow,
  SosTitle,
  SosNotice,
  SosSkeleton,
  SosEmptyState,
  SosModal,
} from '@haruhi/ui'
import { useUserHub } from './useUserHub.js'
import { useConsoleContext } from './console-context.js'

const ctx = useConsoleContext()
const hub = useUserHub(ctx.apiBase)

const loading = ref(true)
const error = ref('')
const okMsg = ref('')
const artPts = ref({ total: 0, history: [] })
const newsPts = ref({ total: 0, history: [] })
const prizes = ref([])
const redemptions = ref([])

function imgUrl(u) {
  if (!u) return undefined
  return /^(https?:|data:|blob:|\/)/.test(u) ? u : `/${u}`
}

async function loadAll() {
  loading.value = true
  error.value = ''
  try {
    const [a, n, pz, rd] = await Promise.all([
      hub.art.points().catch(() => ({ data: { total: 0, history: [] } })),
      hub.news.points().catch(() => ({ data: { total: 0, history: [] } })),
      hub.news.prizes().catch(() => ({ data: [] })),
      hub.news.redemptions().catch(() => ({ data: [] })),
    ])
    artPts.value = { total: a.data?.total || 0, history: a.data?.history || [] }
    newsPts.value = { total: n.data?.total || 0, history: n.data?.history || [] }
    prizes.value = pz.data || []
    redemptions.value = rd.data || []
  } catch (e) {
    error.value = e?.message || '加载失败'
  } finally {
    loading.value = false
  }
}
onMounted(loadAll)

const redeeming = ref(null)
const busy = ref(false)
function askRedeem(p) {
  error.value = ''
  okMsg.value = ''
  redeeming.value = p
}
async function confirmRedeem() {
  busy.value = true
  try {
    const r = await hub.news.redeem(redeeming.value.id)
    okMsg.value = `兑换成功：${redeeming.value.name}，剩余 ${r.data?.total ?? '?'} 团报应援分`
    redeeming.value = null
    await loadAll()
  } catch (e) {
    error.value = e?.message || '兑换失败'
    redeeming.value = null
  } finally {
    busy.value = false
  }
}

function redeemState(p) {
  if (p.stock < 1) return { disabled: true, label: '已兑完' }
  if (newsPts.value.total < p.points) return { disabled: true, label: '积分不足' }
  return { disabled: false, label: '兑换' }
}
function redemptionStatus(s) {
  return { pending: '待发放', fulfilled: '已发放', cancelled: '已取消' }[s] || s
}
</script>

<template>
  <div class="sos-stack huc-page">
    <header class="sos-stack sos-stack--tight">
      <SosEyebrow>积分中心</SosEyebrow>
      <SosTitle as="h1" size="xl">积分与兑换</SosTitle>
      <p class="sos-copy">查看你的两类积分，并用团报应援分兑换应援团周边。</p>
    </header>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
    <SosNotice v-if="okMsg" tone="success">{{ okMsg }}</SosNotice>

    <!-- 积分概览 -->
    <div class="huc__points">
      <div class="huc__point">
        <span class="huc__point-value">{{ artPts.total }}</span>
        <span class="huc__point-label">画廊创作分</span>
      </div>
      <div class="huc__point">
        <span class="huc__point-value">{{ newsPts.total }}</span>
        <span class="huc__point-label">团报应援分（可兑换）</span>
      </div>
    </div>

    <!-- 兑换商城 -->
    <section class="sos-stack">
      <SosTitle as="h2" style="font-size: var(--sos-text-lg)">兑换商城</SosTitle>
      <div v-if="loading" class="huc__grid">
        <SosSkeleton v-for="i in 3" :key="i" variant="block" style="height: 14rem" />
      </div>
      <SosEmptyState v-else-if="!prizes.length" title="暂无可兑换奖品" copy="奖品上架后会显示在这里。" />
      <div v-else class="huc__grid">
        <div v-for="p in prizes" :key="p.id" class="huc-prize">
          <div class="huc-prize__media">
            <img v-if="p.image" :src="imgUrl(p.image)" :alt="p.name" />
          </div>
          <div class="huc-prize__body">
            <h3 class="huc-prize__name">{{ p.name }}</h3>
            <p v-if="p.description" class="huc-prize__desc">{{ p.description }}</p>
            <div class="huc-prize__foot">
              <span class="huc-prize__cost">{{ p.points }} 分</span>
              <span class="huc-prize__stock">库存 {{ p.stock }}</span>
            </div>
            <SosButton
              size="sm"
              :disabled="redeemState(p).disabled"
              @click="askRedeem(p)"
            >
              {{ redeemState(p).label }}
            </SosButton>
          </div>
        </div>
      </div>
    </section>

    <!-- 我的兑换记录 -->
    <section v-if="redemptions.length" class="sos-stack">
      <SosTitle as="h2" style="font-size: var(--sos-text-lg)">兑换记录</SosTitle>
      <div class="huc__rows">
        <div v-for="r in redemptions" :key="r.id" class="huc__row">
          <div class="huc__row-main">
            <div class="huc__row-title">
              {{ r.prize_name }}
              <SosBadge variant="default">{{ redemptionStatus(r.status) }}</SosBadge>
            </div>
            <div class="huc__row-meta">
              -{{ r.points_cost }} 分
              <span v-if="r.created_at"> · {{ String(r.created_at).slice(0, 10) }}</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 积分流水 -->
    <div class="huc__ledger-cols">
      <SosCard as="section">
        <SosTitle as="h3" style="font-size: var(--sos-text-md)">团报积分流水</SosTitle>
        <ul class="huc__ledger">
          <li v-for="(h, i) in newsPts.history" :key="i" class="huc__ledger-item">
            <span class="huc__ledger-reason">{{ h.reason || '—' }}</span>
            <span class="huc__ledger-change">{{ h.change }}</span>
          </li>
          <li v-if="!newsPts.history.length" class="huc__ledger-empty">暂无记录</li>
        </ul>
      </SosCard>
      <SosCard as="section">
        <SosTitle as="h3" style="font-size: var(--sos-text-md)">画廊积分流水</SosTitle>
        <ul class="huc__ledger">
          <li v-for="(h, i) in artPts.history" :key="i" class="huc__ledger-item">
            <span class="huc__ledger-reason">
              {{ h.note || '积分' }}<template v-if="h.artwork_title"> · {{ h.artwork_title }}</template>
            </span>
            <span class="huc__ledger-change">+{{ h.points }}</span>
          </li>
          <li v-if="!artPts.history.length" class="huc__ledger-empty">暂无记录</li>
        </ul>
      </SosCard>
    </div>

    <!-- 兑换确认 -->
    <SosModal
      :open="!!redeeming"
      title="确认兑换"
      @update:open="(v) => { if (!v) redeeming = null }"
    >
      <p class="sos-copy">
        用 <strong>{{ redeeming?.points }}</strong> 团报应援分兑换「{{ redeeming?.name }}」？
        <br />当前余额 {{ newsPts.total }} 分，兑换后剩余
        {{ newsPts.total - (redeeming?.points || 0) }} 分。
      </p>
      <template #footer>
        <SosButton variant="ghost" @click="redeeming = null">取消</SosButton>
        <SosButton :loading="busy" @click="confirmRedeem">确认兑换</SosButton>
      </template>
    </SosModal>
  </div>
</template>
