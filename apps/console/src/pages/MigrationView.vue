<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { api, store, refreshAdmin, flash, errMsg } from '../api'

interface OrphanItem {
  id: string
  title: string
  by: string | null
  at: string | null
}

const MODULES = [
  { key: 'art', label: '画廊作品' },
  { key: 'news', label: '团报文章' },
  { key: 'exam', label: '考试试卷' },
]

const module = ref('art')
const q = ref('')
const page = ref(1)
const total = ref(0)
const items = ref<OrphanItem[]>([])
const loading = ref(false)
const checked = reactive<Record<string, boolean>>({})
const targetUserId = ref<number | null>(null)
const binding = ref(false)

const pageSize = 20
const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)))
const checkedIds = computed(() => Object.keys(checked).filter((k) => checked[k]))
const moduleLabel = computed(() => MODULES.find((m) => m.key === module.value)?.label || '')
const targetUser = computed(() => store.users.find((u) => u.id === targetUserId.value))

function clearChecked() {
  for (const k in checked) delete checked[k]
}

async function load() {
  loading.value = true
  try {
    const r = await api.get(
      `/admin/migration/orphans?module=${module.value}&q=${encodeURIComponent(q.value.trim())}&page=${page.value}`,
    )
    items.value = r.items || []
    total.value = r.total || 0
  } catch (e) {
    flash(errMsg(e), 'err')
  } finally {
    loading.value = false
  }
}

function switchModule(m: string) {
  if (module.value === m) return
  module.value = m
  page.value = 1
  clearChecked()
  load()
}
function doSearch() {
  page.value = 1
  load()
}
function go(p: number) {
  page.value = Math.min(Math.max(1, p), totalPages.value)
  load()
}
function toggleAll(ev: Event) {
  const on = (ev.target as HTMLInputElement).checked
  for (const it of items.value) checked[it.id] = on
}

async function bind() {
  if (!checkedIds.value.length) {
    flash('请先勾选要迁移的内容', 'err')
    return
  }
  if (!targetUserId.value) {
    flash('请选择目标用户', 'err')
    return
  }
  const u = targetUser.value
  if (
    !confirm(
      `把选中的 ${checkedIds.value.length} 项「${moduleLabel.value}」绑定到「${u?.displayName || u?.username}」？`,
    )
  )
    return
  binding.value = true
  try {
    const r = await api.post('/admin/migration/bind', {
      module: module.value,
      ids: checkedIds.value,
      userId: targetUserId.value,
    })
    flash(`已绑定 ${r.bound} 项到该用户`)
    clearChecked()
    await load()
  } catch (e) {
    flash(errMsg(e), 'err')
  } finally {
    binding.value = false
  }
}

onMounted(() => {
  if (!store.loaded) refreshAdmin().catch((e) => flash(errMsg(e), 'err'))
  load()
})
</script>

<template>
  <p class="muted" style="margin-bottom: 16px">
    把历史游客内容（尚未绑定账号的旧作品 / 文章 / 试卷）归属到现有用户账户。仅作用于「未归属」条目，已归属的不受影响。
  </p>

  <div class="toolbar">
    <div class="seg">
      <button
        v-for="m in MODULES"
        :key="m.key"
        :class="module === m.key ? 'primary' : ''"
        @click="switchModule(m.key)"
      >
        {{ m.label }}
      </button>
    </div>
    <input v-model="q" placeholder="搜索标题 / 原署名…" @keyup.enter="doSearch" />
    <button @click="doSearch">搜索</button>
    <span class="spacer"></span>
    <span class="muted small">共 {{ total }} 项待归属</span>
  </div>

  <section class="card">
    <table class="data">
      <thead>
        <tr>
          <th style="width: 40px"><input type="checkbox" @change="toggleAll" /></th>
          <th style="width: 90px">ID</th>
          <th>标题</th>
          <th>原署名 / 标识</th>
          <th style="width: 170px">时间</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="it in items" :key="it.id">
          <td><input v-model="checked[it.id]" type="checkbox" /></td>
          <td class="muted small">{{ it.id }}</td>
          <td>{{ it.title || '（无标题）' }}</td>
          <td class="muted small">{{ it.by || '—' }}</td>
          <td class="muted small">{{ it.at || '—' }}</td>
        </tr>
        <tr v-if="!loading && !items.length">
          <td colspan="5" class="muted" style="padding: 24px; text-align: center">
            没有待归属的内容
          </td>
        </tr>
      </tbody>
    </table>
    <div
      v-if="totalPages > 1"
      class="row"
      style="justify-content: center; gap: 12px; margin-top: 12px"
    >
      <button :disabled="page <= 1" @click="go(page - 1)">上一页</button>
      <span class="muted small">{{ page }} / {{ totalPages }}</span>
      <button :disabled="page >= totalPages" @click="go(page + 1)">下一页</button>
    </div>
  </section>

  <section class="card">
    <div class="row" style="gap: 12px; flex-wrap: wrap">
      <span>已选 <strong>{{ checkedIds.length }}</strong> 项，绑定到：</span>
      <select v-model="targetUserId">
        <option :value="null">— 选择目标用户 —</option>
        <option v-for="u in store.users" :key="u.id" :value="u.id">
          {{ u.displayName || u.username }}（{{ u.username }}）
        </option>
      </select>
      <span class="spacer"></span>
      <button
        class="primary"
        :disabled="binding || !checkedIds.length || !targetUserId"
        @click="bind"
      >
        绑定到该用户
      </button>
    </div>
  </section>
</template>
