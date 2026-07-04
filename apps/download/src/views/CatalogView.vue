<script setup>
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosSearch, SosNotice, SosEmptyState, SosSkeleton } from '@haruhi/ui'
import ResourceEntry from '@/components/ResourceEntry.vue'
import CategoryBody from '@/components/CategoryBody.vue'
import { useIndex } from '@/lib/store'
import { flattenEntries, searchEntries, findCategory, fmtRelative } from '@/lib/format'

const route = useRoute()
const router = useRouter()
const store = useIndex()

const tree = computed(() => store.data?.tree || [])
const topCats = computed(() => tree.value.filter((n) => n.kind === 'category'))
const empty = computed(() => store.loaded && !topCats.value.length)
// 提交缺失资源：固定的邮件反馈入口（前端写死，不再走语雀 LINK 项）
const submitMail = `mailto:haruhifanclub@outlook.com?subject=${encodeURIComponent(
  '凉宫春日资源站 · 缺失资源反馈',
)}`
const flatAll = computed(() => flattenEntries(tree.value))

// 检索关键词与所选分类（均同步到 URL query，可分享/可回退）
const q = ref(typeof route.query.q === 'string' ? route.query.q : '')
const cat = ref(typeof route.query.cat === 'string' ? route.query.cat : '')
const query = computed(() => q.value.trim())

const activeCat = computed(() => (cat.value ? findCategory(tree.value, cat.value) : null))
// 当前检索范围：选中分类则限定其下条目，否则全部
const scopeFlat = computed(() => {
  if (!activeCat.value) return flatAll.value
  return flattenEntries(activeCat.value.children).map((e) => ({ ...e, top: activeCat.value.title }))
})
const results = computed(() => (query.value ? searchEntries(scopeFlat.value, query.value) : []))

watch([q, cat], () => {
  const nq = {}
  if (query.value) nq.q = query.value
  if (cat.value) nq.cat = cat.value
  router.replace({ query: nq })
})
watch(
  () => route.query,
  (qq) => {
    const nq = typeof qq.q === 'string' ? qq.q : ''
    const nc = typeof qq.cat === 'string' ? qq.cat : ''
    if (nq !== q.value) q.value = nq
    if (nc !== cat.value) cat.value = nc
  },
)

function selectCat(id) {
  cat.value = id
  if (typeof window !== 'undefined') window.scrollTo({ top: 0, behavior: 'smooth' })
}

const metaText = computed(() => {
  const d = store.data
  if (!d) return ''
  if (query.value) return `在 ${scopeFlat.value.length} 项中找到 ${results.value.length} 项`
  const upd = d.contentUpdatedAt ? ` · 更新于 ${fmtRelative(d.contentUpdatedAt)}` : ''
  return `共 ${d.stats.entries} 项 · ${d.stats.topCategories} 类${upd}`
})

const resultLabel = computed(() => {
  let s = `找到 ${results.value.length} 项`
  if (activeCat.value) s += ` · ${activeCat.value.title}`
  return s
})
</script>

<template>
  <div class="dl-page dl-catalog">
    <SosNotice v-if="store.error" tone="danger">{{ store.error }}</SosNotice>
    <SosNotice v-else-if="empty" tone="info">资源索引正在首次同步，请稍后刷新页面再试。</SosNotice>

    <div v-else class="dl-catalog__body">
      <!-- 左侧：搜索 + 分类过滤（sticky 面板） -->
      <aside class="dl-side">
        <div class="dl-side__search">
          <SosSearch v-model="q" placeholder="搜索资源…" />
        </div>
        <p class="dl-side__meta">{{ metaText }}</p>

        <nav class="dl-side__list">
          <button
            type="button"
            class="dl-side__link"
            :class="{ 'is-active': !cat }"
            @click="selectCat('')"
          >
            <span>全部资源</span>
            <span class="dl-side__n">{{ store.data?.stats.entries }}</span>
          </button>
          <button
            v-for="c in topCats"
            :key="c.id"
            type="button"
            class="dl-side__link"
            :class="{ 'is-active': cat === c.id }"
            @click="selectCat(c.id)"
          >
            <span>{{ c.title }}</span>
            <span class="dl-side__n">{{ c.count }}</span>
          </button>
        </nav>

        <div class="dl-side__ext">
          <a :href="submitMail">
            <svg class="dl-side__mail" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M4 6h16a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1z" stroke-linejoin="round" />
              <path d="m4 7 8 6 8-6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            提交缺失资源
          </a>
        </div>
      </aside>

      <!-- 右侧内容 -->
      <div class="dl-content">
        <div v-if="store.loading" class="dl-entrylist">
          <SosSkeleton v-for="i in 10" :key="i" variant="text" style="margin: 12px 16px" />
        </div>

        <!-- 搜索结果 -->
        <template v-else-if="query">
          <div class="dl-content__head">
            <h1 class="dl-content__title">搜索结果</h1>
            <span class="dl-content__n">{{ resultLabel }}</span>
          </div>
          <div v-if="results.length" class="dl-entrylist">
            <ResourceEntry
              v-for="e in results"
              :key="e.id"
              :entry="e"
              :query="q"
              show-path
              show-desc
            />
          </div>
          <SosEmptyState v-else title="没有匹配的资源" copy="换个关键词，或在左侧切换分类范围。" />
        </template>

        <!-- 单个分类 -->
        <section v-else-if="activeCat" class="dl-cat-block">
          <div class="dl-cat-block__head">
            <h1 class="dl-cat-block__title">{{ activeCat.title }}</h1>
            <span class="dl-cat-block__n">{{ activeCat.count }} 项</span>
          </div>
          <CategoryBody :node="activeCat" :depth="0" />
        </section>

        <!-- 全部：按分类分组 -->
        <template v-else>
          <section
            v-for="c in topCats"
            :id="`cat-${c.id}`"
            :key="c.id"
            class="dl-cat-block"
          >
            <div class="dl-cat-block__head">
              <h2 class="dl-cat-block__title">{{ c.title }}</h2>
              <span class="dl-cat-block__n">{{ c.count }} 项</span>
            </div>
            <CategoryBody :node="c" :depth="0" />
          </section>
        </template>
      </div>
    </div>
  </div>
</template>
