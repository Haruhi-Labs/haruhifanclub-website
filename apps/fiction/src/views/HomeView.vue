<script setup>
import { ref, onMounted, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { SosButton, SosSkeleton } from '@haruhi/ui'
import StoryCard from '@/components/StoryCard.vue'
import { getSpotlight } from '@/api'
import { CATEGORIES } from '@/lib/format'

const loading = ref(true)
const data = ref({ featured: [], latest: [], popular: [], updated: [] })

const hasAny = computed(
  () => (data.value.latest || []).length > 0 || (data.value.updated || []).length > 0,
)

onMounted(async () => {
  try {
    const r = await getSpotlight()
    data.value = { featured: r.featured, latest: r.latest, popular: r.popular, updated: r.updated }
  } catch {
    /* 首屏容错：接口异常时退化为空态 */
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="fiction-page fiction-page--wide">
    <!-- 分类导航 -->
    <nav class="fic-cats">
      <RouterLink to="/library" class="fic-cats__all">全部</RouterLink>
      <RouterLink v-for="c in CATEGORIES" :key="c.slug" :to="`/library?category=${c.slug}`" class="fic-cats__chip">
        {{ c.label }}
      </RouterLink>
    </nav>

    <!-- 精选佳作 -->
    <section v-if="data.featured.length" class="fiction-rail">
      <div class="fiction-rail__head">
        <h2 class="fiction-rail__title">精选佳作</h2>
      </div>
      <div class="fiction-grid">
        <StoryCard v-for="s in data.featured.slice(0, 10)" :key="s.id" :story="s" />
      </div>
    </section>

    <!-- 加载骨架 -->
    <section v-if="loading" class="fiction-rail">
      <div class="fiction-grid">
        <div v-for="i in 6" :key="i">
          <SosSkeleton variant="block" style="aspect-ratio: 3/4" />
          <SosSkeleton variant="text" style="margin-top: 8px" />
        </div>
      </div>
    </section>

    <template v-else-if="hasAny">
      <!-- 最近更新 -->
      <section v-if="data.updated.length" class="fiction-rail">
        <div class="fiction-rail__head">
          <h2 class="fiction-rail__title">最近更新</h2>
          <RouterLink to="/library?sort=updated" class="fiction-rail__more">查看全部 →</RouterLink>
        </div>
        <div class="fiction-grid">
          <StoryCard v-for="s in data.updated.slice(0, 6)" :key="s.id" :story="s" />
        </div>
      </section>

      <div class="fic-split">
        <!-- 人气排行 -->
        <section v-if="data.popular.length" class="fiction-rail fic-split__side">
          <div class="fiction-rail__head">
            <h2 class="fiction-rail__title">人气排行</h2>
          </div>
          <ol class="fiction-rank">
            <li v-for="(s, i) in data.popular.slice(0, 5)" :key="s.id" class="fiction-rank__item">
              <span class="fiction-rank__no">{{ i + 1 }}</span>
              <StoryCard :story="s" layout="compact" />
            </li>
          </ol>
        </section>

        <!-- 最新发布 -->
        <section v-if="data.latest.length" class="fiction-rail fic-split__main">
          <div class="fiction-rail__head">
            <h2 class="fiction-rail__title">新作上架</h2>
            <RouterLink to="/library" class="fiction-rail__more">查看全部 →</RouterLink>
          </div>
          <div class="fiction-grid">
            <StoryCard v-for="s in data.latest.slice(0, 6)" :key="s.id" :story="s" />
          </div>
        </section>
      </div>
    </template>

    <!-- 空态 -->
    <section v-else class="fic-empty">
      <h2>书库正在等待第一个故事</h2>
      <p>成为第一位在同人文库留下作品的团员吧。</p>
      <SosButton variant="primary" size="lg" @click="$router.push('/write')">去创作</SosButton>
    </section>
  </div>
</template>

<style scoped>
/* 分类导航（首页顶部第一块） */
.fic-cats {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
  padding-bottom: var(--sos-space-5);
  margin-bottom: var(--sos-space-2);
  border-bottom: 1px solid var(--sos-border-subtle);
}
.fic-cats__chip,
.fic-cats__all {
  text-decoration: none;
  font-size: var(--sos-text-sm);
  color: var(--sos-text-secondary);
  padding: 6px 14px;
  border-radius: var(--sos-radius-full);
  background: var(--sos-bg-subtle);
  transition: all 0.15s ease;
}
.fic-cats__all {
  color: var(--sos-accent-contrast);
  background: var(--sos-accent);
}
.fic-cats__chip:hover {
  background: var(--sos-accent-soft);
  color: var(--sos-accent);
}

.fic-split {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: var(--sos-space-8);
  align-items: start;
}

.fic-empty {
  text-align: center;
  padding: var(--sos-space-11) 0;
}
.fic-empty h2 {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-2xl);
  margin: 0 0 var(--sos-space-2);
}
.fic-empty p {
  color: var(--sos-text-secondary);
  margin: 0 0 var(--sos-space-5);
}

@media (max-width: 860px) {
  .fic-split {
    grid-template-columns: 1fr;
  }
}
</style>
