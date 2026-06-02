<template>
    <Transition name="search-modal">
        <div
            v-if="store.isSearchOpen"
            class="search-overlay"
            @click.self="store.toggleSearch"
        >
            <button
                @click="store.toggleSearch"
                class="close-button"
            >
                <svg class="close-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M6 18L18 6M6 6l12 12"></path></svg>
            </button>

            <div class="search-container animate-up">
                <div class="search-input-wrapper">
                    <input
                        ref="inputRef"
                        type="text"
                        v-model="store.searchQuery"
                        @keyup.enter="performSearch"
                        placeholder="搜索: 凉宫春日..."
                        class="search-input"
                    >

                    <button
                        @click="performSearch"
                        class="search-submit"
                    >
                        <span>ENTER</span>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="submit-icon">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3" />
                        </svg>
                    </button>
                </div>

                <div class="hot-searches">
                    <span>热门搜索：</span>
                    <span class="hot-search-item" @click="quickSearch('凉宫春日')">凉宫春日</span>
                    <span class="hot-search-item" @click="quickSearch('漫无止境的八月')">漫无止境的八月</span>
                    <span class="hot-search-item" @click="quickSearch('长门有希')">长门有希</span>
                </div>
            </div>
        </div>
    </Transition>
</template>

<script setup>
import { ref, watch, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useMainStore } from '@/stores/main';

const store = useMainStore();
const router = useRouter();
const inputRef = ref(null);

watch(() => store.isSearchOpen, (val) => {
    if (val) nextTick(() => inputRef.value?.focus());
});

const performSearch = () => {
    if (!store.searchQuery) return;
    router.push(`/search?q=${encodeURIComponent(store.searchQuery)}`); // 建议带上参数
    store.toggleSearch();
};

// 辅助功能：点击推荐词直接搜索
const quickSearch = (term) => {
    store.searchQuery = term;
    performSearch();
};
</script>

<style scoped>
/* Search Overlay */
.search-overlay {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 50;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 8rem;
    padding-left: 1rem;
    padding-right: 1rem;
    background-color: rgba(255, 255, 255, 0.9);
    backdrop-filter: blur(24px);
    transition-property: color, background-color, border-color;
    transition-duration: 300ms;
}

/* Close Button */
.close-button {
    position: absolute;
    top: 1.5rem;
    right: 1.5rem;
    padding: 0.5rem;
    color: #6b7280;
    border-radius: 9999px;
    transition: all 300ms;
}

.close-button:hover {
    color: #000000;
    background-color: rgba(0, 0, 0, 0.05);
}

.close-icon {
    width: 2rem;
    height: 2rem;
}

/* Search Container */
.search-container {
    width: 100%;
    max-width: 48rem;
    position: relative;
}

.search-input-wrapper {
    position: relative;
}

/* Search Input */
.search-input {
    width: 100%;
    font-size: 2.25rem;
    line-height: 2.5rem;
    font-weight: 700;
    background: transparent;
    border: none;
    border-bottom: 2px solid #e5e7eb;
    padding-top: 1.5rem;
    padding-bottom: 1.5rem;
    padding-right: 8rem;
    font-family: "Noto Serif SC", serif;
    transition-property: color, background-color, border-color;
    transition-duration: 300ms;
}

@media (min-width: 768px) {
    .search-input {
        font-size: 3.75rem;
        line-height: 1;
    }
}

.search-input:focus {
    outline: none;
    border-color: #000000;
}

.search-input::placeholder {
    color: #d1d5db;
}

/* Search Submit Button */
.search-submit {
    position: absolute;
    right: 0;
    bottom: 1.5rem;
    font-size: 1.25rem;
    line-height: 1.75rem;
    font-weight: 500;
    color: #9ca3af;
    transition-property: color, background-color, border-color;
    transition-duration: 150ms;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.search-submit:hover {
    color: #000000;
}

.submit-icon {
    width: 1.25rem;
    height: 1.25rem;
}

/* Hot Searches */
.hot-searches {
    margin-top: 2rem;
    display: flex;
    gap: 1rem;
    font-size: 0.875rem;
    line-height: 1.25rem;
    color: #6b7280;
}

.hot-search-item {
    cursor: pointer;
}

.hot-search-item:hover {
    color: #000000;
    text-decoration: underline;
}

/* Vue Transition 动画样式
  效果：背景淡入淡出 + 内容微微上浮/下沉 + 缩放
*/

/* 进场和离场的持续状态 */
.search-modal-enter-active,
.search-modal-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); /* 使用更自然的贝塞尔曲线 */
}

/* 进场初始状态 / 离场结束状态 */
.search-modal-enter-from,
.search-modal-leave-to {
  opacity: 0;
  transform: scale(0.98) translateY(10px); /* 微微缩小并下沉，产生上浮感 */
  backdrop-filter: blur(0px); /* 模糊度也做动画 */
}

/* 进场结束状态 / 离场初始状态 */
.search-modal-enter-to,
.search-modal-leave-from {
  opacity: 1;
  transform: scale(1) translateY(0);
  backdrop-filter: blur(16px);
}
</style>
