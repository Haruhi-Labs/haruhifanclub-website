<script setup>
import { ref, reactive, onMounted } from 'vue'
import {
  SosButton,
  SosBadge,
  SosEyebrow,
  SosTitle,
  SosNotice,
  SosSkeleton,
  SosEmptyState,
  SosModal,
  SosField,
  SosInput,
  SosTextarea,
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
    hidden: { label: '已下架', tone: 'default' },
    rejected: { label: '未通过', tone: 'danger' },
  }
  return map[s] || { label: '已发布', tone: 'success' }
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const r = await hub.news.articles()
    items.value = r.data || []
  } catch (e) {
    error.value = e?.message || '加载失败'
  } finally {
    loading.value = false
  }
}
onMounted(load)

// 编辑（meta：标题/副标题/摘要/标签；正文编辑请前往团报编辑器）
const editing = ref(null)
const form = reactive({ title: '', subtitle: '', summary: '', tags: '' })
const saving = ref(false)
function openEdit(a) {
  editing.value = a
  form.title = a.title || ''
  form.subtitle = a.subtitle || ''
  form.summary = a.summary || ''
  form.tags = (a.tags || []).join(' ')
}
async function saveEdit() {
  if (!form.title.trim()) {
    error.value = '标题不能为空'
    return
  }
  saving.value = true
  try {
    await hub.news.updateArticle(editing.value.id, {
      title: form.title.trim(),
      subtitle: form.subtitle.trim(),
      summary: form.summary.trim(),
      tags: form.tags.split(/[\s,，]+/).filter(Boolean),
    })
    okMsg.value = '文章已更新'
    editing.value = null
    await load()
  } catch (e) {
    error.value = e?.message || '保存失败'
  } finally {
    saving.value = false
  }
}

// 下架
const removing = ref(null)
async function confirmRemove() {
  try {
    await hub.news.deleteArticle(removing.value.id)
    okMsg.value = '文章已下架'
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
      <SosTitle as="h1" size="xl">我的文章</SosTitle>
      <p class="sos-copy">管理你在团报投稿的文章——编辑信息、摘要与标签，或下架。</p>
    </header>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
    <SosNotice v-if="okMsg" tone="success">{{ okMsg }}</SosNotice>

    <div v-if="loading" class="huc__rows">
      <SosSkeleton v-for="i in 4" :key="i" variant="block" style="height: 4rem" />
    </div>
    <SosEmptyState
      v-else-if="!items.length"
      title="还没有投稿"
      copy="去团报投稿你的第一篇文章吧，发布后会出现在这里。"
    />
    <div v-else class="huc__rows">
      <div v-for="a in items" :key="a.id" class="huc__row">
        <div class="huc__row-main">
          <div class="huc__row-title">
            {{ a.title || '无标题' }}
            <SosBadge :variant="statusMeta(a.status).tone">
              {{ statusMeta(a.status).label }}
            </SosBadge>
          </div>
          <div class="huc__row-meta">
            <span v-if="a.date">{{ a.date }}</span>
            <span v-if="a.preview"> · {{ a.preview.slice(0, 40) }}</span>
          </div>
        </div>
        <div class="huc__row-actions">
          <SosButton size="sm" variant="ghost" @click="openEdit(a)">编辑</SosButton>
          <SosButton
            v-if="a.status !== 'hidden'"
            size="sm"
            variant="ghost"
            @click="removing = a"
          >
            下架
          </SosButton>
        </div>
      </div>
    </div>

    <!-- 编辑弹窗 -->
    <SosModal
      :open="!!editing"
      title="编辑文章"
      @update:open="(v) => { if (!v) editing = null }"
    >
      <form class="sos-stack" @submit.prevent="saveEdit">
        <SosField label="标题" required>
          <SosInput v-model="form.title" maxlength="120" />
        </SosField>
        <SosField label="副标题">
          <SosInput v-model="form.subtitle" maxlength="160" />
        </SosField>
        <SosField label="摘要" help="列表与分享卡片展示的简介">
          <SosTextarea v-model="form.summary" :rows="3" maxlength="400" />
        </SosField>
        <SosField label="标签" help="空格或逗号分隔">
          <SosInput v-model="form.tags" />
        </SosField>
        <SosNotice tone="info">正文内容请在团报「编辑器」中修改。</SosNotice>
      </form>
      <template #footer>
        <SosButton variant="ghost" @click="editing = null">取消</SosButton>
        <SosButton :loading="saving" @click="saveEdit">保存</SosButton>
      </template>
    </SosModal>

    <!-- 下架确认 -->
    <SosModal
      :open="!!removing"
      title="下架文章"
      @update:open="(v) => { if (!v) removing = null }"
    >
      <p class="sos-copy">
        确定下架「{{ removing?.title || '无标题' }}」吗？下架后将从团报隐藏，数据保留，可联系管理员恢复。
      </p>
      <template #footer>
        <SosButton variant="ghost" @click="removing = null">取消</SosButton>
        <SosButton variant="danger" @click="confirmRemove">确认下架</SosButton>
      </template>
    </SosModal>
  </div>
</template>
