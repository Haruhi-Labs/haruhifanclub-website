<template>
  <div class="upload-page">
    <section class="container-card">
      <!-- 头部区域 -->
      <header class="page-header">
        <div class="header-text">
          <h1 class="main-title">投稿上传</h1>
        </div>
        <div class="header-decoration">
          <img src="../assets/kon.webp" alt="Decoration" class="deco-img" />
        </div>
      </header>

      <!-- 表单主体 -->
      <form class="main-form" @submit.prevent="submit">
        
        <!-- 区块 1: 基础信息 -->
        <section class="form-section">
          <div class="section-head">
            <h2 class="section-title">基础信息</h2>
            <span class="section-desc">填写作品的基本资料</span>
          </div>

          <div class="form-grid">
            <div class="form-group" style="grid-column: span 2;">
              <label class="form-label">作品名称 <span class="req">*</span></label>
              <input class="form-input" v-model="title" placeholder="请输入作品名称" required />
            </div>
          </div>

          <div class="form-group">
            <label class="form-label">作品描述 <span class="opt">（选填）</span></label>
            <textarea class="form-textarea" v-model="description" placeholder="请描述作品内容、创作思路或来源说明…"></textarea>
          </div>
        </section>

        <!-- 区块 2: 分类与署名 -->
        <section class="form-section">
          <div class="section-head">
            <h2 class="section-title">分类与署名</h2>
            <span class="section-desc">确认作品来源与版权归属</span>
          </div>

          <div class="form-grid">
            <div class="form-group">
              <label class="form-label">图片来源 <span class="req">*</span></label>
              <div class="segment-control">
                <button type="button" :class="['segment-btn', sourceType==='personal' && 'active']" @click="sourceType='personal'" data-sfx="click">
                  <span class="icon">🎨</span> 个人作品
                </button>
                <button type="button" :class="['segment-btn', sourceType==='network' && 'active']" @click="sourceType='network'" data-sfx="click">
                  <span class="icon">🌐</span> 网络转载&其它
                </button>
              </div>
            </div>

            <div class="form-group">
              <label class="form-label">内容划分 <span class="req">*</span></label>
              <div class="segment-control">
                <button type="button" :class="['segment-btn', contentType==='haruhi' && 'active']" @click="contentType='haruhi'" data-sfx="click">
                  凉宫内容
                </button>
                <button type="button" :class="['segment-btn', contentType==='other' && 'active']" @click="contentType='other'" data-sfx="click">
                  非凉宫内容
                </button>
              </div>
            </div>
          </div>

          <!-- 个人作品专属逻辑 -->
          <transition name="fade-slide">
            <div v-if="sourceType==='personal'" class="conditional-block">
              <div class="form-group">
                <label class="form-label">创作者唯一ID <span class="req">*</span></label>
                <div class="input-with-action">
                  <input class="form-input" v-model="uid" placeholder="请输入你的唯一ID" />
                  <button class="action-btn" type="button" @click="checkUid" :disabled="checkingUid || !uid.trim()" data-sfx="click">
                    {{ checkingUid ? '检测中…' : '检测' }}
                  </button>
                </div>
                
                <!-- UID 状态反馈 -->
                <div class="uid-feedback" v-if="uidHint">
                  <div class="status-badge" :class="uidHintClass">
                    {{ uidHint }}
                  </div>
                  <img v-if="uidAvatar" class="creator-avatar" :src="uidAvatar" alt="Avatar" />
                </div>

                <div class="info-card">
                  <p>进行创作者注册才能获取唯一ID，并允许上传个人作品哦！</p>
                  <p class="sub-info">可加入绘画部联系 <strong>律纪</strong> 或 <strong>阿笑</strong> 成为创作者🥰～<br>绘画部群号：627992968，阿笑QQ：2452812504，律纪QQ：3105285630<br>进行创作者注册主要是为了避免冒领身份和过滤AI作品</p>
                </div>
              </div>
            </div>
          </transition>

          <!-- 网络图片专属逻辑 -->
          <transition name="fade-slide">
            <div v-if="sourceType==='network'" class="conditional-block">
              <div class="form-group" style="margin-bottom: 24px;">
                <label class="form-label">上传者显示名 <span class="opt">（可选）</span></label>
                <input class="form-input" v-model="uploaderName" placeholder="例如：昵称 / 匿名" />
                <p class="form-hint">仅用于展示，不填写则显示为默认名称。</p>
              </div>

              <div class="form-group">
                <label class="form-label">网络图片来源链接 <span class="opt">（可选）</span></label>
                <input class="form-input" v-model="originUrl" placeholder="https://..." />
                <p class="form-hint warning">⚠️ 上传他人作品必须取得原作者授权并标注来源！</p>
              </div>
            </div>
          </transition>
        </section>

        <!-- 区块 3: 标签与授权 -->
        <section class="form-section">
          <div class="section-head">
            <h2 class="section-title">标签与授权</h2>
          </div>

          <div class="form-grid">
            <div class="form-group" style="grid-column: span 2;">
              <label class="form-label">标签 <span class="opt">（可选）</span></label>
              <div class="input-with-action">
                <input
                  class="form-input"
                  v-model="tagDraft"
                  placeholder="输入标签后按回车或点击添加"
                  @keydown.enter.prevent="addTag"
                />
                <button class="action-btn secondary" type="button" @click="addTag" :disabled="!tagDraft.trim()" data-sfx="click">添加</button>
              </div>
              
              <div class="tags-container" v-if="tags.length">
                <transition-group name="list">
                  <span class="tag-pill" v-for="t in tags" :key="t">
                    <span class="tag-text">#{{ t }}</span>
                    <button class="tag-remove" type="button" title="删除" @click="removeTag(t)" data-sfx="click">×</button>
                  </span>
                </transition-group>
              </div>
              <div class="tags-actions" v-if="tags.length">
                 <button class="text-btn" type="button" @click="clearTags" data-sfx="click">清空所有标签</button>
              </div>
            </div>

            <transition name="fade-slide">
              <div class="form-group" v-if="sourceType==='personal'" style="grid-column: span 2;">
                <label class="form-label">授权许可设置</label>
                
                <div class="license-split-layout">
                  <div class="license-col">
                    <div class="license-header">对大众/网络的授权</div>
                    <p class="form-hint" style="margin-bottom: 12px;">这些授权信息将公开显示在图片详情页。</p>
                    <div class="checkbox-list">
                      <label class="checkbox-card" v-for="opt in NET_LICENSE_OPTIONS" :key="opt">
                        <input type="checkbox" :value="opt" v-model="netLicenses" class="chk-input" />
                        <span class="chk-custom"></span>
                        <span class="chk-label">{{ opt }}</span>
                      </label>
                    </div>
                  </div>

                  <div class="license-col">
                    <div class="license-header">对应援团的特别授权 <span class="badge-private">后台可见</span></div>
                    <p class="form-hint" style="margin-bottom: 12px;">这些信息仅在后台可见，用于社团内部企划或周边制作参考。</p>
                    <div class="checkbox-list">
                      <label class="checkbox-card" v-for="opt in GROUP_LICENSE_OPTIONS" :key="opt">
                        <input type="checkbox" :value="opt" v-model="groupLicenses" class="chk-input" />
                        <span class="chk-custom"></span>
                        <span class="chk-label">{{ opt }}</span>
                      </label>
                    </div>
                  </div>
                </div>

              </div>
            </transition>
          </div>
        </section>

        <!-- 区块 4: 图片上传 (更新为多图模式) -->
        <section class="form-section upload-section">
          <div class="section-head">
            <h2 class="section-title">图片文件</h2>
            <span class="section-desc">支持 JPG, PNG, WebP 格式。支持拖拽排序，第一张将作为封面。</span>
          </div>

          <!-- 已选图片列表 -->
          <div class="files-manager" v-if="filesList.length > 0">
            <transition-group name="list" tag="div" class="files-grid">
              <div 
                v-for="(item, index) in filesList" 
                :key="item.id" 
                class="file-card"
                :class="{ 'is-cover': index === 0 }"
                draggable="true"
                @dragstart="onDragStart($event, index)"
                @dragover.prevent
                @dragenter.prevent
                @drop="onDrop($event, index)"
              >
                <div class="file-thumb">
                  <img :src="item.preview" />
                  <div class="cover-badge" v-if="index === 0">封面</div>
                  <button type="button" class="remove-btn" @click="removeFile(index)">✕</button>
                </div>
                <div class="file-meta">
                  <div class="file-name" :title="item.file.name">{{ item.file.name }}</div>
                  <div class="file-size">{{ (item.file.size / 1024 / 1024).toFixed(2) }} MB</div>
                </div>
              </div>
            </transition-group>
          </div>

          <!-- 上传区域 -->
          <div class="file-upload-wrapper">
             <div class="file-drop-area" :class="{ 'has-file': filesList.length > 0 }">
                <!-- 注意：开启了 multiple -->
                <input class="file-input-hidden" type="file" accept="image/*" multiple @change="onFilesAdded" id="fileUpload" />
                <label for="fileUpload" class="file-drop-label">
                   <div class="upload-icon">📂</div>
                   <div class="upload-text">
                     <strong v-if="filesList.length === 0">点击或拖拽上传图片</strong>
                     <strong v-else>继续添加图片</strong>
                   </div>
                   <div class="file-info" v-if="filesList.length > 0">
                     <div class="file-meta">已选择 {{ filesList.length }} 张图片</div>
                   </div>
                </label>
             </div>
             <p class="form-hint center">系统将自动压缩生成预览图，并保留原图用于下载。</p>
          </div>

          <div class="form-actions">
            <div v-if="msg" class="message-box" :class="{ error: isError, success: !isError }">
              {{ msg }}
            </div>
            <button class="submit-btn" :disabled="submitting || filesList.length === 0" data-sfx="click">
              <span v-if="submitting" class="spinner"></span>
              {{ submitting ? statusMsg : '🚀 确认并提交' }}
            </button>
          </div>
        </section>
      </form>
    </section>
  </div>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import { api } from '../services/api' 
import { compressToWebP } from '../utils/imageCompressor.js'

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
const uploaderName = ref('')
const title = ref('')
const description = ref('')

const tagDraft = ref('')
const tags = ref([])

const sourceType = ref('personal')
const contentType = ref('haruhi')
const originUrl = ref('')

// 改为文件列表：{ id, file, preview }
const filesList = ref([])

const uid = ref('')
const uidHint = ref('')
const uidAvatar = ref('')
const uidExists = ref(false)
const checkingUid = ref(false)

// 授权状态拆分
const netLicenses = ref([])
const groupLicenses = ref([])

const msg = ref('')
const isError = ref(false)
const submitting = ref(false)
const statusMsg = ref('提交中…')

const uidHintClass = computed(() => uidHint.value.includes('✅') ? 'ok' : (uidHint.value.includes('❌') ? 'bad' : ''))

// --- 方法 ---

// 多图添加
async function onFilesAdded(e) {
  const addedFiles = Array.from(e.target.files || [])
  if (addedFiles.length === 0) return

  // 清空 value 允许重复添加同名文件
  e.target.value = ''

  for (const f of addedFiles) {
    const previewUrl = URL.createObjectURL(f)
    filesList.value.push({
      id: Date.now() + Math.random(),
      file: f,
      preview: previewUrl
    })
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
}
function removeTag(t){ tags.value = tags.value.filter(x => x !== t) }
function clearTags(){ tags.value = []; tagDraft.value = '' }

async function checkUid(){
  const u = uid.value.trim()
  if(!u) return
  checkingUid.value = true
  uidHint.value = ''
  uidAvatar.value = ''
  uidExists.value = false
  try{
    const r = await api.verifyCreator(u)
    if(r.exists){
      uidExists.value = true
      uidAvatar.value = r.creator?.avatar_url || ''
      uidHint.value = '✅ 唯一ID存在'
    }else{
      uidExists.value = false
      uidHint.value = '❌ 唯一ID不存在（请检查输入或联系管理员登记）'
    }
  }catch(e){
    uidExists.value = false
    uidHint.value = `检测失败：${e.message}`
  }finally{
    checkingUid.value = false
  }
}

watch(sourceType, (v) => {
  if(v !== 'personal'){
    uid.value = ''
    uidHint.value = ''
    uidAvatar.value = ''
    uidExists.value = false
    netLicenses.value = []
    groupLicenses.value = []
  }
})

async function submit(){
  msg.value = ''
  isError.value = false
  
  if(filesList.value.length === 0){ msg.value = '请至少选择一张图片'; isError.value = true; return }
  if(!title.value.trim()){
    msg.value = '作品名称为必填'; isError.value = true;
    return
  }

  if(sourceType.value === 'personal'){
    const u = uid.value.trim()
    if(!u){ msg.value = '个人作品必须填写唯一ID'; isError.value = true; return }
    if(!uidExists.value){
      await checkUid()
      if(!uidExists.value){
        msg.value = '唯一ID不存在，无法提交个人作品'; isError.value = true;
        return
      }
    }
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

    fd.append('uploader_name', uploaderName.value.trim())
    fd.append('title', title.value.trim())
    fd.append('description', description.value.trim())
    fd.append('tags', tags.value.join(' '))
    fd.append('source_type', sourceType.value)
    fd.append('content_type', contentType.value)
    fd.append('origin_url', originUrl.value.trim())

    if(sourceType.value === 'personal'){
      fd.append('uploader_uid', uid.value.trim())
      
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
/* 变量映射：适配现代风格 CSS 变量 */
.upload-page {
  --primary: hsl(172, 78%, 42%);
  --primary-soft: hsl(172, 65%, 92%);
  --primary-hover: hsl(172, 80%, 38%);
  --text-main: hsl(210, 20%, 14%);
  --text-sub: hsl(210, 10%, 45%);
  --bg-input: rgba(255, 255, 255, 0.6);
  --border: rgba(0, 0, 0, 0.08);
  --shadow-sm: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
  --shadow-md: 0 10px 15px -3px rgba(0, 0, 0, 0.08);
  --radius-lg: 18px;
  --radius-md: 12px;
}

/* 容器布局 */
.container-card {
  max-width: 900px;
  margin: 0 auto;
  padding-bottom: 60px;
}

/* 头部 Header */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 32px;
  padding: 0 12px;
}

.header-text {
  z-index: 2;
}

.main-title {
  font-size: 36px;
  font-weight: 950;
  letter-spacing: -0.5px;
  margin: 0;
  background: linear-gradient(135deg, var(--text-main) 0%, #4b5563 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.sub-title {
  margin: 8px 0 0;
  font-size: 16px;
  color: var(--text-sub);
  font-weight: 500;
}

.header-decoration .deco-img {
  height: 160px;
  object-fit: contain;
  filter: drop-shadow(0 8px 16px rgba(0,0,0,0.1));
  transform: translateY(10px);
  transition: transform 0.3s ease;
}
.header-decoration:hover .deco-img {
  transform: translateY(0) rotate(2deg);
}

/* 表单主体 */
.main-form {
  background: rgba(255, 255, 255, 0.55);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.8);
  box-shadow: 
    0 20px 40px rgba(0,0,0,0.05),
    inset 0 1px 0 rgba(255,255,255,0.8);
  padding: 8px; /* Inner padding for sections */
}

/* 表单区块 Section */
.form-section {
  padding: 32px;
  border-bottom: 1px solid rgba(0,0,0,0.04);
}
.form-section:last-child {
  border-bottom: none;
}

.section-head {
  margin-bottom: 24px;
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.section-title {
  font-size: 20px;
  font-weight: 800;
  color: var(--text-main);
  margin: 0;
  position: relative;
  padding-left: 14px;
}
.section-title::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 16px;
  background: var(--primary);
  border-radius: 2px;
}

.section-desc {
  font-size: 13px;
  color: var(--text-sub);
  font-weight: 500;
}

/* 表单栅格与控件 */
.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  margin-bottom: 24px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-main);
  display: flex;
  align-items: center;
  gap: 4px;
}
.form-label .req { color: #ef4444; }
.form-label .opt { color: var(--text-sub); font-weight: 400; font-size: 12px; }

/* 输入框样式 */
.form-input, .form-textarea {
  width: 100%;
  padding: 12px 16px;
  border-radius: var(--radius-md);
  border: 1px solid transparent;
  background: var(--bg-input);
  color: var(--text-main);
  font-size: 15px;
  font-weight: 600;
  transition: all 0.2s cubic-bezier(0.2, 0.8, 0.2, 1);
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.02);
}

.form-input:hover, .form-textarea:hover {
  background: rgba(255, 255, 255, 0.9);
}

.form-input:focus, .form-textarea:focus {
  background: #fff;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-soft);
  outline: none;
}

.form-textarea {
  min-height: 120px;
  resize: vertical;
  line-height: 1.6;
}

.form-hint {
  font-size: 13px;
  color: var(--text-sub);
  margin-top: 4px;
}
.form-hint.warning { color: #d97706; }
.form-hint.center { text-align: center; }

/* Segment Control */
.segment-control {
  display: flex;
  background: rgba(0,0,0,0.04);
  padding: 4px;
  border-radius: 14px;
  gap: 4px;
}

.segment-btn {
  flex: 1;
  border: none;
  background: transparent;
  padding: 10px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 700;
  color: var(--text-sub);
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.segment-btn:hover { color: var(--text-main); }
.segment-btn.active {
  background: #fff;
  color: var(--primary);
  box-shadow: 0 2px 8px rgba(0,0,0,0.08);
}
.segment-btn .icon { font-size: 16px; }

/* 复合输入框 (Action Button) */
.input-with-action {
  display: flex;
  gap: 10px;
}
.action-btn {
  padding: 0 20px;
  border-radius: var(--radius-md);
  border: none;
  background: var(--text-main);
  color: #fff;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.action-btn:hover:not(:disabled) { transform: translateY(-1px); background: #000; }

.action-btn.secondary {
  background: rgba(0,0,0,0.05);
  color: var(--text-main);
}
.action-btn.secondary:hover:not(:disabled) { background: rgba(0,0,0,0.1); }

/* UID 验证反馈 */
.uid-feedback {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
  padding: 10px 14px;
  background: rgba(255,255,255,0.5);
  border-radius: 12px;
}
.status-badge { font-size: 14px; font-weight: 700; }
.status-badge.ok { color: #10b981; }
.status-badge.bad { color: #ef4444; }
.creator-avatar {
  width: 32px; height: 32px;
  border-radius: 50%;
  border: 2px solid #fff;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.info-card {
  margin-top: 12px;
  padding: 16px;
  background: #eff6ff;
  border-radius: 12px;
  font-size: 13px;
  color: #1e40af;
  line-height: 1.5;
}
.info-card p { margin: 0; }
.info-card .sub-info { margin-top: 6px; opacity: 0.8; font-size: 12px; }

/* Tags */
.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}
.tag-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #fff;
  border: 1px solid rgba(0,0,0,0.06);
  border-radius: 20px;
  font-size: 13px;
  color: var(--primary);
  font-weight: 700;
  box-shadow: 0 2px 4px rgba(0,0,0,0.03);
}
.tag-remove {
  border: none;
  background: transparent;
  color: #9ca3af;
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 0 2px;
  border-radius: 4px;
}
.tag-remove:hover { color: #ef4444; background: rgba(239,68,68,0.1); }
.text-btn {
  margin-top: 8px;
  background: none;
  border: none;
  color: var(--text-sub);
  font-size: 12px;
  cursor: pointer;
  text-decoration: underline;
}
.text-btn:hover { color: var(--text-main); }

/* License Cards & Layout */
.license-split-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  margin-top: 10px;
}

.license-col {
  display: flex;
  flex-direction: column;
}

.license-header {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-sub);
  margin-bottom: 6px;
  display: flex;
  justify-content: space-between;
}
.badge-private {
  font-size: 10px;
  background: #f3f4f6;
  padding: 2px 6px;
  border-radius: 4px;
  color: #6b7280;
}

.checkbox-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.checkbox-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: var(--bg-input);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}
.checkbox-card:hover { background: #fff; box-shadow: var(--shadow-sm); }
.checkbox-card:has(.chk-input:checked) {
  background: #f0fdfa; /* Teal-50 */
  border-color: var(--primary);
}

.chk-input { display: none; }
.chk-custom {
  width: 18px; height: 18px;
  border: 2px solid #d1d5db;
  border-radius: 5px;
  position: relative;
  transition: all 0.2s;
  flex-shrink: 0;
  background: #fff;
}
.chk-input:checked + .chk-custom {
  background: var(--primary);
  border-color: var(--primary);
}
.chk-input:checked + .chk-custom::after {
  content: '';
  position: absolute;
  left: 5px; top: 1px;
  width: 5px; height: 10px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}
.chk-label {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.3;
}

/* File Upload */
.upload-section { background: rgba(255,255,255,0.4); }

.file-drop-area {
  position: relative;
  border: 2px dashed #cbd5e1;
  border-radius: 20px;
  background: rgba(255,255,255,0.5);
  transition: all 0.3s ease;
  overflow: hidden;
}
.file-drop-area:hover {
  border-color: var(--primary);
  background: rgba(255,255,255,0.8);
}
.file-drop-area.has-file {
  border-style: solid;
  border-color: var(--primary);
  background: #f0fdfa;
}

.file-input-hidden {
  position: absolute;
  width: 100%; height: 100%;
  opacity: 0;
  cursor: pointer;
  z-index: 10;
}

.file-drop-label {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  text-align: center;
  cursor: pointer;
}

.upload-icon { font-size: 48px; margin-bottom: 12px; opacity: 0.8; }
.upload-text { font-size: 16px; color: var(--text-main); }

.file-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  animation: slideUp 0.3s ease;
}
.file-icon { font-size: 32px; margin-bottom: 8px; }
.file-name { font-weight: 700; color: var(--text-main); margin-bottom: 4px; word-break: break-all; max-width: 300px; }
.file-meta { font-size: 12px; color: var(--text-sub); }
.file-change-hint {
  margin-top: 12px;
  font-size: 12px;
  color: var(--primary);
  font-weight: 700;
  background: rgba(255,255,255,0.8);
  padding: 4px 12px;
  border-radius: 99px;
}

/* Form Actions */
.form-actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  margin-top: 32px;
}

.message-box {
  padding: 12px 20px;
  border-radius: 12px;
  font-weight: 700;
  font-size: 14px;
  text-align: center;
  width: 100%;
  animation: fadeIn 0.3s;
}
.message-box.error { background: #fee2e2; color: #b91c1c; }
.message-box.success { background: #d1fae5; color: #047857; }

.submit-btn {
  position: relative;
  width: 100%;
  max-width: 320px;
  padding: 16px;
  border: none;
  border-radius: 16px;
  background: linear-gradient(135deg, var(--text-main) 0%, #111827 100%);
  color: #fff;
  font-size: 18px;
  font-weight: 800;
  letter-spacing: 0.5px;
  cursor: pointer;
  box-shadow: 0 10px 25px -5px rgba(0,0,0,0.2);
  transition: all 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}
.submit-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 15px 30px -5px rgba(0,0,0,0.3);
  background: linear-gradient(135deg, #000 0%, #1f2937 100%);
}
.submit-btn:active:not(:disabled) { transform: translateY(0); }
.submit-btn:disabled { opacity: 0.6; cursor: not-allowed; filter: grayscale(1); }

.spinner {
  width: 20px; height: 20px;
  border: 3px solid rgba(255,255,255,0.3);
  border-radius: 50%;
  border-top-color: #fff;
  animation: spin 0.8s linear infinite;
}

/* 动画 */
@keyframes spin { to { transform: rotate(360deg); } }
@keyframes slideUp { from { transform: translateY(10px); opacity: 0; } to { transform: translateY(0); opacity: 1; } }
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }

.fade-slide-enter-active, .fade-slide-leave-active { transition: all 0.3s ease; max-height: 200px; opacity: 1; overflow: hidden; }
.fade-slide-enter-from, .fade-slide-leave-to { max-height: 0; opacity: 0; transform: translateY(-10px); margin-top: 0; padding-top: 0; padding-bottom: 0; }

.list-enter-active, .list-leave-active { transition: all 0.3s ease; }
.list-enter-from, .list-leave-to { opacity: 0; transform: translateX(-10px); }

/* --- 新增：文件列表样式 (在原有样式之后追加) --- */
.files-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}
.file-card {
  background: #fff;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 10px rgba(0,0,0,0.05);
  cursor: grab;
  position: relative;
  transition: transform 0.2s, box-shadow 0.2s;
}
.file-card:active { cursor: grabbing; transform: scale(1.02); }
.file-card.is-cover {
  outline: 3px solid var(--primary);
  outline-offset: 2px;
}

.file-thumb { height: 120px; position: relative; background: #eee; }
.file-thumb img { width: 100%; height: 100%; object-fit: cover; }

.remove-btn {
  position: absolute; top: 4px; right: 4px;
  width: 24px; height: 24px;
  border-radius: 50%;
  background: rgba(0,0,0,0.5);
  color: #fff;
  border: none;
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  font-size: 14px;
  transition: background 0.2s;
}
.remove-btn:hover { background: rgba(239, 68, 68, 0.9); }

.cover-badge {
  position: absolute; bottom: 0; left: 0; right: 0;
  background: rgba(16, 185, 129, 0.9); /* primary color variant */
  color: #fff;
  font-size: 10px; text-align: center;
  padding: 2px 0; font-weight: 700;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .page-header { flex-direction: column; align-items: flex-start; gap: 16px; margin-bottom: 24px; }
  .header-decoration { display: none; } /* 移动端隐藏装饰图 */
  
  .form-grid { grid-template-columns: 1fr; gap: 20px; }
  .form-section { padding: 20px 16px; }
  
  .license-split-layout { grid-template-columns: 1fr; gap: 24px; }
}
</style>