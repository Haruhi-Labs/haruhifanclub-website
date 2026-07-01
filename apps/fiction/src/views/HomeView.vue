<script setup>
import { ref, onMounted, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { SosButton, SosSkeleton } from '@haruhi/ui'
import StoryCard from '@/components/StoryCard.vue'
import CoverImage from '@/components/CoverImage.vue'
import { getSpotlight } from '@/api'
import { CATEGORIES, compact, wordLabel } from '@/lib/format'

const loading = ref(true)
const data = ref({ featured: [], latest: [], popular: [], updated: [] })

const hero = computed(() => data.value.featured[0] || data.value.updated[0] || data.value.latest[0] || null)
const featuredRest = computed(() => data.value.featured.slice(1, 5))
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
    <!-- Hero -->
    <section class="fic-hero">
      <div class="fic-hero__intro">
        <p class="fic-hero__eyebrow">凉宫春日应援团 · 同人文库</p>
        <h1 class="fic-hero__title">在这里，续写<br />那个不会结束的夏天</h1>
        <p class="fic-hero__lead">阅读团员们创作的凉宫春日同人小说，或提笔写下你心中的 SOS 团故事。</p>
        <div class="fic-hero__cta">
          <SosButton variant="primary" size="lg" @click="$router.push('/library')">进入书库</SosButton>
          <SosButton variant="secondary" size="lg" @click="$router.push('/write')">开始创作</SosButton>
        </div>
      </div>

      <RouterLink v-if="hero" :to="`/story/${hero.id}`" class="fic-hero__feature">
        <div class="fic-hero__feature-cover">
          <CoverImage :path="hero.coverPath" :title="hero.title" :category="hero.category" />
        </div>
        <div class="fic-hero__feature-body">
          <span class="fic-hero__badge">编辑推荐</span>
          <h2>{{ hero.title }}</h2>
          <p class="fic-hero__feature-author">{{ hero.authorName }}</p>
          <p class="fic-hero__feature-summary">{{ hero.summary || '暂无简介' }}</p>
          <p class="fic-hero__feature-stats">
            {{ wordLabel(hero.wordCount) }} · {{ compact(hero.viewCount) }} 阅读 · {{ compact(hero.likeCount) }} 喜欢
          </p>
        </div>
      </RouterLink>
      <div v-else-if="loading" class="fic-hero__feature">
        <SosSkeleton variant="block" style="aspect-ratio: 3/4; width: 150px" />
        <div style="flex: 1"><SosSkeleton variant="title" /><SosSkeleton variant="text" /><SosSkeleton variant="text" /></div>
      </div>
    </section>

    <!-- 分类导航 -->
    <nav class="fic-cats">
      <RouterLink to="/library" class="fic-cats__all">全部</RouterLink>
      <RouterLink v-for="c in CATEGORIES" :key="c.slug" :to="`/library?category=${c.slug}`" class="fic-cats__chip">
        {{ c.label }}
      </RouterLink>
    </nav>

    <!-- 推荐其余（≥2 才成组展示，避免孤卡） -->
    <section v-if="featuredRest.length >= 2" class="fiction-rail">
      <div class="fiction-rail__head">
        <h2 class="fiction-rail__title">精选佳作</h2>
      </div>
      <div class="fiction-grid">
        <StoryCard v-for="s in featuredRest" :key="s.id" :story="s" />
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
.fic-hero {
  display: grid;
  grid-template-columns: 1.05fr 0.95fr;
  gap: clamp(24px, 4vw, 56px);
  align-items: center;
  padding: clamp(28px, 4.5vw, 56px);
  margin-bottom: var(--sos-space-8);
  border-radius: 24px;
  border: 1px solid var(--sos-border-subtle);
  background:
    radial-gradient(90% 130% at 6% -12%, var(--sos-accent-soft), transparent 52%),
    linear-gradient(135deg, color-mix(in srgb, var(--sos-accent-soft) 45%, var(--sos-bg-surface)), var(--sos-bg-surface));
}
.fic-hero__eyebrow {
  color: var(--sos-accent);
  font-weight: 600;
  font-size: var(--sos-text-sm);
  letter-spacing: var(--sos-tracking-wide);
  margin: 0 0 var(--sos-space-3);
}
.fic-hero__title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-hero);
  line-height: var(--sos-leading-tight);
  font-weight: 700;
  margin: 0 0 var(--sos-space-4);
}
.fic-hero__lead {
  font-size: var(--sos-text-lg);
  color: var(--sos-text-secondary);
  line-height: var(--sos-leading-body);
  max-width: 34ch;
  margin: 0 0 var(--sos-space-6);
}
.fic-hero__cta {
  display: flex;
  gap: var(--sos-space-3);
  flex-wrap: wrap;
}
.fic-hero__feature {
  display: flex;
  gap: var(--sos-space-5);
  padding: var(--sos-space-5);
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-lg);
  box-shadow: var(--sos-shadow-float);
  text-decoration: none;
  color: inherit;
  transition: box-shadow 0.18s ease, transform 0.18s ease;
}
.fic-hero__feature:hover {
  box-shadow: var(--sos-shadow-lg, var(--sos-shadow-float));
  transform: translateY(-3px);
}
.fic-hero__feature-cover {
  width: 162px;
  flex: none;
}
.fic-hero__feature-body {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-2);
  min-width: 0;
}
.fic-hero__badge {
  align-self: flex-start;
  font-size: var(--sos-text-2xs);
  color: #fff;
  background: var(--sos-accent);
  border-radius: var(--sos-radius-full);
  padding: 2px 10px;
  font-weight: 600;
}
.fic-hero__feature-body h2 {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-xl);
  margin: 0;
  line-height: var(--sos-leading-snug);
}
.fic-hero__feature-author {
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
  margin: 0;
}
.fic-hero__feature-summary {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-sm);
  line-height: var(--sos-leading-body);
  margin: 0;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}
.fic-hero__feature-stats {
  margin-top: auto;
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}

.fic-cats {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
  padding: var(--sos-space-4) 0;
  border-block: 1px solid var(--sos-border-subtle);
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
  .fic-hero {
    grid-template-columns: 1fr;
    gap: var(--sos-space-6);
  }
  .fic-split {
    grid-template-columns: 1fr;
  }
}
</style>
