<template>
  <div class="store-page">

    <!-- ================= 背景层设计 ================= -->
    <div class="bg-layer">
        <!-- 0. 基础底色 -->
        <div class="bg-base"></div>

        <!-- 1. 背景底图 -->
        <!-- 电脑端 -->
        <div
            class="bg-image bg-image--desktop"
            style="background-image: url('ok.webp'); filter: blur(2px) brightness(0.9) contrast(1.1);"
        ></div>
        <!-- 手机端 -->
        <div
            class="bg-image bg-image--mobile"
            style="background-image: url('ko3.webp'); filter: blur(1px) brightness(0.8);"
        ></div>

        <!-- 1.5 氛围渐变遮罩 -->
        <div class="bg-gradient-overlay"></div>

        <!-- 2. 动态呼吸光斑 -->
        <div class="glow-orb glow-orb--purple animate-pulse-slow"></div>
        <div class="glow-orb glow-orb--blue animate-pulse-slow delay-1000"></div>
        <!-- 中间的高光点缀 -->
        <div class="glow-orb glow-orb--pink animate-float"></div>

        <!-- 3. 全局噪点纹理 -->
        <div class="noise-texture" style="background-image: url('data:image/svg+xml,%3Csvg viewBox=%220 0 200 200%22 xmlns=%22http://www.w3.org/2000/svg%22%3E%3Cfilter id=%22noiseFilter%22%3E%3CfeTurbulence type=%22fractalNoise%22 baseFrequency=%220.65%22 numOctaves=%223%22 stitchTiles=%22stitch%22/%3E%3C/filter%3E%3Crect width=%22100%25%22 height=%22100%25%22 filter=%22url(%23noiseFilter)%22/%3E%3C/svg%3E');"></div>
    </div>

    <!-- 顶部占位 -->
    <div class="top-spacer"></div>

    <!-- 页面主体 -->
    <div class="page-content">

        <!-- Header -->
        <div class="page-header animate-slide-in">
            <!-- 标题区 -->
            <div class="header-title-area">
                <div class="header-title-row">
                    <h1 class="page-title">
                        积分奖品兑换
                    </h1>
                    <!-- 凉宫春日可爱贴纸 -->
                    <div class="sticker-wrapper animate-wiggle-flip">
                        <img src="https://img.remit.ee/api/file/BQACAgUAAyEGAASHRsPbAAEMN05pMsT1f7SH0ZuKr6pElHlz3jYGtwACdSUAAhSkmVUFXNg3Vl8qQzYE.png" alt="" class="sticker-img">
                    </div>
                </div>

                <p class="header-subtitle">
                    <span class="subtitle-line"></span>
                    <span>这是团长大人对忠诚团员的奖励哦~</span>
                </p>
                <!-- 赞助提示语 -->
                <p class="header-sponsor-tip">
                    <span class="sponsor-asterisk">*</span>
                    想要赞助奖品的话，随时私聊春日酱哦！
                </p>
            </div>
        </div>

        <!-- 积分查询搜索框 -->
        <div class="search-section animate-slide-in delay-75">
            <div class="search-wrapper group">
                <div class="search-glow"></div>

                <!-- 搜索框本体 -->
                <div class="search-box">
                    <input
                        v-model="searchId"
                        @input="handleInput"
                        @focus="handleFocus"
                        @blur="handleBlur"
                        type="text"
                        placeholder="输入自己的ID来查询积分吧。"
                        class="search-input"
                        @keyup.enter="handleSearch"
                        autocomplete="off"
                    >
                    <button
                        @click="handleSearch"
                        :disabled="isSearching"
                        class="search-btn"
                    >
                        <span v-if="isSearching">查询中...</span>
                        <span v-else>查询</span>
                        <svg v-if="!isSearching" xmlns="http://www.w3.org/2000/svg" class="search-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>
                    </button>
                </div>

                <!-- 自动补全下拉菜单 -->
                <Transition name="fade">
                    <div v-if="showSuggestions" class="suggestions-dropdown no-scrollbar">
                        <!-- 加载中 -->
                        <div v-if="isLoadingSuggestions" class="suggestion-loading">
                            <svg class="spinner" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                <circle class="spinner-track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="spinner-head" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            正在搜索...
                        </div>

                        <!-- 有结果 -->
                        <ul v-else-if="suggestions.length > 0">
                             <li
                                v-for="(suggestion, index) in suggestions"
                                :key="index"
                                @mousedown.prevent="selectSuggestion(suggestion)"
                                class="suggestion-item"
                            >
                                <span>{{ suggestion }}</span>
                                <span class="suggestion-select-hint">选择</span>
                            </li>
                        </ul>

                        <!-- 无结果 (仅当有输入内容但无结果时显示) -->
                        <div v-else-if="searchId && searchId.length > 0" class="suggestion-empty">
                            未找到匹配的用户ID
                        </div>
                    </div>
                </Transition>
            </div>
        </div>

        <!-- 过滤器 -->
        <div class="filter-bar animate-slide-in delay-100 no-scrollbar">
            <button
                v-for="cat in categories"
                :key="cat.id"
                @click="activeCategory = cat.id"
                class="filter-btn"
                :class="activeCategory === cat.id ? 'filter-btn--active' : 'filter-btn--inactive'"
            >
                <span class="filter-btn-label">{{ cat.name }}</span>
            </button>
        </div>

        <!-- 核心展示区：Bento Grid (完整商品列表) -->
        <div class="bento-grid">

            <div
                v-for="(item, index) in filteredItems"
                :key="item.id"
                class="store-card group"
                :class="getItemSpanClass(item.size, index)"
                @click="toggleItemActive(item.id)"
                @mousemove="handleMouseMove($event, index)"
                @mouseleave="handleMouseLeave(index)"
                :ref="el => itemRefs[index] = el"
            >
                <!-- [电脑端交互层] -->
                <div
                    class="card-click-layer"
                    @click.stop="openImageModal(item.image)"
                ></div>

                <!-- 卡片背景 -->
                <div class="card-bg"></div>

                <!-- 呼吸光效边框 -->
                <div class="card-hover-glow"></div>

                <!-- 商品图片 -->
                <div class="card-image-wrapper">
                    <img
                        :src="item.image"
                        class="card-image"
                        :style="item.imageStyle"
                        onerror="this.style.display='none'"
                        alt="商品图片"
                    >
                    <!-- 渐变遮罩 -->
                    <div class="card-image-overlay"></div>
                </div>

                <!-- 稀有度光晕 -->
                <div class="card-rarity">
                    <span class="rarity-label" :style="{ color: item.color, textShadow: `0 0 10px ${item.color}` }">{{ item.rarity }}</span>
                    <div class="rarity-line" :style="{ background: `linear-gradient(to right, transparent, ${item.color})` }"></div>
                </div>

                <!-- 装饰性角标 -->
                <div class="corner-mark corner-mark--top-left"></div>
                <div class="corner-mark corner-mark--bottom-right"></div>

                <!-- 信息内容区 -->
                <div class="card-info">

                    <div
                        class="card-info-inner"
                        :class="activeItemId === item.id ? 'card-info-inner--active' : 'card-info-inner--default'"
                    >
                        <h3 class="card-title">
                            {{ item.name }}
                        </h3>

                        <div class="card-meta">
                            <!-- 价格 -->
                            <div class="card-price">
                                <span class="price-value">{{ item.points }}</span>
                                <span class="price-unit">积分</span>
                            </div>

                            <!-- 库存点 -->
                            <div class="card-stock">
                                <span class="stock-dot" :class="item.stock < 5 ? 'stock-dot--low' : 'stock-dot--ok'"></span>
                                <span class="stock-text">{{ item.stock > 0 ? `剩余数量:${item.stock}` : 'SOLD' }}</span>
                            </div>
                        </div>
                    </div>

                    <!-- 悬浮/点击后展现 -->
                    <div
                        class="card-detail"
                        :class="activeItemId === item.id ? 'card-detail--active' : 'card-detail--default'"
                    >
                        <p class="card-description">
                            {{ item.description }}
                        </p>

                        <!-- 查看大图按钮 (仅手机端显示) -->
                        <div class="card-view-image-mobile">
                            <button
                                @click.stop="openImageModal(item.image)"
                                class="view-image-btn"
                            >
                                查看大图 <span class="view-image-arrow">↗</span>
                            </button>
                        </div>

                        <!-- 兑换按钮 -->
                        <button
                            @click.stop="redeemItem(item)"
                            :disabled="item.stock <= 0"
                            class="redeem-btn clip-path-button"
                        >
                            <span class="redeem-btn-text">{{ item.stock <= 0 ? '缺货' : '兑换请私聊春日酱~' }}</span>
                        </button>
                    </div>
                </div>

            </div>

        </div>

        <!-- 底部加载更多 -->
        <div class="page-footer">
            <div class="footer-content">
                <div class="footer-line"></div>
                <span class="footer-text">End of Archive</span>
            </div>
        </div>

    </div>

    <!-- 积分详情查询结果弹窗 (已对接后端) -->
    <Transition name="fade">
        <div v-if="showPointsModal" class="modal-overlay" @click="closePointsModal">
            <div class="points-modal" @click.stop>
                <!-- 关闭按钮 -->
                <button @click="closePointsModal" class="modal-close-btn">
                    <svg xmlns="http://www.w3.org/2000/svg" class="modal-close-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>

                <!-- 内容区 -->
                <div class="points-modal-content">
                    <div class="points-user-id">用户：{{ pointsResult.nickname || searchId }}</div>
                    <div class="points-total">
                        {{ pointsResult.total }}
                    </div>

                    <div class="points-history-section">
                        <div class="points-history-header">
                            <span>变动时间</span>
                            <span>原因</span>
                            <span>额度</span>
                        </div>
                        <ul class="points-history-list no-scrollbar">
                            <li v-for="(record, idx) in pointsResult.history" :key="idx" class="points-history-item">
                                <span class="history-date">{{ record.date }}</span>
                                <span class="history-reason" :title="record.reason">{{ record.reason }}</span>
                                <span :class="record.change.startsWith('+') ? 'history-change--positive' : 'history-change--negative'" class="history-change">{{ record.change }}</span>
                            </li>
                        </ul>
                        <div v-if="pointsResult.history.length === 0" class="points-empty">暂无积分变动记录</div>
                    </div>
                </div>
            </div>
        </div>
    </Transition>

    <!-- 大图预览弹窗 -->
    <Transition name="fade">
      <div v-if="showImageModal" class="modal-overlay" @click="closeImageModal">
        <button class="image-modal-close">
          <svg xmlns="http://www.w3.org/2000/svg" class="image-modal-close-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
        <div class="image-modal-content" @click.stop>
          <img :src="selectedImage" alt="大图预览" class="image-modal-img" />
        </div>
      </div>
    </Transition>

    <!-- 全局兑换成功弹窗 -->
    <Transition name="pop-in">
        <div v-if="showSuccessModal" class="success-modal-overlay">
            <div class="success-modal">
                <div class="success-modal-topline"></div>
                <div class="success-modal-glow"></div>
                <div class="success-modal-icon">
                    <svg xmlns="http://www.w3.org/2000/svg" class="modal-close-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg>
                </div>
                <h3 class="success-modal-title">NOTICE</h3>
                <p class="success-modal-text">都说了去找春日酱，怎么还点，笨蛋笨蛋！</p>
                <button @click="showSuccessModal = false" class="success-modal-close-btn">Close</button>
            </div>
        </div>
    </Transition>

  </div>
</template>

<script setup>
import { ref, computed, reactive, onMounted } from 'vue';
import { useMainStore } from '@/stores/main';

const store = useMainStore();
const activeCategory = ref('all');
const showSuccessModal = ref(false);
const itemRefs = reactive({});

// 激活的卡片ID，用于移动端点击显示/隐藏详情
const activeItemId = ref(null);

// 大图预览相关状态
const showImageModal = ref(false);
const selectedImage = ref('');

// 积分查询相关状态
const searchId = ref('');
const isSearching = ref(false);
const showPointsModal = ref(false);
const pointsResult = reactive({
    total: 0,
    nickname: '',
    history: []
});

// 自动补全建议状态
const suggestions = ref([]);
const showSuggestions = ref(false);
const isLoadingSuggestions = ref(false);

const categories = [
    { id: 'all', name: '全部奖品' },
    { id: 'anime', name: '动漫周边' },
    { id: 'paint', name: '绘画用具' },
    { id: 'computer', name: '电脑配件' },
    { id: 'book', name: '书籍' },
    { id: 'peripheral', name: '外设' },
    { id: 'instrument', name: '乐器' },
    { id: 'game', name: '游戏' },
];

// 修改点 1: 移除硬编码的 items 数组，改为从 Store 获取
const items = computed(() => store.prizes);

// 过滤逻辑
const filteredItems = computed(() => {
    if (activeCategory.value === 'all') return items.value;
    return items.value.filter(i => i.category === activeCategory.value);
});

// Grid Span 类名生成
const getItemSpanClass = (size, index) => {
    switch (size) {
        case 'large': return 'span-large';
        case 'wide': return 'span-wide';
        case 'tall': return 'span-tall';
        default: return '';
    }
};

const handleMouseMove = (e, index) => {
    // 预留视差逻辑
};

const handleMouseLeave = (index) => {
    // 预留重置逻辑
};

// 点击卡片切换激活状态（用于移动端显示详情）
const toggleItemActive = (id) => {
    // 只有在移动端（<768px）时才启用点击切换详情逻辑
    if (window.innerWidth >= 768) return;

    if (activeItemId.value === id) {
        activeItemId.value = null; // 再次点击关闭
    } else {
        activeItemId.value = id;
    }
};

const redeemItem = (item) => {
    showSuccessModal.value = true;
};

// 打开大图弹窗
const openImageModal = (imageUrl) => {
  selectedImage.value = imageUrl;
  showImageModal.value = true;
  document.body.style.overflow = 'hidden'; // 防止背景滚动
};

// 关闭大图弹窗
const closeImageModal = () => {
  showImageModal.value = false;
  selectedImage.value = '';
  document.body.style.overflow = ''; // 恢复背景滚动
};

// ---------------- 搜索建议逻辑 ----------------

// 简单的防抖函数
const debounce = (fn, delay) => {
    let timeoutId;
    return (...args) => {
        if (timeoutId) clearTimeout(timeoutId);
        timeoutId = setTimeout(() => {
            fn(...args);
        }, delay);
    };
};

// 执行搜索建议
const fetchSuggestions = async () => {
    if (!searchId.value || searchId.value.trim().length < 1) {
        suggestions.value = [];
        isLoadingSuggestions.value = false;
        return;
    }

    isLoadingSuggestions.value = true;

    // 安全调用：检查 store.searchUsers 是否存在
    if (typeof store.searchUsers !== 'function') {
        console.warn('store.searchUsers action is missing. Please update src/stores/main.js');
        suggestions.value = [];
        isLoadingSuggestions.value = false;
        return;
    }

    try {
        const results = await store.searchUsers(searchId.value);
        suggestions.value = Array.isArray(results) ? results : [];
    } catch (error) {
        console.error("Search failed:", error);
        suggestions.value = [];
    } finally {
        isLoadingSuggestions.value = false;
    }
};

// 防抖后的输入处理
const debouncedFetch = debounce(fetchSuggestions, 300);

const handleInput = () => {
    // 输入时立即显示（此时可能显示"正在搜索"或者上次结果）
    showSuggestions.value = true;
    debouncedFetch();
};

const handleFocus = () => {
    // 聚焦时如果框里有字，也触发一下
    if (searchId.value) {
        showSuggestions.value = true;
        debouncedFetch();
    }
};

const handleBlur = () => {
    // 延迟隐藏，以便点击事件能被捕获
    setTimeout(() => {
        showSuggestions.value = false;
    }, 200);
};

const selectSuggestion = (suggestion) => {
    searchId.value = suggestion;
    showSuggestions.value = false;
    // 选择后直接触发查询
    handleSearch();
};

// ---------------- 真实积分查询逻辑 ----------------

const handleSearch = async () => {
    showSuggestions.value = false; // 查询时关闭建议
    if (!searchId.value.trim()) {
        alert("请输入ID");
        return;
    }
    isSearching.value = true;
    try {
        const data = await store.fetchUserPoints(searchId.value);
        if (data) {
            pointsResult.total = data.total;
            pointsResult.nickname = data.nickname || '';
            pointsResult.history = data.history;
            showPointsModal.value = true;
        } else {
            alert('未找到该用户的积分记录，或网络连接失败');
        }
    } catch (e) {
        console.error(e);
        alert('查询出错');
    } finally {
        isSearching.value = false;
    }
};

const closePointsModal = () => {
    showPointsModal.value = false;
};

// 修改点 2: 在组件挂载时从后端获取奖品数据
onMounted(() => {
    store.fetchPrizes();
});
</script>

<style scoped>
/* ========================================
   Selection
   ======================================== */
.store-page ::selection {
    background: #a855f7;
    color: var(--sos-bg-surface);
}

/* ========================================
   Page Root
   ======================================== */
.store-page {
    min-height: 100vh;
    padding-bottom: 5rem; /* pb-20 */
    position: relative;
    overflow-x: hidden;
    background-color: #050505;
}

/* ========================================
   Background Layer
   ======================================== */
.bg-layer {
    position: fixed;
    top: 0; right: 0; bottom: 0; left: 0;
    z-index: 0;
    pointer-events: none;
}

.bg-base {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    background-color: var(--sos-text-primary);
}

.bg-image {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
    opacity: 0.6;
}

.bg-image--desktop {
    display: none;
}

.bg-image--mobile {
    display: block;
}

@media (min-width: 768px) {
    .bg-image--desktop {
        display: block;
    }
    .bg-image--mobile {
        display: none;
    }
}

.bg-gradient-overlay {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    background: linear-gradient(to bottom, rgba(0,0,0,0.6), rgba(88,28,135,0.1), rgba(0,0,0,0.9));
}

/* Glow orbs */
.glow-orb {
    position: absolute;
    border-radius: 9999px;
}

.glow-orb--purple {
    top: -20%;
    left: -10%;
    width: 80vw;
    height: 80vw;
    background: rgba(147, 51, 234, 0.2);
    filter: blur(100px);
}

.glow-orb--blue {
    bottom: -20%;
    right: -10%;
    width: 80vw;
    height: 80vw;
    background: rgba(37, 99, 235, 0.15);
    filter: blur(100px);
}

.glow-orb--pink {
    top: 30%;
    left: 20%;
    width: 40vw;
    height: 40vw;
    background: rgba(236, 72, 153, 0.1);
    filter: blur(120px);
}

.noise-texture {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    opacity: 0.04;
}

/* ========================================
   Top Spacer
   ======================================== */
.top-spacer {
    height: 6rem; /* h-24 */
}

@media (min-width: 768px) {
    .top-spacer {
        height: 8rem; /* md:h-32 */
    }
}

/* ========================================
   Page Content
   ======================================== */
.page-content {
    position: relative;
    z-index: 10;
    max-width: 1400px;
    margin-left: auto;
    margin-right: auto;
    padding-left: 1rem;
    padding-right: 1rem;
}

@media (min-width: 768px) {
    .page-content {
        padding-left: 2rem;
        padding-right: 2rem;
    }
}

/* ========================================
   Page Header
   ======================================== */
.page-header {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    gap: 1.5rem;
    text-align: center;
}

@media (min-width: 768px) {
    .page-header {
        flex-direction: row;
        align-items: flex-end;
        text-align: left;
    }
}

.header-title-area {
    position: relative;
    z-index: 10;
    width: 100%;
}

@media (min-width: 768px) {
    .header-title-area {
        width: auto;
    }
}

.header-title-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    margin-bottom: 1rem;
}

@media (min-width: 768px) {
    .header-title-row {
        justify-content: flex-start;
    }
}

.page-title {
    font-size: 3rem; /* text-5xl */
    line-height: 1;
    font-weight: 900;
    font-family: "Noto Serif SC", serif;
    letter-spacing: -0.025em;
    color: transparent;
    background-image: linear-gradient(to bottom, var(--sos-bg-surface), #ede9fe, var(--sos-text-tertiary));
    -webkit-background-clip: text;
    background-clip: text;
    filter: drop-shadow(0 0 25px rgba(168,85,247,0.3));
}

@media (min-width: 768px) {
    .page-title {
        font-size: 6rem; /* text-8xl */
    }
}

.sticker-wrapper {
    width: 3rem;
    height: 3rem;
    flex-shrink: 0;
    transform-origin: bottom;
}

@media (min-width: 768px) {
    .sticker-wrapper {
        width: 5rem;
        height: 5rem;
    }
}

.sticker-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    filter: drop-shadow(0 0 10px rgba(255,100,100,0.5));
}

.header-subtitle {
    color: var(--sos-border-strong);
    font-family: "Noto Serif SC", serif;
    font-style: italic;
    font-size: 1.125rem;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
}

@media (min-width: 768px) {
    .header-subtitle {
        font-size: 1.25rem;
        justify-content: flex-start;
    }
}

.subtitle-line {
    display: inline-block;
    width: 3rem;
    height: 2px;
    background: linear-gradient(to right, #a855f7, transparent);
}

.header-sponsor-tip {
    color: var(--sos-border-strong);
    font-size: 0.75rem;
    margin-top: 0.75rem;
    font-family: "Noto Sans SC", sans-serif;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
}

@media (min-width: 768px) {
    .header-sponsor-tip {
        font-size: 0.875rem;
        justify-content: flex-start;
        padding-left: 0.25rem;
    }
}

.sponsor-asterisk {
    color: #c084fc;
}

/* ========================================
   Search Section
   ======================================== */
.search-section {
    display: flex;
    justify-content: center;
    margin-bottom: 2.5rem;
    position: relative;
    z-index: 30;
}

@media (min-width: 768px) {
    .search-section {
        justify-content: flex-start;
    }
}

.search-wrapper {
    position: relative;
    width: 100%;
    max-width: 28rem; /* max-w-md */
}

.search-glow {
    position: absolute;
    inset: -0.125rem;
    background: linear-gradient(to right, #9333ea, #2563eb);
    border-radius: 0.5rem;
    filter: blur(4px);
    opacity: 0.3;
    transition: opacity 0.5s;
}

.search-wrapper:hover .search-glow {
    opacity: 0.7;
}

.search-box {
    position: relative;
    display: flex;
    align-items: center;
    background-color: var(--sos-text-primary);
    border-radius: 0.5rem;
    padding: 0.25rem;
    border: 1px solid rgba(255,255,255,0.1);
    z-index: 20;
}

.search-input {
    width: 100%;
    background: transparent;
    font-size: 0.875rem;
    color: var(--sos-bg-surface);
    padding: 0.5rem 1rem;
    outline: none;
}

.search-input::placeholder {
    color: var(--sos-text-secondary);
}

@media (min-width: 768px) {
    .search-input {
        font-size: 1rem;
    }
}

.search-btn {
    background: rgba(255,255,255,0.1);
    color: var(--sos-bg-surface);
    padding: 0.5rem 1.5rem;
    border-radius: 0.375rem;
    font-weight: 700;
    font-size: 0.875rem;
    transition: background-color 0.2s, color 0.2s;
    border: 1px solid rgba(255,255,255,0.1);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    white-space: nowrap;
    cursor: pointer;
}

.search-btn:hover {
    background: rgba(255,255,255,0.2);
}

.search-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.search-icon {
    width: 1rem;
    height: 1rem;
}

/* ========================================
   Suggestions Dropdown
   ======================================== */
.suggestions-dropdown {
    position: absolute;
    left: 0;
    right: 0;
    top: 100%;
    margin-top: 0.5rem;
    background: rgba(17,17,17,0.95);
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 0.5rem;
    box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
    z-index: 50;
    overflow: hidden;
    max-height: 15rem;
    overflow-y: auto;
}

.suggestion-loading {
    padding: 0.75rem 1rem;
    font-size: 0.875rem;
    color: var(--sos-text-secondary);
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.spinner {
    animation: spin 1s linear infinite;
    height: 1rem;
    width: 1rem;
    color: #a855f7;
}

.spinner-track {
    opacity: 0.25;
}

.spinner-head {
    opacity: 0.75;
}

@keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

.suggestion-item {
    padding: 0.75rem 1rem;
    font-size: 0.875rem;
    color: var(--sos-border-strong);
    cursor: pointer;
    transition: background-color 0.2s, color 0.2s;
    border-bottom: 1px solid rgba(255,255,255,0.05);
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.suggestion-item:last-child {
    border-bottom: 0;
}

.suggestion-item:hover {
    background: rgba(147,51,234,0.2);
    color: var(--sos-bg-surface);
}

.suggestion-select-hint {
    font-size: 10px;
    color: var(--sos-text-secondary);
    opacity: 0;
    transition: opacity 0.2s, color 0.2s;
}

.suggestion-item:hover .suggestion-select-hint {
    color: #c084fc;
    opacity: 1;
}

.suggestion-empty {
    padding: 0.75rem 1rem;
    font-size: 0.875rem;
    color: var(--sos-text-tertiary);
    font-style: italic;
    text-align: center;
}

/* ========================================
   Filter Bar
   ======================================== */
.filter-bar {
    display: flex;
    gap: 1rem;
    margin-bottom: 3rem;
    overflow-x: auto;
    padding-bottom: 1rem;
    position: relative;
    z-index: 20;
}

.filter-btn {
    padding: 0.625rem 1.5rem;
    border-radius: 9999px;
    font-size: 0.875rem;
    font-weight: 700;
    border: 1px solid transparent;
    transition: all 0.5s;
    white-space: nowrap;
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    cursor: pointer;
}

.filter-btn--active {
    background: rgba(168,85,247,0.2);
    border-color: #c084fc;
    color: var(--sos-bg-surface);
    box-shadow: 0 0 20px rgba(168,85,247,0.4);
}

.filter-btn--inactive {
    border-color: rgba(255,255,255,0.05);
    background: rgba(0,0,0,0.2);
    color: var(--sos-text-tertiary);
}

.filter-btn--inactive:hover {
    color: var(--sos-bg-surface);
    border-color: rgba(168,85,247,0.5);
    background: rgba(168,85,247,0.1);
}

.filter-btn-label {
    position: relative;
    z-index: 10;
}

/* ========================================
   Bento Grid
   ======================================== */
.bento-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1.5rem;
    grid-auto-rows: 320px;
    grid-auto-flow: row dense;
    padding-bottom: 5rem;
    position: relative;
    z-index: 20;
}

@media (min-width: 768px) {
    .bento-grid {
        grid-template-columns: repeat(2, 1fr);
    }
}

@media (min-width: 1024px) {
    .bento-grid {
        grid-template-columns: repeat(4, 1fr);
    }
}

/* Grid span classes (replaces getItemSpanClass Tailwind output) */
@media (min-width: 768px) {
    .span-large {
        grid-column: span 2;
        grid-row: span 2;
    }
    .span-wide {
        grid-column: span 2;
        grid-row: span 1;
    }
    .span-tall {
        grid-column: span 1;
        grid-row: span 2;
    }
}

/* ========================================
   Store Card
   ======================================== */
.store-card {
    position: relative;
    overflow: hidden;
    border-radius: 0.75rem;
    border: 1px solid rgba(255,255,255,0.1);
    transition: all 0.5s;
    cursor: pointer;
}

.store-card:hover {
    z-index: 30;
    transform: scale(1.02);
    border-color: rgba(168,85,247,0.5);
    box-shadow: 0 20px 50px rgba(0,0,0,0.5);
}

.card-click-layer {
    display: none;
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    z-index: 20;
    cursor: zoom-in;
}

@media (min-width: 768px) {
    .card-click-layer {
        display: block;
    }
}

.card-bg {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    background: linear-gradient(to bottom right, #2a2a2a, #1a1a1a, #050505);
    z-index: 0;
}

.card-hover-glow {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    opacity: 0;
    transition: opacity 0.7s;
    z-index: 10;
    pointer-events: none;
    background: linear-gradient(to bottom right, rgba(168,85,247,0.1), transparent, rgba(59,130,246,0.1));
    mix-blend-mode: overlay;
}

.store-card:hover .card-hover-glow {
    opacity: 1;
}

.card-image-wrapper {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    width: 100%;
    height: 100%;
    z-index: 10;
}

.card-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.7s ease-out;
    opacity: 0.9;
}

.store-card:hover .card-image {
    transform: scale(1.1);
    opacity: 1;
}

.card-image-overlay {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
    background: linear-gradient(to top, var(--sos-text-primary), rgba(10,10,10,0.4), transparent);
    opacity: 0.9;
}

.card-rarity {
    position: absolute;
    top: 0;
    right: 0;
    padding: 1.25rem;
    z-index: 30;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.25rem;
    opacity: 0.7;
    transition: opacity 0.3s;
    pointer-events: none;
}

.store-card:hover .card-rarity {
    opacity: 1;
}

.rarity-label {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.3em;
    filter: drop-shadow(0 1px 2px rgba(0,0,0,0.1)) drop-shadow(0 1px 1px rgba(0,0,0,0.06));
}

.rarity-line {
    width: 4rem;
    height: 1px;
}

/* Corner marks */
.corner-mark {
    position: absolute;
    z-index: 20;
    width: 0.5rem;
    height: 0.5rem;
    opacity: 0;
    transition: all 0.3s;
    pointer-events: none;
}

.store-card:hover .corner-mark {
    opacity: 1;
}

.corner-mark--top-left {
    top: 1rem;
    left: 1rem;
    border-top: 1px solid rgba(255,255,255,0.3);
    border-left: 1px solid rgba(255,255,255,0.3);
}

.corner-mark--bottom-right {
    bottom: 1rem;
    right: 1rem;
    border-bottom: 1px solid rgba(255,255,255,0.3);
    border-right: 1px solid rgba(255,255,255,0.3);
}

/* Card info area */
.card-info {
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    padding: 1.5rem;
    z-index: 30;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    height: 100%;
    pointer-events: none;
}

.card-info-inner {
    transform: translateY(0);
    transition: transform 0.5s ease-out;
}

.card-info-inner--default {
    transform: translateY(0.5rem);
}

.card-info-inner--active {
    transform: translateY(0);
}

@media (min-width: 768px) {
    .store-card:hover .card-info-inner--default {
        transform: translateY(0);
    }
}

.card-title {
    font-size: 1.5rem;
    font-weight: 700;
    font-family: "Noto Serif SC", serif;
    color: var(--sos-bg-surface);
    margin-bottom: 0.5rem;
    filter: drop-shadow(0 10px 8px rgba(0,0,0,0.04)) drop-shadow(0 4px 3px rgba(0,0,0,0.1));
    line-height: 1.25;
    max-width: 95%;
}

.card-meta {
    display: flex;
    align-items: center;
    gap: 0.75rem;
}

.card-price {
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    background: rgba(255,255,255,0.05);
    border: 1px solid rgba(255,255,255,0.1);
    padding: 0.25rem 0.75rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    color: #ede9fe;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    transition: all 0.3s;
    box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1);
}

@media (min-width: 768px) {
    .store-card:hover .card-price {
        background: #9333ea;
        border-color: #a855f7;
        color: var(--sos-bg-surface);
    }
}

.price-value {
    font-weight: 700;
    letter-spacing: -0.025em;
}

.price-unit {
    font-size: 10px;
    opacity: 0.7;
}

.card-stock {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    background: rgba(0,0,0,0.2);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    border: 1px solid rgba(255,255,255,0.05);
}

.stock-dot {
    width: 0.375rem;
    height: 0.375rem;
    border-radius: 9999px;
}

.stock-dot--ok {
    background-color: #34d399;
    color: #34d399;
    box-shadow: 0 0 5px currentColor;
}

.stock-dot--low {
    background-color: var(--sos-danger);
    color: var(--sos-danger);
    box-shadow: 0 0 5px currentColor;
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

.stock-text {
    font-size: 10px;
    color: var(--sos-text-tertiary);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    letter-spacing: 0.05em;
}

/* Card detail (hover/active expand) */
.card-detail {
    overflow: hidden;
    transition: all 0.5s;
    transition-delay: 0.1s;
}

.card-detail--default {
    height: 0;
    opacity: 0;
    margin-top: 0;
}

.card-detail--active {
    height: auto;
    opacity: 1;
    margin-top: 1.25rem;
}

@media (min-width: 768px) {
    .store-card:hover .card-detail--default {
        height: auto;
        opacity: 1;
        margin-top: 1.25rem;
    }
}

.card-description {
    font-size: 0.75rem;
    color: rgba(209,213,219,0.8);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    margin-bottom: 1rem;
    font-family: "Noto Sans SC", sans-serif;
    line-height: 1.625;
    letter-spacing: 0.025em;
    border-left: 2px solid rgba(255,255,255,0.2);
    padding-left: 0.75rem;
}

.card-view-image-mobile {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 0.75rem;
    pointer-events: auto;
}

@media (min-width: 768px) {
    .card-view-image-mobile {
        display: none;
    }
}

.view-image-btn {
    font-size: 10px;
    color: #d8b4fe;
    border: none;
    background: none;
    border-bottom: 1px solid rgba(216,180,254,0.5);
    padding-bottom: 0.125rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    cursor: pointer;
    transition: color 0.2s, border-color 0.2s;
}

.view-image-btn:hover {
    color: var(--sos-bg-surface);
    border-bottom-color: var(--sos-bg-surface);
}

.view-image-arrow {
    font-size: 0.75rem;
}

/* Redeem button */
.redeem-btn {
    width: 100%;
    padding: 0.75rem 0;
    background: var(--sos-bg-surface);
    color: var(--sos-text-primary);
    font-weight: 700;
    font-size: 0.75rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    border: none;
    cursor: pointer;
    transition: all 0.3s;
    position: relative;
    overflow: hidden;
    box-shadow: 0 0 20px rgba(255,255,255,0.1);
    pointer-events: auto;
}

.redeem-btn:hover {
    background: #a855f7;
    color: var(--sos-bg-surface);
    box-shadow: 0 0 30px rgba(168,85,247,0.4);
}

.redeem-btn:disabled {
    background: var(--sos-text-primary);
    color: var(--sos-text-secondary);
    cursor: not-allowed;
}

.redeem-btn-text {
    position: relative;
    z-index: 10;
}

/* ========================================
   Page Footer
   ======================================== */
.page-footer {
    text-align: center;
    padding-bottom: 3rem;
    position: relative;
    z-index: 20;
}

.footer-content {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    opacity: 0.3;
    transition: opacity 0.5s;
    cursor: default;
}

.footer-content:hover {
    opacity: 1;
}

.footer-line {
    width: 1px;
    height: 4rem;
    background: linear-gradient(to bottom, transparent, var(--sos-bg-surface), transparent);
}

.footer-text {
    font-size: 10px;
    letter-spacing: 0.5em;
    text-transform: uppercase;
    font-weight: 300;
    color: var(--sos-bg-surface);
}

/* ========================================
   Modal Overlay (shared)
   ======================================== */
.modal-overlay {
    position: fixed;
    top: 0; right: 0; bottom: 0; left: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0,0,0,0.9);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    padding: 1rem;
}

@media (min-width: 768px) {
    .modal-overlay {
        padding: 2rem;
    }
}

/* ========================================
   Points Modal
   ======================================== */
.points-modal {
    background: rgba(17,17,17,0.9);
    border: 1px solid rgba(168,85,247,0.3);
    padding: 2rem;
    border-radius: 0.75rem;
    max-width: 28rem;
    width: 100%;
    position: relative;
    overflow: hidden;
    box-shadow: 0 0 50px rgba(168,85,247,0.15);
}

.modal-close-btn {
    position: absolute;
    top: 1rem;
    right: 1rem;
    color: var(--sos-text-secondary);
    background: none;
    border: none;
    cursor: pointer;
    transition: color 0.2s;
}

.modal-close-btn:hover {
    color: var(--sos-bg-surface);
}

.modal-close-icon {
    width: 1.5rem;
    height: 1.5rem;
}

.points-modal-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
}

.points-user-id {
    font-size: 0.75rem;
    color: #c084fc;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-bottom: 0.5rem;
}

.points-total {
    font-size: 3rem;
    font-weight: 900;
    color: var(--sos-bg-surface);
    margin-bottom: 1.5rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    letter-spacing: -0.05em;
    filter: drop-shadow(0 0 10px rgba(255,255,255,0.5));
}

.points-history-section {
    width: 100%;
    border-top: 1px solid rgba(255,255,255,0.1);
    padding-top: 1rem;
}

.points-history-header {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--sos-text-secondary);
    margin-bottom: 0.75rem;
    padding: 0 0.5rem;
}

.points-history-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 200px;
    overflow-y: auto;
}

.points-history-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.875rem;
    padding: 0.5rem;
    border-radius: 0.25rem;
    transition: background-color 0.2s;
}

.points-history-item:hover {
    background: rgba(255,255,255,0.05);
}

.history-date {
    color: var(--sos-text-tertiary);
    font-size: 0.75rem;
}

.history-reason {
    color: var(--sos-bg-surface);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 120px;
}

.history-change {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-weight: 700;
}

.history-change--positive {
    color: #4ade80;
}

.history-change--negative {
    color: #f87171;
}

.points-empty {
    color: var(--sos-text-secondary);
    font-size: 0.75rem;
    padding: 1rem 0;
}

/* ========================================
   Image Modal
   ======================================== */
.image-modal-close {
    position: absolute;
    top: 1rem;
    right: 1rem;
    color: rgba(255,255,255,0.5);
    background: none;
    border: none;
    cursor: pointer;
    transition: color 0.2s;
    z-index: 101;
}

@media (min-width: 768px) {
    .image-modal-close {
        top: 2rem;
        right: 2rem;
    }
}

.image-modal-close:hover {
    color: var(--sos-bg-surface);
}

.image-modal-close-icon {
    width: 2rem;
    height: 2rem;
}

@media (min-width: 768px) {
    .image-modal-close-icon {
        width: 2.5rem;
        height: 2.5rem;
    }
}

.image-modal-content {
    position: relative;
    max-width: 100%;
    max-height: 100%;
    overflow: hidden;
    border-radius: 0.5rem;
    box-shadow: 0 0 50px rgba(168,85,247,0.2);
}

.image-modal-img {
    max-width: 100%;
    max-height: 90vh;
    object-fit: contain;
}

/* ========================================
   Success Modal
   ======================================== */
.success-modal-overlay {
    position: fixed;
    top: 0; right: 0; bottom: 0; left: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}

.success-modal {
    background: rgba(17,17,17,0.9);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
    border: 1px solid rgba(255,255,255,0.1);
    padding: 2.5rem;
    border-radius: 0;
    text-align: center;
    box-shadow: 0 0 100px rgba(100,50,250,0.3);
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    max-width: 24rem;
    width: 100%;
    position: relative;
    overflow: hidden;
}

.success-modal-topline {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 0.25rem;
    background: linear-gradient(to right, transparent, #c084fc, transparent);
}

.success-modal-glow {
    position: absolute;
    top: -5rem;
    left: -5rem;
    width: 10rem;
    height: 10rem;
    background: rgba(168,85,247,0.2);
    filter: blur(50px);
    border-radius: 9999px;
}

.success-modal-icon {
    width: 4rem;
    height: 4rem;
    border-radius: 9999px;
    border: 1px solid rgba(168,85,247,0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 1.5rem;
    color: #c084fc;
    box-shadow: 0 0 20px rgba(168,85,247,0.2);
}

.success-modal-title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--sos-bg-surface);
    margin-bottom: 0.5rem;
    font-family: "Noto Serif SC", serif;
    letter-spacing: 0.025em;
}

.success-modal-text {
    color: var(--sos-text-tertiary);
    font-size: 0.75rem;
    margin-bottom: 2rem;
    letter-spacing: 0.05em;
}

.success-modal-close-btn {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--sos-bg-surface);
    border: 1px solid rgba(255,255,255,0.2);
    padding: 0.5rem 2rem;
    background: none;
    cursor: pointer;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    transition: background-color 0.2s, color 0.2s;
}

.success-modal-close-btn:hover {
    background: var(--sos-bg-surface);
    color: var(--sos-text-primary);
}

/* ========================================
   Utility Classes (kept from original)
   ======================================== */

/* 隐藏滚动条 */
.no-scrollbar::-webkit-scrollbar {
    display: none;
}
.no-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
}

/* 按钮斜切角效果 */
.clip-path-button {
    clip-path: polygon(10px 0, 100% 0, 100% calc(100% - 10px), calc(100% - 10px) 100%, 0 100%, 0 10px);
}

/* 呼吸动画 */
@keyframes pulse-slow {
    0%, 100% { transform: scale(1); opacity: 0.2; }
    50% { transform: scale(1.3); opacity: 0.6; }
}
.animate-pulse-slow {
    animation: pulse-slow 6s infinite ease-in-out;
}

@keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-30px); }
}
.animate-float {
    animation: float 7s ease-in-out infinite;
}

@keyframes wiggle-flip {
    0% { transform: rotate(-5deg); }
    25% { transform: rotate(5deg); }
    50% { transform: rotate(-5deg) scaleX(-1); }
    75% { transform: rotate(5deg) scaleX(-1); }
    100% { transform: rotate(-5deg); }
}
.animate-wiggle-flip {
    animation: wiggle-flip 3s infinite ease-in-out;
}

.number-flow {
    font-variant-numeric: tabular-nums;
}

/* 边框发光动画 */
.border-glow {
    box-shadow: inset 0 0 20px rgba(168, 85, 247, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.2);
}

/* 进入动画 */
.animate-slide-in {
    animation: slideUpFade 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    opacity: 0;
    transform: translateY(20px);
}

.delay-100 { animation-delay: 0.1s; }
.delay-75 { animation-delay: 0.075s; }
.delay-1000 { animation-delay: 1s; }

@keyframes slideUpFade {
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

/* 弹窗动画 */
.pop-in-enter-active, .pop-in-leave-active { transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
.pop-in-enter-from, .pop-in-leave-to { opacity: 0; transform: scale(0.8) translateY(10px); }

/* 淡入淡出动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-5px);
}
</style>
