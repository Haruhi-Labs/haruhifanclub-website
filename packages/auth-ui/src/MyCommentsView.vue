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

async function load() {
  loading.value = true
  error.value = ''
  try {
    const r = await hub.art.comments({ pageSize: 60 })
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
    await hub.art.deleteComment(removing.value.id)
    okMsg.value = '评论已删除'
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
      <SosTitle as="h1" size="xl">我的评论</SosTitle>
      <p class="sos-copy">你在画廊作品下发表的评论，可在此删除。</p>
    </header>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
    <SosNotice v-if="okMsg" tone="success">{{ okMsg }}</SosNotice>

    <div v-if="loading" class="huc__rows">
      <SosSkeleton v-for="i in 4" :key="i" variant="block" style="height: 3.5rem" />
    </div>
    <SosEmptyState
      v-else-if="!items.length"
      title="还没有评论"
      copy="去画廊看看作品，留下你的第一条评论吧。"
    />
    <div v-else class="huc__rows">
      <div v-for="c in items" :key="c.id" class="huc__row">
        <div class="huc__row-main">
          <div class="huc__row-title">{{ c.body }}</div>
          <div class="huc__row-meta">
            <span v-if="c.artwork_title">作品：{{ c.artwork_title }}</span>
            <SosBadge v-if="c.status === 'hidden'" variant="default">已隐藏</SosBadge>
            <span v-if="c.created_at"> · {{ String(c.created_at).slice(0, 10) }}</span>
            <span v-if="c.like_total"> · ♥ {{ c.like_total }}</span>
          </div>
        </div>
        <div class="huc__row-actions">
          <SosButton
            v-if="c.status !== 'hidden'"
            size="sm"
            variant="ghost"
            @click="removing = c"
          >
            删除
          </SosButton>
        </div>
      </div>
    </div>

    <SosModal
      :open="!!removing"
      title="删除评论"
      @update:open="(v) => { if (!v) removing = null }"
    >
      <p class="sos-copy">确定删除这条评论吗？此操作会将其从作品下隐藏。</p>
      <template #footer>
        <SosButton variant="ghost" @click="removing = null">取消</SosButton>
        <SosButton variant="danger" @click="confirmRemove">确认删除</SosButton>
      </template>
    </SosModal>
  </div>
</template>
