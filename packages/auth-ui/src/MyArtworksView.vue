<script setup>
import { ref, reactive, onMounted } from 'vue'
import {
  SosButton,
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
import { SosArtworkCard } from '@haruhi/ui/recipes'
import { useUserHub } from './useUserHub.js'
import { useConsoleContext } from './console-context.js'

const ctx = useConsoleContext()
const hub = useUserHub(ctx.apiBase)

const FILTERS = [
  { key: 'all', label: '全部' },
  { key: 'approved', label: '已发布' },
  { key: 'pending', label: '审核中' },
  { key: 'hidden', label: '已下架' },
]
const filter = ref('all')
const items = ref([])
const total = ref(0)
const loading = ref(true)
const error = ref('')
const okMsg = ref('')

function imgUrl(u) {
  if (!u) return undefined
  return /^(https?:|data:|blob:|\/)/.test(u) ? u : `/${u}`
}
function statusLabel(s) {
  return (
    { approved: '已发布', pending: '审核中', flagged: '待复核', rejected: '未通过', hidden: '已下架' }[
      s
    ] || s
  )
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const r = await hub.art.artworks({ status: filter.value, pageSize: 60 })
    items.value = r.data || []
    total.value = r.total || 0
  } catch (e) {
    error.value = e?.message || '加载失败'
  } finally {
    loading.value = false
  }
}
function setFilter(k) {
  if (filter.value === k) return
  filter.value = k
  load()
}
onMounted(load)

// 编辑
const editing = ref(null)
const form = reactive({ title: '', description: '', tags: '', origin_url: '' })
const saving = ref(false)
function openEdit(a) {
  editing.value = a
  form.title = a.title || ''
  form.description = a.description || ''
  form.tags = (a.tags || []).join(' ')
  form.origin_url = a.origin_url || ''
}
async function saveEdit() {
  if (!form.title.trim()) {
    error.value = '标题不能为空'
    return
  }
  saving.value = true
  try {
    await hub.art.updateArtwork(editing.value.id, { ...form })
    okMsg.value = '作品已更新'
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
    await hub.art.deleteArtwork(removing.value.id)
    okMsg.value = '作品已下架'
    removing.value = null
    await load()
  } catch (e) {
    error.value = e?.message || '操作失败'
    removing.value = null
  }
}

// 认领历史匿名作品
const claiming = ref(false)
async function claimHistory() {
  claiming.value = true
  error.value = ''
  okMsg.value = ''
  try {
    const r = await hub.art.claim()
    okMsg.value = r?.claimed ? `已认领 ${r.claimed} 项历史内容` : '未发现可认领的历史内容'
    await load()
  } catch (e) {
    error.value = e?.message || '认领失败'
  } finally {
    claiming.value = false
  }
}
</script>

<template>
  <div class="sos-stack huc-page">
    <header class="sos-stack sos-stack--tight">
      <SosEyebrow>我的内容</SosEyebrow>
      <SosTitle as="h1" size="xl">我的作品</SosTitle>
      <p class="sos-copy">管理你在画廊发布的绘画作品——编辑信息或下架。</p>
    </header>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
    <SosNotice v-if="okMsg" tone="success">{{ okMsg }}</SosNotice>

    <div class="huc__toolbar">
      <SosButton
        v-for="f in FILTERS"
        :key="f.key"
        size="sm"
        :variant="filter === f.key ? 'primary' : 'ghost'"
        @click="setFilter(f.key)"
      >
        {{ f.label }}
      </SosButton>
      <span class="huc__toolbar-spacer" />
      <SosButton size="sm" variant="secondary" :loading="claiming" @click="claimHistory">
        认领历史作品
      </SosButton>
    </div>

    <div v-if="loading" class="huc__grid">
      <SosSkeleton v-for="i in 6" :key="i" variant="block" style="height: 16rem" />
    </div>
    <SosEmptyState
      v-else-if="!items.length"
      title="还没有作品"
      copy="去画廊上传你的第一件作品吧，发布后会出现在这里。"
    />
    <div v-else class="huc__grid">
      <div v-for="a in items" :key="a.id" class="huc-art-cell">
        <SosArtworkCard
          :title="a.title || '未命名'"
          :image="imgUrl(a.image_url)"
          :category="statusLabel(a.status)"
          :tags="a.tags || []"
          :likes="a.like_total"
          @click="openEdit(a)"
        />
        <div class="huc-art-cell__actions">
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
      title="编辑作品"
      @update:open="(v) => { if (!v) editing = null }"
    >
      <form class="sos-stack" @submit.prevent="saveEdit">
        <SosField label="标题" required>
          <SosInput v-model="form.title" maxlength="200" />
        </SosField>
        <SosField label="简介">
          <SosTextarea v-model="form.description" :rows="3" maxlength="4000" />
        </SosField>
        <SosField label="标签" help="空格或逗号分隔">
          <SosInput v-model="form.tags" />
        </SosField>
        <SosField label="来源链接">
          <SosInput v-model="form.origin_url" placeholder="https://…" />
        </SosField>
      </form>
      <template #footer>
        <SosButton variant="ghost" @click="editing = null">取消</SosButton>
        <SosButton :loading="saving" @click="saveEdit">保存</SosButton>
      </template>
    </SosModal>

    <!-- 下架确认 -->
    <SosModal
      :open="!!removing"
      title="下架作品"
      @update:open="(v) => { if (!v) removing = null }"
    >
      <p class="sos-copy">
        确定下架「{{ removing?.title || '未命名' }}」吗？下架后将从画廊隐藏，数据保留，可联系管理员恢复。
      </p>
      <template #footer>
        <SosButton variant="ghost" @click="removing = null">取消</SosButton>
        <SosButton variant="danger" @click="confirmRemove">确认下架</SosButton>
      </template>
    </SosModal>
  </div>
</template>
