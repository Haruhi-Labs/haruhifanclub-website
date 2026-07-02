<script setup>
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { SosButton, SosSelect, SosSwitch, SosBadge, useToast } from '@haruhi/ui'
import CoverImage from '@/components/CoverImage.vue'
import {
  myStory,
  updateStory,
  deleteStory,
  publishStory,
  unpublishStory,
  createChapter,
  updateChapter,
  deleteChapter,
  reorderChapters,
  uploadCover,
} from '@/api'
import { CATEGORIES, RATINGS, wordLabel, fmtDate } from '@/lib/format'

const route = useRoute()
const router = useRouter()
const toast = useToast()

const id = computed(() => Number(route.params.id))
const loading = ref(true)
const saving = ref(false)
const story = ref(null)
const chapters = ref([])
const form = ref({ title: '', summary: '', category: 'other', contentRating: 'general', isCompleted: false, coverPath: null, tags: [] })
const tagInput = ref('')

async function load() {
  loading.value = true
  try {
    const r = await myStory(id.value)
    story.value = r.story
    chapters.value = r.chapters
    form.value = {
      title: r.story.title,
      summary: r.story.summary,
      category: r.story.category,
      contentRating: r.story.contentRating,
      isCompleted: r.story.isCompleted,
      coverPath: r.story.coverPath,
      tags: [...(r.story.tags || [])],
    }
  } catch (e) {
    toast.danger(e.status === 403 ? '无权管理该作品' : '作品加载失败')
    router.replace('/write')
  } finally {
    loading.value = false
  }
}

async function saveMeta() {
  if (!form.value.title.trim()) {
    toast.danger('标题不能为空')
    return
  }
  saving.value = true
  try {
    await updateStory(id.value, { ...form.value })
    toast.success('已保存')
    await load()
  } catch (e) {
    toast.danger(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

function addTag() {
  const t = tagInput.value.trim().replace(/[,，]/g, '')
  if (t && !form.value.tags.includes(t) && form.value.tags.length < 12) form.value.tags.push(t)
  tagInput.value = ''
}
function removeTag(t) {
  form.value.tags = form.value.tags.filter((x) => x !== t)
}

async function onCover(e) {
  const file = e.target.files?.[0]
  if (!file) return
  try {
    const r = await uploadCover(file)
    form.value.coverPath = r.path
    toast.success('封面已上传，记得保存')
  } catch (er) {
    toast.danger(er.message || '上传失败')
  }
  e.target.value = ''
}

function removeCover() {
  form.value.coverPath = null
  toast.success('已移除封面，保存后生效')
}

async function newChapter() {
  try {
    const r = await createChapter(id.value, { title: `第${chapters.value.length + 1}章` })
    router.push(`/write/${id.value}/chapter/${r.id}`)
  } catch (e) {
    toast.danger(e.message || '创建失败')
  }
}

async function toggleChapter(c) {
  try {
    await updateChapter(id.value, c.id, { status: c.status === 'published' ? 'draft' : 'published' })
    await load()
  } catch (e) {
    toast.danger(e.message || '操作失败')
  }
}
async function removeChapter(c) {
  if (!window.confirm(`确定删除章节「${c.title}」？此操作不可恢复。`)) return
  try {
    await deleteChapter(id.value, c.id)
    await load()
  } catch (e) {
    toast.danger(e.message || '删除失败')
  }
}
async function move(i, dir) {
  const j = i + dir
  if (j < 0 || j >= chapters.value.length) return
  const arr = [...chapters.value]
  ;[arr[i], arr[j]] = [arr[j], arr[i]]
  chapters.value = arr
  try {
    await reorderChapters(id.value, arr.map((c) => c.id))
  } catch {
    await load()
  }
}

async function onPublish() {
  try {
    await publishStory(id.value)
    toast.success('作品已发布')
    await load()
  } catch (e) {
    toast.danger(e.message || '发布失败')
  }
}
async function onUnpublish() {
  try {
    await unpublishStory(id.value)
    await load()
  } catch (e) {
    toast.danger(e.message || '撤回失败')
  }
}
async function onDelete() {
  if (!window.confirm('确定下架并删除该作品？读者将无法再看到。')) return
  try {
    await deleteStory(id.value)
    toast.success('已下架')
    router.push('/write')
  } catch (e) {
    toast.danger(e.message || '删除失败')
  }
}

watch(id, load, { immediate: true })
</script>

<template>
  <div class="fiction-page se">
    <div v-if="loading" class="se__loading">加载中…</div>

    <template v-else-if="story">
      <div class="se__head">
        <RouterLink to="/write" class="se__back">‹ 创作中心</RouterLink>
        <div class="se__head-right">
          <SosBadge v-if="story.status === 'published'" variant="success">已发布</SosBadge>
          <SosBadge v-else-if="story.status === 'hidden'" variant="danger">已下架</SosBadge>
          <SosBadge v-else variant="outline">草稿</SosBadge>
          <RouterLink v-if="story.status === 'published'" :to="`/story/${story.id}`" class="sos-button sos-button--ghost sos-button--sm">查看</RouterLink>
        </div>
      </div>

      <div class="se__grid">
        <!-- 元信息 -->
        <section class="se__meta">
          <h2 class="se__section-title">作品信息</h2>
          <div class="se__cover">
            <CoverImage :path="form.coverPath" :title="form.title" :category="form.category" />
            <div class="se__cover-actions">
              <label class="sos-button sos-button--secondary sos-button--sm">
                {{ form.coverPath ? '更换封面' : '上传封面' }}
                <input type="file" accept="image/*" hidden @change="onCover" />
              </label>
              <button
                v-if="form.coverPath"
                type="button"
                class="se__cover-remove"
                @click="removeCover"
              >
                移除封面
              </button>
            </div>
            <p class="se__cover-hint">不上传将使用按分类自动生成的封面</p>
          </div>
          <div class="se__fields">
            <label class="se__field">
              <span>标题</span>
              <input v-model="form.title" maxlength="120" placeholder="作品标题" />
            </label>
            <label class="se__field">
              <span>简介</span>
              <textarea v-model="form.summary" rows="4" maxlength="2000" placeholder="向读者介绍你的故事……"></textarea>
            </label>
            <div class="se__row">
              <label class="se__field">
                <span>分类</span>
                <SosSelect v-model="form.category">
                  <option v-for="c in CATEGORIES" :key="c.slug" :value="c.slug">{{ c.label }}</option>
                </SosSelect>
              </label>
              <label class="se__field">
                <span>分级</span>
                <SosSelect v-model="form.contentRating">
                  <option v-for="r in RATINGS" :key="r.slug" :value="r.slug">{{ r.label }}</option>
                </SosSelect>
              </label>
            </div>
            <label class="se__field">
              <span>标签</span>
              <div class="se__tags">
                <span v-for="t in form.tags" :key="t" class="se__tag">{{ t }}<button @click="removeTag(t)">✕</button></span>
                <input v-model="tagInput" class="se__taginput" placeholder="回车添加" @keydown.enter.prevent="addTag" @keydown="(e) => (e.key === ',' || e.key === '，') && (e.preventDefault(), addTag())" />
              </div>
            </label>
            <label class="se__switch">
              <SosSwitch v-model="form.isCompleted" />
              <span>已完结</span>
            </label>
            <div class="se__meta-actions">
              <SosButton variant="primary" :loading="saving" @click="saveMeta">保存作品信息</SosButton>
            </div>
          </div>
        </section>

        <!-- 章节 -->
        <section class="se__chapters">
          <div class="se__section-head">
            <h2 class="se__section-title">章节（{{ chapters.length }}）</h2>
            <SosButton variant="primary" size="sm" @click="newChapter">＋ 新建章节</SosButton>
          </div>

          <p v-if="!chapters.length" class="se__empty">还没有章节，点「新建章节」开始写第一章吧。</p>
          <ul v-else class="se__chlist">
            <li v-for="(c, i) in chapters" :key="c.id" class="se__chitem">
              <div class="se__ch-move">
                <button :disabled="i === 0" @click="move(i, -1)">▲</button>
                <button :disabled="i === chapters.length - 1" @click="move(i, 1)">▼</button>
              </div>
              <div class="se__ch-main">
                <RouterLink :to="`/write/${story.id}/chapter/${c.id}`" class="se__ch-title">
                  {{ i + 1 }}. {{ c.title }}
                </RouterLink>
                <div class="se__ch-meta">
                  <SosBadge :variant="c.status === 'published' ? 'success' : 'outline'">
                    {{ c.status === 'published' ? '已发布' : '草稿' }}
                  </SosBadge>
                  <span>{{ wordLabel(c.wordCount) }}</span>
                  <span>{{ fmtDate(c.updatedAt) }}</span>
                </div>
              </div>
              <div class="se__ch-ops">
                <button @click="toggleChapter(c)">{{ c.status === 'published' ? '转草稿' : '发布' }}</button>
                <button class="se__del" @click="removeChapter(c)">删除</button>
              </div>
            </li>
          </ul>

          <div class="se__publish">
            <SosButton v-if="story.status !== 'published'" variant="primary" @click="onPublish">发布作品</SosButton>
            <SosButton v-else variant="secondary" @click="onUnpublish">撤回作品</SosButton>
            <SosButton variant="danger" @click="onDelete">删除作品</SosButton>
          </div>
        </section>
      </div>
    </template>
  </div>
</template>

<style scoped>
.se__loading {
  text-align: center;
  padding: var(--sos-space-10);
  color: var(--sos-text-tertiary);
}
.se__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--sos-space-6);
}
.se__back {
  color: var(--sos-text-secondary);
  text-decoration: none;
}
.se__back:hover {
  color: var(--sos-accent);
}
.se__head-right {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}
.se__grid {
  display: grid;
  grid-template-columns: 340px 1fr;
  gap: var(--sos-space-8);
  align-items: start;
}
.se__section-title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-lg);
  margin: 0 0 var(--sos-space-4);
}
.se__cover {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sos-space-3);
}
.se__cover > .fiction-cover {
  width: 160px;
}
.se__cover-actions {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}
.se__cover-remove {
  border: none;
  background: none;
  cursor: pointer;
  font-size: var(--sos-text-sm);
  color: var(--sos-text-tertiary);
}
.se__cover-remove:hover {
  color: var(--sos-danger);
}
.se__cover-hint {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
  margin: 0;
  text-align: center;
}
.se__fields {
  margin-top: var(--sos-space-5);
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-4);
}
.se__field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.se__field > span {
  font-size: var(--sos-text-sm);
  color: var(--sos-text-secondary);
}
.se__field input,
.se__field textarea {
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-md);
  padding: var(--sos-space-3);
  font: inherit;
  color: var(--sos-text-primary);
  background: var(--sos-bg-surface);
  resize: vertical;
}
.se__field input:focus,
.se__field textarea:focus,
.se__taginput:focus {
  outline: none;
  border-color: var(--sos-accent);
}
.se__row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sos-space-3);
}
.se__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-md);
  padding: 8px;
}
.se__tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: var(--sos-accent-soft);
  color: var(--sos-accent);
  border-radius: var(--sos-radius-full);
  padding: 2px 4px 2px 10px;
  font-size: var(--sos-text-xs);
}
.se__tag button {
  border: none;
  background: none;
  cursor: pointer;
  color: inherit;
  opacity: 0.7;
}
.se__taginput {
  border: none;
  background: transparent;
  flex: 1;
  min-width: 90px;
  font: inherit;
  color: var(--sos-text-primary);
}
.se__switch {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
  font-size: var(--sos-text-sm);
  color: var(--sos-text-secondary);
}
.se__section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--sos-space-4);
}
.se__empty {
  color: var(--sos-text-tertiary);
  padding: var(--sos-space-6);
  text-align: center;
  background: var(--sos-bg-subtle);
  border-radius: var(--sos-radius-md);
}
.se__chlist {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-2);
}
.se__chitem {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
  padding: var(--sos-space-3);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-md);
  background: var(--sos-bg-surface);
}
.se__ch-move {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.se__ch-move button {
  border: none;
  background: var(--sos-bg-subtle);
  cursor: pointer;
  border-radius: var(--sos-radius-xs);
  width: 22px;
  height: 18px;
  font-size: 9px;
  color: var(--sos-text-secondary);
}
.se__ch-move button:disabled {
  opacity: 0.3;
}
.se__ch-main {
  flex: 1;
  min-width: 0;
}
.se__ch-title {
  font-weight: 600;
  color: var(--sos-text-primary);
  text-decoration: none;
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.se__ch-title:hover {
  color: var(--sos-accent);
}
.se__ch-meta {
  display: flex;
  align-items: center;
  gap: var(--sos-space-2);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
  margin-top: 4px;
}
.se__ch-ops {
  display: flex;
  gap: var(--sos-space-2);
}
.se__ch-ops button {
  border: none;
  background: none;
  cursor: pointer;
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-xs);
}
.se__ch-ops button:hover {
  color: var(--sos-accent);
}
.se__ch-ops .se__del:hover {
  color: var(--sos-danger);
}
.se__publish {
  display: flex;
  gap: var(--sos-space-3);
  margin-top: var(--sos-space-7);
  padding-top: var(--sos-space-5);
  border-top: 1px solid var(--sos-border-subtle);
}

@media (max-width: 820px) {
  .se__grid {
    grid-template-columns: 1fr;
  }
}
</style>
