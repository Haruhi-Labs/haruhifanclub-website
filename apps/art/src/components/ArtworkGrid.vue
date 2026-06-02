<script setup>
import { computed } from 'vue'
// import { useRouter } from 'vue-router' // No longer needed for author navigation

const props = defineProps({
  // 兼容多种父组件传参命名
  items: { type: Array, default: undefined },
  artworks: { type: Array, default: undefined },
  list: { type: Array, default: undefined },
  data: { type: Array, default: undefined },

  // --- 新增：分页相关 Props ---
  page: { type: Number, default: 1 },
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
  'prevPage', 'nextPage' // --- 新增：翻页事件 ---
])

// const router = useRouter() 

const list = computed(() => {
  return props.items ?? props.artworks ?? props.list ?? props.data ?? []
})

function imgSrc(item){
  return item?.image_url || item?.imageUrl || item?.url || ''
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
  if(uid) return uid
  if(name) return name
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
  
  imgStyle[item.id] = {
    objectFit: isExtreme ? 'cover' : 'contain',
    // 统一居中，对于 contain 模式会留白，对于 cover 模式会裁切
    objectPosition: 'center center'
  }
}
</script>

<template>
  <div class="gallery">
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
            <!-- 背景模糊层：填补留白 -->
            <div 
              class="art-card__blur-bg"
              :style="{ backgroundImage: `url(${imgSrc(it)})` }"
            ></div>

            <img
              class="art-card__img"
              :src="imgSrc(it)"
              :alt="it.title || 'artwork'"
              loading="lazy"
              :style="imgStyle[it.id] || { objectFit: 'cover' }"
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

    <div class="pagination-twin" v-if="page > 1 || hasMore">
      <div class="twin-branch left" :class="{ disabled: page <= 1 }">
        <button 
          class="twin-btn prev" 
          @click="emit('prevPage')" 
          :disabled="page <= 1 || loading"
          data-sfx="click"
        >
          <span class="icon">←</span>
          <span class="text">PREV</span>
        </button>
      </div>

      <div class="twin-knot">
        <span class="page-num">{{ page }}</span>
      </div>

      <div class="twin-branch right" :class="{ disabled: !hasMore }">
        <button 
          class="twin-btn next" 
          @click="emit('nextPage')" 
          :disabled="!hasMore || loading"
          data-sfx="click"
        >
          <span class="text">NEXT</span>
          <span class="icon">→</span>
        </button>
      </div>
    </div>

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
  
  --neon-cyan: #00f2ff;
  --neon-purple: #e9b5fd;
  --neon-glow: rgba(0, 242, 255, 0.4);
  
  --text-main: rgba(255, 255, 255, 0.95);
  --text-muted: rgba(255, 255, 255, 0.55);

  --shadow-media: 0 10px 30px -5px rgba(0, 242, 255, 0.15); 

  /* Twin Theme Colors (Green) */
  --twin-primary: #10b981; /* Emerald 500 */
  --twin-dark: #064e3b;    /* Emerald 900 */
  --twin-light: #a7f3d0;   /* Emerald 200 */
  --twin-glow: rgba(16, 185, 129, 0.5);

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
  background: rgba(30, 21, 21, 0.82);
  border: 1px solid rgba(255,255,255,0.15);
  border-radius: 24px;

  transform-style: preserve-3d;
  /* Flattens preserve-3d (same as the removed backdrop-filter did),
     prevents hover on one card from shifting siblings in the perspective scene */
  isolation: isolate;

  transition: transform 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
  box-shadow: 0 0 20px rgba(0, 242, 255, 0.12), 0 0 10px rgba(188, 19, 254, 0.1), inset 0 0 15px rgba(0,0,0,0.4);

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
  box-shadow: 0 30px 80px rgba(0, 242, 255, 0.4), 0 0 0 1px rgba(255,255,255,0.4);
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
  color: #fff;
  border-bottom-color: #fff;
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
  background: rgba(30, 45, 50, 0.55);
  color: var(--neon-cyan);
  border-color: rgba(29, 170, 178, 0.3);
  box-shadow: 0 0 10px rgba(0, 242, 255, 0.1);
}

.badge--personal {
  background: rgba(50, 25, 60, 0.55);
  color: var(--neon-purple);
  border-color: rgba(188, 19, 254, 0.3);
  box-shadow: 0 0 10px rgba(188, 19, 254, 0.1);
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
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.721);
  border: 1px solid rgba(255,255,255,0.1);
  color: var(--text-main);
  cursor: pointer;
  transition: all 0.3s ease;
  pointer-events: auto;
}

.like-pill .heart {
  width: 16px;
  height: 16px;
  color: #ff4757;
}
.like-pill b { display: none; }
.like-pill .count { font-family: monospace; font-size: 14px; }

.like-pill:hover {
  background: rgb(105, 216, 244);
  border-color: #ff4757;
  box-shadow: 0 0 15px rgba(255, 71, 87, 0.3);
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
  background: rgba(255, 255, 255, 0.827);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 99px;
  padding: 4px 10px;
  font-size: 13px;
  color: solid(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}

.tag-chip:hover {
  background: rgba(255,255,255,0.1);
  color: #fff;
  border-color: rgba(255,255,255,0.3);
  text-shadow: 0 0 5px rgba(255,255,255,0.5);
}

/* =========================
   Twin Pagination (艺术化双子翻页)
   ========================= */
.pagination-twin {
  display: flex;
  justify-content: center;
  align-items: center;
  margin-top: 80px;
  margin-bottom: 40px;
  gap: 0; /* 必须为0，确保视觉连接 */
  position: relative;
  height: 60px;
}

/* 中心的核心结 */
.twin-knot {
  position: relative;
  width: 50px;
  height: 50px;
  border-radius: 50%;
  background: radial-gradient(circle, var(--twin-light), var(--twin-primary));
  box-shadow: 0 0 20px var(--twin-glow), inset 0 0 10px rgba(255,255,255,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 5;
  border: 2px solid #fff;
}

.page-num {
  font-family: monospace;
  font-weight: 900;
  font-size: 18px;
  color: var(--twin-dark);
}

/* 左右分支容器 */
.twin-branch {
  position: relative;
  display: flex;
  align-items: center;
  transition: all 0.4s ease;
  opacity: 1;
}

/* 不可用时隐藏分支 */
.twin-branch.disabled {
  opacity: 0;
  pointer-events: none;
  transform: scale(0.8);
}

/* 按钮基础样式 */
.twin-btn {
  background: transparent;
  border: 2px solid var(--twin-primary);
  color: var(--twin-light);
  height: 40px;
  padding: 0 24px;
  font-family: monospace;
  font-weight: 700;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 10px;
  position: relative;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  overflow: hidden;
  background: rgba(0, 0, 0, 0.15);
}

.twin-btn:hover {
  background: var(--twin-primary);
  color: var(--twin-dark);
  box-shadow: 0 0 15px var(--twin-glow);
  text-shadow: none;
}

.twin-btn:active {
  transform: scale(0.95);
}

/* 左侧按钮：连接到右侧结 */
.twin-branch.left .twin-btn {
  border-right: none; 
  border-radius: 20px 0 0 20px;
  padding-right: 30px; /* 留出空间给结 */
  margin-right: -10px; /* 缩进 */
}

/* 右侧按钮：连接到左侧结 */
.twin-branch.right .twin-btn {
  border-left: none; 
  border-radius: 0 20px 20px 0;
  padding-left: 30px; /* 留出空间给结 */
  margin-left: -10px; /* 缩进 */
}

</style>
