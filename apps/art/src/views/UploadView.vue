<template>
  <div class="upload-page">
    <div class="upload-shell">
      <!-- 页头 -->
      <header class="upload-hero">
        <div class="upload-hero__copy">
          <span class="upload-hero__eyebrow">绘画部 · 作品投稿</span>
          <h1 class="upload-hero__title">投稿上传</h1>
          <p class="upload-hero__lede">分享你的画作。作品将以账号昵称署名，登录后会自动保留草稿。</p>
        </div>
        <img src="../assets/kon.webp" alt="" class="upload-hero__deco" />
      </header>

      <form class="upload-layout" @submit.prevent="submit">
        <!-- 主列：信息表单（纵向流动，内容多也不溢出） -->
        <div class="upload-main">
          <!-- 基础信息 -->
          <section class="upload-block">
            <header class="upload-block__head">
              <h2 class="upload-block__title">基础信息</h2>
              <span class="upload-block__hint">作品的标题与简介</span>
            </header>
            <div class="upload-block__body">
              <div class="sos-field">
                <label class="sos-field__label">作品名称 <span class="sos-field__required">*</span></label>
                <input class="sos-input" v-model="title" placeholder="请输入作品名称" required />
              </div>
              <div class="sos-field">
                <label class="sos-field__label">作品描述 <span class="sos-field__optional">（选填）</span></label>
                <textarea class="sos-textarea" v-model="description" placeholder="描述作品内容、创作思路或来源说明…"></textarea>
              </div>
            </div>
          </section>

          <!-- 分类与署名 -->
          <section class="upload-block">
            <header class="upload-block__head">
              <h2 class="upload-block__title">分类与署名</h2>
              <span class="upload-block__hint">作品来源与版权归属</span>
            </header>
            <div class="upload-block__body">
              <div class="upload-pair">
                <div class="sos-field">
                  <label class="sos-field__label">图片来源 <span class="sos-field__required">*</span></label>
                  <div class="sos-segmented" role="group" aria-label="图片来源">
                    <button type="button" class="sos-segmented__item" :aria-pressed="sourceType==='personal'" @click="sourceType='personal'" data-sfx="click">🎨 个人作品</button>
                    <button type="button" class="sos-segmented__item" :aria-pressed="sourceType==='network'" @click="sourceType='network'" data-sfx="click">🌐 网络转载</button>
                  </div>
                </div>
                <div class="sos-field">
                  <label class="sos-field__label">内容划分 <span class="sos-field__required">*</span></label>
                  <div class="sos-segmented" role="group" aria-label="内容划分">
                    <button type="button" class="sos-segmented__item" :aria-pressed="contentType==='haruhi'" @click="contentType='haruhi'" data-sfx="click">凉宫内容</button>
                    <button type="button" class="sos-segmented__item" :aria-pressed="contentType==='other'" @click="contentType='other'" data-sfx="click">非凉宫</button>
                  </div>
                </div>
              </div>

              <div class="sos-field">
                <label class="sos-field__label">作者署名</label>
                <input v-if="isLoggedIn" class="sos-input" :value="authorName || '请先在「个人中心 → 资料」填写昵称'" readonly />
                <div v-else class="upload-loginhint">
                  <span>登录后以账号昵称署名，返回时会保留已填写内容。</span>
                  <button type="button" class="sos-button sos-button--primary sos-button--sm" @click="goLogin" data-sfx="click">登录 / 注册</button>
                </div>
                <p class="sos-field__help">作者身份由当前登录账号确定，前端不再手动填写或校验 UID。</p>
              </div>

              <transition name="upl-collapse">
                <div class="sos-field" v-if="sourceType==='network'">
                  <label class="sos-field__label">网络来源链接 <span class="sos-field__optional">（可选）</span></label>
                  <input class="sos-input" v-model="originUrl" placeholder="https://..." />
                  <p class="sos-field__help upload-warn">⚠️ 上传他人作品必须取得原作者授权并标注来源。</p>
                </div>
              </transition>
            </div>
          </section>

          <!-- 标签与授权 -->
          <section class="upload-block">
            <header class="upload-block__head">
              <h2 class="upload-block__title">标签与授权</h2>
              <span class="upload-block__hint">便于检索与二次创作授权</span>
            </header>
            <div class="upload-block__body">
              <div class="sos-field">
                <label class="sos-field__label">标签 <span class="sos-field__optional">（可选）</span></label>
                <div class="upload-taginput">
                  <div class="upload-taginput__shell">
                    <input
                      class="sos-input"
                      v-model="tagDraft"
                      placeholder="输入标签后回车或点添加"
                      @focus="showTagSuggestions = true"
                      @blur="hideTagSuggestions"
                      @keydown.enter.prevent="addTag"
                    />
                    <div v-if="showTagSuggestions && suggestedTags.length" class="upload-suggest">
                      <div class="upload-suggest__head">
                        <span>{{ tagDraft.trim() ? '匹配标签' : '推荐标签' }}</span>
                        <small>来自画廊</small>
                      </div>
                      <button
                        v-for="item in suggestedTags"
                        :key="item.name"
                        type="button"
                        class="upload-suggest__item"
                        @mousedown.prevent="pickSuggestedTag(item.name)"
                        data-sfx="click"
                      >
                        <span>#{{ item.name }}</span>
                        <em>{{ item.count }} 件</em>
                      </button>
                    </div>
                  </div>
                  <button class="sos-button sos-button--secondary" type="button" @click="addTag" :disabled="!tagDraft.trim()" data-sfx="click">添加</button>
                </div>

                <div class="upload-tags" v-if="tags.length">
                  <transition-group name="upl-tag">
                    <span class="upload-tag" v-for="t in tags" :key="t">
                      <span>#{{ t }}</span>
                      <button class="upload-tag__x" type="button" title="删除" @click="removeTag(t)" data-sfx="click">×</button>
                    </span>
                  </transition-group>
                </div>
                <div v-if="tags.length">
                  <button class="sos-button sos-button--link sos-button--sm" type="button" @click="clearTags" data-sfx="click">清空所有标签</button>
                </div>
              </div>

              <transition name="upl-collapse">
                <div class="sos-field" v-if="sourceType==='personal'">
                  <label class="sos-field__label">授权许可设置</label>
                  <div class="upload-license">
                    <div class="upload-license__col">
                      <div class="upload-license__head">对大众 / 网络</div>
                      <p class="sos-field__help">将公开显示在作品详情页。</p>
                      <label class="upload-chk" v-for="opt in NET_LICENSE_OPTIONS" :key="opt">
                        <input type="checkbox" :value="opt" v-model="netLicenses" />
                        <span>{{ opt }}</span>
                      </label>
                    </div>
                    <div class="upload-license__col">
                      <div class="upload-license__head">对应援团 <span class="upload-tag-mini">后台可见</span></div>
                      <p class="sos-field__help">仅后台可见，用于社团企划参考。</p>
                      <label class="upload-chk" v-for="opt in GROUP_LICENSE_OPTIONS" :key="opt">
                        <input type="checkbox" :value="opt" v-model="groupLicenses" />
                        <span>{{ opt }}</span>
                      </label>
                    </div>
                  </div>
                </div>
              </transition>
            </div>
          </section>
        </div>

        <!-- 副列：图片上传 + 提交（桌面 sticky 跟随） -->
        <aside class="upload-aside">
          <section class="upload-block upload-block--media">
            <header class="upload-block__head">
              <h2 class="upload-block__title">作品图片 <span class="sos-field__required">*</span></h2>
              <span class="upload-block__hint">第一张作为封面，可拖拽排序</span>
            </header>
            <div class="upload-block__body">
              <div class="upload-files" v-if="filesList.length">
                <transition-group name="upl-tag" tag="div" class="upload-files__grid">
                  <div
                    v-for="(item, index) in filesList"
                    :key="item.id"
                    class="upload-file"
                    :class="{ 'is-cover': index === 0 }"
                    draggable="true"
                    @dragstart="onDragStart($event, index)"
                    @dragover.prevent
                    @dragenter.prevent
                    @drop="onDrop($event, index)"
                  >
                    <img :src="item.preview" alt="" />
                    <span class="upload-file__cover" v-if="index === 0">封面</span>
                    <button type="button" class="upload-file__x" @click="removeFile(index)" title="移除">✕</button>
                  </div>
                </transition-group>
              </div>

              <label class="upload-drop" :class="{ 'is-filled': filesList.length }" for="fileUpload">
                <input class="upload-drop__input" type="file" accept="image/*" multiple @change="onFilesAdded" id="fileUpload" />
                <span class="upload-drop__icon" aria-hidden="true">{{ filesList.length ? '＋' : '📁' }}</span>
                <strong>{{ filesList.length ? '继续添加图片' : '点击或拖拽上传图片' }}</strong>
                <small v-if="filesList.length">已选 {{ filesList.length }} 张</small>
                <small v-else>JPG / PNG / WebP · 自动压缩生成预览、保留原图</small>
              </label>
            </div>
          </section>

          <div class="upload-cta">
            <transition name="upl-collapse">
              <p v-if="msg" class="upload-msg" :class="{ 'is-error': isError, 'is-ok': !isError }">{{ msg }}</p>
            </transition>
            <button
              class="sos-button sos-button--primary sos-button--lg sos-button--block"
              :disabled="submitting || (isLoggedIn && filesList.length === 0)"
              data-sfx="click"
            >
              <span v-if="submitting" class="upload-spinner" aria-hidden="true"></span>
              {{ submitButtonLabel }}
            </button>
          </div>
        </aside>
      </form>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '../services/api'
import { compressToWebP } from '../utils/imageCompressor.js'
import { useSession } from '@haruhi/auth-ui'
import { saveUploadDraft, takeUploadDraft, clearUploadDraft } from '../composables/uploadDraft.js'
import { seedArtworks } from '../mock/seedData.js'

const router = useRouter()
const route = useRoute()
const session = useSession('/api')
const authorName = computed(() => session.state.user?.nickname || '')
const isLoggedIn = computed(() => !!session.state.user)

function goLogin() {
  router.push({ path: '/login', query: { redirect: route.fullPath } })
}

// --- 常量定义 ---
const NET_LICENSE_OPTIONS = [
  '可在b站、小红书等社交媒体转载',
  '允许用于视频等个人创作',
  '允许用于制作无料发放'
]

const GROUP_LICENSE_OPTIONS = [
  '允许应援团社交媒体官方账号转载',
  '允许用于应援团官方视频/游戏等创作企划',
  '允许制作无料周边发放',
  '允许制作贩售周边'
]

// --- 状态定义 ---
const title = ref('')
const description = ref('')

const tagDraft = ref('')
const tags = ref([])
const showTagSuggestions = ref(false)

const sourceType = ref('personal')
const contentType = ref('haruhi')
const originUrl = ref('')

// 改为文件列表：{ id, file, preview }
const filesList = ref([])

// 授权状态拆分
const netLicenses = ref([])
const groupLicenses = ref([])

const msg = ref('')
const isError = ref(false)
const submitting = ref(false)
const statusMsg = ref('提交中…')

const submitButtonLabel = computed(() => {
  if (submitting.value) return statusMsg.value
  if (!isLoggedIn.value) return '登录后提交作品'
  return '🚀 确认并提交'
})

const galleryTagStats = computed(() => {
  const counts = new Map()
  for (const artwork of seedArtworks) {
    for (const tag of artwork.tags || []) {
      const name = normalizeOneTag(tag)
      if (!name) continue
      counts.set(name, (counts.get(name) || 0) + 1)
    }
  }

  return Array.from(counts.entries())
    .map(([name, count]) => ({ name, count }))
    .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name, 'zh-Hans-CN'))
})

const suggestedTags = computed(() => {
  const query = normalizeOneTag(tagDraft.value).toLowerCase()
  const selected = new Set(tags.value.map(tag => tag.toLowerCase()))
  return galleryTagStats.value
    .filter(item => !selected.has(item.name.toLowerCase()))
    .filter(item => !query || item.name.toLowerCase().includes(query))
    .slice(0, 8)
})

onMounted(() => {
  if (!session.state.ready) session.refresh()
  const draft = takeUploadDraft()
  if (!draft) return

  title.value = draft.title || ''
  description.value = draft.description || ''
  tags.value = Array.isArray(draft.tags) ? [...draft.tags] : []
  sourceType.value = draft.sourceType || 'personal'
  contentType.value = draft.contentType || 'haruhi'
  originUrl.value = draft.originUrl || ''
  netLicenses.value = Array.isArray(draft.netLicenses) ? [...draft.netLicenses] : []
  groupLicenses.value = Array.isArray(draft.groupLicenses) ? [...draft.groupLicenses] : []
  filesList.value = Array.isArray(draft.filesList) ? draft.filesList : []
})

onBeforeUnmount(() => {
  const hasContent =
    title.value.trim() ||
    description.value.trim() ||
    tags.value.length ||
    filesList.value.length ||
    originUrl.value.trim() ||
    netLicenses.value.length ||
    groupLicenses.value.length

  if (!hasContent) {
    clearUploadDraft()
    return
  }

  saveUploadDraft({
    title: title.value,
    description: description.value,
    tags: [...tags.value],
    sourceType: sourceType.value,
    contentType: contentType.value,
    originUrl: originUrl.value,
    netLicenses: [...netLicenses.value],
    groupLicenses: [...groupLicenses.value],
    filesList: filesList.value
  })
})

// --- 方法 ---

// 单张原图大小上限（MB），与后端 MAX_IMAGE_BYTES 一致；超限当场提醒并跳过，
// 避免到提交时才收到后端 400「文件过大」。
const MAX_IMAGE_MB = 60

// 多图添加
async function onFilesAdded(e) {
  const addedFiles = Array.from(e.target.files || [])
  if (addedFiles.length === 0) return

  // 清空 value 允许重复添加同名文件
  e.target.value = ''

  const tooBig = []
  for (const f of addedFiles) {
    if (f.size > MAX_IMAGE_MB * 1024 * 1024) {
      tooBig.push(`${f.name}（${(f.size / 1024 / 1024).toFixed(1)}MB）`)
      continue
    }
    const previewUrl = URL.createObjectURL(f)
    filesList.value.push({
      id: Date.now() + Math.random(),
      file: f,
      preview: previewUrl
    })
  }

  if (tooBig.length) {
    msg.value = `以下图片超过 ${MAX_IMAGE_MB}MB 上限，已跳过：${tooBig.join('、')}。请压缩后再上传。`
    isError.value = true
  } else {
    // 本次选择都合规：清掉可能残留的旧错误提示
    if (isError.value) { msg.value = ''; isError.value = false }
  }
}

// 移除图片
function removeFile(index) {
  const item = filesList.value[index]
  if (item && item.preview) URL.revokeObjectURL(item.preview)
  filesList.value.splice(index, 1)
}

// 拖拽排序
let dragIndex = -1
function onDragStart(e, index) {
  dragIndex = index
  e.dataTransfer.effectAllowed = 'move'
  e.dataTransfer.dropEffect = 'move'
}
function onDrop(e, dropIndex) {
  if (dragIndex === -1 || dragIndex === dropIndex) return
  const item = filesList.value[dragIndex]
  filesList.value.splice(dragIndex, 1)
  filesList.value.splice(dropIndex, 0, item)
  dragIndex = -1
}

function normalizeOneTag(s){
  let t = String(s || '').trim()
  if(!t) return ''
  if(t.startsWith('#')) t = t.slice(1).trim()
  t = t.replace(/\s+/g, ' ')
  return t
}

function addTag(){
  const t = normalizeOneTag(tagDraft.value)
  if(!t) return
  const key = t.toLowerCase()
  const exists = tags.value.some(x => x.toLowerCase() === key)
  if(!exists) tags.value.push(t)
  tagDraft.value = ''
  showTagSuggestions.value = true
}
function removeTag(t){ tags.value = tags.value.filter(x => x !== t) }
function clearTags(){ tags.value = []; tagDraft.value = '' }
function pickSuggestedTag(tag){
  tagDraft.value = tag
  addTag()
}
function hideTagSuggestions(){
  window.setTimeout(() => {
    showTagSuggestions.value = false
  }, 120)
}

watch(sourceType, (v) => {
  if(v !== 'personal'){
    netLicenses.value = []
    groupLicenses.value = []
  }
})

async function submit(){
  msg.value = ''
  isError.value = false

  if(!isLoggedIn.value){
    goLogin()
    return
  }

  if(filesList.value.length === 0){ msg.value = '请至少选择一张图片'; isError.value = true; return }
  if(!title.value.trim()){
    msg.value = '作品名称为必填'; isError.value = true;
    return
  }

  submitting.value = true
  statusMsg.value = '处理图片中…'

  try{
    const fd = new FormData()

    // --- 批量处理图片 ---
    for (const item of filesList.value) {
      let compressedBlob = null
      try {
        // 压缩到 WebP, 90% 质量, 最大宽 1920
        compressedBlob = await compressToWebP(item.file, 0.9, 1920)
      } catch (err) {
        console.warn('图片压缩失败，将使用原图作为展示图:', err)
        compressedBlob = item.file
      }

      // 使用 images (复数) 字段上传
      fd.append('images', compressedBlob, 'display.webp')
      fd.append('originals', item.file)
    }

    fd.append('title', title.value.trim())
    fd.append('description', description.value.trim())
    fd.append('tags', tags.value.join(' '))
    fd.append('source_type', sourceType.value)
    fd.append('content_type', contentType.value)
    fd.append('origin_url', originUrl.value.trim())

    if(sourceType.value === 'personal'){
      const combinedLicenses = [
        ...netLicenses.value.map(x => `NET:${x}`),
        ...groupLicenses.value.map(x => `GROUP:${x}`)
      ]
      fd.append('licenses', JSON.stringify(combinedLicenses))
    }

    statusMsg.value = '上传中…'
    const r = await api.uploadArtwork(fd)

    // 根据返回的状态显示不同的提示
    if (r.status === 'approved') {
      msg.value = r.pointsAdded
        ? '发布成功！✅（AI审核通过，积分已发放）'
        : '发布成功！✅（AI审核通过）'
    } else if (r.status === 'flagged') {
      msg.value = '提交成功，但内容需人工复核 🤖'
    } else {
      msg.value = '提交成功，进入待审核队列'
    }

    // reset form
    title.value = ''
    description.value = ''
    originUrl.value = ''

    // 清空文件列表
    filesList.value.forEach(i => URL.revokeObjectURL(i.preview))
    filesList.value = []

    // Reset file input value manually
    const fileInput = document.getElementById('fileUpload')
    if(fileInput) fileInput.value = ''

    clearTags()
    netLicenses.value = []
    groupLicenses.value = []
    clearUploadDraft()
  }catch(e){
    msg.value = `提交失败：${e.message}`
    isError.value = true
  }finally{
    submitting.value = false
    statusMsg.value = '提交'
  }
}
</script>

<style scoped>
.upload-page {
  --upl-accent: var(--sos-accent, hsl(172, 70%, 42%));
  --upl-accent-strong: color-mix(in srgb, var(--upl-accent) 78%, #0b3a36);
  --upl-accent-soft: color-mix(in srgb, var(--upl-accent) 13%, transparent);
  --upl-glass: color-mix(in srgb, #ffffff 70%, transparent);
  --upl-glass-line: color-mix(in srgb, #ffffff 85%, transparent);
  --upl-text: var(--sos-text-primary, #16242b);
  --upl-muted: var(--sos-text-secondary, #5b6b72);
  padding-bottom: var(--sos-space-8, 48px);
}

.upload-shell {
  max-width: 1160px;
  margin: 0 auto;
  padding: 0 var(--sos-space-4, 16px);
}

/* ---------- 页头 ---------- */
.upload-hero {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--sos-space-4);
  padding: var(--sos-space-3) var(--sos-space-1) var(--sos-space-5);
}
.upload-hero__eyebrow {
  display: inline-block;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.16em;
  color: var(--upl-accent-strong);
  padding: 4px 11px;
  border-radius: 999px;
  background: var(--upl-accent-soft);
}
.upload-hero__title {
  margin: 12px 0 6px;
  font-size: clamp(28px, 4vw, 40px);
  font-weight: 850;
  letter-spacing: -0.02em;
  color: var(--upl-text);
}
.upload-hero__lede {
  margin: 0;
  color: var(--upl-muted);
  font-size: 15px;
  max-width: 46ch;
}
.upload-hero__deco {
  height: clamp(96px, 12vw, 152px);
  object-fit: contain;
  filter: drop-shadow(0 14px 24px rgba(18, 80, 70, 0.2));
  transform: translateY(6px);
}

/* ---------- 两列布局 ---------- */
.upload-layout {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--sos-space-4);
  align-items: start;
}
@media (min-width: 960px) {
  .upload-layout {
    grid-template-columns: minmax(0, 1.55fr) minmax(322px, 1fr);
    gap: var(--sos-space-5);
  }
}

.upload-main,
.upload-aside {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-4);
  min-width: 0;
}
@media (min-width: 960px) {
  .upload-aside {
    position: sticky;
    /* 顶部 SosAppbar 是 sticky header（高 --sos-header-height），副列须停在它下方，避免被遮挡 */
    top: calc(var(--sos-header-height, 4rem) + var(--sos-space-4));
  }
}

/* ---------- 玻璃区块 ---------- */
.upload-block {
  position: relative;
  background: var(--upl-glass);
  -webkit-backdrop-filter: blur(18px) saturate(1.3);
  backdrop-filter: blur(18px) saturate(1.3);
  border: 1px solid var(--upl-glass-line);
  border-radius: 20px;
  box-shadow:
    0 20px 44px -26px rgba(20, 60, 60, 0.42),
    inset 0 1px 0 rgba(255, 255, 255, 0.72);
  padding: clamp(18px, 2.4vw, 26px);
}
.upload-block__head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  flex-wrap: wrap;
  margin-bottom: var(--sos-space-4);
}
.upload-block__title {
  position: relative;
  margin: 0;
  padding-left: 14px;
  font-size: 17px;
  font-weight: 800;
  color: var(--upl-text);
}
.upload-block__title::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 16px;
  border-radius: 2px;
  background: linear-gradient(var(--upl-accent), color-mix(in srgb, var(--upl-accent) 55%, #ec4faf));
}
.upload-block__hint {
  font-size: 12.5px;
  color: var(--upl-muted);
}
.upload-block__body {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-4);
}

/* 字段双列 */
.upload-pair {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--sos-space-4);
}
@media (min-width: 540px) {
  .upload-pair { grid-template-columns: 1fr 1fr; }
}

/* 分段控件撑满 */
.sos-segmented { display: flex; width: 100%; }
.sos-segmented__item { flex: 1; justify-content: center; gap: 6px; }

.sos-field .sos-input[readonly] {
  background: color-mix(in srgb, var(--upl-accent) 5%, #f4f7f8);
  cursor: default;
}
.upload-warn {
  color: var(--sos-warning, #b4690e);
  font-weight: 600;
}

/* 未登录提示 */
.upload-loginhint {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 12px;
  background: var(--upl-accent-soft);
  border: 1px dashed color-mix(in srgb, var(--upl-accent) 40%, transparent);
}
.upload-loginhint > span {
  flex: 1;
  min-width: 12em;
  font-size: 13px;
  color: var(--upl-text);
}

/* 标签输入 */
.upload-taginput {
  display: flex;
  gap: 10px;
  align-items: stretch;
}
.upload-taginput__shell {
  position: relative;
  flex: 1;
  min-width: 0;
}
.upload-suggest {
  position: absolute;
  z-index: 30;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  background: #fff;
  border: 1px solid var(--sos-border-default, rgba(0, 0, 0, 0.1));
  border-radius: 14px;
  box-shadow: 0 18px 42px -16px rgba(20, 40, 50, 0.34);
  padding: 8px;
  max-height: 290px;
  overflow: auto;
}
.upload-suggest__head {
  display: flex;
  justify-content: space-between;
  padding: 4px 8px 8px;
  font-size: 12px;
  font-weight: 700;
  color: var(--upl-muted);
}
.upload-suggest__item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  padding: 8px 10px;
  border: none;
  background: transparent;
  border-radius: 9px;
  cursor: pointer;
  font-size: 13px;
  color: var(--upl-text);
}
.upload-suggest__item:hover { background: var(--upl-accent-soft); }
.upload-suggest__item em { font-style: normal; color: var(--upl-muted); font-size: 11.5px; }

.upload-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.upload-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 6px 5px 11px;
  border-radius: 999px;
  background: var(--upl-accent-soft);
  color: var(--upl-accent-strong);
  font-size: 13px;
  font-weight: 600;
}
.upload-tag__x {
  border: none;
  background: rgba(0, 0, 0, 0.07);
  color: inherit;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  cursor: pointer;
  line-height: 1;
  font-size: 13px;
}
.upload-tag__x:hover { background: var(--sos-danger, #e5484d); color: #fff; }

/* 授权双栏 */
.upload-license {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--sos-space-3);
}
@media (min-width: 620px) {
  .upload-license { grid-template-columns: 1fr 1fr; }
}
.upload-license__col {
  background: color-mix(in srgb, #fff 48%, transparent);
  border: 1px solid var(--upl-glass-line);
  border-radius: 14px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.upload-license__head {
  font-size: 13px;
  font-weight: 700;
  color: var(--upl-text);
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
}
.upload-tag-mini {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 7px;
  border-radius: 999px;
  background: var(--upl-accent-soft);
  color: var(--upl-accent-strong);
}
.upload-chk {
  display: flex;
  align-items: center;
  gap: 9px;
  font-size: 13px;
  color: var(--upl-text);
  cursor: pointer;
  padding: 5px 0;
}
.upload-chk input {
  accent-color: var(--upl-accent);
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

/* ---------- 图片副栏 ---------- */
.upload-block--media .upload-block__body { gap: var(--sos-space-3); }
.upload-files__grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(84px, 1fr));
  gap: 10px;
}
.upload-file {
  position: relative;
  aspect-ratio: 1;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--upl-glass-line);
  cursor: grab;
  box-shadow: 0 8px 18px -12px rgba(20, 40, 50, 0.55);
}
.upload-file:active { cursor: grabbing; }
.upload-file img { width: 100%; height: 100%; object-fit: cover; display: block; }
.upload-file.is-cover { outline: 2px solid var(--upl-accent); outline-offset: -2px; }
.upload-file__cover {
  position: absolute;
  left: 6px;
  top: 6px;
  font-size: 10px;
  font-weight: 700;
  color: #fff;
  background: var(--upl-accent);
  padding: 2px 7px;
  border-radius: 999px;
}
.upload-file__x {
  position: absolute;
  right: 4px;
  top: 4px;
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
  cursor: pointer;
  font-size: 11px;
  line-height: 1;
}
.upload-file__x:hover { background: var(--sos-danger, #e5484d); }

.upload-drop {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  text-align: center;
  padding: 30px 16px;
  border-radius: 16px;
  cursor: pointer;
  border: 1.5px dashed color-mix(in srgb, var(--upl-accent) 42%, transparent);
  background: color-mix(in srgb, var(--upl-accent) 6%, #fff);
  color: var(--upl-text);
  transition: border-color 0.2s, background 0.2s, transform 0.2s;
}
.upload-drop:hover {
  border-color: var(--upl-accent);
  background: color-mix(in srgb, var(--upl-accent) 11%, #fff);
  transform: translateY(-1px);
}
.upload-drop.is-filled { padding: 18px 16px; }
.upload-drop__input { display: none; }
.upload-drop__icon { font-size: 30px; line-height: 1; }
.upload-drop strong { font-size: 14px; font-weight: 700; }
.upload-drop small { font-size: 12px; color: var(--upl-muted); }

/* ---------- 提交区 ---------- */
.upload-cta {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.upload-msg {
  margin: 0;
  padding: 10px 14px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 600;
}
.upload-msg.is-error { background: var(--sos-danger-soft, #fdecec); color: var(--sos-danger, #c0392b); }
.upload-msg.is-ok { background: var(--sos-success-soft, #e7f7ee); color: var(--sos-success, #1f9254); }
.upload-spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  margin-right: 8px;
  border: 2px solid rgba(255, 255, 255, 0.4);
  border-top-color: #fff;
  border-radius: 50%;
  vertical-align: -3px;
  animation: upl-spin 0.7s linear infinite;
}
@keyframes upl-spin { to { transform: rotate(360deg); } }

/* ---------- 过渡 ---------- */
.upl-collapse-enter-active,
.upl-collapse-leave-active { transition: all 0.28s ease; max-height: 360px; overflow: hidden; }
.upl-collapse-enter-from,
.upl-collapse-leave-to { max-height: 0; opacity: 0; transform: translateY(-6px); }
.upl-tag-enter-active,
.upl-tag-leave-active { transition: all 0.25s ease; }
.upl-tag-enter-from,
.upl-tag-leave-to { opacity: 0; transform: scale(0.85); }

@media (max-width: 600px) {
  .upload-hero__deco { display: none; }
}
</style>
