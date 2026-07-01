<script setup>
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosPagination, SosSkeleton, SosEmptyState } from '@haruhi/ui'
import StoryCard from '@/components/StoryCard.vue'
import { myBookmarks } from '@/api'

const route = useRoute()
const router = useRouter()
const loading = ref(true)
const stories = ref([])
const pagination = ref({ page: 1, totalPages: 1, total: 0 })

async function load(page) {
  loading.value = true
  try {
    const r = await myBookmarks({ page, pageSize: 24 })
    stories.value = r.stories
    pagination.value = r.pagination
  } catch {
    stories.value = []
  } finally {
    loading.value = false
  }
}

watch(
  () => Number(route.query.page) || 1,
  (p) => load(p),
  { immediate: true },
)
</script>

<template>
  <div class="fiction-page">
    <header class="bm__head">
      <h1 class="bm__title">我的书架</h1>
      <p class="bm__sub">共收藏 {{ pagination.total }} 部作品</p>
    </header>

    <div v-if="loading" class="fic-list">
      <div v-for="i in 8" :key="i" class="fic-skel">
        <SosSkeleton variant="title" />
        <SosSkeleton variant="text" />
        <SosSkeleton variant="text" style="width: 60%" />
      </div>
    </div>

    <template v-else-if="stories.length">
      <div class="fic-list">
        <StoryCard v-for="s in stories" :key="s.id" :story="s" />
      </div>
      <div v-if="pagination.totalPages > 1" class="bm__pager">
        <SosPagination
          :model-value="pagination.page"
          :page-count="pagination.totalPages"
          @update:model-value="router.push({ query: { page: $event } })"
        />
      </div>
    </template>

    <SosEmptyState v-else title="书架还是空的" copy="在作品页点「收藏」，就能把喜欢的故事放进这里。">
      <template #actions>
        <RouterLink to="/library" class="sos-button sos-button--primary">去书库逛逛</RouterLink>
      </template>
    </SosEmptyState>
  </div>
</template>

<style scoped>
.bm__head {
  margin-bottom: var(--sos-space-6);
}
.bm__title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-2xl);
  margin: 0;
}
.bm__sub {
  color: var(--sos-text-secondary);
  margin: 6px 0 0;
}
.bm__pager {
  display: flex;
  justify-content: center;
  margin-top: var(--sos-space-8);
}
</style>
