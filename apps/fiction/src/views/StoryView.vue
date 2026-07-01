<script setup>
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { SosButton, SosSkeleton, SosBadge, useToast } from '@haruhi/ui'
import CoverImage from '@/components/CoverImage.vue'
import CommentSection from '@/components/CommentSection.vue'
import { getStory, bumpView, toggleLike, toggleBookmark, getProgress, session } from '@/api'
import { categoryLabel, wordLabel, compact, fmtDate, readingMinutes } from '@/lib/format'

const route = useRoute()
const router = useRouter()
const toast = useToast()

const id = computed(() => Number(route.params.id))
const loading = ref(true)
const notFound = ref(false)
const story = ref(null)
const chapters = ref([])
const liked = ref(false)
const bookmarked = ref(false)
const isAuthor = ref(false)
const resumeChapterId = ref(null)

const user = computed(() => session.state.user)
const firstChapterId = computed(() => chapters.value[0]?.id)
const readTarget = computed(() => resumeChapterId.value || firstChapterId.value)

async function load() {
  loading.value = true
  notFound.value = false
  try {
    const r = await getStory(id.value)
    story.value = r.story
    chapters.value = r.chapters
    liked.value = r.liked
    bookmarked.value = r.bookmarked
    isAuthor.value = r.isAuthor
    bumpView(id.value).catch(() => {})
    if (user.value) {
      getProgress(id.value)
        .then((p) => {
          if (p.progress && p.progress.chapterId) resumeChapterId.value = p.progress.chapterId
        })
        .catch(() => {})
    }
  } catch (e) {
    if (e.status === 404) notFound.value = true
  } finally {
    loading.value = false
  }
}

function requireLogin() {
  if (user.value) return true
  router.push({ name: 'login', query: { redirect: route.fullPath } })
  return false
}

async function onLike() {
  if (!requireLogin()) return
  try {
    const r = await toggleLike(id.value)
    liked.value = r.liked
    story.value.likeCount = r.likeCount
  } catch (e) {
    toast.danger(e.message || '操作失败')
  }
}
async function onBookmark() {
  if (!requireLogin()) return
  try {
    const r = await toggleBookmark(id.value)
    bookmarked.value = r.bookmarked
    story.value.bookmarkCount = r.bookmarkCount
    toast.success(r.bookmarked ? '已加入书架' : '已移出书架')
  } catch (e) {
    toast.danger(e.message || '操作失败')
  }
}

watch(id, load, { immediate: true })
</script>

<template>
  <div class="fiction-page story">
    <div v-if="loading" class="story__hero">
      <SosSkeleton variant="block" style="width: 200px; aspect-ratio: 3/4" />
      <div style="flex: 1">
        <SosSkeleton variant="title" />
        <SosSkeleton variant="text" />
        <SosSkeleton variant="text" />
      </div>
    </div>

    <div v-else-if="notFound" class="story__missing">
      <h2>作品不存在或已下架</h2>
      <RouterLink to="/library" class="sos-button sos-button--primary">返回书库</RouterLink>
    </div>

    <template v-else-if="story">
      <!-- 作品头 -->
      <section class="story__hero">
        <div class="story__cover">
          <CoverImage :path="story.coverPath" :title="story.title" :category="story.category" />
        </div>
        <div class="story__info">
          <div class="story__tagline">
            <RouterLink :to="`/library?category=${story.category}`" class="story__cat">
              {{ categoryLabel(story.category) }}
            </RouterLink>
            <SosBadge v-if="story.isCompleted" variant="success">已完结</SosBadge>
            <SosBadge v-else variant="accent">连载中</SosBadge>
            <SosBadge v-if="story.contentRating === 'mature'" variant="danger">限制级</SosBadge>
            <SosBadge v-else-if="story.contentRating === 'teen'" variant="outline">青少年</SosBadge>
          </div>
          <h1 class="story__title">{{ story.title }}</h1>
          <p class="story__author">作者：{{ story.authorName || '佚名' }}</p>

          <ul class="story__stats">
            <li><b>{{ wordLabel(story.wordCount) }}</b><span>字数</span></li>
            <li><b>{{ story.chapterCount }}</b><span>章节</span></li>
            <li><b>{{ compact(story.viewCount) }}</b><span>阅读</span></li>
            <li><b>{{ compact(story.likeCount) }}</b><span>点赞</span></li>
            <li><b>{{ compact(story.bookmarkCount) }}</b><span>收藏</span></li>
          </ul>

          <p v-if="story.summary" class="story__summary">{{ story.summary }}</p>

          <ul v-if="story.tags && story.tags.length" class="story__tags">
            <li v-for="t in story.tags" :key="t">
              <RouterLink :to="`/library?tag=${encodeURIComponent(t)}`">#{{ t }}</RouterLink>
            </li>
          </ul>

          <div class="story__actions">
            <SosButton
              v-if="readTarget"
              variant="primary"
              size="lg"
              @click="router.push(`/story/${story.id}/chapter/${readTarget}`)"
            >
              {{ resumeChapterId ? '继续阅读' : '开始阅读' }}
            </SosButton>
            <SosButton :variant="liked ? 'primary' : 'secondary'" @click="onLike">
              {{ liked ? '❤ 已赞' : '♡ 点赞' }} {{ compact(story.likeCount) }}
            </SosButton>
            <SosButton :variant="bookmarked ? 'primary' : 'secondary'" @click="onBookmark">
              {{ bookmarked ? '★ 已收藏' : '☆ 收藏' }}
            </SosButton>
            <RouterLink v-if="isAuthor" :to="`/write/${story.id}`" class="sos-button sos-button--ghost">
              管理作品
            </RouterLink>
          </div>
        </div>
      </section>

      <!-- 目录 -->
      <section class="story__toc">
        <div class="story__toc-head">
          <h2>目录</h2>
          <span>{{ chapters.length }} 章</span>
        </div>
        <ol v-if="chapters.length" class="story__chapters">
          <li v-for="(c, i) in chapters" :key="c.id">
            <RouterLink :to="`/story/${story.id}/chapter/${c.id}`">
              <span class="story__ch-no">{{ i + 1 }}</span>
              <span class="story__ch-title">{{ c.title }}</span>
              <span class="story__ch-meta">{{ readingMinutes(c.wordCount) }} 分钟 · {{ fmtDate(c.publishedAt) }}</span>
            </RouterLink>
          </li>
        </ol>
        <p v-else class="story__toc-empty">作者还没有发布章节。</p>
      </section>

      <!-- 评论 -->
      <CommentSection :story-id="story.id" />
    </template>
  </div>
</template>

<style scoped>
.story__hero {
  display: flex;
  gap: var(--sos-space-8);
  align-items: flex-start;
}
.story__cover {
  width: 210px;
  flex: none;
}
.story__info {
  flex: 1;
  min-width: 0;
}
.story__tagline {
  display: flex;
  align-items: center;
  gap: var(--sos-space-2);
  margin-bottom: var(--sos-space-3);
}
.story__cat {
  color: var(--sos-accent);
  font-weight: 600;
  font-size: var(--sos-text-sm);
  text-decoration: none;
}
.story__title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-3xl);
  line-height: var(--sos-leading-tight);
  margin: 0 0 var(--sos-space-2);
}
.story__author {
  color: var(--sos-text-secondary);
  margin: 0 0 var(--sos-space-5);
}
.story__stats {
  list-style: none;
  display: flex;
  gap: var(--sos-space-7);
  padding: var(--sos-space-4) 0;
  margin: 0 0 var(--sos-space-5);
  border-block: 1px solid var(--sos-border-subtle);
}
.story__stats li {
  display: flex;
  flex-direction: column;
}
.story__stats b {
  font-family: var(--sos-font-reading);
  font-size: var(--sos-text-lg);
  color: var(--sos-text-primary);
}
.story__stats span {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.story__summary {
  color: var(--sos-text-secondary);
  line-height: var(--sos-leading-body);
  white-space: pre-wrap;
  margin: 0 0 var(--sos-space-4);
}
.story__tags {
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
  padding: 0;
  margin: 0 0 var(--sos-space-6);
}
.story__tags a {
  color: var(--sos-text-secondary);
  background: var(--sos-bg-subtle);
  border-radius: var(--sos-radius-full);
  padding: 3px 12px;
  font-size: var(--sos-text-xs);
  text-decoration: none;
}
.story__tags a:hover {
  color: var(--sos-accent);
  background: var(--sos-accent-soft);
}
.story__actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-3);
  align-items: center;
}
.story__toc {
  margin-top: var(--sos-space-9);
}
.story__toc-head {
  display: flex;
  align-items: baseline;
  gap: var(--sos-space-3);
  margin-bottom: var(--sos-space-4);
}
.story__toc-head h2 {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-xl);
  margin: 0;
}
.story__toc-head span {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-sm);
}
.story__chapters {
  list-style: none;
  padding: 0;
  margin: 0;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2px var(--sos-space-6);
  counter-reset: ch;
}
.story__chapters a {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
  padding: var(--sos-space-3) var(--sos-space-3);
  border-radius: var(--sos-radius-sm);
  text-decoration: none;
  color: var(--sos-text-primary);
  border-bottom: 1px solid var(--sos-border-subtle);
}
.story__chapters a:hover {
  background: var(--sos-accent-soft);
}
.story__ch-no {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-sm);
  width: 2em;
  flex: none;
}
.story__ch-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.story__ch-meta {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-xs);
  flex: none;
}
.story__missing {
  text-align: center;
  padding: var(--sos-space-11) 0;
}
.story__missing h2 {
  font-family: var(--sos-display-family, var(--sos-font-display));
  margin-bottom: var(--sos-space-5);
}

@media (max-width: 720px) {
  .story__hero {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }
  .story__cover {
    width: 160px;
  }
  .story__stats {
    justify-content: space-between;
    gap: var(--sos-space-3);
    width: 100%;
  }
  .story__tagline,
  .story__actions,
  .story__tags {
    justify-content: center;
  }
  .story__chapters {
    grid-template-columns: 1fr;
  }
}
</style>
