<script setup>
import { ref, onMounted } from 'vue'
import {
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

const items = ref([])
const loading = ref(true)
const error = ref('')
const okMsg = ref('')

function statusMeta(s) {
  const map = {
    published: { label: '已发布', tone: 'success' },
    pending: { label: '审核中', tone: 'signal' },
    locked: { label: '已锁定', tone: 'default' },
    hidden: { label: '已下架', tone: 'default' },
  }
  return map[s] || { label: s || '—', tone: 'default' }
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const r = await hub.exam.exams()
    items.value = r.data || []
  } catch (e) {
    error.value = e?.message || '加载失败'
  } finally {
    loading.value = false
  }
}
onMounted(load)

const removing = ref(null)
async function confirmRemove() {
  try {
    await hub.exam.deleteExam(removing.value.id)
    okMsg.value = '试卷已下架'
    removing.value = null
    await load()
  } catch (e) {
    error.value = e?.message || '操作失败'
    removing.value = null
  }
}
</script>

<template>
  <div class="sos-stack huc-page">
    <header class="sos-stack sos-stack--tight">
      <SosEyebrow>我的内容</SosEyebrow>
      <SosTitle as="h1" size="xl">我的考试</SosTitle>
      <p class="sos-copy">管理你创建的检定试卷——查看状态与访问量，或下架。</p>
    </header>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
    <SosNotice v-if="okMsg" tone="success">{{ okMsg }}</SosNotice>

    <div v-if="loading" class="huc__rows">
      <SosSkeleton v-for="i in 3" :key="i" variant="block" style="height: 4rem" />
    </div>
    <SosEmptyState
      v-else-if="!items.length"
      title="还没有试卷"
      copy="去考场创建你的第一份检定试卷吧。"
    />
    <div v-else class="huc__rows">
      <div v-for="e in items" :key="e.id" class="huc__row">
        <div class="huc__row-main">
          <div class="huc__row-title">
            {{ e.title || '未命名试卷' }}
            <SosBadge :variant="statusMeta(e.status).tone">{{ statusMeta(e.status).label }}</SosBadge>
          </div>
          <div class="huc__row-meta">
            <span v-if="e.subtitle">{{ e.subtitle }} · </span>
            <span>访问 {{ e.visit_count ?? 0 }}</span>
            <span v-if="e.created_at"> · {{ String(e.created_at).slice(0, 10) }}</span>
          </div>
        </div>
        <div class="huc__row-actions">
          <SosButton
            v-if="e.status !== 'hidden'"
            size="sm"
            variant="ghost"
            @click="removing = e"
          >
            下架
          </SosButton>
        </div>
      </div>
    </div>

    <SosModal
      :open="!!removing"
      title="下架试卷"
      @update:open="(v) => { if (!v) removing = null }"
    >
      <p class="sos-copy">
        确定下架「{{ removing?.title || '未命名试卷' }}」吗？下架后将从考场隐藏，数据保留。
      </p>
      <template #footer>
        <SosButton variant="ghost" @click="removing = null">取消</SosButton>
        <SosButton variant="danger" @click="confirmRemove">确认下架</SosButton>
      </template>
    </SosModal>
  </div>
</template>
