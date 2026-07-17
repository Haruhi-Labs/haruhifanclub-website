<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import {
  SosButton,
  SosEyebrow,
  SosTitle,
  SosNotice,
  SosSkeleton,
  SosEmptyState,
  SosModal,
  SosPagination,
  SosField,
  SosInput,
  SosTextarea,
  SosSwitch,
} from '@haruhi/ui'
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
const page = ref(1)
const pageSize = 24
const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)))
const loading = ref(true)
const error = ref('')
const okMsg = ref('')
const exhibitSaving = ref(new Set())

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

let loadSeq = 0
async function load() {
  const seq = ++loadSeq
  loading.value = true
  error.value = ''
  try {
    const r = await hub.art.artworks({ status: filter.value, page: page.value, pageSize })
    if (seq !== loadSeq) return // 丢弃过期响应，避免旧筛选/页结果覆盖当前
    items.value = r.data || []
    total.value = r.total || 0
  } catch (e) {
    if (seq === loadSeq) error.value = e?.message || '加载失败'
  } finally {
    if (seq === loadSeq) loading.value = false
  }
}
function setFilter(k) {
  if (filter.value === k) return
  filter.value = k
  page.value = 1
  load()
}
function go(p) {
  const np = Math.min(Math.max(1, p), totalPages.value)
  if (np === page.value) return
  page.value = np
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

async function toggleExhibit(artwork, enabled) {
  if (exhibitSaving.value.has(artwork.id)) return
  const previous = Boolean(artwork.exhibit_enabled)
  artwork.exhibit_enabled = enabled
  exhibitSaving.value = new Set(exhibitSaving.value).add(artwork.id)
  error.value = ''
  try {
    const result = await hub.art.updateArtwork(artwork.id, { exhibit_enabled: enabled })
    artwork.exhibit_enabled = Boolean(result.exhibit_enabled)
  } catch (e) {
    artwork.exhibit_enabled = previous
    error.value = e?.message || '展位状态更新失败'
  } finally {
    const next = new Set(exhibitSaving.value)
    next.delete(artwork.id)
    exhibitSaving.value = next
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
      <article v-for="a in items" :key="a.id" class="huc-mcard">
        <button class="huc-mcard__media" type="button" title="编辑作品" @click="openEdit(a)">
          <img v-if="a.image_url" :src="imgUrl(a.image_url)" :alt="a.title || '作品'" />
          <span class="huc-mcard__status" :data-status="a.status">{{ statusLabel(a.status) }}</span>
        </button>
        <div class="huc-mcard__body">
          <h3 class="huc-mcard__title">{{ a.title || '未命名' }}</h3>
          <div v-if="a.tags && a.tags.length" class="huc-mcard__tags">
            <span v-for="t in a.tags.slice(0, 3)" :key="t">{{ t }}</span>
          </div>
          <div v-if="a.exhibit_eligible" class="huc-mcard__exhibit">
            <SosSwitch
              :model-value="Boolean(a.exhibit_enabled)"
              :disabled="exhibitSaving.has(a.id)"
              @update:model-value="value => toggleExhibit(a, value)"
            >
              展位展示
            </SosSwitch>
          </div>
          <div class="huc-mcard__foot">
            <span class="huc-mcard__likes">♥ {{ a.like_total ?? 0 }}</span>
            <span class="huc__toolbar-spacer" />
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
      </article>
    </div>

    <SosPagination
      v-if="!loading && totalPages > 1"
      :model-value="page"
      :page-count="totalPages"
      @update:model-value="go"
    />

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
