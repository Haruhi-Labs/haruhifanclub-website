<script setup>
import { ref, computed, onMounted } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { SosCard, SosButton, SosEyebrow, SosTitle, SosSkeleton, SosNotice } from '@haruhi/ui'
import { useSession } from './useSession.js'
import { useUserHub } from './useUserHub.js'
import { useConsoleContext } from './console-context.js'

const ctx = useConsoleContext()
const session = useSession(ctx.apiBase)
const hub = useUserHub(ctx.apiBase)
const router = useRouter()
const user = computed(() => session.state.user)

const summary = ref(null)
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    summary.value = await hub.summary()
  } catch (e) {
    error.value = e?.message || '加载失败'
  } finally {
    loading.value = false
  }
})

function pendingText(g) {
  return g && g.pending ? `${g.pending} 条审核中` : ''
}

const stats = computed(() => {
  const s = summary.value || {}
  return [
    {
      key: 'artworks',
      label: '画廊作品',
      value: s.artworks?.total ?? 0,
      sub: pendingText(s.artworks),
      to: `${ctx.basePath}/artworks`,
    },
    {
      key: 'articles',
      label: '团报文章',
      value: s.articles?.total ?? 0,
      sub: pendingText(s.articles),
      to: `${ctx.basePath}/articles`,
    },
    {
      key: 'exams',
      label: '我的考试',
      value: s.exams?.total ?? 0,
      sub: '',
      to: `${ctx.basePath}/exams`,
    },
    {
      key: 'comments',
      label: '我的评论',
      value: s.comments ?? 0,
      sub: '',
      to: `${ctx.basePath}/comments`,
    },
  ]
})

const points = computed(() => summary.value?.points || { art: 0, news: 0 })
const redemptions = computed(() => summary.value?.redemptions ?? 0)
</script>

<template>
  <div class="sos-stack huc-page">
    <header class="sos-stack sos-stack--tight">
      <SosEyebrow>个人中心</SosEyebrow>
      <SosTitle as="h1" size="xl">你好，{{ user?.nickname || '同好' }}</SosTitle>
      <p class="sos-copy">在这里统一管理你在应援团各处发布的内容、积分与账号。</p>
    </header>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>

    <!-- 内容统计 -->
    <div class="huc__stat-grid">
      <template v-if="loading">
        <SosCard v-for="i in 4" :key="i">
          <SosSkeleton variant="block" style="height: 4.5rem" />
        </SosCard>
      </template>
      <template v-else>
        <RouterLink v-for="st in stats" :key="st.key" :to="st.to" class="huc__stat">
          <span class="huc__stat-value">{{ st.value }}</span>
          <span class="huc__stat-label">{{ st.label }}</span>
          <span v-if="st.sub" class="huc__stat-sub">{{ st.sub }}</span>
        </RouterLink>
      </template>
    </div>

    <!-- 积分概览 -->
    <SosCard as="section">
      <div class="sos-stack">
        <div class="sos-cluster">
          <SosTitle as="h2" style="font-size: var(--sos-text-lg)">我的积分</SosTitle>
          <SosButton variant="secondary" size="sm" @click="router.push(`${ctx.basePath}/points`)">
            去兑换 →
          </SosButton>
        </div>
        <div class="huc__points">
          <div class="huc__point">
            <span class="huc__point-value">{{ points.art }}</span>
            <span class="huc__point-label">画廊创作分</span>
          </div>
          <div class="huc__point">
            <span class="huc__point-value">{{ points.news }}</span>
            <span class="huc__point-label">团报应援分</span>
          </div>
          <div class="huc__point">
            <span class="huc__point-value">{{ redemptions }}</span>
            <span class="huc__point-label">兑换记录</span>
          </div>
        </div>
      </div>
    </SosCard>
  </div>
</template>
