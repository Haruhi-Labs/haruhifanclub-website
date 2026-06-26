<script setup>
import { computed } from 'vue'
import { thumbUrl } from '../services/api.js'
// import { useRouter } from 'vue-router' // No longer needed for author navigation

const props = defineProps({
  // 兼容多种父组件传参命名
  items: { type: Array, default: undefined },
  artworks: { type: Array, default: undefined },
  list: { type: Array, default: undefined },
  data: { type: Array, default: undefined },

  // --- 新增：分页相关 Props ---
  page: { type: Number, default: 1 },
  pageCount: { type: Number, default: 1 },
  hasMore: { type: Boolean, default: false },
  loading: { type: Boolean, default: false },

  // 兼容回调
  onOpen: { type: Function, default: null },
  onLike: { type: Function, default: null },
  onTag: { type: Function, default: null }
})

const emit = defineEmits([
  'open', 'like', 'tag',
  'select', 'itemClick', 'click', 'likeArtwork', 'tagClick',
  'author',
  'prevPage', 'nextPage', 'goPage' // --- 新增：翻页事件 ---
])

// const router = useRouter() 

const list = computed(() => {
  return props.items ?? props.artworks ?? props.list ?? props.data ?? []
})

// 分页页码列表（含省略号），逻辑对齐设计系统 SosPagination
const pageList = computed(() => {
  const total = props.pageCount
  const current = props.page
  const span = 1
  if (total <= 1) return [1]
  const range = new Set([1, total])
  for (let i = current - span; i <= current + span; i += 1) {
    if (i >= 1 && i <= total) range.add(i)
  }
  const sorted = [...range].sort((a, b) => a - b)
  const out = []
  let prev = 0
  for (const p of sorted) {
    if (prev && p - prev > 1) out.push('…')
    out.push(p)
    prev = p
  }
  return out
})

function goPage(p) {
  if (typeof p !== 'number') return
  const next = Math.min(Math.max(p, 1), props.pageCount)
  if (next !== props.page) emit('goPage', next)
}

function imgSrc(item){
  return item?.image_url || item?.imageUrl || item?.url || ''
}

// 网格统一用 640px 缩略图：卡片 <img> 与模糊背景层共用同一 URL，
// 浏览器只下载/解码一次；展示原图留给弹窗的渐进加载
function thumbSrc(item){
  return thumbUrl(imgSrc(item), 640)
}

function isPersonal(item){
  return item?.source_type === 'personal'
}

function badgeText(item){
  return isPersonal(item) ? '个人作品' : '网络&其它'
}

function badgeClass(item){
  return isPersonal(item) ? 'badge badge--personal' : 'badge badge--network'
}

function likeCount(item){
  return Number(item?.like_total || 0)
}

function displayUploader(item){
  const uid = (item?.uploader_uid || '').trim()
  const name = (item?.uploader_name || '').trim()
  // 优先显示昵称快照；uid（统一账号下形如 u{id}）只作回退，避免作者名显示成 uXX
  if(name) return name
  if(uid) return uid
  return '匿名'
}

function isAuthorClickable(item){
  const uid = (item?.uploader_uid || '').trim()
  return !!uid && isPersonal(item)
}

function openCard(item){
  emit('open', item)
  // 兼容旧事件
  emit('select', item)
  emit('itemClick', item)
  emit('click', item)
}

function like(item, e){
  // 阻止冒泡，防止触发外层卡片的点击
  e?.stopPropagation?.()
  emit('like', item)
  emit('likeArtwork', item)
}

function goAuthor(item, e){
  e?.stopPropagation?.()
  const uid = (item?.uploader_uid || '').trim()
  if(!uid) return
  // 旧逻辑：router.push
  // 新逻辑：emit author 事件，让父组件处理筛选
  const name = (item?.uploader_name || '').trim()
  emit('author', { uid, name: name || uid })
}

function clickTag(tag, item, e){
  e?.stopPropagation?.()
  emit('tag', tag)
  emit('tagClick', tag)
}

// --- 智能缩放逻辑 ---
import { reactive } from 'vue'

const imgStyle = reactive({})

// 与模板 :key 同一回退链：id 缺失时不让多个项落进同一个 undefined 桶串样式
function itemKey(item) {
  return item?.id ?? item?.file_path ?? item?.title ?? imgSrc(item)
}

function onImgLoad(item, e) {
  const el = e.target
  if (!el) return
  
  const w = el.naturalWidth
  const h = el.naturalHeight
  
  // 避免除零
  if (!h) return 

  const ratio = w / h
  
  // 阈值设定：
  // 极端比例定义：更宽松一点，避免正常竖图(9:16=0.56)被裁切
  // 取 < 0.45 (极高) 或 > 2.2 (极宽) 作为极端阈值
  // 极端比例 -> cover (保证填满，避免主体看起来太小)
  // 正常比例 -> contain (完整展示)
  
  const isExtreme = ratio < 0.42 || ratio > 2.3

  imgStyle[itemKey(item)] = {
    objectFit: isExtreme ? 'cover' : 'contain',
    // 统一居中，对于 contain 模式会留白，对于 cover 模式会裁切
    objectPosition: 'center center'
  }
}

// --- 视口外暂停浮动动画：滚出屏幕的卡片不再产生每帧合成开销 ---
import { ref, onMounted, onUpdated, onBeforeUnmount } from 'vue'

const galleryEl = ref(null)
let cardObserver = null

function observeCards() {
  if (!cardObserver || !galleryEl.value) return
  // 先断开旧观察再全量重扫：翻页/筛选移除的卡片节点不再被观察器持有
  cardObserver.disconnect()
  galleryEl.value.querySelectorAll('.art-card-wrap').forEach(el => cardObserver.observe(el))
}

onMounted(() => {
  cardObserver = new IntersectionObserver((entries) => {
    for (const en of entries) {
      en.target.style.animationPlayState = en.isIntersecting ? '' : 'paused'
    }
  }, { rootMargin: '100px' })
  observeCards()
})

onUpdated(observeCards)

onBeforeUnmount(() => {
  cardObserver?.disconnect()
  cardObserver = null
})
</script>

<template>
  <div class="gallery" ref="galleryEl">
    <div class="grid">
      <div
        v-for="it in list"
        :key="it.id ?? it.file_path ?? it.title"
        class="art-card-wrap"
      >
        <article
          class="art-card"
          role="button"
          tabindex="0"
          @click="openCard(it)"
          @keydown.enter="openCard(it)"
        >
          <div class="art-card__media">
            <!-- 背景模糊层：填补留白（与 <img> 共用缩略图，零额外请求） -->
            <div
              class="art-card__blur-bg"
              :style="{ backgroundImage: `url(${thumbSrc(it)})` }"
            ></div>

            <img
              class="art-card__img"
              :src="thumbSrc(it)"
              :alt="it.title || 'artwork'"
              loading="lazy"
              decoding="async"
              :style="imgStyle[itemKey(it)] || { objectFit: 'cover' }"
              @load="onImgLoad(it, $event)"
            />
          </div>

          <div class="art-card__body">
            <div class="art-card__meta">
              <span :class="badgeClass(it)">{{ badgeText(it) }}</span>

              <button
                class="like-pill"
                type="button"
                @click.stop="(e) => like(it, e)"
                data-sfx="click"
                aria-label="点赞"
              >
                <svg class="heart" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <path
                    d="M12 21s-7-4.6-9.4-8.7C.6 9.1 2.2 5.9 5.6 5.1c2-.5 4 .3 5.3 1.8 1.3-1.5 3.3-2.3 5.3-1.8 3.4.8 5 4 3 7.2C19 16.4 12 21 12 21z"
                    fill="currentColor"
                  />
                </svg>
                <b>赞</b>
                <span class="count">{{ likeCount(it) }}</span>
              </button>
            </div>

            <div class="art-card__title">
              {{ it.title }}
            </div>

            <div class="byline">
              <span class="byline__k">上传者：</span>

              <a
                v-if="isAuthorClickable(it)"
                class="byline__a"
                href="javascript:void(0)"
                @click.stop="(e) => goAuthor(it, e)"
                data-sfx="click"
              >
                {{ displayUploader(it) }}
              </a>
              <span v-else class="byline__v">{{ displayUploader(it) }}</span>
            </div>

            <div class="tags" v-if="Array.isArray(it.tags) && it.tags.length">
              <button
                v-for="t in it.tags.slice(0, 6)"
                :key="t"
                class="tag-chip"
                type="button"
                @click.stop="(e) => clickTag(t, it, e)"
                data-sfx="click"
              >
                #{{ t }}
              </button>
            </div>
          </div>
        </article>
      </div>
    </div>

    <!-- 翻页：采用设计系统统一分页规范 .sos-pagination 外观（桥接，随 art 表达主题化） -->
    <nav class="gallery-pager sos-pagination" v-if="pageCount > 1" aria-label="分页">
      <button
        type="button"
        class="sos-pagination__item"
        :disabled="page <= 1 || loading"
        aria-label="上一页"
        data-sfx="click"
        @click="goPage(page - 1)"
      >‹</button>
      <template v-for="(p, i) in pageList">
        <span v-if="p === '…'" :key="`gap-${i}`" class="sos-pagination__item" aria-hidden="true">…</span>
        <button
          v-else
          :key="p"
          type="button"
          class="sos-pagination__item"
          :aria-current="p === page ? 'page' : undefined"
          data-sfx="click"
          @click="goPage(p)"
        >{{ p }}</button>
      </template>
      <button
        type="button"
        class="sos-pagination__item"
        :disabled="page >= pageCount || loading"
        aria-label="下一页"
        data-sfx="click"
        @click="goPage(page + 1)"
      >›</button>
    </nav>

  </div>
</template>

<style scoped>
/* =========================
   核心主题配置
========================= */
.gallery {
  --bg-deep: #525289; 
  --glass-bg: rgba(30, 21, 21, 0.694);
  --glass-border: rgba(151, 68, 68, 0.1);
  
  /* 弃用电光 neon，改取 art 表达的青绿 / 粉柔光（在暗底上提亮以保证可读） */
  --neon-cyan: color-mix(in srgb, var(--sos-accent) 48%, white);
  --neon-purple: color-mix(in srgb, var(--sos-accent-2) 52%, white);
  --neon-glow: color-mix(in srgb, var(--sos-accent) 38%, transparent);

  /* 青调夜色玻璃画框底（对齐 .sos-artwork-card 的 night-900 + accent） */
  --card-bg: linear-gradient(
    180deg,
    color-mix(in srgb, var(--sos-night-900) 86%, var(--sos-accent)),
    var(--sos-night-900)
  );

  --text-main: rgba(255, 255, 255, 0.96);
  --text-muted: rgba(255, 255, 255, 0.64);

  --shadow-media: 0 14px 32px -10px rgba(18, 50, 60, 0.5);

  width: min(1450px, calc(100% - 40px));
  margin: 60px auto 120px;
}

/* =========================
   布局 Grid
========================= */
.grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 100px 50px;
}

@keyframes float-card {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

.art-card-wrap {
  width: 100%;
  animation: float-card 6.8s ease-in-out infinite;
  /* 永久动画的卡片预提升为独立合成层，浮动只走 GPU 合成、不重栅格化 */
  will-change: transform;
}

.art-card-wrap:nth-child(2n) {
  animation-delay: -2.2s;
}

.art-card-wrap:nth-child(3n) {
  animation-delay: -4.3s;
}

.art-card-wrap:hover {
  animation-play-state: paused;
}

/* 弹窗打开时暂停浮动：卡片被全屏 backdrop-filter 遮罩盖住，
   继续动画会让遮罩每帧重模糊，白白消耗 GPU */
body.modal-open .art-card-wrap {
  animation-play-state: paused;
}

@media (max-width: 1100px) {
  .grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 90px 30px;
  }
}

/* --- 手机端适配 (单列 + 紧凑) --- */
@media (max-width: 768px) {
  .grid {
    /* 强制单列 */
    grid-template-columns: 1fr;
    gap: 60px 0; /* 减小垂直间距 */
  }
  .gallery { width: calc(100% - 24px); } /* 稍微加宽一点显示区域 */
}

/* =========================
   Card: 3D 底座
========================= */
.art-card {
  position: relative;
  background: var(--card-bg);
  border: 1px solid rgba(255,255,255,0.14);
  border-radius: var(--sos-card-radius, 24px);

  transform-style: preserve-3d;
  /* Flattens preserve-3d (same as the removed backdrop-filter did),
     prevents hover on one card from shifting siblings in the perspective scene */
  isolation: isolate;

  transition: transform 0.4s cubic-bezier(0.25, 0.8, 0.25, 1), box-shadow 0.3s ease;
  box-shadow: 0 16px 38px -16px rgba(18, 50, 60, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.08);

  cursor: pointer;
  outline: none;
  overflow: visible;

  padding-bottom: 12px;

  width: 100%;
  max-width: 380px;
  margin: 0 auto;

  transform: translateZ(0);
}

.art-card:hover {
  transform: translateY(-10px) scale(1.02) rotateX(2deg);
  z-index: 20; 
}

.art-card:focus-visible {
  border-color: var(--neon-cyan);
  box-shadow: 0 0 0 2px var(--neon-cyan);
}

/* =========================
   Media: 悬浮图片层 (Z=24px)
========================= */
.art-card__media {
  position: relative;
  aspect-ratio: 4 / 3;
  width: 110%;
  margin-left: -5%;
  margin-top: -15%;
  border-radius: 20px;
  background: #000;
  overflow: hidden;

  transform-style: preserve-3d;
  transform: translateZ(24px) translateY(-22px) scale(1.02);

  box-shadow: var(--shadow-media), 0 25px 50px rgba(0,0,0,0.5);
  transition: box-shadow 0.3s ease;
  border: 1px solid rgba(255,255,255,0.15);
  backface-visibility: hidden;
}

.art-card:hover .art-card__media {
  box-shadow:
    0 30px 70px -20px color-mix(in srgb, var(--sos-accent) 42%, transparent),
    0 0 0 1px rgba(255, 255, 255, 0.32);
}

.art-card__img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transform: scale(1.02);
  transition: transform 0.5s ease;
  position: relative;
  z-index: 2; /* 确保在模糊层之上 */
}

/* 模糊背景层 — 恢复毛玻璃效果 */
.art-card__blur-bg {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background-size: cover;
  background-position: center;
  opacity: 0.6;
  transform: scale(1.4);
  filter: blur(20px);
  z-index: 1;
}

.art-card:hover .art-card__img {
  transform: scale(1.1);
}

/* =========================
   Body Layout
========================= */
.art-card__body {
  padding: 18px 20px 8px; 
  display: grid;
  gap: 6px;
  
  transform: translateZ(10px);
  transform-style: preserve-3d;
}

/* 手机端进一步紧凑化 */
@media (max-width: 768px) {
  .art-card__body {
    padding: 12px 16px 6px; /* 减少内边距 */
    gap: 4px; /* 极小间距 */
  }
  .art-card {
    padding-bottom: 8px; /* 减少底部留白 */
  }
}

.art-card__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 2px;
  
  position: relative;
  transform: translateZ(30px); 
}

/* =========================
   Typography
========================= */
.art-card__title {
  font-family: sans-serif;
  font-weight: 700;
  color: var(--text-main);
  font-size: 22px; 
  line-height: 1.1;
  text-shadow: 0 2px 4px rgba(0,0,0,0.5);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* 手机端标题稍微缩小一点，避免太满 */
@media (max-width: 768px) {
  .art-card__title {
    font-size: 20px;
  }
}

.byline {
  font-size: 15px; 
  line-height: 1.2;
  color: var(--text-muted);
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  align-items: baseline;
  
  transform: translateZ(30px);
}
.byline__k { opacity: 0.6; }
.byline__v { color: rgb(255, 255, 255); }
.byline__a {
  color: var(--neon-cyan);
  text-decoration: none;
  border-bottom: 1px dashed rgba(110, 203, 208, 0.3);
  text-shadow: 0 0 8px var(--neon-glow);
  transition: all 0.2s;
  cursor: pointer;
}
.byline__a:hover {
  color: var(--sos-bg-surface);
  border-bottom-color: var(--sos-bg-surface);
  text-shadow: 0 0 12px var(--neon-cyan);
}

/* =========================
   组件：Badge
========================= */
.badge {
  display: inline-flex;
  align-items: center;
  height: 24px;
  padding: 0 10px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  border: 1px solid transparent;
}

.badge--network {
  background: color-mix(in srgb, var(--sos-night-900) 70%, var(--sos-accent));
  color: var(--neon-cyan);
  border-color: color-mix(in srgb, var(--sos-accent) 40%, transparent);
}

.badge--personal {
  background: color-mix(in srgb, var(--sos-night-900) 72%, var(--sos-accent-2));
  color: var(--neon-purple);
  border-color: color-mix(in srgb, var(--sos-accent-2) 40%, transparent);
}

/* =========================
   组件：Like Pill
========================= */
.like-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 12px;
  border-radius: var(--sos-radius-full, 16px);
  /* 暗底上的玻璃片：白计数清晰可读（此前白底白字看不见） */
  background: rgba(255, 255, 255, 0.12);
  border: 1px solid rgba(255, 255, 255, 0.18);
  color: var(--text-main);
  cursor: pointer;
  transition: all 0.3s ease;
  pointer-events: auto;
}

.like-pill .heart {
  width: 16px;
  height: 16px;
  color: var(--sos-accent-2); /* art 粉色点赞 */
}
.like-pill b { display: none; }
.like-pill .count { font-family: monospace; font-size: 14px; color: var(--text-main); }

.like-pill:hover {
  background: rgba(255, 255, 255, 0.2);
  border-color: color-mix(in srgb, var(--sos-accent-2) 55%, transparent);
  box-shadow: 0 0 14px color-mix(in srgb, var(--sos-accent-2) 32%, transparent);
}
.like-pill:active { transform: scale(0.95); }

/* =========================
   组件：Tag Chip
========================= */
.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
  transform: translateZ(30px);
}

.tag-chip {
  /* 暗底上的半透明白片 + 浅色字（此前 color:solid(...) 非法 + 近白底，白字白底看不见） */
  background: rgba(255, 255, 255, 0.14);
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: var(--sos-radius-full, 99px);
  padding: 4px 10px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.9);
  cursor: pointer;
  transition: all 0.2s;
}

.tag-chip:hover {
  background: rgba(255, 255, 255, 0.24);
  color: #fff;
  border-color: color-mix(in srgb, var(--sos-accent) 55%, transparent);
}

/* =========================
   翻页：采用设计系统统一分页规范 .sos-pagination（随 art 表达主题化）
   ========================= */
.gallery-pager {
  display: flex;
  justify-content: center;
  margin-top: 72px;
  margin-bottom: 24px;
}

</style>
