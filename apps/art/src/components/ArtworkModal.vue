<template>
  <div v-if="visible" class="garden-overlay" @click.self="close">
    <section class="garden-modal" role="dialog" aria-modal="true">
      
      <div class="deco-vine"></div>
      <div class="deco-flower"></div>

      <!-- 图片展示区域 -->
      <div 
        ref="mediaContainer"
        class="modal__media" 
        :class="{ 'is-mobile-expanded': isMobileExpanded }"
        @wheel.prevent="handleWheel"
        @touchstart.passive="handleTouchStart"
        @touchmove="handleTouchMove"
        @touchend.passive="handleTouchEnd"
      >
        <!-- 图片本体：增加 transition 用于切换动画 -->
        <transition :name="slideDirection">
          <img 
            :key="currentIndex"
            :src="displaySrc" 
            :alt="art?.title || 'artwork'" 
            :style="imageTransformStyle"
            class="zoomable-image"
            draggable="false"
            @mousedown="startDrag"
            @click="handleImageClick"
          />
        </transition>

        <!-- 悬浮导航箭头 (仅当有多张图片且未放大/全屏时显示) -->
        <!-- 移动端通过滑动切换，这里也可以保留箭头但要在 CSS 中处理 hover 问题 -->
        <div v-if="images.length > 1 && !isMobileExpanded" class="gallery-nav prev" @click.stop="prevImage" title="上一张">
          <svg viewBox="0 0 24 24"><path d="M15 18l-6-6 6-6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </div>
        <div v-if="images.length > 1 && !isMobileExpanded" class="gallery-nav next" @click.stop="nextImage" title="下一张">
          <svg viewBox="0 0 24 24"><path d="M9 18l6-6-6-6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </div>

        <!-- 底部圆点指示器 -->
        <div v-if="images.length > 1 && !isMobileExpanded" class="gallery-dots">
          <span 
            v-for="(img, idx) in images" 
            :key="idx"
            class="dot"
            :class="{ active: idx === currentIndex }"
            @click.stop="gotoImage(idx)"
          ></span>
        </div>

        <!-- 桌面端：缩放控制栏 -->
        <div class="zoom-controls" v-if="!isMobileExpanded" @click.stop>
          <button class="zoom-btn" @click="zoomOut" title="缩小" type="button">−</button>
          <span class="zoom-text">{{ Math.round(scale * 100) }}%</span>
          <button class="zoom-btn" @click="zoomIn" title="放大" type="button">+</button>
          <button class="zoom-btn reset-btn" @click="resetZoom" title="还原" type="button">↺</button>
        </div>

        <!-- 移动端全屏模式下的提示/关闭按钮 -->
        <div v-if="isMobileExpanded" class="mobile-close-hint">
          <span v-if="images.length > 1">{{ currentIndex + 1 }} / {{ images.length }} · </span>
          再次点击图片收起
        </div>
      </div>

      <div class="modal__side">
        <div class="modal__head">
          <h2>{{ art?.title }}</h2>
          <div class="actions">
            <!-- 下载按钮：下载当前查看的图片 -->
            <a 
              v-if="currentOriginalUrl" 
              :href="currentOriginalUrl" 
              :download="downloadFilename"
              class="icon-btn download-btn" 
              title="下载原图"
              data-sfx="click"
              @click.stop
            >
              <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="7 10 12 15 17 10"></polyline>
                <line x1="12" y1="15" x2="12" y2="3"></line>
              </svg>
            </a>
            <button class="icon-btn" type="button" @click="close" data-sfx="click">✕</button>
          </div>
        </div>

        <div class="kv">
          <b>上传者</b>
          <div>{{ art?.uploader_uid || art?.uploader_name || '匿名' }}</div>

          <b>上传时间</b>
          <div>{{ art?.created_at ? new Date(art.created_at).toLocaleString() : '-' }}</div>

          <b>作品描述</b>
          <div class="small-muted">{{ art?.description || '（无）' }}</div>
        </div>

        <div v-if="tags.length" class="tags" style="margin: 8px 0 0;">
          <button
            v-for="t in tags"
            :key="t"
            class="tag-chip"
            type="button"
            @click="clickTag(t)"
            data-sfx="click"
          >
            #{{ t }}
          </button>
        </div>

        <div class="sep"></div>

        <div>
          <div class="small-muted" style="font-weight: 950; color:var(--accent);">授权许可 (网络/公开)</div>
          <div v-if="publicLicenses.length === 0" class="small-muted">（未勾选任何网络公开授权）</div>
          <ul v-else class="license-list">
            <li v-for="x in publicLicenses" :key="x">{{ x }}</li>
          </ul>
        </div>

        <div class="sep"></div>

        <div>
          <div class="small-muted" style="font-weight: 950; color:var(--accent);">评论</div>

          <div v-if="loadingComments" class="small-muted" style="margin-top: 6px;">风正在传送回音…</div>

          <div v-else style="display:flex; flex-direction:column; gap:10px; margin-top: 10px;">
            <div v-if="comments.length === 0" class="small-muted">暂无回音，在此刻留下你的足迹吧～</div>

            <div
              v-for="c in comments"
              :key="c.id"
              class="panel comment-card"
            >
              <div style="display:flex; align-items:center; justify-content:space-between; gap:10px;">
                <div style="font-weight:950; color:var(--text-deep);">
                  {{ c.user_name }}
                  <span class="small-muted" style="font-weight:400; font-family:serif; margin-left:6px; font-style:italic;">
                    {{ c.created_at ? new Date(c.created_at).toLocaleString() : '' }}
                  </span>
                </div>

                <button class="like-pill" type="button" @click="() => likeComment(c)" data-sfx="click">
                  <svg class="heart" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                    <path d="M12 21s-7-4.6-9.4-8.7C.6 9.1 2.2 5.9 5.6 5.1c2-.5 4 .3 5.3 1.8 1.3-1.5 3.3-2.3 5.3-1.8 3.4.8 5 4 3 7.2C19 16.4 12 21 12 21z"
                      fill="currentColor"/>
                  </svg>
                  <span class="count">{{ Number(c.like_total || 0) }}</span>
                </button>
              </div>

              <div style="margin-top:8px; font-weight:500; line-height:1.6; color:var(--text-main);">
                {{ c.body }}
              </div>
            </div>
          </div>

          <div class="sep"></div>

          <div class="panel input-panel">
            <div class="small-muted" style="font-weight: 950; margin-bottom:8px; color:var(--accent);">讲两句？</div>

            <div class="search input-row" style="margin-bottom:10px;">
              <span class="hint">署名</span>
              <input v-model="commentName" placeholder="无需登录，但可不要冒充别人哦！" />
            </div>

            <div class="search input-row" style="height:auto; padding: 10px 12px;">
              <textarea
                v-model="commentBody"
                rows="3"
                placeholder="下面我来简单喵两句.."
                style="width:100%; border:none; outline:none; background:transparent; resize: vertical; font-weight:500;"
              ></textarea>
            </div>

            <div style="margin-top:10px; display:flex; justify-content:flex-end;">
              <button class="btn btn--accent" type="button" @click="postComment" :aria-disabled="posting ? 'true':'false'" data-sfx="click">
                {{ posting ? '传送中…' : '发送' }}
              </button>
            </div>
          </div>
        </div>

      </div>
    </section>
  </div>
</template>

<script setup>
import { computed, ref, watch, onMounted } from 'vue'
import { api } from '../services/api.js'

const props = defineProps({
  modelValue: { type: Boolean, default: undefined },
  open: { type: Boolean, default: undefined },
  item: { type: Object, default: null },
  artwork: { type: Object, default: null }
})

const emit = defineEmits(['update:modelValue', 'close', 'tag', 'like'])

/* ----------------------------------
   基础显示逻辑
---------------------------------- */
const visible = computed(() => {
  if(props.modelValue !== undefined) return !!props.modelValue
  if(props.open !== undefined) return !!props.open
  return false
})

const art = computed(() => props.item || props.artwork || null)

// --- 画廊多图逻辑 ---
const currentIndex = ref(0)
const slideDirection = ref('fade') // 'slide-left', 'slide-right'

// 获取图片列表，兼容单图数据
const images = computed(() => {
  const a = art.value
  if (!a) return []
  if (Array.isArray(a.images) && a.images.length > 0) return a.images
  // 兼容旧数据
  return [{
    image_url: a.image_url || a.imageUrl || a.url || '',
    original_url: a.original_url || a.originalUrl || a.url || ''
  }]
})

const currentImgSrc = computed(() => images.value[currentIndex.value]?.image_url || '')
const currentOriginalUrl = computed(() => images.value[currentIndex.value]?.original_url || '')

// --- 渐进式加载逻辑 ---
const displaySrc = ref('')

watch([currentIndex, art], () => {
  // 1. 立即显示缩略图/压缩图
  const lowRes = currentImgSrc.value
  displaySrc.value = lowRes
  
  // 2. 后台加载原图
  const highRes = currentOriginalUrl.value
  if (highRes && highRes !== lowRes) {
    const img = new Image()
    img.src = highRes
    img.onload = () => {
      // 只有当当前应该显示的图片仍是这张原图时，才进行替换
      // 防止快速切换时，上一张的原图加载完成覆盖了当前图
      if (currentOriginalUrl.value === highRes) {
        displaySrc.value = highRes
      }
    }
  }
}, { immediate: true })

// 切换图片
function nextImage() {
  if (images.value.length <= 1) return
  slideDirection.value = 'slide-left' // 向左滑
  if (currentIndex.value < images.value.length - 1) {
    currentIndex.value++
  } else {
    currentIndex.value = 0 // 循环
  }
  resetZoom()
}

function prevImage() {
  if (images.value.length <= 1) return
  slideDirection.value = 'slide-right' // 向右滑
  if (currentIndex.value > 0) {
    currentIndex.value--
  } else {
    currentIndex.value = images.value.length - 1
  }
  resetZoom()
}

function gotoImage(idx) {
  slideDirection.value = idx > currentIndex.value ? 'slide-left' : 'slide-right'
  currentIndex.value = idx
  resetZoom()
}

// 监听 item 变化，重置 index
watch(art, () => {
  currentIndex.value = 0
  resetZoom()
})

// 生成下载文件名
const downloadFilename = computed(() => {
  const a = art.value
  if(!a) return 'artwork.jpg'
  
  // 添加序号后缀
  const idxStr = images.value.length > 1 ? `_${currentIndex.value + 1}` : ''
  
  let dateStr = '00000000'
  if(a.created_at) {
    const d = new Date(a.created_at)
    dateStr = `${d.getFullYear()}${String(d.getMonth()+1).padStart(2,'0')}${String(d.getDate()).padStart(2,'0')}`
  }
  let author = (a.uploader_name || a.uploader_uid || '匿名').trim()
  let title = (a.title || '作品').trim()
  const safe = (s) => String(s).replace(/[\\/:*?"<>|]/g, '_').trim()
  const baseName = `${dateStr}-${safe(author)}-${safe(title)}${idxStr}`
  let ext = 'jpg'
  const url = currentOriginalUrl.value
  if(url){
    const cleanUrl = url.split(/[?#]/)[0]
    const parts = cleanUrl.split('.')
    if(parts.length > 1) ext = parts[parts.length - 1]
  }
  return `${baseName}.${ext}`
})

const tags = computed(() => Array.isArray(art.value?.tags) ? art.value.tags : [])
const publicLicenses = computed(() => {
  const all = Array.isArray(art.value?.licenses) ? art.value.licenses : []
  return all.filter(l => l.startsWith('NET:')).map(l => l.replace('NET:', ''))
})

/* ----------------------------------
   图片缩放与拖拽逻辑 (核心增强)
---------------------------------- */
const scale = ref(1)
const translateX = ref(0)
const translateY = ref(0)
const isDragging = ref(false)
const mediaContainer = ref(null)

// 移动端专用状态
const isMobileExpanded = ref(false) // 是否处于全屏查看模式
const lastTouchDistance = ref(0) // 用于双指缩放计算
const swipeOffset = ref(0) // 新增：滑动时的实时位移偏移量

// 计算样式
const imageTransformStyle = computed(() => {
  // 如果正在拖拽或者正在滑动，移除 transition 以保证跟手
  const hasTransition = !isDragging.value && Math.abs(swipeOffset.value) === 0
  
  // 最终的 X 轴偏移是：缩放平移量 + 滑动偏移量
  const finalX = translateX.value + swipeOffset.value
  
  return {
    transform: `translate(${finalX}px, ${translateY.value}px) scale(${scale.value})`,
    cursor: isDragging.value ? 'grabbing' : (scale.value > 1 ? 'grab' : 'default'),
    transition: hasTransition ? 'transform 0.25s cubic-bezier(0.2, 0.8, 0.2, 1)' : 'none'
  }
})

// --- 通用操作 ---
function resetZoom() {
  scale.value = 1
  translateX.value = 0
  translateY.value = 0
  swipeOffset.value = 0
}

function zoomIn() {
  scale.value = Math.min(scale.value + 0.5, 5) // 最大5倍
}

function zoomOut() {
  scale.value = Math.max(scale.value - 0.5, 0.5) // 最小0.5倍
  if(scale.value <= 1) {
    translateX.value = 0
    translateY.value = 0
  }
}

// 鼠标滚轮缩放
function handleWheel(e) {
  // 如果没有全屏且不是Expanded模式，允许滚轮缩放
  e.preventDefault()
  const delta = e.deltaY > 0 ? -0.2 : 0.2
  const newScale = Math.max(0.5, Math.min(5, scale.value + delta))
  scale.value = newScale
  if (newScale <= 1) {
    translateX.value = 0
    translateY.value = 0
  }
}

// --- 桌面端鼠标拖拽 ---
let startX = 0, startY = 0, initialTranslateX = 0, initialTranslateY = 0

function startDrag(e) {
  // 只在左键点击且已放大时允许拖动 (或者在移动端全屏模式下)
  // 注意：touchstart 也会触发 mousedown，这里做个区分
  if (e.type === 'mousedown' && e.button !== 0) return
  if (scale.value <= 1 && !isMobileExpanded.value) return 

  isDragging.value = true
  startX = e.type.includes('mouse') ? e.clientX : e.touches[0].clientX
  startY = e.type.includes('mouse') ? e.clientY : e.touches[0].clientY
  initialTranslateX = translateX.value
  initialTranslateY = translateY.value
  
  // 绑定全局移动事件，防止鼠标移出图片范围
  if(e.type === 'mousedown') {
    window.addEventListener('mousemove', onDrag)
    window.addEventListener('mouseup', stopDrag)
  }
}

function onDrag(e) {
  if (!isDragging.value) return
  e.preventDefault()
  const clientX = e.type.includes('mouse') ? e.clientX : e.touches[0].clientX
  const clientY = e.type.includes('mouse') ? e.clientY : e.touches[0].clientY
  
  const deltaX = clientX - startX
  const deltaY = clientY - startY
  
  translateX.value = initialTranslateX + deltaX
  translateY.value = initialTranslateY + deltaY
}

function stopDrag() {
  isDragging.value = false
  window.removeEventListener('mousemove', onDrag)
  window.removeEventListener('mouseup', stopDrag)
}

// --- 移动端手势逻辑 (点击、双指、滑动切换) ---

let touchStartX = 0
let touchStartY = 0
let isSwiping = false // 标记是否判定为滑动

function handleImageClick(e) {
  const isMobile = window.innerWidth <= 800
  // 如果正在滑动中，不触发点击
  if (isSwiping || Math.abs(swipeOffset.value) > 5) return

  if (isMobile) {
    // 移动端：直接切换全屏模式
    if (!isMobileExpanded.value) {
      isMobileExpanded.value = true
      resetZoom()
    } else {
      isMobileExpanded.value = false
      resetZoom()
    }
  }
}

function handleTouchStart(e) {
  if (e.touches.length === 2) {
    // 双指：开始缩放
    isDragging.value = false // 此时优先缩放
    lastTouchDistance.value = Math.hypot(e.touches[0].clientX - e.touches[1].clientX, e.touches[0].clientY - e.touches[1].clientY)
  } else if (e.touches.length === 1) {
    touchStartX = e.touches[0].clientX
    touchStartY = e.touches[0].clientY
    isSwiping = false
    swipeOffset.value = 0
    
    // 单指：如果已放大，准备拖拽
    if (scale.value > 1 || isMobileExpanded.value) {
      startDrag(e)
    }
  }
}

function handleTouchMove(e) {
  if (e.touches.length === 2) {
    // 双指缩放逻辑
    e.preventDefault() // 防止页面滚动
    const dist = Math.hypot(e.touches[0].clientX - e.touches[1].clientX, e.touches[0].clientY - e.touches[1].clientY)
    if (lastTouchDistance.value > 0) {
      const ratio = dist / lastTouchDistance.value
      let newScale = scale.value * ratio
      newScale = Math.max(0.5, Math.min(newScale, 5))
      scale.value = newScale
    }
    lastTouchDistance.value = dist
  } else if (e.touches.length === 1) {
    // 单指逻辑
    if (isDragging.value) {
      // 放大后的拖拽
      onDrag(e)
    } else if (scale.value === 1 && !isMobileExpanded.value && images.value.length > 1) {
      // 未放大状态：检测左右滑动
      const currentX = e.touches[0].clientX
      const currentY = e.touches[0].clientY
      const diffX = currentX - touchStartX
      const diffY = currentY - touchStartY

      // 判定意图：横向移动大于纵向移动，且移动了一定距离
      if (!isSwiping) {
        if (Math.abs(diffX) > Math.abs(diffY) && Math.abs(diffX) > 10) {
          isSwiping = true
        }
      }

      if (isSwiping) {
        e.preventDefault() // 阻止垂直滚动
        swipeOffset.value = diffX // 图片跟随手指移动
      }
    }
  }
}

function handleTouchEnd(e) {
  if (e.touches.length < 2) {
    lastTouchDistance.value = 0
  }
  
  if (isDragging.value) {
    isDragging.value = false
  }

  // 左右滑动结算
  if (scale.value === 1 && !isMobileExpanded.value && isSwiping) {
    const threshold = 80 // 滑动阈值
    if (Math.abs(swipeOffset.value) > threshold) {
      if (swipeOffset.value > 0) {
        prevImage()
      } else {
        nextImage()
      }
    }
    // 无论是否切换，都需要重置偏移量（如果未切换，CSS transition 会让它弹回去）
    swipeOffset.value = 0
    isSwiping = false
  }
}

/* ----------------------------------
   评论与杂项
---------------------------------- */
const comments = ref([])
const loadingComments = ref(false)
const commentName = ref('')
const commentBody = ref('')
const posting = ref(false)

function close(){
  emit('update:modelValue', false)
  emit('close')
  // 关闭时重置状态
  setTimeout(() => {
    resetZoom()
    isMobileExpanded.value = false
  }, 200)
}

async function loadComments(){
  if(!art.value?.id) return
  loadingComments.value = true
  try{
    const r = await api.listComments(art.value.id)
    comments.value = r.data || []
  }catch{
    comments.value = []
  }finally{
    loadingComments.value = false
  }
}

async function postComment(){
  if(!art.value?.id) return
  const name = commentName.value.trim()
  const body = commentBody.value.trim()
  if(!name || !body) return

  posting.value = true
  try{
    await api.postComment({ artwork_id: art.value.id, user_name: name, body })
    commentBody.value = ''
    await loadComments()
  }finally{
    posting.value = false
  }
}

async function likeComment(c){
  try{
    await api.likeComment(c.id)
    c.like_total = Number(c.like_total || 0) + 1
  }catch(e){ }
}

function clickTag(t){
  emit('tag', t)
  close()
}

watch(visible, (v) => {
  if(v){
    loadComments()
    document.documentElement.style.overflow = 'hidden'
  }else{
    document.documentElement.style.overflow = ''
  }
})

onMounted(() => {
  if(visible.value) loadComments()
})
</script>

<style scoped>
/* =========================
   主题变量定义
========================= */
.garden-overlay {
  --mist-bg: #f4f6f9; 
  --vine-green: #6b8c85; 
  --flower-purple: #bfa2db; 
  --flower-pink: #f3d1dc; 
  --accent: #7a8b99; 
  
  --text-deep: #434d58; 
  --text-main: #5d6d7e; 
  
  --paper-texture: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)' opacity='0.04'/%3E%3C/svg%3E");
  
  position: fixed;
  inset: 0;
  background: rgba(20, 25, 35, 0.4);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  z-index: 999;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.4s ease-out;
}

@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

.garden-modal {
  width: 90vw;
  height: 85vh;
  max-width: 1100px;
  position: relative;
  background: linear-gradient(135deg, #fffcf9 0%, #f7f9fc 100%);
  overflow: visible;
  border: 4px double rgba(107, 140, 133, 0.2); 
  border-radius: 20px;
  box-shadow: 
    0 20px 60px rgba(0, 0, 0, 0.15),
    0 0 0 1px rgba(255, 255, 255, 0.5) inset;
  display: grid;
  grid-template-columns: 1fr 380px;
}

.garden-modal::before {
  content: "";
  position: absolute;
  inset: -6px;
  border-radius: 24px;
  border: 2px solid transparent;
  background: linear-gradient(45deg, var(--vine-green), transparent 40%, transparent 60%, var(--vine-green)) border-box; 
  -webkit-mask: linear-gradient(#fff 0 0) padding-box, linear-gradient(#fff 0 0);
  mask: linear-gradient(#fff 0 0) padding-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  opacity: 0.5;
  pointer-events: none;
  z-index: 10;
}

.deco-flower {
  position: absolute;
  top: -20px;
  left: -20px;
  width: 120px;
  height: 120px;
  background: radial-gradient(circle at 40% 40%, var(--flower-purple), transparent 70%);
  filter: blur(20px);
  opacity: 0.6;
  z-index: 11;
  pointer-events: none;
}

.deco-vine {
  position: absolute;
  bottom: -30px;
  right: -30px;
  width: 150px;
  height: 150px;
  background: radial-gradient(circle, var(--vine-green), transparent 70%);
  filter: blur(30px);
  opacity: 0.4;
  z-index: 11;
  pointer-events: none;
}

@media (max-width: 800px){
  .garden-modal{ grid-template-columns: 1fr; grid-template-rows: 40vh 1fr; height:90vh; }
  .garden-modal::before { display: none; }
}

/* --- 媒体展示区域 --- */
.modal__media {
  background: rgba(240, 242, 245, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden; /* 防止图片放大溢出 */
  border-radius: 16px 0 0 16px;
  user-select: none;
}

/* 移动端全屏模式样式 */
.modal__media.is-mobile-expanded {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: #000;
  border-radius: 0;
  width: 100vw;
  height: 100vh;
}
.modal__media.is-mobile-expanded img {
  object-fit: contain; 
}
.mobile-close-hint {
  position: absolute;
  bottom: 30px;
  left: 50%;
  transform: translateX(-50%);
  color: rgba(255,255,255,0.7);
  background: rgba(0,0,0,0.5);
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  pointer-events: none;
}

.modal__media::after {
  content: "";
  position: absolute;
  inset: 0;
  box-shadow: inset 0 0 20px rgba(107, 140, 133, 0.1);
  pointer-events: none;
}

/* 移除遮罩以支持纯净查看 */
.modal__media.is-mobile-expanded::after {
  display: none;
}

.zoomable-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  display: block;
  /* 移除混合模式以免影响查看原图细节，或者在非expanded模式下保留 */
  /* mix-blend-mode: multiply; */
  transform-origin: center center;
  /* will-change: transform; removed to fix blurry scaling on some browsers */
}

/* --- 悬浮导航箭头 --- */
.gallery-nav {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 40px;
  height: 40px;
  background: rgba(0,0,0,0.3);
  color: #fff;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
  opacity: 0;
  z-index: 10;
}
.gallery-nav svg {
  width: 24px;
  height: 24px;
}

/* 仅在支持 Hover 的设备上（桌面端）启用悬浮显示逻辑 */
@media (hover: hover) {
  .modal__media:hover .gallery-nav {
    opacity: 1;
  }
}

.gallery-nav:hover {
  background: rgba(0,0,0,0.6);
  transform: translateY(-50%) scale(1.1);
}
.gallery-nav.prev {
  left: 16px;
}
.gallery-nav.next {
  right: 16px;
}

/* --- 底部圆点指示器 --- */
.gallery-dots {
  position: absolute;
  bottom: 60px; /* 缩放控制栏上方 */
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 6px;
  z-index: 10;
}
.dot {
  width: 8px;
  height: 8px;
  background: rgba(0,0,0,0.2);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s;
}
.dot:hover {
  background: rgba(0,0,0,0.5);
}
.dot.active {
  background: #fff;
  box-shadow: 0 0 4px rgba(0,0,0,0.3);
  transform: scale(1.2);
}

/* --- 缩放控制条 (桌面端) --- */
.zoom-controls {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(255, 255, 255, 0.9);
  padding: 6px 10px;
  border-radius: 99px;
  display: flex;
  align-items: center;
  gap: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  border: 1px solid rgba(255,255,255,0.6);
  z-index: 20;
}
.zoom-btn {
  width: 28px; height: 28px;
  border-radius: 50%;
  border: 1px solid rgba(107, 140, 133, 0.2);
  background: #fff;
  color: var(--text-deep);
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  font-size: 16px;
  line-height: 1;
  transition: all 0.2s;
}
.zoom-btn:hover {
  background: var(--flower-pink);
  border-color: var(--flower-pink);
  color: #fff;
}
.zoom-text {
  font-size: 12px;
  font-family: monospace;
  color: var(--text-deep);
  min-width: 36px;
  text-align: center;
}
.reset-btn { font-size: 12px; }

/* --- 右侧侧边栏 --- */
.modal__side {
  display: flex; flex-direction: column;
  background: rgba(255, 255, 255, 0.72);
  padding: 24px;
  overflow-y: auto;
  border-left: 1px solid rgba(107, 140, 133, 0.1);
  background-image: var(--paper-texture);
  font-family: "Georgia", "Songti SC", "SimSun", serif;
}

.modal__side::-webkit-scrollbar { width: 4px; }
.modal__side::-webkit-scrollbar-thumb { background: rgba(107, 140, 133, 0.3); border-radius: 4px; }
.modal__side::-webkit-scrollbar-track { background: transparent; }

.modal__head {
  display: flex; align-items: flex-start; justify-content: space-between;
  gap: 12px;
  margin-bottom: 20px;
}
.modal__head h2 {
  font-size: 22px;
  font-weight: 600;
  color: var(--text-deep);
  line-height: 1.4;
  letter-spacing: 0.5px;
  text-shadow: 0 2px 10px rgba(191, 162, 219, 0.2);
}

.actions {
  display: flex;
  gap: 8px;
}

.icon-btn {
  flex-shrink: 0;
  width: 32px; height: 32px;
  border: 1px solid rgba(107, 140, 133, 0.2);
  background: rgba(255, 255, 255, 0.6);
  border-radius: 50%;
  color: var(--vine-green);
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex; align-items: center; justify-content: center;
}
.icon-btn:hover {
  background: #fff;
  box-shadow: 0 0 10px var(--flower-pink);
  transform: rotate(90deg);
  border-color: var(--flower-pink);
}

/* 下载按钮特殊样式 */
.download-btn:hover {
  color: var(--flower-purple);
  transform: translateY(-2px); /* 只有下载按钮向上浮动，不旋转 */
  box-shadow: 0 4px 10px rgba(191, 162, 219, 0.3);
}

.kv {
  display: grid;
  grid-template-columns: 70px 1fr;
  gap: 8px 12px;
  font-size: 13px;
  margin-bottom: 16px;
  color: var(--text-main);
}
.kv b {
  font-weight: 600;
  color: var(--vine-green); 
  text-align: right;
  opacity: 0.8;
}
.small-muted {
  font-size: 13px;
  color: var(--text-main);
  opacity: 0.8;
  line-height: 1.6;
}

.tags { display: flex; flex-wrap: wrap; gap: 8px; }
.tag-chip {
  border: 1px solid rgba(191, 162, 219, 0.3);
  background: linear-gradient(to bottom, rgba(255,255,255,0.8), rgba(243, 209, 220, 0.1));
  padding: 5px 12px;
  border-radius: 12px 2px 12px 2px;
  font-size: 12px;
  color: #7b6c88;
  cursor: pointer;
  transition: all 0.2s;
  font-family: sans-serif;
}
.tag-chip:hover {
  background: #fff;
  border-color: var(--flower-purple);
  box-shadow: 0 4px 12px rgba(191, 162, 219, 0.2);
  transform: translateY(-1px);
}

.sep {
  height: 1px;
  background: linear-gradient(to right, transparent, rgba(107, 140, 133, 0.2), transparent);
  margin: 20px 0;
}

.license-list {
  margin: 6px 0 0 0;
  padding-left: 18px;
  font-size: 13px; opacity: 0.8; line-height: 1.6;
  color: var(--text-deep);
  list-style-type: square;
}

.panel {
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 4px 12px rgba(107, 140, 133, 0.05);
}

.comment-card {
  padding: 14px;
  border-radius: 12px;
  margin-bottom: 12px;
  transition: transform 0.2s;
}
.comment-card:hover {
  background: rgba(255, 255, 255, 0.8);
  transform: translateX(2px);
}

.input-panel {
  padding: 16px;
  border-radius: 16px;
  background: rgba(244, 246, 249, 0.5);
  border: 1px dashed rgba(107, 140, 133, 0.3);
}

.search {
  display: flex; align-items: center;
  background: rgba(255, 255, 255, 0.7);
  border-bottom: 1px solid rgba(107, 140, 133, 0.3);
  border-radius: 4px 4px 0 0;
  padding: 0 8px;
  height: 40px;
  transition: all 0.2s;
}
.input-row { border: none; border-bottom: 1px solid rgba(107, 140, 133, 0.2); }
.input-row:focus-within {
  background: #fff;
  border-bottom-color: var(--flower-purple);
}

.search .hint {
  font-size: 12px; color: var(--accent); opacity: 0.6; margin-right: 8px;
  font-family: sans-serif;
}
.search input, .search textarea {
  border: none; background: transparent; outline: none;
  flex: 1; font-size: 14px; color: var(--text-deep);
  font-family: "Georgia", serif;
}
.search textarea::placeholder, .search input::placeholder {
  color: var(--text-main); opacity: 0.4; font-style: italic;
}

.btn--accent {
  background: linear-gradient(135deg, var(--text-main), var(--text-deep));
  color: #fff;
  border: 0; padding: 6px 20px; border-radius: 999px;
  font-size: 13px; cursor: pointer;
  box-shadow: 0 4px 10px rgba(67, 77, 88, 0.2);
  transition: all 0.3s;
  letter-spacing: 1px;
}
.btn--accent:hover {
  background: linear-gradient(135deg, var(--flower-purple), #9b80b9);
  box-shadow: 0 6px 14px rgba(155, 128, 185, 0.3);
  transform: translateY(-1px);
}
.btn--accent:disabled { opacity: 0.6; cursor: not-allowed; transform: none; box-shadow: none; }

.like-pill {
  display: flex; align-items: center; gap: 6px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.5);
  padding: 4px 10px; border-radius: 999px;
  font-size: 12px;
  color: var(--accent);
  cursor: pointer;
  transition: all 0.2s;
}
.like-pill:hover {
  background: #fff;
  color: var(--flower-purple);
  box-shadow: 0 2px 8px rgba(191, 162, 219, 0.2);
}
.heart { width: 14px; height: 14px; color: currentColor; }

/* 切换动画 - 添加部分 */
.slide-left-enter-active, .slide-left-leave-active, .slide-right-enter-active, .slide-right-leave-active, .fade-enter-active, .fade-leave-active {
  transition: all 0.3s ease;
  position: absolute;
  width: 100%;
  height: 100%;
  object-fit: contain;
}
.fade-enter-from, .fade-leave-to { opacity: 0; }
.slide-left-enter-from { transform: translateX(20px); opacity: 0; }
.slide-left-leave-to { transform: translateX(-20px); opacity: 0; }
.slide-right-enter-from { transform: translateX(-20px); opacity: 0; }
.slide-right-leave-to { transform: translateX(20px); opacity: 0; }
</style>