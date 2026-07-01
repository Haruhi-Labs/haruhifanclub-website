<script setup>
import { ref, computed, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { SosButton, SosAvatar, useToast } from '@haruhi/ui'
import { listComments, postComment, deleteMyComment, session } from '@/api'
import { fmtRelative } from '@/lib/format'

const props = defineProps({
  storyId: { type: [Number, String], required: true },
  chapterId: { type: [Number, String], default: null },
})

const route = useRoute()
const toast = useToast()
const comments = ref([])
const total = ref(0)
const loading = ref(true)
const draft = ref('')
const replyTo = ref(null)
const replyDraft = ref('')
const posting = ref(false)

const user = computed(() => session.state.user)

const byId = computed(() => new Map(comments.value.map((c) => [c.id, c])))
function rootOf(c) {
  let cur = c
  const seen = new Set()
  while (cur.parentId && byId.value.has(cur.parentId) && !seen.has(cur.id)) {
    seen.add(cur.id)
    cur = byId.value.get(cur.parentId)
  }
  return cur.id
}
const tops = computed(() => comments.value.filter((c) => !c.parentId))
const repliesByRoot = computed(() => {
  const m = new Map()
  for (const c of comments.value) {
    if (!c.parentId) continue
    const root = rootOf(c)
    if (!m.has(root)) m.set(root, [])
    m.get(root).push(c)
  }
  return m
})

async function load() {
  loading.value = true
  try {
    const params = { pageSize: 100 }
    if (props.chapterId != null) params.chapterId = props.chapterId
    const r = await listComments(props.storyId, params)
    comments.value = r.comments
    total.value = r.pagination.total
  } catch {
    comments.value = []
  } finally {
    loading.value = false
  }
}

async function send(body, parentId = null) {
  const text = (body || '').trim()
  if (!text) return
  posting.value = true
  try {
    const payload = { body: text }
    if (props.chapterId != null) payload.chapterId = Number(props.chapterId)
    if (parentId != null) payload.parentId = parentId
    await postComment(props.storyId, payload)
    draft.value = ''
    replyDraft.value = ''
    replyTo.value = null
    await load()
    toast.success('评论已发布')
  } catch (e) {
    toast.danger(e.message || '发布失败')
  } finally {
    posting.value = false
  }
}

async function remove(id) {
  try {
    await deleteMyComment(id)
    await load()
  } catch (e) {
    toast.danger(e.message || '删除失败')
  }
}

function targetName(c) {
  return byId.value.get(c.parentId)?.authorName
}
function canDelete(c) {
  return user.value && c.authorUserId === user.value.id
}

watch(
  () => [props.storyId, props.chapterId],
  () => load(),
  { immediate: true },
)
</script>

<template>
  <section class="cmt">
    <h3 class="cmt__title">评论 <span>{{ total }}</span></h3>

    <div v-if="user" class="cmt__editor">
      <SosAvatar :name="user.nickname || user.username" size="md" />
      <div class="cmt__editor-body">
        <textarea
          v-model="draft"
          class="cmt__textarea"
          rows="3"
          maxlength="2000"
          placeholder="说点什么吧……（请友善交流）"
        ></textarea>
        <div class="cmt__editor-actions">
          <SosButton variant="primary" :loading="posting" :disabled="!draft.trim()" @click="send(draft)">
            发表评论
          </SosButton>
        </div>
      </div>
    </div>
    <div v-else class="cmt__login">
      <RouterLink :to="{ name: 'login', query: { redirect: route.fullPath } }">登录</RouterLink>
      后即可参与讨论。
    </div>

    <div v-if="loading" class="cmt__loading">加载评论中…</div>
    <p v-else-if="!tops.length" class="cmt__empty">还没有评论，来抢占第一个沙发。</p>

    <ul v-else class="cmt__list">
      <li v-for="c in tops" :key="c.id" class="cmt__item">
        <SosAvatar :name="c.authorName" size="md" />
        <div class="cmt__main">
          <div class="cmt__head">
            <span class="cmt__author">{{ c.authorName }}</span>
            <span class="cmt__time">{{ fmtRelative(c.createdAt) }}</span>
          </div>
          <p class="cmt__body">{{ c.body }}</p>
          <div class="cmt__ops">
            <button v-if="user" @click="replyTo = replyTo === c.id ? null : c.id">回复</button>
            <button v-if="canDelete(c)" class="cmt__del" @click="remove(c.id)">删除</button>
          </div>

          <!-- 回复列表 -->
          <ul v-if="repliesByRoot.get(c.id)" class="cmt__replies">
            <li v-for="r in repliesByRoot.get(c.id)" :key="r.id" class="cmt__reply">
              <SosAvatar :name="r.authorName" size="sm" />
              <div class="cmt__main">
                <div class="cmt__head">
                  <span class="cmt__author">{{ r.authorName }}</span>
                  <span v-if="targetName(r) && targetName(r) !== c.authorName" class="cmt__at">
                    ▸ {{ targetName(r) }}
                  </span>
                  <span class="cmt__time">{{ fmtRelative(r.createdAt) }}</span>
                </div>
                <p class="cmt__body">{{ r.body }}</p>
                <div class="cmt__ops">
                  <button v-if="user" @click="replyTo = replyTo === r.id ? null : r.id">回复</button>
                  <button v-if="canDelete(r)" class="cmt__del" @click="remove(r.id)">删除</button>
                </div>
                <div v-if="replyTo === r.id" class="cmt__replybox">
                  <textarea v-model="replyDraft" rows="2" class="cmt__textarea" :placeholder="`回复 @${r.authorName}`"></textarea>
                  <SosButton size="sm" variant="primary" :loading="posting" @click="send(replyDraft, r.id)">回复</SosButton>
                </div>
              </div>
            </li>
          </ul>

          <div v-if="replyTo === c.id" class="cmt__replybox">
            <textarea v-model="replyDraft" rows="2" class="cmt__textarea" :placeholder="`回复 @${c.authorName}`"></textarea>
            <SosButton size="sm" variant="primary" :loading="posting" @click="send(replyDraft, c.id)">回复</SosButton>
          </div>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.cmt {
  margin-top: var(--sos-space-8);
}
.cmt__title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-xl);
  margin: 0 0 var(--sos-space-5);
}
.cmt__title span {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-md);
}
.cmt__editor {
  display: flex;
  gap: var(--sos-space-3);
  margin-bottom: var(--sos-space-6);
}
.cmt__editor-body {
  flex: 1;
}
.cmt__textarea {
  width: 100%;
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-md);
  background: var(--sos-bg-surface);
  padding: var(--sos-space-3);
  font: inherit;
  color: var(--sos-text-primary);
  resize: vertical;
}
.cmt__textarea:focus {
  outline: none;
  border-color: var(--sos-accent);
  box-shadow: 0 0 0 3px var(--sos-accent-soft);
}
.cmt__editor-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: var(--sos-space-2);
}
.cmt__login {
  padding: var(--sos-space-4);
  background: var(--sos-bg-subtle);
  border-radius: var(--sos-radius-md);
  color: var(--sos-text-secondary);
  margin-bottom: var(--sos-space-6);
}
.cmt__login a {
  color: var(--sos-accent);
  font-weight: 600;
}
.cmt__loading,
.cmt__empty {
  color: var(--sos-text-tertiary);
  padding: var(--sos-space-4) 0;
}
.cmt__list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-6);
}
.cmt__item,
.cmt__reply {
  display: flex;
  gap: var(--sos-space-3);
}
.cmt__main {
  flex: 1;
  min-width: 0;
}
.cmt__head {
  display: flex;
  align-items: baseline;
  gap: var(--sos-space-2);
}
.cmt__author {
  font-weight: 600;
  color: var(--sos-text-primary);
  font-size: var(--sos-text-sm);
}
.cmt__at {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-xs);
}
.cmt__time {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-xs);
}
.cmt__body {
  margin: 4px 0;
  color: var(--sos-text-primary);
  line-height: var(--sos-leading-body);
  white-space: pre-wrap;
  word-break: break-word;
}
.cmt__ops {
  display: flex;
  gap: var(--sos-space-3);
}
.cmt__ops button {
  border: none;
  background: none;
  cursor: pointer;
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-xs);
  padding: 0;
}
.cmt__ops button:hover {
  color: var(--sos-accent);
}
.cmt__ops .cmt__del:hover {
  color: var(--sos-danger);
}
.cmt__replies {
  list-style: none;
  margin: var(--sos-space-3) 0 0;
  padding: var(--sos-space-3) 0 0 0;
  border-top: 1px dashed var(--sos-border-subtle);
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-4);
}
.cmt__replybox {
  margin-top: var(--sos-space-2);
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-2);
  align-items: flex-end;
}
.cmt__replybox .cmt__textarea {
  width: 100%;
}
</style>
