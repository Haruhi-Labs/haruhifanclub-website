<template>
  <!-- 保持 overflow-x-hidden 防止横向滚动 -->
  <div class="activity-page">

    <!-- ==================== 0. 动态背景层 (保留原样) ==================== -->
    <div class="bg-layer-fixed">
      <div class="bg-layer-black"></div>
      <div
        class="bg-layer-image"
        style="background-image: url('ok3.webp');"
      ></div>
      <div class="cyber-grid bg-layer-grid"></div>
      <canvas ref="bgFxCanvas" class="bg-fx-canvas"></canvas>
      <!-- 背景光晕 -->
      <div class="bg-glow bg-glow--magenta"></div>
      <div class="bg-glow bg-glow--cyan"></div>
    </div>

    <!-- 拖尾特效层 -->
    <canvas ref="trailCanvas" class="trail-canvas"></canvas>

    <!-- ==================== 页面主体内容 ==================== -->
    <div class="page-content">

      <!-- 页头：标题容器居中 -->
      <div class="page-header group">
        <div class="activity-title-wrap">
          <h1
            class="cyber-glitch-title page-title"
            :class="{ 'title-power-off': isPowerOff }"
            data-text="活动中心"
          >
            活动中心
          </h1>
        </div>
        <div class="subtitle-row">
          <div class="subtitle-line animate-pulse"></div>
          <p class="subtitle-text glitch-sub" data-text="SOS BRIGADE">
            跟着团长一起参与好玩的活动吧！
          </p>
          <div class="subtitle-line animate-pulse"></div>
        </div>
      </div>

      <!-- 活动列表 -->
      <div class="activity-grid" ref="activityGridRef">

        <div
          v-for="(activity, index) in paginatedActivities"
          :key="activity.id"
          class="activity-col"
          :data-activity-id="String(activity.id)"
          :ref="(el) => setCardColRef(el, activity.id)"
          :class="{
            'activity-col--offset-top': index % 2 !== 0,
            'activity-col--offset-bottom': index % 2 === 0
          }"
        >
          <!-- 3D 容器 -->
          <div
            class="tilt-container"
            :style="getTiltStyle(index, activity.id)"
          >
            <div
              class="cyber-card group"
              :class="{ 'mobile-active': mobileActiveId === activity.id }"
              :style="getFloatAnimation(index, activity.id)"
              @mouseenter="!isMobile && (hoverId = activity.id)"
              @mouseleave="!isMobile && (hoverId = null)"
              @click="handleCardClick(activity)"
            >
              <!-- LAYER 0: 霓虹边框与背景 -->
              <div
                class="card-neon-border"
                :style="getLayerStyle(activity.id, 0, 20)"
              >
                <div class="neon-glow-outer animate-gradient-border"></div>
                <div class="neon-glow-inner animate-gradient-border"></div>
                <div class="neon-bg-fill"></div>
              </div>

              <!-- LAYER 1: 图片层 -->
              <div
                class="card-image-wrap"
                :style="getLayerStyle(activity.id, 30)"
              >
                <img
                  :src="getActivityImage(activity)"
                  class="card-image"
                  loading="lazy"
                  decoding="async"
                  @error="handleImgError"
                >
                <div class="card-image-noise"></div>
                <div class="card-image-gradient"></div>

                <!-- 状态角标 -->
                <div
                  class="card-badge-position"
                  :class="index % 2 === 0 ? 'card-badge-position--left' : 'card-badge-position--right'"
                  style="transform: translateZ(10px);"
                >
                  <div
                    class="card-badge"
                    :class="index % 2 === 0 ? 'card-badge--skew-right' : 'card-badge--skew-left'"
                  >
                    {{ activity.status }}
                  </div>
                </div>
              </div>

              <!-- LAYER 2: 信息层 -->
              <div
                class="card-info"
                :class="index % 2 === 0 ? 'card-info--right' : 'card-info--left'"
                :style="getLayerStyle(activity.id, 60)"
              >
                <h3 class="card-title">
                  {{ activity.title }}
                </h3>

                <p
                  class="card-intro"
                  :class="index % 2 === 0 ? 'card-intro--right-aligned' : 'card-intro--left-aligned'"
                >
                  {{ activity.intro }}
                </p>

                <!-- 底部奖励栏 -->
                <div
                  class="card-rewards"
                  :class="{ 'card-rewards--event': !hasPointsReward(activity) }"
                  :style="getLayerStyle(activity.id, 20, 20)"
                >
                  <template v-if="hasPointsReward(activity)">
                    <!-- 顶部：总奖池 -->
                    <div
                      class="reward-pool-row"
                      :class="index % 2 === 0 ? 'reward-pool-row--reverse' : ''"
                    >
                      <span class="reward-label">REWARD POOL</span>
                      <span class="reward-value">{{ formatPoints(activity.totalPoints) }}</span>
                    </div>

                    <!-- 底部：单次奖励 -->
                    <div
                      class="reward-action-row"
                      :class="index % 2 === 0 ? 'reward-action-row--right' : ''"
                    >
                      <div
                        class="reward-action-indicator animate-pulse"
                        :class="index % 2 === 0 ? 'reward-action-indicator--right' : 'reward-action-indicator--left'"
                      ></div>
                      <span class="reward-action-points">{{ formatDeltaPoints(activity.pointsPerAction) }}</span>
                      <span class="reward-action-name">/ {{ activity.actionName || '活动行为' }}</span>
                    </div>
                  </template>

                  <template v-else>
                    <div
                      class="reward-pool-row"
                      :class="index % 2 === 0 ? 'reward-pool-row--reverse' : ''"
                    >
                      <span class="reward-label">ACTIVITY MODE</span>
                      <span class="reward-value reward-value--event">{{ getActivityModeLabel(activity) }}</span>
                    </div>

                    <div
                      class="reward-action-row reward-action-row--event"
                      :class="index % 2 === 0 ? 'reward-action-row--right' : ''"
                    >
                      <div
                        class="reward-action-indicator animate-pulse"
                        :class="index % 2 === 0 ? 'reward-action-indicator--right' : 'reward-action-indicator--left'"
                      ></div>
                      <span class="reward-action-points reward-action-points--event">{{ getEventActionText(activity) }}</span>
                      <span class="reward-action-name reward-action-name--event">{{ getEventMetaText(activity) }}</span>
                    </div>
                  </template>
                </div>

                <!-- 移动端按钮 -->
                <div class="card-mobile-cta"
                     :style="{ maxHeight: mobileActiveId === activity.id ? '60px' : '0', opacity: mobileActiveId === activity.id ? '1' : '0' }">
                  <button class="card-mobile-btn clip-path-button"
                          style="transform: translateZ(20px);"
                          @click.stop="openDetailModal(activity)">
                    查看详情
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

      </div>

      <!-- 分页控件 -->
      <nav v-if="totalPages > 1" class="cyber-pagination" aria-label="活动分页">
        <button
          class="cyber-page-btn cyber-page-btn--nav"
          :disabled="currentPage === 1"
          @click="goToPage(currentPage - 1)"
        >
          <span>&lt; PREV</span>
        </button>

        <template v-for="(p, i) in pageNumbers" :key="`p-${p}-${i}`">
          <span
            v-if="i > 0 && p - pageNumbers[i - 1] > 1"
            class="cyber-page-ellipsis"
          >…</span>
          <button
            class="cyber-page-btn cyber-page-btn--num"
            :class="{ 'is-active': p === currentPage }"
            @click="goToPage(p)"
          >
            {{ String(p).padStart(2, '0') }}
          </button>
        </template>

        <button
          class="cyber-page-btn cyber-page-btn--nav"
          :disabled="currentPage === totalPages"
          @click="goToPage(currentPage + 1)"
        >
          <span>NEXT &gt;</span>
        </button>

        <span class="cyber-page-meta">
          {{ String(currentPage).padStart(2, '0') }} / {{ String(totalPages).padStart(2, '0') }}
        </span>
      </nav>
    </div>

    <!-- ==================== 详情弹窗 ==================== -->
    <Transition name="cyber-pop">
      <div v-if="selectedActivity" class="modal-overlay" @click="closeModal">
        <div class="modal-backdrop"></div>

        <div class="modal-container" @click.stop>
          <div class="modal-top-bar"></div>

          <div class="modal-scroll-area custom-scrollbar">
            <div class="modal-hero">
              <img
                :src="getActivityImage(selectedActivity)"
                class="modal-hero-img"
                decoding="async"
                @error="handleImgError"
              >
              <div class="modal-hero-overlay"></div>
              <div class="modal-hero-fade"></div>
            </div>

            <div class="modal-body">
              <div class="modal-watermark">SOS</div>

              <h2 class="modal-title">
                {{ selectedActivity.title }}
              </h2>

              <div class="modal-tags">
                <span class="modal-tag modal-tag--cyan">ID: {{ String(selectedActivity.id).padStart(3, '0') }}</span>
                <span class="modal-tag modal-tag--magenta">TYPE: {{ selectedActivity.type }}</span>
              </div>

              <div class="prose prose-invert prose-lg modal-prose">
                <p class="modal-intro-text">{{ selectedActivity.intro }}</p>
                <!-- 修改点：使用 formatContent 处理后的 HTML，支持链接点击 -->
                <div v-html="formatContent(selectedActivity.detail)" class="modal-detail-content activity-detail-content"></div>
              </div>

              <div class="modal-footer">
                <div class="modal-waiting-text animate-pulse">
                  &gt; WAITING FOR INPUT...
                </div>
                <button @click="closeModal" class="modal-close-btn">
                  <span class="modal-close-btn-text">CLOSE TERMINAL</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { useMainStore } from '@/stores/main';

// --- 数据部分 ---
const store = useMainStore();
const activities = computed(() => store.activities);

// 分页：整个列表数据量大（存在多张 base64 data URI 图片），一次性渲染会卡顿
const PAGE_SIZE = 12;
const currentPage = ref(1);
const totalPages = computed(() => Math.max(1, Math.ceil(activities.value.length / PAGE_SIZE)));
const paginatedActivities = computed(() => {
  const start = (currentPage.value - 1) * PAGE_SIZE;
  return activities.value.slice(start, start + PAGE_SIZE);
});
const pageNumbers = computed(() => {
  const total = totalPages.value;
  const cur = currentPage.value;
  const pages = new Set([1, total, cur, cur - 1, cur + 1]);
  return Array.from(pages)
    .filter((n) => n >= 1 && n <= total)
    .sort((a, b) => a - b);
});
const activityGridRef = ref(null);
const goToPage = (p) => {
  const next = Math.min(Math.max(1, p), totalPages.value);
  if (next === currentPage.value) return;
  currentPage.value = next;
  // 切页后回到列表顶部
  if (activityGridRef.value?.scrollIntoView) {
    activityGridRef.value.scrollIntoView({ behavior: 'smooth', block: 'start' });
  } else {
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }
};
watch(activities, () => {
  if (currentPage.value > totalPages.value) currentPage.value = totalPages.value;
});

const FALLBACK_ACTIVITY_IMAGE = `data:image/svg+xml;utf8,${encodeURIComponent(
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600">
    <defs>
      <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="#0a0a0a"/>
        <stop offset="100%" stop-color="#111827"/>
      </linearGradient>
    </defs>
    <rect width="800" height="600" fill="url(#bg)"/>
    <rect x="18" y="18" width="764" height="564" rx="18" fill="none" stroke="#00ffff" stroke-opacity="0.25" stroke-width="2"/>
    <text x="400" y="290" text-anchor="middle" fill="#00ffff" font-size="54" font-family="monospace" font-weight="700">SOS</text>
    <text x="400" y="340" text-anchor="middle" fill="#9ca3af" font-size="20" font-family="monospace">Activity</text>
  </svg>`
)}`;

const handleImgError = (e) => {
  const img = e?.target;
  if (!img || img.dataset?.fallbackApplied === '1') return;

  img.dataset.fallbackApplied = '1';
  img.onerror = null;
  img.src = FALLBACK_ACTIVITY_IMAGE;
};

// 规范化图片源：空值/非法值直接使用占位图，避免 <img src=""> 在部分浏览器上既不报错也不显示
const getActivityImage = (activity) => {
  const raw = activity?.image;
  if (typeof raw !== 'string') return FALLBACK_ACTIVITY_IMAGE;
  const trimmed = raw.trim();
  if (!trimmed) return FALLBACK_ACTIVITY_IMAGE;
  // 修正不必要的 ./ 相对前缀，避免在子路由下解析错误
  if (trimmed.startsWith('./')) return '/' + trimmed.slice(2);
  return trimmed;
};

// --- 交互逻辑 ---
const isMobile = ref(false);
const hoverId = ref(null);
const mobileActiveId = ref(null);
const selectedActivity = ref(null);
const activityConfigs = ref([]);
const cardVisibilityMap = ref({});
const observedCardEls = new Map();
let cardIntersectionObserver = null;

/* ★ 星空配置 */
const STAR_COUNT_DESKTOP = 96;
const STAR_COUNT_MOBILE = 68;
const STREAK_COUNT = 8;
const BG_TARGET_FPS_DESKTOP = 30;
const BG_TARGET_FPS_MOBILE = 24;
const BG_MAX_DPR_DESKTOP = 1.25;
const BG_MAX_DPR_MOBILE = 1.0;

/* Canvas Trail 配置 */
const MAX_CANVAS_DPR = 1.5;
const TRAIL_TARGET_FPS = 45;
const TRAIL_FRAME_MS = 1000 / TRAIL_TARGET_FPS;
const TRAIL_IDLE_STOP_MS = 180;
const PARTICLES_PER_INPUT = 2;
const INPUT_SPAWN_INTERVAL_MS = 16;
const MAX_PARTICLES_DESKTOP = 220;
const MAX_PARTICLES_MOBILE = 120;

/* 标题断电 */
const isPowerOff = ref(false);
const flickerTimeouts = [];
let isDestroyed = false;
let isPageVisible = true;

const randomRange = (min, max) => Math.random() * (max - min) + min;
const isCardInViewport = (activityId) => cardVisibilityMap.value[String(activityId)] !== false;
const parseNumber = (value) => {
  if (value === '' || value === null || value === undefined) return 0;
  const n = Number(value);
  return Number.isFinite(n) ? n : 0;
};
const normalizeLabel = (value) => String(value || '').trim();

const hasPointsReward = (activity) => (
  parseNumber(activity?.totalPoints) !== 0 || parseNumber(activity?.pointsPerAction) !== 0
);

const formatPoints = (value) => parseNumber(value).toLocaleString();

const formatDeltaPoints = (value) => {
  const n = parseNumber(value);
  if (n > 0) return `+${n.toLocaleString()}`;
  return n.toLocaleString();
};

const getActivityModeLabel = (activity) => {
  const type = normalizeLabel(activity?.type);
  const status = normalizeLabel(activity?.status);
  if (type) return type;
  if (status) return status;
  return '综合活动';
};

const getEventActionText = (activity) => {
  const actionName = normalizeLabel(activity?.actionName);
  if (actionName) return actionName;
  return '参与活动';
};

const getEventMetaText = (activity) => {
  const status = normalizeLabel(activity?.status);
  if (status) return `状态 / ${status}`;
  return '非积分活动';
};

const checkMobile = () => {
  isMobile.value = window.innerWidth < 1024;
};

const initConfigs = () => {
  activityConfigs.value = activities.value.map((_, index) => {
    const isLeft = index % 2 === 0;
    const baseAngle = isLeft ? 10 : -10;

    const durX = randomRange(3, 5) + 's';
    const durY = randomRange(4, 7) + 's';
    const delay = randomRange(0, 2) + 's';
    return { baseAngle, durX, durY, delay };
  });
};

watch(activities, () => {
  initConfigs();
}, { immediate: true });

watch(activities, (list) => {
  const idSet = new Set(list.map((item) => String(item.id)));
  const nextVisibilityMap = {};

  Object.entries(cardVisibilityMap.value).forEach(([id, visible]) => {
    if (idSet.has(id)) nextVisibilityMap[id] = visible;
  });
  cardVisibilityMap.value = nextVisibilityMap;

  Array.from(observedCardEls.keys()).forEach((id) => {
    if (idSet.has(id)) return;
    const el = observedCardEls.get(id);
    if (el && cardIntersectionObserver) cardIntersectionObserver.unobserve(el);
    observedCardEls.delete(id);
  });
}, { immediate: true });

const initCardObserver = () => {
  if (cardIntersectionObserver || typeof window === 'undefined') return;

  if (!('IntersectionObserver' in window)) {
    const nextVisibilityMap = { ...cardVisibilityMap.value };
    observedCardEls.forEach((_, id) => {
      nextVisibilityMap[id] = true;
    });
    cardVisibilityMap.value = nextVisibilityMap;
    return;
  }

  cardIntersectionObserver = new IntersectionObserver((entries) => {
    let changed = false;
    const nextVisibilityMap = { ...cardVisibilityMap.value };

    entries.forEach((entry) => {
      const id = entry.target?.dataset?.activityId;
      if (!id) return;

      const isVisible = entry.isIntersecting || entry.intersectionRatio > 0;
      if (nextVisibilityMap[id] === isVisible) return;

      nextVisibilityMap[id] = isVisible;
      changed = true;
    });

    if (changed) cardVisibilityMap.value = nextVisibilityMap;
  }, {
    root: null,
    rootMargin: '180px 0px 180px 0px',
    threshold: [0, 0.01, 0.1]
  });

  observedCardEls.forEach((el, id) => {
    if (!el) return;
    el.dataset.activityId = id;
    cardIntersectionObserver.observe(el);
  });
};

const setCardColRef = (el, activityId) => {
  const id = String(activityId);
  const previousEl = observedCardEls.get(id);

  if (!el) {
    if (previousEl && cardIntersectionObserver) cardIntersectionObserver.unobserve(previousEl);
    observedCardEls.delete(id);
    return;
  }

  if (previousEl === el) return;

  if (previousEl && previousEl !== el && cardIntersectionObserver) {
    cardIntersectionObserver.unobserve(previousEl);
  }

  el.dataset.activityId = id;
  observedCardEls.set(id, el);

  if (cardIntersectionObserver) {
    cardIntersectionObserver.observe(el);
  }
};

const teardownCardObserver = () => {
  if (cardIntersectionObserver) {
    cardIntersectionObserver.disconnect();
    cardIntersectionObserver = null;
  }
  observedCardEls.clear();
  cardVisibilityMap.value = {};
};

/* 背景 Canvas 特效（星空 + 流光） */
const bgFxCanvas = ref(null);
let bgFxCtx = null;
let bgFxAnimationId = null;
let bgCanvasWidth = 0;
let bgCanvasHeight = 0;
let lastBgFrameTime = 0;
let starField = [];
let streakField = [];
const starSpriteCache = new Map();
const streakSpriteCache = new Map();
let starCoreSprite = null;

const getStarCount = () => (isMobile.value ? STAR_COUNT_MOBILE : STAR_COUNT_DESKTOP);

const createStarConfig = () => {
  const colors = ['#ffffff', '#a0ffff', '#ffb3ff'];
  return {
    xRatio: Math.random(),
    yRatio: Math.random(),
    scale: 0.5 + Math.random() * 1.5,
    delay: Math.random() * 6,
    duration: 3 + Math.random() * 5,
    opacity: 0.2 + Math.random() * 0.6,
    blur: Math.random() * 1,
    color: colors[Math.floor(Math.random() * colors.length)],
    twinkleFreq: 0.14 + Math.random() * 0.28,
    sparkleFreq: 1.0 + Math.random() * 1.6,
    pulseFreq: 0.07 + Math.random() * 0.18,
    twinkleOffset: Math.random() * Math.PI * 2,
    sparkleOffset: Math.random() * Math.PI * 2,
    pulseOffset: Math.random() * Math.PI * 2,
    twinklePower: 0.8 + Math.random() * 0.9,
    pulseSharpness: 5 + Math.random() * 3,
    driftAmp: 0.6 + Math.random() * 1.6,
    driftFreq: 0.04 + Math.random() * 0.08,
    driftOffset: Math.random() * Math.PI * 2
  };
};

const createStreakConfig = (index) => ({
  yRatio: Math.random(),
  length: Math.random() * 300 + 100,
  duration: Math.random() * 4 + 3,
  delay: Math.random() * 5,
  thickness: Math.random() * 1.4 + 0.8,
  color: index % 2 === 0 ? '#0ff' : '#f0f',
  maxAlpha: 0.5
});

const initBackgroundFx = () => {
  starField = Array.from({ length: getStarCount() }, () => createStarConfig());
  streakField = Array.from({ length: STREAK_COUNT }, (_, i) => createStreakConfig(i));
};

const createStarSprite = (color) => {
  const sprite = document.createElement('canvas');
  const size = 40;
  sprite.width = size;
  sprite.height = size;
  const spriteCtx = sprite.getContext('2d');
  if (!spriteCtx) return null;

  const gradient = spriteCtx.createRadialGradient(size / 2, size / 2, 1, size / 2, size / 2, size / 2);
  gradient.addColorStop(0, color);
  gradient.addColorStop(0.35, color);
  gradient.addColorStop(1, 'rgba(255,255,255,0)');

  spriteCtx.fillStyle = gradient;
  spriteCtx.fillRect(0, 0, size, size);
  return sprite;
};

const getStarCoreSprite = () => {
  if (starCoreSprite) return starCoreSprite;

  const sprite = document.createElement('canvas');
  const size = 18;
  sprite.width = size;
  sprite.height = size;
  const spriteCtx = sprite.getContext('2d');
  if (!spriteCtx) return null;

  const gradient = spriteCtx.createRadialGradient(size / 2, size / 2, 0.5, size / 2, size / 2, size / 2);
  gradient.addColorStop(0, 'rgba(255,255,255,1)');
  gradient.addColorStop(0.4, 'rgba(255,255,255,0.9)');
  gradient.addColorStop(1, 'rgba(255,255,255,0)');

  spriteCtx.fillStyle = gradient;
  spriteCtx.fillRect(0, 0, size, size);
  starCoreSprite = sprite;
  return starCoreSprite;
};

const getStarSprite = (color) => {
  if (!starSpriteCache.has(color)) {
    starSpriteCache.set(color, createStarSprite(color));
  }
  return starSpriteCache.get(color);
};

const createStreakSprite = (color) => {
  const sprite = document.createElement('canvas');
  sprite.width = 256;
  sprite.height = 6;
  const spriteCtx = sprite.getContext('2d');
  if (!spriteCtx) return null;

  const gradient = spriteCtx.createLinearGradient(0, 0, sprite.width, 0);
  gradient.addColorStop(0, 'rgba(0,0,0,0)');
  gradient.addColorStop(0.5, color);
  gradient.addColorStop(1, 'rgba(0,0,0,0)');
  spriteCtx.fillStyle = gradient;
  spriteCtx.fillRect(0, 0, sprite.width, sprite.height);
  return sprite;
};

const getStreakSprite = (color) => {
  if (!streakSpriteCache.has(color)) {
    streakSpriteCache.set(color, createStreakSprite(color));
  }
  return streakSpriteCache.get(color);
};

const resizeBgFxCanvas = () => {
  if (!bgFxCanvas.value) return;

  bgCanvasWidth = window.innerWidth;
  bgCanvasHeight = window.innerHeight;
  const dprLimit = isMobile.value ? BG_MAX_DPR_MOBILE : BG_MAX_DPR_DESKTOP;
  const dpr = Math.min(window.devicePixelRatio || 1, dprLimit);

  bgFxCanvas.value.style.width = `${bgCanvasWidth}px`;
  bgFxCanvas.value.style.height = `${bgCanvasHeight}px`;
  bgFxCanvas.value.width = Math.floor(bgCanvasWidth * dpr);
  bgFxCanvas.value.height = Math.floor(bgCanvasHeight * dpr);

  if (bgFxCtx) {
    bgFxCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }
};

const drawStar = (star, timeSec) => {
  const twinklePhase = timeSec * star.twinkleFreq * Math.PI * 2 + star.twinkleOffset;
  const twinkleBase = 0.5 + 0.5 * Math.sin(twinklePhase);
  const twinkle = Math.pow(twinkleBase, star.twinklePower);
  const sparkle = 0.5 + 0.5 * Math.sin(timeSec * star.sparkleFreq * Math.PI * 2 + star.sparkleOffset);
  const pulseWave = Math.max(0, Math.sin(timeSec * star.pulseFreq * Math.PI * 2 + star.pulseOffset));
  const pulse = Math.pow(pulseWave, star.pulseSharpness);

  const driftPhase = timeSec * star.driftFreq * Math.PI * 2 + star.driftOffset;
  const x = star.xRatio * bgCanvasWidth + Math.sin(driftPhase) * star.driftAmp;
  const y = star.yRatio * bgCanvasHeight + Math.cos(driftPhase * 0.9) * star.driftAmp * 0.7;

  const size = star.scale * (5.6 + twinkle * 5.4 + sparkle * 1.9 + pulse * 3.8) + star.blur * 0.8;
  const alpha = Math.min(1, star.opacity * (0.08 + twinkle * 0.72 + sparkle * 0.16 + pulse * 0.65));

  const sprite = getStarSprite(star.color);
  const coreSprite = getStarCoreSprite();
  if (!sprite || !coreSprite) return;

  bgFxCtx.globalAlpha = alpha;
  bgFxCtx.drawImage(sprite, x - size / 2, y - size / 2, size, size);

  const coreSize = size * (0.24 + sparkle * 0.1 + pulse * 0.14);
  bgFxCtx.globalAlpha = Math.min(1, alpha * (1.1 + pulse * 0.25));
  bgFxCtx.drawImage(coreSprite, x - coreSize / 2, y - coreSize / 2, coreSize, coreSize);

  if (pulse > 0.72 && size > 4) {
    const glintLen = size * (0.8 + pulse * 0.65);
    bgFxCtx.globalAlpha = Math.min(0.35, alpha * pulse * 0.45);
    bgFxCtx.fillStyle = '#ffffff';
    bgFxCtx.fillRect(x - glintLen / 2, y - 0.55, glintLen, 1.1);
    bgFxCtx.fillRect(x - 0.55, y - glintLen / 2, 1.1, glintLen);
  }
};

const drawStreak = (streak, timeSec) => {
  const progress = ((timeSec + streak.delay) % streak.duration) / streak.duration;
  const x = -streak.length + progress * (bgCanvasWidth + streak.length * 2);
  const y = streak.yRatio * bgCanvasHeight;
  const ramp = progress < 0.2 ? progress / 0.2 : (progress > 0.8 ? (1 - progress) / 0.2 : 1);
  const alpha = Math.max(0, ramp) * streak.maxAlpha;
  const sprite = getStreakSprite(streak.color);
  if (!sprite) return;

  bgFxCtx.globalAlpha = alpha;
  bgFxCtx.drawImage(sprite, x, y - streak.thickness / 2, streak.length, streak.thickness);
};

const animateBackgroundFx = (now) => {
  if (!bgFxCtx || !bgFxCanvas.value || !isPageVisible) return;

  const bgFrameMs = 1000 / (isMobile.value ? BG_TARGET_FPS_MOBILE : BG_TARGET_FPS_DESKTOP);
  if (now - lastBgFrameTime < bgFrameMs) {
    bgFxAnimationId = requestAnimationFrame(animateBackgroundFx);
    return;
  }
  lastBgFrameTime = now;

  bgFxCtx.clearRect(0, 0, bgCanvasWidth, bgCanvasHeight);
  bgFxCtx.save();
  bgFxCtx.globalCompositeOperation = 'screen';

  const timeSec = now / 1000;
  for (let i = 0; i < streakField.length; i++) drawStreak(streakField[i], timeSec);
  for (let i = 0; i < starField.length; i++) drawStar(starField[i], timeSec);

  bgFxCtx.restore();
  bgFxAnimationId = requestAnimationFrame(animateBackgroundFx);
};

const startBackgroundFx = () => {
  if (bgFxAnimationId || !bgFxCtx || !bgFxCanvas.value || !isPageVisible) return;
  bgFxAnimationId = requestAnimationFrame(animateBackgroundFx);
};

const stopBackgroundFx = () => {
  if (!bgFxAnimationId) return;
  cancelAnimationFrame(bgFxAnimationId);
  bgFxAnimationId = null;
  lastBgFrameTime = 0;
};

// 修改点：检查 hover 状态，如果是 hover，则角度归零（或保留很小的角度），方便对齐
const getTiltStyle = (index, activityId) => {
  const config = activityConfigs.value[index];
  if (!config) return {};

  if (isMobile.value) {
    return { transform: 'none' };
  }

  // 检查是否激活
  const isActive = hoverId.value === activityId;
  // 分页后 config 与实际位置可能错位，直接用当前 index 奇偶决定倾斜方向
  const baseAngle = index % 2 === 0 ? 10 : -10;
  // 如果激活，强制回正到 0deg，这样就不会有斜向视差，保证完全对齐
  const currentAngle = isActive ? 0 : baseAngle;

  return {
    transform: `perspective(1000px) rotateY(${currentAngle}deg)`,
    transition: 'transform 0.5s ease-out' // 确保回正过程平滑
  };
};

// 修改点：Hover 时的整体浮动逻辑
const getFloatAnimation = (index, activityId) => {
  const config = activityConfigs.value[index];
  if (!config) return {};

  if (!isCardInViewport(activityId) && hoverId.value !== activityId && mobileActiveId.value !== activityId) {
    return { animation: 'none' };
  }

  if (!isMobile.value && hoverId.value === activityId) {
    return {
      // Hover 时整体稍微放大并浮起，注意 rotateY(0deg) 配合 getTiltStyle 里的 0deg 确保完全正对屏幕
      transform: 'rotateY(0deg) scale(1.02) translateZ(40px)',
      zIndex: 50,
      boxShadow: '0 0 60px rgba(0, 255, 255, 0.3)'
    };
  }

  if (isMobile.value && mobileActiveId.value === activityId) {
    return {
      transform: 'scale(1.02) translateZ(20px)',
      zIndex: 50,
      borderColor: '#0ff'
    };
  }

  return {
    '--float-dur-y': config.durY,
    '--float-dur-x': config.durX,
    '--float-delay': config.delay,
    animation: `cyber-float-y var(--float-dur-y) ease-in-out infinite alternate var(--float-delay)`
  };
};

// 新增点：核心对齐逻辑
// 在 Hover 时，强行将所有层级的 Z 轴设为相同的值（HOVER_Z），消除层级视差
const getLayerStyle = (activityId, defaultZ, hoverZ = 20) => {
  const isActive = (!isMobile.value && hoverId.value === activityId) || (isMobile.value && mobileActiveId.value === activityId);

  // 如果是激活状态，使用统一的 hoverZ (默认20px)
  // 如果不是，使用各自原本的 defaultZ (如30px, 60px) 保持静止时的立体感
  const zValue = isActive ? hoverZ : defaultZ;

  return {
    transform: `translateZ(${zValue}px)`,
    // 添加 transition 确保 Z 轴变化平滑，不会突变
    transition: 'transform 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94)'
  };
};

const handleCardClick = (activity) => {
  if (isMobile.value) {
    mobileActiveId.value = mobileActiveId.value === activity.id ? null : activity.id;
  } else {
    openDetailModal(activity);
  }
};

const openDetailModal = (activity) => {
  selectedActivity.value = activity;
  document.body.style.overflow = 'hidden';
};

const closeModal = () => {
  selectedActivity.value = null;
  document.body.style.overflow = '';
};

// 修改点：增强的 formatContent，支持 URL 自动转换为链接
const formatContent = (text) => {
  if (!text) return '';
  // 1. 转义 HTML (避免XSS，如果需要的话。这里简单处理，因为数据是静态的)
  let content = text.replace(/</g, "&lt;").replace(/>/g, "&gt;");

  // 2. 替换换行符为 <br>
  content = content.replace(/\n/g, '<br>');

  // 3. 识别 URL 并转换为 <a href>
  // 匹配 http:// 或 https:// 开头的链接
  const urlRegex = /(https?:\/\/[^\s<]+)/g;
  content = content.replace(urlRegex, (url) => {
    return `<a href="${url}" target="_blank" rel="noopener noreferrer" class="detail-link">${url}</a>`;
  });

  // 4. 简单的 Markdown 粗体支持 (**text**)
  content = content.replace(/\*\*(.*?)\*\*/g, '<strong class="detail-bold">$1</strong>');

  return content;
};

/* ---------- 断电/Canvas 逻辑 (保持不变) ---------- */
const wait = (ms) => new Promise((resolve) => {
  const id = setTimeout(() => resolve(), ms);
  flickerTimeouts.push(id);
});

const runFlickerSequence = async () => {
  if (isDestroyed) return;
  const flashesCount = Math.floor(Math.random() * 3) + 2;
  let longOffIndex = -1;
  if (flashesCount >= 3) longOffIndex = Math.floor(Math.random() * flashesCount);

  for (let i = 0; i < flashesCount; i++) {
    isPowerOff.value = true;
    const duration = i === longOffIndex ? randomRange(700, 1300) : randomRange(80, 200);
    await wait(duration);
    isPowerOff.value = false;
    if (i < flashesCount - 1) await wait(randomRange(60, 260));
  }
};

const startFlickerLoop = () => {
  const loop = async () => {
    if (isDestroyed) return;
    await wait(randomRange(5000, 9000));
    if (isDestroyed) return;
    await runFlickerSequence();
    if (!isDestroyed) loop();
  };
  loop();
};

// Canvas Trail
const trailCanvas = ref(null);
let ctx = null;
let particles = [];
let animationFrameId = null;
let canvasWidth = 0;
let canvasHeight = 0;
let lastTrailFrameTime = 0;
let lastSpawnTime = 0;
let pendingPointer = null;
let trailIdleSince = 0;

class Particle {
  constructor(x, y) {
    this.x = x;
    this.y = y;
    this.size = Math.random() * 3 + 1;
    this.speedX = Math.random() * 2 - 1;
    this.speedY = Math.random() * 2 - 1;
    this.life = 1.0;
    this.decay = 0.03 + Math.random() * 0.03;
    this.color = '#0ff';
  }
  update() {
    this.x += this.speedX;
    this.y += this.speedY;
    this.life -= this.decay;
    this.size *= 0.95;
  }
  draw(context) {
    context.fillStyle = this.color;
    context.globalAlpha = this.life * 0.5;
    context.beginPath();
    context.arc(this.x, this.y, this.size, 0, Math.PI * 2);
    context.fill();
  }
}

const getParticleLimit = () => (isMobile.value ? MAX_PARTICLES_MOBILE : MAX_PARTICLES_DESKTOP);

const handleInputEvent = (x, y) => {
  pendingPointer = { x, y };
  trailIdleSince = 0;
  startTrailAnimation();
};

const onMouseMove = (e) => handleInputEvent(e.clientX, e.clientY);
const onTouchMove = (e) => {
  if (!e.touches || e.touches.length === 0) return;
  const touch = e.touches[0];
  handleInputEvent(touch.clientX, touch.clientY);
};

const animateTrail = (now) => {
  if (!ctx || !trailCanvas.value || !isPageVisible) return;

  if (now - lastTrailFrameTime < TRAIL_FRAME_MS) {
    animationFrameId = requestAnimationFrame(animateTrail);
    return;
  }
  lastTrailFrameTime = now;

  if (pendingPointer && now - lastSpawnTime >= INPUT_SPAWN_INTERVAL_MS) {
    const limit = getParticleLimit();
    const spawnCount = Math.min(PARTICLES_PER_INPUT, Math.max(0, limit - particles.length));
    for (let i = 0; i < spawnCount; i++) particles.push(new Particle(pendingPointer.x, pendingPointer.y));
    pendingPointer = null;
    lastSpawnTime = now;
  }

  ctx.clearRect(0, 0, canvasWidth, canvasHeight);
  for (let i = 0; i < particles.length; i++) {
    particles[i].update();
    particles[i].draw(ctx);
    if (particles[i].life <= 0) {
      particles.splice(i, 1);
      i--;
    }
  }

  const limit = getParticleLimit();
  if (particles.length > limit) {
    particles.splice(0, particles.length - limit);
  }

  if (!pendingPointer && particles.length === 0) {
    if (!trailIdleSince) trailIdleSince = now;
    if (now - trailIdleSince >= TRAIL_IDLE_STOP_MS) {
      stopTrailAnimation();
      return;
    }
  } else {
    trailIdleSince = 0;
  }

  animationFrameId = requestAnimationFrame(animateTrail);
};

const resizeCanvas = () => {
  if (!trailCanvas.value) return;

  canvasWidth = window.innerWidth;
  canvasHeight = window.innerHeight;
  const dpr = Math.min(window.devicePixelRatio || 1, MAX_CANVAS_DPR);

  trailCanvas.value.style.width = `${canvasWidth}px`;
  trailCanvas.value.style.height = `${canvasHeight}px`;
  trailCanvas.value.width = Math.floor(canvasWidth * dpr);
  trailCanvas.value.height = Math.floor(canvasHeight * dpr);

  if (ctx) {
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }
};

const startTrailAnimation = () => {
  if (animationFrameId || !ctx || !trailCanvas.value || !isPageVisible) return;
  animationFrameId = requestAnimationFrame(animateTrail);
};

const stopTrailAnimation = () => {
  if (!animationFrameId) return;
  cancelAnimationFrame(animationFrameId);
  animationFrameId = null;
  trailIdleSince = 0;
};

const handleWindowResize = () => {
  const prevStarCount = getStarCount();
  checkMobile();
  if (getStarCount() !== prevStarCount) {
    initBackgroundFx();
  }
  resizeBgFxCanvas();
  resizeCanvas();
};

const handleVisibilityChange = () => {
  isPageVisible = !document.hidden;
  if (isPageVisible) {
    startBackgroundFx();
    startTrailAnimation();
  } else {
    stopBackgroundFx();
    stopTrailAnimation();
  }
};

onMounted(() => {
  if (store.activities.length === 0) {
    store.fetchActivities();
  }
  checkMobile();
  initCardObserver();
  initBackgroundFx();
  startFlickerLoop();
  isPageVisible = !document.hidden;

  window.addEventListener('resize', handleWindowResize);
  document.addEventListener('visibilitychange', handleVisibilityChange);
  if (bgFxCanvas.value) {
    bgFxCtx = bgFxCanvas.value.getContext('2d', { alpha: true, desynchronized: true }) || bgFxCanvas.value.getContext('2d');
    if (bgFxCtx) {
      resizeBgFxCanvas();
      startBackgroundFx();
    }
  }
  if (trailCanvas.value) {
    ctx = trailCanvas.value.getContext('2d', { alpha: true, desynchronized: true }) || trailCanvas.value.getContext('2d');
    if (ctx) {
      resizeCanvas();
      window.addEventListener('mousemove', onMouseMove, { passive: true });
      window.addEventListener('touchmove', onTouchMove, { passive: true });
      startTrailAnimation();
    }
  }
});

onUnmounted(() => {
  isDestroyed = true;
  teardownCardObserver();
  window.removeEventListener('resize', handleWindowResize);
  document.removeEventListener('visibilitychange', handleVisibilityChange);
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('touchmove', onTouchMove);
  stopBackgroundFx();
  stopTrailAnimation();
  starField = [];
  streakField = [];
  starSpriteCache.clear();
  streakSpriteCache.clear();
  starCoreSprite = null;
  particles = [];
  pendingPointer = null;
  flickerTimeouts.forEach((id) => clearTimeout(id));
});
</script>

<style scoped>
/* ================= 字体设置 ================= */
@font-face {
  font-family: "CyberHanzi";
  src: url("/fonts/CyberHanzi.ttf") format("truetype");
  font-weight: 400 900;
  font-style: normal;
  font-display: swap;
}

/* ================= 页面根 ================= */
.activity-page {
  min-height: 100vh;
  position: relative;
  overflow-x: hidden;
  background-color: #050505;
  color: #fff;
  font-family: "Noto Sans SC", sans-serif;
}

.activity-page ::selection {
  background: #f0f;
  color: #fff;
}

/* ================= 背景层 ================= */
.bg-layer-fixed {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  z-index: 0;
  pointer-events: none;
}

.bg-layer-black {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  background-color: #000;
}

.bg-layer-image {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  opacity: 0.4;
  animation: blur-pulse 8s ease-in-out infinite alternate;
}

.bg-layer-grid {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  opacity: 0.3;
  mix-blend-mode: overlay;
}

.bg-fx-canvas {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.bg-glow {
  position: absolute;
  width: 60vw;
  height: 60vw;
  border-radius: 9999px;
  filter: blur(120px);
  mix-blend-mode: screen;
}

.bg-glow--magenta {
  top: -10%;
  left: -10%;
  background: rgba(255, 0, 255, 0.2);
  animation: pulse-deep 6s ease-in-out infinite alternate;
}

.bg-glow--cyan {
  bottom: -10%;
  right: -10%;
  background: rgba(0, 255, 255, 0.2);
  animation: pulse-deep 6s ease-in-out infinite alternate;
  animation-delay: 1000ms;
}

/* ================= 拖尾画布 ================= */
.trail-canvas {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  z-index: 50;
  pointer-events: none;
  mix-blend-mode: screen;
}

/* ================= 页面主体 ================= */
.page-content {
  position: relative;
  z-index: 10;
  width: 100%;
  max-width: 1600px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1.5rem;
  padding-right: 1.5rem;
  padding-top: 6rem;
  padding-bottom: 8rem;
}

@media (min-width: 768px) {
  .page-content {
    padding-left: 3rem;
    padding-right: 3rem;
  }
}

/* ================= 页头 ================= */
.page-header {
  text-align: center;
  margin-bottom: 5rem;
  position: relative;
  user-select: none;
}

@media (min-width: 768px) {
  .page-header {
    margin-bottom: 8rem;
  }
}

.page-title {
  position: relative;
  display: inline-block;
  font-size: 3.75rem;
  font-weight: 900;
  font-style: italic;
  letter-spacing: -0.05em;
  line-height: 1;
}

@media (min-width: 640px) {
  .page-title {
    font-size: 6rem;
  }
}

@media (min-width: 1024px) {
  .page-title {
    font-size: 11rem;
  }
}

.subtitle-row {
  margin-top: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1.5rem;
}

.subtitle-line {
  height: 2px;
  width: 3rem;
  background-color: #0ff;
}

@media (min-width: 768px) {
  .subtitle-line {
    width: 6rem;
  }
}

.subtitle-text {
  color: #d1d5db;
  font-family: "Noto Sans SC", sans-serif;
  font-size: 0.75rem;
  letter-spacing: 0.3em;
  font-weight: 700;
  text-transform: uppercase;
  max-width: 80vw;
}

@media (min-width: 768px) {
  .subtitle-text {
    font-size: 1.125rem;
  }
}

/* ================= 活动网格 ================= */
.activity-grid {
  display: grid;
  grid-template-columns: 1fr;
  column-gap: 3rem;
  row-gap: 5rem;
  width: 100%;
}

@media (min-width: 1024px) {
  .activity-grid {
    grid-template-columns: repeat(2, 1fr);
    column-gap: 8rem;
    row-gap: 0;
  }
}

.activity-col {
  width: 100%;
  display: flex;
  justify-content: center;
}

@media (min-width: 1024px) {
  .activity-col--offset-top {
    margin-top: 8rem;
  }
  .activity-col--offset-bottom {
    margin-bottom: 8rem;
  }
}

/* ================= 3D 容器 ================= */
.tilt-container {
  transform-style: preserve-3d;
  transform-origin: center center;
  width: 100%;
  max-width: 600px;
}

.cyber-card {
  transform-style: preserve-3d;
  background: rgba(10, 10, 10, 0.4);
  position: relative;
  width: 100%;
  border-radius: 1.5rem;
  transition: transform 500ms ease-out, box-shadow 500ms ease-out, border-color 500ms ease-out;
}

/* ================= 卡片：霓虹边框 ================= */
.card-neon-border {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  border-radius: 1.5rem;
  z-index: 0;
  pointer-events: none;
  transition: transform 500ms ease-out;
}

.neon-glow-outer {
  position: absolute;
  top: -3px;
  right: -3px;
  bottom: -3px;
  left: -3px;
  border-radius: 1.5rem;
  background: linear-gradient(to right, #0ff, #f0f, #0ff);
  opacity: 0.5;
  filter: blur(10px);
}

.neon-glow-inner {
  position: absolute;
  top: -1px;
  right: -1px;
  bottom: -1px;
  left: -1px;
  border-radius: 1.5rem;
  background: linear-gradient(to right, #0ff, #f0f, #0ff);
  opacity: 0.8;
}

.neon-bg-fill {
  position: absolute;
  top: 1px;
  right: 1px;
  bottom: 1px;
  left: 1px;
  border-radius: 22px;
  background: rgba(5, 5, 5, 0.95);
  backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

/* ================= 卡片：图片层 ================= */
.card-image-wrap {
  position: relative;
  height: 12rem;
  margin-left: 0.5rem;
  margin-right: 0.5rem;
  margin-top: 0.5rem;
  border-radius: 1rem 1rem 0.375rem 0.375rem;
  overflow: hidden;
  background-color: #111827;
  z-index: 10;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
  transition: transform 500ms ease-out;
}

@media (min-width: 640px) {
  .card-image-wrap {
    height: 14rem;
  }
}

@media (min-width: 1024px) {
  .card-image-wrap {
    height: 16rem;
  }
}

.card-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0.9;
  transition: transform 700ms, opacity 700ms;
}

.group:hover .card-image {
  opacity: 1;
  transform: scale(1.1);
}

.card-image-noise {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  background-image: url('https://media.giphy.com/media/xTiTnxpQ3ghPiB2Hp6/giphy.gif');
  background-size: cover;
  opacity: 0.05;
  mix-blend-mode: overlay;
  pointer-events: none;
}

.card-image-gradient {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  background: linear-gradient(to top, #0a0a0a, transparent, transparent);
}

/* ================= 卡片：状态角标 ================= */
.card-badge-position {
  position: absolute;
  top: 1rem;
}

.card-badge-position--left {
  left: 1rem;
}

.card-badge-position--right {
  right: 1rem;
}

.card-badge {
  background: rgba(0, 0, 0, 0.9);
  border: 1px solid #0ff;
  color: #0ff;
  font-size: 0.75rem;
  font-weight: 900;
  padding: 0.25rem 0.75rem;
  box-shadow: 0 0 15px #0ff;
  letter-spacing: 0.1em;
}

@media (min-width: 768px) {
  .card-badge {
    font-size: 0.875rem;
  }
}

.card-badge--skew-right {
  transform: skewX(15deg);
}

.card-badge--skew-left {
  transform: skewX(-15deg);
}

/* ================= 卡片：信息层 ================= */
.card-info {
  position: relative;
  z-index: 20;
  display: flex;
  flex-direction: column;
  height: 100%;
  transition: transform 500ms ease-out;
  padding: 1.5rem;
}

@media (min-width: 640px) {
  .card-info {
    padding: 1.5rem 2.5rem 2rem 2.5rem;
  }
}

.card-info--right {
  text-align: left;
}

@media (min-width: 1024px) {
  .card-info--right {
    text-align: right;
  }
}

.card-info--left {
  text-align: left;
}

/* ================= 卡片标题 ================= */
.card-title {
  font-size: 1.5rem;
  font-weight: 900;
  font-style: italic;
  font-family: "Noto Sans SC", sans-serif;
  color: #fff;
  margin-bottom: 0.5rem;
  transition: color 300ms;
  filter: drop-shadow(0 10px 8px rgba(0, 0, 0, 0.04)) drop-shadow(0 4px 3px rgba(0, 0, 0, 0.1));
  letter-spacing: -0.025em;
  line-height: 1.25;
}

@media (min-width: 640px) {
  .card-title {
    font-size: 2.25rem;
  }
}

.group:hover .card-title {
  color: #0ff;
}

/* ================= 卡片介绍 ================= */
.card-intro {
  color: #9ca3af;
  font-size: 0.875rem;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin-bottom: 1rem;
  font-family: "Noto Sans SC", sans-serif;
  line-height: 1.625;
  letter-spacing: 0.025em;
  transition: color 300ms;
  border-color: #f0f;
}

@media (min-width: 640px) {
  .card-intro {
    font-size: 1rem;
  }
}

.group:hover .card-intro {
  color: #e5e7eb;
}

.card-intro--left-aligned {
  border-left: 2px solid;
  padding-left: 0.75rem;
}

.card-intro--right-aligned {
  border-left: 2px solid;
  padding-left: 0.75rem;
}

@media (min-width: 1024px) {
  .card-intro--right-aligned {
    border-left: 0;
    padding-left: 0;
    border-right: 2px solid;
    padding-right: 0.75rem;
  }
}

/* ================= 奖励栏 ================= */
.card-rewards {
  margin-top: auto;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding-top: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  transition: transform 500ms ease-out;
}

.card-rewards--event {
  gap: 0.625rem;
}

.reward-pool-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}

@media (min-width: 1024px) {
  .reward-pool-row--reverse {
    flex-direction: row-reverse;
  }
}

.reward-label {
  font-size: 10px;
  color: #6b7280;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

@media (min-width: 640px) {
  .reward-label {
    font-size: 0.75rem;
  }
}

.reward-value {
  font-size: 1.125rem;
  font-weight: 900;
  color: #f0f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  letter-spacing: -0.05em;
  filter: drop-shadow(0 0 5px #f0f);
}

.reward-value--event {
  color: #0ff;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  filter: drop-shadow(0 0 6px rgba(0, 255, 255, 0.9));
}

@media (min-width: 640px) {
  .reward-value {
    font-size: 1.5rem;
  }
}

.reward-action-row {
  position: relative;
  overflow: hidden;
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.2);
  padding: 0.75rem;
  border-radius: 0.25rem;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  transition: background-color 300ms;
}

@media (min-width: 1024px) {
  .reward-action-row--right {
    justify-content: flex-end;
  }
}

.group:hover .reward-action-row {
  background: rgba(0, 255, 255, 0.1);
}

.reward-action-row--event {
  background: rgba(255, 0, 255, 0.08);
  border-color: rgba(255, 0, 255, 0.28);
}

.group:hover .reward-action-row--event {
  background: rgba(255, 0, 255, 0.14);
}

.reward-action-indicator {
  width: 0.25rem;
  height: 100%;
  position: absolute;
  top: 0;
  background-color: #0ff;
}

.reward-action-indicator--left {
  left: 0;
}

.reward-action-indicator--right {
  left: 0;
}

@media (min-width: 1024px) {
  .reward-action-indicator--right {
    left: auto;
    right: 0;
  }
}

.reward-action-points {
  color: #0ff;
  font-weight: 900;
  font-size: 1.125rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.reward-action-points--event {
  color: #f5f5f5;
  font-size: 1rem;
  letter-spacing: 0.05em;
}

.reward-action-name {
  color: #9ca3af;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.reward-action-name--event {
  color: #d1d5db;
  letter-spacing: 0.08em;
}

/* ================= 移动端操作按钮 ================= */
.card-mobile-cta {
  margin-top: 1rem;
  overflow: hidden;
  transition: max-height 300ms ease, opacity 300ms ease;
}

@media (min-width: 768px) {
  .card-mobile-cta {
    display: none;
  }
}

.card-mobile-btn {
  width: 100%;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  background-color: #f0f;
  color: #fff;
  font-weight: 900;
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  box-shadow: 0 0 15px #f0f;
}

/* ================= 弹窗 ================= */
.modal-overlay {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

@media (min-width: 768px) {
  .modal-overlay {
    padding: 2rem;
  }
}

.modal-backdrop {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  background: rgba(5, 5, 5, 0.95);
  backdrop-filter: blur(24px);
}

.modal-container {
  position: relative;
  width: 100%;
  max-width: 64rem;
  background-color: #000;
  border: 1px solid #0ff;
  box-shadow: 0 0 50px rgba(0, 255, 255, 0.2);
  overflow: hidden;
  transform: translateZ(0);
  display: flex;
  flex-direction: column;
  max-height: 90vh;
  border-radius: 0.5rem;
}

.modal-top-bar {
  height: 0.25rem;
  background: linear-gradient(to right, #0ff, #f0f, #0ff);
  width: 100%;
  animation: gradient-slide 3s linear infinite;
  background-size: 200% 100%;
}

.modal-scroll-area {
  position: relative;
  z-index: 10;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  flex: 1;
}

.modal-hero {
  width: 100%;
  height: 12rem;
  position: relative;
  flex-shrink: 0;
}

@media (min-width: 768px) {
  .modal-hero {
    height: 20rem;
  }
}

.modal-hero-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  filter: grayscale(20%) contrast(1.25);
}

.modal-hero-overlay {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  background: rgba(0, 0, 0, 0.4);
}

.modal-hero-fade {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 6rem;
  background: linear-gradient(to top, #000, transparent);
}

.modal-body {
  width: 100%;
  padding: 1.5rem;
  background-color: #000;
  position: relative;
}

@media (min-width: 768px) {
  .modal-body {
    padding: 2.5rem;
  }
}

.modal-watermark {
  position: absolute;
  top: 1rem;
  right: 1rem;
  font-size: 6rem;
  font-weight: 900;
  color: rgba(255, 255, 255, 0.02);
  pointer-events: none;
  user-select: none;
  font-style: italic;
  line-height: 1;
}

@media (min-width: 768px) {
  .modal-watermark {
    top: 2.5rem;
    right: 2.5rem;
    font-size: 8rem;
  }
}

.modal-title {
  font-size: 1.5rem;
  font-weight: 900;
  font-style: italic;
  color: #fff;
  margin-bottom: 1.5rem;
  filter: drop-shadow(2px 2px 0 #f0f);
  position: relative;
  z-index: 10;
  line-height: 1.25;
}

@media (min-width: 768px) {
  .modal-title {
    font-size: 3rem;
  }
}

.modal-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-bottom: 2rem;
  font-size: 0.75rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-weight: 700;
}

.modal-tag {
  padding: 0.25rem 0.5rem;
}

.modal-tag--cyan {
  background: rgba(0, 255, 255, 0.1);
  color: #0ff;
  border: 1px solid rgba(0, 255, 255, 0.3);
}

.modal-tag--magenta {
  background: rgba(255, 0, 255, 0.1);
  color: #f0f;
  border: 1px solid rgba(255, 0, 255, 0.3);
}

/* prose classes are global; override max-width here */
.modal-prose {
  max-width: none;
  font-family: "Noto Sans SC", sans-serif;
  line-height: 1.625;
  position: relative;
  z-index: 10;
}

.modal-intro-text {
  font-weight: 700;
  font-size: 1.125rem;
  color: #fff;
  margin-bottom: 1rem;
  border-left: 4px solid #0ff;
  padding-left: 1rem;
}

@media (min-width: 768px) {
  .modal-intro-text {
    font-size: 1.25rem;
  }
}

.modal-detail-content {
  opacity: 0.8;
  font-size: 0.875rem;
  color: #d1d5db;
}

@media (min-width: 768px) {
  .modal-detail-content {
    font-size: 1rem;
  }
}

.modal-footer {
  margin-top: 2.5rem;
  padding-top: 1.5rem;
  border-top: 1px dashed #333;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-waiting-text {
  color: #f0f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-size: 0.75rem;
}

.modal-close-btn {
  padding: 0.5rem 1.5rem;
  background-color: #fff;
  color: #000;
  font-weight: 900;
  transition: background-color 300ms;
  transform: skewX(-10deg);
  font-size: 0.75rem;
  cursor: pointer;
}

@media (min-width: 768px) {
  .modal-close-btn {
    font-size: 0.875rem;
  }
}

.modal-close-btn:hover {
  background-color: #0ff;
}

.modal-close-btn-text {
  display: inline-block;
  transform: skewX(10deg);
}

/* ================= JS formatContent 中生成的类 (使用 :deep) ================= */
:deep(.detail-link) {
  color: #0ff;
  text-decoration: underline;
  text-decoration-thickness: 1px;
  text-underline-offset: 4px;
  transition: all 300ms;
  word-break: break-all;
}

:deep(.detail-link:hover) {
  color: #f0f;
  text-decoration-color: #f0f;
}

:deep(.detail-bold) {
  color: #fff;
  font-size: 1.125rem;
}

/* ================= 标题赛博朋克特效（重构） ================= */

/* 包裹层：负责 TV 扫描线 / 噪点 等整体叠加 */
.activity-title-wrap {
  position: relative;
  display: inline-block;
  padding: 0.5rem 1.5rem;
  isolation: isolate; /* 让混合模式只影响本区域 */
}

/* 细横向扫描线：TV 屏幕的"条纹感" */
.activity-title-wrap::before {
  content: "";
  position: absolute;
  inset: -25%;
  pointer-events: none;
  z-index: 0;
  opacity: 0.18;
  mix-blend-mode: soft-light;
  background-image:
    repeating-linear-gradient(
      to bottom,
      rgba(255, 255, 255, 0.12) 0px,
      rgba(255, 255, 255, 0.12) 1px,
      transparent 1px,
      transparent 3px
    );
  animation: scanlines-move 6s linear infinite;
}

/* 竖向扫描条：不一直存在，而是每隔一段时间扫一次 */
.activity-title-wrap::after {
  content: "";
  position: absolute;
  left: -8%;
  width: 116%;
  top: -40%;
  height: 180%;
  pointer-events: none;
  z-index: 1;
  background: linear-gradient(
    to bottom,
    transparent 0%,
    rgba(255, 255, 255, 0.0) 35%,
    rgba(180, 255, 255, 0.55) 50%,
    rgba(255, 255, 255, 0.0) 65%,
    transparent 100%
  );
  mix-blend-mode: screen;
  opacity: 0;
  animation: title-scan 5s ease-in-out infinite;
}

/* 主文字：霓虹渐变 + 强外发光 + 轻微呼吸 */
.activity-title-wrap .cyber-glitch-title {
  font-family: "CyberHanzi", system-ui, sans-serif !important;
  position: relative;
  z-index: 2;
  white-space: nowrap;
  letter-spacing: 0.04em;

  background-image: linear-gradient(
    120deg,
    #ffe66b 0%,
    #fffdf3 35%,
    #6ff7ff 70%,
    #ffe66b 100%
  );
  background-size: 220% 220%;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent !important;

  text-shadow:
    0 0 8px rgba(255, 245, 200, 0.9),
    0 0 22px rgba(111, 247, 255, 0.95),
    0 0 38px rgba(0, 255, 255, 0.95);

  will-change: opacity, transform, text-shadow;
  animation:
    title-glow 4.5s ease-in-out infinite alternate,
    title-flicker-soft 3.2s linear infinite;
}

/* 红 / 青通道错位：左右轻微抖动造成"故障感" */
.activity-title-wrap .cyber-glitch-title::before,
.activity-title-wrap .cyber-glitch-title::after {
  content: attr(data-text);
  position: absolute;
  inset: 0;
  pointer-events: none;
  mix-blend-mode: screen;
}

/* 红通道 */
.activity-title-wrap .cyber-glitch-title::before {
  color: rgba(255, 80, 80, 0.85);
  text-shadow: -2px 0 6px rgba(255, 0, 0, 0.9);
  animation: glitch-red 2.4s infinite steps(1, end);
}

/* 青通道 */
.activity-title-wrap .cyber-glitch-title::after {
  color: rgba(80, 200, 255, 0.95);
  text-shadow: 2px 0 6px rgba(0, 255, 255, 0.9);
  animation: glitch-cyan 3.1s infinite steps(1, end);
}

/* JS 控制的"断电"状态：亮度骤降 + 故障通道关闭 */
.title-power-off {
  opacity: 0.22 !important;
  filter: grayscale(1) brightness(0.5);
  text-shadow: none !important;
  animation: none !important;
}
.title-power-off::before,
.title-power-off::after {
  opacity: 0 !important;
}

/* ================= 动画定义 ================= */

/* 自定义动画：animate-blur-pulse */
@keyframes blur-pulse {
  0% {
    filter: blur(0px);
    opacity: 0.4;
  }
  100% {
    filter: blur(4px);
    opacity: 0.3;
  }
}

/* 自定义动画：animate-pulse (模拟 Tailwind pulse) */
.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

/* 自定义动画：animate-pulse-deep */
@keyframes pulse-deep {
  0% {
    opacity: 0.2;
    transform: scale(1);
  }
  50% {
    opacity: 0.35;
    transform: scale(1.05);
  }
  100% {
    opacity: 0.2;
    transform: scale(1);
  }
}

/* 卡片轻微上下浮动（保留原来的） */
@keyframes cyber-float-y {
  0% { transform: translateY(0px); }
  100% { transform: translateY(-15px); }
}

/* 卡片边框渐变 */
@keyframes gradient-border {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
}
.animate-gradient-border {
  background-size: 200% 200%;
  animation: gradient-border 4s linear infinite;
}

/* 渐变滑动（弹窗顶部线条） */
@keyframes gradient-slide {
  0% { background-position: 0% 50%; }
  100% { background-position: 200% 50%; }
}

/* TV 扫描线的缓慢滚动 */
@keyframes scanlines-move {
  0% { background-position: 0 0; }
  100% { background-position: 0 4px; }
}

/* 竖向扫描条扫过一次 */
@keyframes title-scan {
  0%, 60%, 100% {
    opacity: 0;
    transform: translateY(-120%);
  }
  70% {
    opacity: 0.85;
    transform: translateY(-20%);
  }
  80% {
    opacity: 0;
    transform: translateY(140%);
  }
}

/* 文字发光的"呼吸" */
@keyframes title-glow {
  0% {
    text-shadow:
      0 0 6px rgba(255, 245, 200, 0.7),
      0 0 18px rgba(111, 247, 255, 0.75),
      0 0 26px rgba(0, 255, 255, 0.8);
    transform: translateZ(0) scale(1);
  }
  100% {
    text-shadow:
      0 0 10px rgba(255, 255, 210, 1),
      0 0 30px rgba(111, 247, 255, 1),
      0 0 52px rgba(0, 255, 255, 1);
    transform: translateZ(15px) scale(1.01);
  }
}

/* 轻微亮度抖动：细碎闪烁，不是完全断电 */
@keyframes title-flicker-soft {
  0%, 6%, 8%, 100% { opacity: 1; }
  7% { opacity: 0.85; }

  30% { opacity: 0.96; }
  31% { opacity: 0.72; }
  32% { opacity: 1; }

  60% { opacity: 0.94; }
  61% { opacity: 0.65; }
  62% { opacity: 1; }
}

/* 红色通道 glitch：上下几段随机错位 + 裁剪 */
@keyframes glitch-red {
  0%, 14%, 100% {
    transform: translate(0, 0);
    clip-path: inset(0 0 0 0);
    opacity: 0.4;
  }
  15% {
    transform: translate(-2px, -1px);
    clip-path: inset(0 0 60% 0);
    opacity: 0.95;
  }
  16% {
    transform: translate(1px, 1px);
    clip-path: inset(40% 0 0 0);
    opacity: 0.7;
  }
  50% {
    transform: translate(-1px, 0);
    clip-path: inset(12% 0 65% 0);
    opacity: 0.85;
  }
  51% {
    transform: translate(0, 0);
    clip-path: inset(0 0 0 0);
    opacity: 0.4;
  }
}

/* 青色通道 glitch：另一组错位时间点 */
@keyframes glitch-cyan {
  0%, 24%, 100% {
    transform: translate(0, 0);
    clip-path: inset(0 0 0 0);
    opacity: 0.5;
  }
  25% {
    transform: translate(2px, 1px);
    clip-path: inset(55% 0 0 0);
    opacity: 1;
  }
  26% {
    transform: translate(-1px, -1px);
    clip-path: inset(0 0 45% 0);
    opacity: 0.8;
  }
  70% {
    transform: translate(1px, 0);
    clip-path: inset(20% 0 50% 0);
    opacity: 0.9;
  }
  71% {
    transform: translate(0, 0);
    clip-path: inset(0 0 0 0);
    opacity: 0.5;
  }
}

/* ================= 辅助类 ================= */
.clip-path-button {
  clip-path: polygon(10px 0, 100% 0, 100% calc(100% - 10px), calc(100% - 10px) 100%, 0 100%, 0 10px);
}

.mobile-active {
  transform: scale(1.02);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.2);
  z-index: 50;
  border: 1px solid #0ff;
}

/* 滚动条 */
.custom-scrollbar::-webkit-scrollbar { width: 6px; }
.custom-scrollbar::-webkit-scrollbar-track { background: #000; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #333; border-radius: 3px; }
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: #0ff; }

/* ================= 分页控件 ================= */
.cyber-pagination {
  margin-top: 4rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  font-family: "JetBrains Mono", "Consolas", monospace;
  user-select: none;
}

.cyber-page-btn {
  position: relative;
  min-width: 2.75rem;
  height: 2.5rem;
  padding: 0 0.85rem;
  background: rgba(10, 10, 15, 0.7);
  color: #d4faff;
  border: 1px solid rgba(0, 255, 255, 0.35);
  font-size: 0.85rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  cursor: pointer;
  transition: color 0.2s ease, border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease, transform 0.15s ease;
  clip-path: polygon(8px 0, 100% 0, 100% calc(100% - 8px), calc(100% - 8px) 100%, 0 100%, 0 8px);
}

.cyber-page-btn:hover:not(:disabled) {
  color: #000;
  background: #0ff;
  border-color: #0ff;
  box-shadow: 0 0 18px rgba(0, 255, 255, 0.55);
  transform: translateY(-1px);
}

.cyber-page-btn.is-active {
  color: #000;
  background: linear-gradient(135deg, #0ff 0%, #f0f 100%);
  border-color: #f0f;
  box-shadow: 0 0 20px rgba(255, 0, 255, 0.55);
}

.cyber-page-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.cyber-page-ellipsis {
  padding: 0 0.25rem;
  color: rgba(0, 255, 255, 0.55);
  letter-spacing: 0.1em;
}

.cyber-page-meta {
  margin-left: 0.75rem;
  padding: 0.35rem 0.75rem;
  border: 1px dashed rgba(255, 0, 255, 0.4);
  color: #ff9bff;
  font-size: 0.75rem;
  letter-spacing: 0.12em;
}

@media (max-width: 640px) {
  .cyber-pagination {
    margin-top: 2.5rem;
    gap: 0.35rem;
  }
  .cyber-page-btn {
    min-width: 2.25rem;
    height: 2.25rem;
    padding: 0 0.5rem;
    font-size: 0.75rem;
  }
  .cyber-page-meta {
    width: 100%;
    text-align: center;
    margin: 0.25rem 0 0;
  }
}

/* ================= 移动端性能优化 ================= */
/* 小屏设备上弱化或关闭部分动画，减轻开销 */
@media (max-width: 768px) {
  .activity-title-wrap::before,
  .activity-title-wrap::after {
    opacity: 0.08;
    animation-duration: 10s;
  }

  .activity-title-wrap .cyber-glitch-title {
    /* 只保留呼吸感，去掉高频 flicker，降低 GPU 压力 */
    animation: title-glow 6s ease-in-out infinite alternate;
    text-shadow:
      0 0 6px rgba(255, 245, 200, 0.7),
      0 0 18px rgba(111, 247, 255, 0.75),
      0 0 26px rgba(0, 255, 255, 0.8);
  }

  /* 关闭红/青 glitch 通道，避免频繁 clip-path / steps 动画 */
  .activity-title-wrap .cyber-glitch-title::before,
  .activity-title-wrap .cyber-glitch-title::after {
    animation: none;
    opacity: 0;
  }
}

/* 若系统开启"减少动态效果"，则整体弱化动画 */
@media (prefers-reduced-motion: reduce) {
  .activity-title-wrap::before,
  .activity-title-wrap::after {
    animation: none;
  }
  .activity-title-wrap .cyber-glitch-title {
    animation: none;
  }
  .activity-title-wrap .cyber-glitch-title::before,
  .activity-title-wrap .cyber-glitch-title::after {
    animation: none;
    opacity: 0;
  }
}
</style>
