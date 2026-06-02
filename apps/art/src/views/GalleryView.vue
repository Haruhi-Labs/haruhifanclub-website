<template>
  <section class="container-card">
    <FilterPanel
      v-if="!activeTag && !activeAuthor"
      :content="store.content"
      :sourceMode="store.sourceMode"
      :sortMode="store.sortMode"
      :q="store.q"
      :searchField="store.searchField"
      @update:content="v => { store.setFilters({ content: v }); reload() }"
      @update:sourceMode="v => { store.setFilters({ sourceMode: v }); reload() }"
      @update:sortMode="v => { store.setFilters({ sortMode: v }); reload() }"
      @update:q="v => store.setFilters({ q: v })"
      @update:searchField="v => store.setFilters({ searchField: v })"
      @search="reload"
      @clear="store.setFilters({ searchField: 'all' })"
    />

    <div v-else class="tag-header">
      <div class="tag-info">
        <h2 v-if="activeTag" class="tag-title">标签：{{ activeTag }}</h2>
        <h2 v-else-if="activeAuthor" class="tag-title author-title">
          <span>作者：</span>
          <span class="highlight">{{ activeAuthor.name }}</span>
        </h2>
      </div>
      <button class="btn-return" @click="exitMode">
        <span class="icon">↩</span> 返回画廊
      </button>
    </div>

    <div v-if="store.error" class="errorBox">{{ store.error }}</div>

    <ArtworkGrid
      v-else
      class="main-grid"
      :items="store.list"
      :page="store.page"
      :hasMore="store.hasMore"
      :loading="store.loading"
      @open="openItem"
      @like="likeItem"
      @tag="onTag"
      @author="onAuthor"
      @prevPage="handlePrevPage"
      @nextPage="handleNextPage"
    />

    <div class="statusRow footer-mode">
      <div class="left">
        <span class="muted">排序：{{ sortLabel }}</span>
        <span class="muted">· 第 {{ store.page }} 页</span>
        <span class="muted">· 共 {{ store.total }} 条</span>
        <span class="muted" v-if="store.usingSeed"></span>
      </div>
    </div>

    <ArtworkModal
      :model-value="modalOpen"
      :item="activeItem"
      @update:model-value="val => !val && closeModal()"
      @tag="onTag"
      @close="closeModal"
    />
  </section>
</template>

<script setup>
import { onMounted, onUnmounted, ref, watch, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useGalleryStore } from '../stores/galleryStore.js'
import FilterPanel from '../components/FilterPanel.vue'
import ArtworkGrid from '../components/ArtworkGrid.vue'
import ArtworkModal from '../components/ArtworkModal.vue'

const store = useGalleryStore()
const route = useRoute()
const router = useRouter()

const modalOpen = ref(false)
const activeItem = ref(null)
const activeTag = ref('')
const activeAuthor = ref(null) // { uid, name }

const sortLabel = computed(() => {
  const m = store.sortMode
  if (m === 'likes') return '点赞最高'
  if (m === 'time') return '最新'
  return '随机'
})

// --- 翻页处理 ---
async function handlePrevPage() {
  if (store.page > 1) {
    store.page--
    await store.load()
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }
}

async function handleNextPage() {
  if (store.hasMore) {
    store.page++
    await store.load()
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }
}

// --- 响应式 PageSize 逻辑 ---
function updatePageSize(allowedToReload = true) {
  const isMobile = window.innerWidth <= 768
  const targetSize = isMobile ? 8 : 12

  if (store.limit !== targetSize) {
    store.limit = targetSize
    if (allowedToReload) {
      reload()
    }
  }
}

function reload() {
  store.page = 1
  store.load()
}

function likeItem(it) {
  store.likeArtwork(it)
}

// ---------------------------------------------------------
// URL 驱动的核心逻辑
// ---------------------------------------------------------

function onTag(t) {
  router.push({ query: { ...route.query, tag: t, author: undefined, artwork: undefined } })
}

function onAuthor(authorInfo) {
  router.push({ query: { ...route.query, author: authorInfo.uid, tag: undefined, artwork: undefined } })
}

function openItem(it) {
  router.push({ query: { ...route.query, artwork: it.id } })
}

function closeModal() {
  const q = { ...route.query }
  delete q.artwork
  router.push({ query: q })
}

function exitMode() {
  router.push({ query: {} })
}

// --- 核心：将路由解析逻辑提取出来 ---
function syncStateFromRoute(q) {
  const newTag = q.tag
  const newAuthorUid = q.author
  const newArtworkId = q.artwork

  let needsReload = false

  // A. 处理标签模式变化
  if (newTag && newTag !== activeTag.value) {
    activeTag.value = newTag
    activeAuthor.value = null
    store.setFilters({ q: newTag, searchField: 'tag' })
    needsReload = true
  } else if (!newTag && activeTag.value) {
    activeTag.value = ''
    if (!newAuthorUid) {
      store.setFilters({ q: '', searchField: 'all' })
      needsReload = true
    }
  }

  // B. 处理作者模式变化
  if (newAuthorUid) {
    if (!activeAuthor.value || activeAuthor.value.uid !== newAuthorUid) {
      activeAuthor.value = { uid: newAuthorUid, name: newAuthorUid }
      activeTag.value = ''
      store.setFilters({ q: newAuthorUid, searchField: 'uid' })
      needsReload = true
    }
  } else if (!newAuthorUid && activeAuthor.value) {
    activeAuthor.value = null
    if (!newTag) {
      store.setFilters({ q: '', searchField: 'all' })
      needsReload = true
    }
  }

  // C. 处理详情弹窗
  if (newArtworkId) {
    // Always fetch detail to ensure we have full images list (and comments etc.)
    // The store handles caching/optimization internally now
    store.fetchArtworkById(newArtworkId).then(item => {
      if (item) {
        activeItem.value = item
        modalOpen.value = true
      }
    })
  } else {
    modalOpen.value = false
    activeItem.value = null
  }

  // 普通搜索参数 q
  if (q.q && q.q !== store.q) {
    store.setFilters({ q: q.q })
    needsReload = true
  }

  return needsReload
}

// 监听路由变化
watch(() => route.query, (newQ) => {
  const shouldLoad = syncStateFromRoute(newQ)
  if (shouldLoad) reload()
}, { immediate: false })

// 监听列表数据变化（修正作者名/打开详情）
watch(() => store.list, (list) => {
  if (!list || list.length === 0) return

  if (activeAuthor.value && activeAuthor.value.name === activeAuthor.value.uid) {
    const first = list[0]
    if (first && first.uploader_name) {
      activeAuthor.value.name = first.uploader_name
    }
  }

  const targetId = route.query.artwork
  if (targetId && !modalOpen.value) {
    const target = list.find(i => String(i.id) === String(targetId))
    if (target) {
      activeItem.value = target
      modalOpen.value = true
    }
  }
})

// ✅ 修复：必须用同一个函数引用，remove 才有效
let _resizeTimer = null
const onResize = () => {
  clearTimeout(_resizeTimer)
  _resizeTimer = setTimeout(() => updatePageSize(true), 150)
}

// --- 初始化顺序修复 ---
onMounted(() => {
  // 1. 先确定尺寸（不触发 reload）
  updatePageSize(false)

  // 2. 再根据 URL 设置 Filter
  const needsReload = syncStateFromRoute(route.query)

  // 3. 最后统一加载一次
  if (needsReload || store.list.length === 0) {
    reload()
  }

  // 4. 绑定监听
  window.addEventListener('resize', onResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
  clearTimeout(_resizeTimer)
})
</script>

<style scoped>
.container-card {
  max-width: 1450px;
  margin: 0 auto;
  position: relative;
}

.main-grid { margin-top: 90px; }

.statusRow { margin:14px 0; display:flex; justify-content:space-between; align-items:center; }
.statusRow.footer-mode { margin-top: 30px; border-top: 1px dashed rgba(0,0,0,0.08); padding-top: 20px; }
.left{ display:flex; gap:10px; align-items:center; flex-wrap: wrap; }
.errorBox{ padding:12px; background:#fee; color:red; border-radius:8px; }
.muted{ color:#999; font-size:12px; }

.tag-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 0;
  margin-bottom: 20px;
  border-bottom: 1px dashed rgba(0,0,0,0.08);
  animation: fadeIn 0.4s ease;
}

.tag-title {
  font-size: 26px;
  font-weight: 950;
  color: #1a1a1a;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 12px;
}

.tag-title::before {
  content: '#';
  color: var(--neon-cyan, #00f2ff);
  font-size: 32px;
  font-weight: 300;
  transform: translateY(-2px);
}

.author-title::before {
  content: '@';
  color: #e9b5fd;
}
.author-title .highlight {
  color: #1a1a1a;
}

.btn-return {
  padding: 10px 20px;
  background: #ffffff;
  border: 1px solid #e5e5e5;
  border-radius: 99px;
  cursor: pointer;
  font-weight: 800;
  color: #555;
  transition: all 0.2s cubic-bezier(0.25, 0.8, 0.25, 1);
  display: flex;
  align-items: center;
  gap: 8px;
  box-shadow: 0 4px 10px rgba(0,0,0,0.03);
}

.btn-return:hover {
  background: #fff;
  color: #000;
  border-color: #ccc;
  transform: translateX(-4px);
  box-shadow: 0 6px 15px rgba(0,0,0,0.08);
}

.btn-return:active {
  transform: translateX(-4px) scale(0.96);
}

.icon {
  font-size: 18px;
  line-height: 1;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

@media (max-width: 768px) {
  .tag-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
    padding: 16px 0;
  }
  .btn-return {
    width: 100%;
    justify-content: center;
  }
}
</style>
