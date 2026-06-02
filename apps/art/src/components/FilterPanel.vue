<template>
  <div class="filter-panel-3d">
    <div class="panel-group">
      <div class="control-box content-box">
        <div class="c-label">内容</div>
        <div class="toggle-track content-track">
          <div
            class="slider-bg"
            :class="{
              'pos-0': content === 'mix',
              'pos-1': content === 'haruhi',
              'pos-2': content === 'other'
            }"
          ></div>

          <button
            class="toggle-item"
            :class="{ active: content === 'mix' }"
            type="button"
            data-sfx="click"
            @click="emit('update:content', 'mix')"
          >
            混合
          </button>
          <button
            class="toggle-item"
            :class="{ active: content === 'haruhi' }"
            type="button"
            data-sfx="click"
            @click="emit('update:content', 'haruhi')"
          >
            凉宫
          </button>
          <button
            class="toggle-item"
            :class="{ active: content === 'other' }"
            type="button"
            data-sfx="click"
            @click="emit('update:content', 'other')"
          >
            其他
          </button>
        </div>
      </div>

      <div class="control-box source-box">
        <div class="c-label">来源</div>
        <div class="toggle-track source-track">
          
          <img src="../assets/lucky.webp" class="mascot-img" alt="decoration" />

          <div
            class="slider-bg"
            :class="{
              'pos-0': sourceMode === 'all' || sourceMode === 'balanced',
              'pos-1': sourceMode === 'network',
              'pos-2': sourceMode === 'personal'
            }"
          ></div>

          <button
            class="toggle-item"
            :class="{ active: sourceMode === 'all' || sourceMode === 'balanced' }"
            type="button"
            data-sfx="click"
            @click="emit('update:sourceMode', 'all')"
          >
            全部
          </button>
          <button
            class="toggle-item"
            :class="{ active: sourceMode === 'network' }"
            type="button"
            data-sfx="click"
            @click="emit('update:sourceMode', 'network')"
          >
            网络
          </button>
          <button
            class="toggle-item"
            :class="{ active: sourceMode === 'personal' }"
            type="button"
            data-sfx="click"
            @click="emit('update:sourceMode', 'personal')"
          >
            个人
          </button>
        </div>
      </div>
    </div>

    <!-- ✅ 排序 -->
    <div class="panel-group sort-group">
      <div class="control-box sort-box">
        <div class="c-label">排序</div>

        <!-- ✅ 轨道空白处也可点（避免“点了没反应”的体验） -->
        <div class="toggle-track sort-track" @click="onSortTrackClick">
          <div
            class="slider-bg"
            :class="{
              'pos-0': sortMode === 'time',
              'pos-1': sortMode === 'likes',
              'pos-2': sortMode === 'random'
            }"
          ></div>

          <button
            class="toggle-item sort-item"
            :class="{ active: sortMode === 'time' }"
            type="button"
            data-sfx="click"
            @click="emit('update:sortMode', 'time')"
          >
            最新
          </button>
          <button
            class="toggle-item sort-item"
            :class="{ active: sortMode === 'likes' }"
            type="button"
            data-sfx="click"
            @click="emit('update:sortMode', 'likes')"
          >
            点赞最高
          </button>
          <button
            class="toggle-item sort-item"
            :class="{ active: sortMode === 'random' }"
            type="button"
            data-sfx="click"
            @click="emit('update:sortMode', 'random')"
          >
            随机
          </button>
        </div>
      </div>
    </div>

    <div class="panel-group search-group">
      <div class="search-wrapper">
        <select
          class="input-3d search-select"
          :value="searchField"
          @change="emit('update:searchField', $event.target.value)"
        >
          <option value="all">综合搜</option>
          <option value="title">搜标题</option>
          <option value="uid">搜作者</option>
          <option v-if="searchField === 'tag'" value="tag">搜标签</option>
        </select>

        <input
          class="input-3d search-input"
          :value="q"
          @input="emit('update:q', $event.target.value)"
          @keydown.enter="emit('search')"
          :placeholder="searchPlaceholder"
          type="search"
        />
        <button class="btn-3d btn-primary" type="button" data-sfx="click" @click="emit('search')">
          GO
        </button>
        <button
          class="btn-3d btn-ghost"
          type="button"
          data-sfx="click"
          @click="emit('update:q',''); emit('clear'); emit('search');"
          :disabled="!String(q||'').trim()"
          style="opacity:.9;"
        >
          清空
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  content: { type: String, default: 'mix' },
  sourceMode: { type: String, default: 'all' },
  sortMode: { type: String, default: 'random' },
  q: { type: String, default: '' },
  searchField: { type: String, default: 'all' }
})

const emit = defineEmits([
  'update:content',
  'update:sourceMode',
  'update:sortMode',
  'update:q',
  'update:searchField',
  'search',
  'clear'
])

const searchPlaceholder = computed(() => {
  const map = {
    all: '搜索所有内容...',
    title: '请输入作品标题...',
    uid: '请输入作者名...',
    tag: '搜索标签...'
  }
  return map[props.searchField] || '搜索作品...'
})

/**
 * ✅ 点击排序轨道空白位置也能切换
 * - 若点击的是按钮本身，交给按钮处理（避免重复触发）
 * - 若点击空白，按点击位置落在哪个 1/3 区间决定模式
 */
function onSortTrackClick(e) {
  const target = e?.target
  if (target && target.closest && target.closest('button')) return

  const el = e?.currentTarget
  if (!el || !el.getBoundingClientRect) return

  const rect = el.getBoundingClientRect()
  const x = (e.clientX ?? 0) - rect.left
  const ratio = rect.width > 0 ? (x / rect.width) : 0
  const idx = Math.max(0, Math.min(2, Math.floor(ratio * 3)))
  const modes = ['time', 'likes', 'random']
  emit('update:sortMode', modes[idx])
}
</script>

<style scoped>
/* =========================================
   通用样式 (电脑端优先)
   ========================================= */
.filter-panel-3d {
  padding: 12px 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px; /* reduced gap */
  margin-bottom: 16px;
  margin-top: 16px;

  /* ✅ 防止被其它层盖住导致“点不到” */
  position: relative;
  z-index: 50;
}

.panel-group { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; } /* reduced gap */
.control-box { display: flex; align-items: center; gap: 8px; } /* reduced gap */
.c-label { font-weight: 950; opacity: .78; font-size: 16px; letter-spacing: .2px; } /* slightly smaller font */

/* 轨道样式 */
.toggle-track {
  position: relative;
  display: grid;
  background: rgba(255,255,255,.5);
  padding: 3px; /* reduced padding */
  border-radius: 999px;
  border: 1px solid rgb(95, 215, 226);
  isolation: isolate;
  overflow: visible;

  /* ✅ 移动端触摸更直接 */
  touch-action: manipulation;
}

/* 装饰图片 */
.mascot-img {
  position: absolute;
  bottom: 95%;
  right: 0px;
  height: 120px;
  pointer-events: none;
  z-index: 10;
  filter: drop-shadow(0 4px 6px rgba(0,0,0,0.1));
  transform: rotate(0deg);
  max-width: none;
  width: auto;
}

.content-track { grid-template-columns: 1fr 1fr 1fr; width: 175px; } /* moderate width */
.source-track { grid-template-columns: 1fr 1fr 1fr; width: 175px; } /* moderate width */

/* 排序：自适应宽度 */
.sort-track{
  grid-template-columns: 1fr 1fr 1fr;
  width: clamp(175px, 24vw, 240px); /* moderate clamp */
  max-width: 100%;
}

/* 滑块样式 */
.slider-bg {
  position: absolute;
  top: 3px; /* moderate padding */
  bottom: 3px;
  background: rgba(192, 120, 241, 0.991);
  border-radius: 999px;
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  z-index: 1;
  box-shadow: 0 4px 12px rgba(0,0,0,.15);

  /* ✅ 关键：滑块层不吃点击/触摸事件 */
  pointer-events: none;
}

/* 滑块位置 */
.content-track .slider-bg { width: calc((100% - 6px) / 3); left: 3px; }
.content-track .slider-bg.pos-0 { transform: translateX(0%); }
.content-track .slider-bg.pos-1 { transform: translateX(100%); }
.content-track .slider-bg.pos-2 { transform: translateX(200%); }

.source-track .slider-bg { width: calc((100% - 6px) / 3); left: 3px; }
.source-track .slider-bg.pos-0 { transform: translateX(0%); }
.source-track .slider-bg.pos-1 { transform: translateX(100%); }
.source-track .slider-bg.pos-2 { transform: translateX(200%); }

.sort-track .slider-bg { width: calc((100% - 6px) / 3); left: 3px; }
.sort-track .slider-bg.pos-0 { transform: translateX(0%); }
.sort-track .slider-bg.pos-1 { transform: translateX(100%); }
.sort-track .slider-bg.pos-2 { transform: translateX(200%); }

/* 按钮样式 */
.toggle-item {
  position: relative;
  z-index: 2;
  border: none;
  background: transparent;
  padding: 5px 0; /* moderate padding */
  font-weight: 900;
  font-size: 14px; /* moderate font */
  color: rgba(0,0,0,.6);
  border-radius: 999px;
  cursor: pointer;
  transition: color 0.2s ease;
  text-align: center;

  /* ✅ 明确可点 */
  pointer-events: auto;
}
.toggle-item:hover { color: rgba(0,0,0,.9); }
.toggle-item.active {
  color: #fff;
  text-shadow: 0 1px 2px rgba(0,0,0,0.2);
}

.sort-item{
  padding: 5px 6px;
  white-space: normal;
  line-height: 1.1;
  word-break: keep-all;
}

/* 搜索区域 */
.search-wrapper { display: flex; gap: 10px; width: 100%; align-items:center; flex-wrap: wrap; }

.input-3d{
  padding: 10px 12px;
  border-radius: 14px;
  border: 1px solid rgba(0,0,0,.12);
  background: rgba(255,255,255,.9);
  font-weight: 850;
  outline: none;
  box-shadow: 0 2px 6px rgba(0,0,0,.03);
  transition: border-color 0.2s, box-shadow 0.2s;
}
.input-3d:focus{
  border-color: rgba(20,184,166,.55);
  box-shadow: 0 0 0 4px rgba(20,184,166,.14);
}

.search-select {
  appearance: none;
  background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23333' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
  background-repeat: no-repeat;
  background-position: right 8px center;
  background-size: 16px;
  padding-right: 30px;
  cursor: pointer;
  width: 100px;
  flex-shrink: 0;
}

.search-input {
  min-width: 200px;
  flex: 1;
}

.btn-3d{
  border: 1px solid rgba(0,0,0,.12);
  border-radius: 14px;
  padding: 10px 12px;
  cursor: pointer;
  font-weight: 950;
  background: rgba(255,255,255,.85);
  transition: transform .12s ease;
  box-shadow: 0 2px 6px rgba(0,0,0,.03);
}
.btn-3d:active{ transform: scale(1.06); }
.btn-primary{
  background: linear-gradient(180deg, rgba(20,184,166,.95), rgba(16,120,110,.95));
  color: white;
  border-color: rgba(16,120,110,.35);
}
.btn-ghost{
  background: rgba(255,255,255,.75);
}

/* =========================================
   手机端自适应优化
   ========================================= */
@media (max-width: 768px) {
  .filter-panel-3d {
    align-items: stretch;
    padding: 0 8px;
    margin-top: 130px;
  }

  .panel-group{
    width: 100%;
  }

  .panel-group:not(.sort-group):not(.search-group) {
    flex-direction: row;
    align-items: center;
    gap: 6px;
    flex-wrap: nowrap;
  }

  .content-box { flex: 2; min-width: 0; }
  .source-box { flex: 3; min-width: 0; }

  .control-box {
    flex-direction: row;
    align-items: center;
    gap: 4px;
  }

  .c-label {
    font-size: 12px;
    opacity: 0.8;
    margin: 0;
    white-space: nowrap;
  }

  .toggle-track {
    width: 100%;
    margin: 0;
    padding: 2px;
  }

  .content-track, .source-track { width: 100%; }

  .toggle-item {
    font-size: 13px;
    padding: 5px 0;
    white-space: nowrap;
  }

  .sort-group{
    flex-wrap: nowrap;
  }
  .sort-box{
    width: 100%;
  }
  .sort-track{
    width: 100%;
  }
  .sort-item{
    white-space: normal;
    padding: 5px 6px;
  }

  .slider-bg { top: 2px; bottom: 2px; }
  .content-track .slider-bg { width: calc((100% - 4px) / 3); left: 2px; }
  .source-track .slider-bg { width: calc((100% - 4px) / 3); left: 2px; }
  .sort-track .slider-bg { width: calc((100% - 4px) / 3); left: 2px; }

  .mascot-img {
    height: 100px;
    bottom: 90%;
    right: 0px;
  }

  .search-wrapper {
    width: 100%;
    flex-wrap: nowrap;
    margin-top: 8px;
  }

  .search-select {
    width: 85px;
    padding-left: 8px;
    font-size: 13px;
  }

  .btn-primary { display: none; }

  .btn-ghost {
    flex-shrink: 0;
    white-space: nowrap;
    padding: 10px 10px;
  }

  .search-input {
    min-width: 0;
  }
}
</style>
