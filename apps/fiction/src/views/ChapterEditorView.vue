<script setup>
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { useRoute, useRouter, RouterLink, onBeforeRouteLeave } from 'vue-router'
import { SosButton, useToast } from '@haruhi/ui'
import RichEditor from '@/components/RichEditor.vue'
import { myChapter, updateChapter } from '@/api'

const route = useRoute()
const router = useRouter()
const toast = useToast()

const storyId = computed(() => Number(route.params.id))
const chapterId = computed(() => Number(route.params.cid))

const loading = ref(true)
const title = ref('')
const content = ref('')
const note = ref('')
const status = ref('draft')
const saveState = ref('saved') // saved | dirty | saving | error
const dirty = ref(false)

const wordCount = computed(() => {
  const text = content.value
    .replace(/<[^>]*>/g, '')
    .replace(/&nbsp;/g, ' ')
    .replace(/&[a-z]+;/gi, '')
  return [...text].filter((c) => !/\s/.test(c)).length
})

let loaded = false
async function load() {
  loading.value = true
  loaded = false
  // 切换章节时取消上一章残留的防抖保存，否则它可能以「新章 id + 旧章内容」触发，覆盖新章
  clearTimeout(timer)
  try {
    const r = await myChapter(storyId.value, chapterId.value)
    title.value = r.chapter.title
    content.value = r.chapter.contentHtml
    note.value = r.chapter.authorNote
    status.value = r.chapter.status
    saveState.value = 'saved'
    dirty.value = false
  } catch (e) {
    toast.danger(e.status === 403 ? '无权编辑该章节' : '章节加载失败')
    router.replace(`/write/${storyId.value}`)
  } finally {
    loading.value = false
    // 下一拍再允许 watch 触发（避免载入即标记 dirty）
    setTimeout(() => (loaded = true), 0)
  }
}

async function save() {
  if (saveState.value === 'saving') return
  saveState.value = 'saving'
  try {
    await updateChapter(storyId.value, chapterId.value, {
      title: title.value,
      contentHtml: content.value,
      authorNote: note.value,
    })
    saveState.value = 'saved'
    dirty.value = false
  } catch (e) {
    saveState.value = 'error'
    toast.danger(e.message || '保存失败')
  }
}

let timer = null
function scheduleSave() {
  if (!loaded) return
  dirty.value = true
  saveState.value = 'dirty'
  clearTimeout(timer)
  timer = setTimeout(save, 1500)
}
watch([title, content, note], scheduleSave)

async function togglePublish() {
  await save()
  const nextStatus = status.value === 'published' ? 'draft' : 'published'
  try {
    await updateChapter(storyId.value, chapterId.value, { status: nextStatus })
    status.value = nextStatus
    toast.success(nextStatus === 'published' ? '章节已发布' : '已转为草稿')
  } catch (e) {
    toast.danger(e.message || '操作失败')
  }
}

const saveLabel = computed(
  () => ({ saved: '已保存', dirty: '未保存', saving: '保存中…', error: '保存失败' })[saveState.value],
)

onBeforeRouteLeave(async () => {
  if (dirty.value) await save()
  return true
})
onBeforeUnmount(() => clearTimeout(timer))

watch([storyId, chapterId], load, { immediate: true })
</script>

<template>
  <div class="ce">
    <header class="ce__bar">
      <RouterLink :to="`/write/${storyId}`" class="ce__back">‹ 返回作品</RouterLink>
      <span class="ce__save" :class="`ce__save--${saveState}`">{{ saveLabel }}</span>
      <span class="ce__wc">{{ wordCount }} 字</span>
      <div class="ce__bar-right">
        <SosButton variant="secondary" size="sm" @click="save">保存草稿</SosButton>
        <SosButton :variant="status === 'published' ? 'ghost' : 'primary'" size="sm" @click="togglePublish">
          {{ status === 'published' ? '转为草稿' : '发布本章' }}
        </SosButton>
      </div>
    </header>

    <div v-if="loading" class="ce__loading">加载中…</div>
    <div v-else class="ce__paper">
      <input v-model="title" class="ce__title" placeholder="章节标题" maxlength="120" />
      <RichEditor v-model="content" placeholder="夜色渐深，故事从这里开始……" />
      <div class="ce__note">
        <label>作者的话（可选，展示在章末）</label>
        <textarea v-model="note" rows="2" maxlength="1000" placeholder="和读者说点什么～"></textarea>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ce__bar {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: var(--sos-space-4);
  padding: var(--sos-space-3) var(--sos-space-5);
  background: var(--sos-bg-surface);
  border-bottom: 1px solid var(--sos-border-subtle);
}
.ce__back {
  color: var(--sos-text-secondary);
  text-decoration: none;
  font-size: var(--sos-text-sm);
}
.ce__back:hover {
  color: var(--sos-accent);
}
.ce__save {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.ce__save--saved {
  color: var(--sos-success);
}
.ce__save--error {
  color: var(--sos-danger);
}
.ce__wc {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
  margin-right: auto;
}
.ce__bar-right {
  display: flex;
  gap: var(--sos-space-2);
}
.ce__loading {
  text-align: center;
  padding: var(--sos-space-10);
  color: var(--sos-text-tertiary);
}
.ce__paper {
  width: min(760px, 100% - 2 * var(--sos-space-5));
  margin: var(--sos-space-7) auto;
}
.ce__title {
  width: 100%;
  border: none;
  background: transparent;
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-2xl);
  font-weight: 700;
  color: var(--sos-text-primary);
  padding: var(--sos-space-2) 0;
  margin-bottom: var(--sos-space-4);
  border-bottom: 2px solid var(--sos-border-subtle);
}
.ce__title:focus {
  outline: none;
  border-color: var(--sos-accent);
}
.ce__note {
  margin-top: var(--sos-space-5);
}
.ce__note label {
  display: block;
  font-size: var(--sos-text-sm);
  color: var(--sos-text-secondary);
  margin-bottom: var(--sos-space-2);
}
.ce__note textarea {
  width: 100%;
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-md);
  padding: var(--sos-space-3);
  font: inherit;
  color: var(--sos-text-primary);
  background: var(--sos-bg-surface);
  resize: vertical;
}
.ce__note textarea:focus {
  outline: none;
  border-color: var(--sos-accent);
}
</style>
