<template>
  <div class="generator-tab">
     <!-- Keep existing generator code -->
     <div class="generator-layout">
      <div class="generator-sidebar">
        <div>
          <h3 class="generator-config-title">生成配置</h3>
          <div class="generator-config-box">
            <div>
              <label class="form-label">基准日期 (截止日)</label>
              <input type="date" v-model="genConfig.date" class="gen-date-input">
            </div>
            <div>
              <label class="form-label form-label-mb3">时间尺度</label>
              <div class="gen-range-grid">
                <button v-for="opt in rangeOptions" :key="opt.value" @click="genConfig.range = opt.value" :class="genConfig.range === opt.value ? 'opt-active' : 'opt-inactive'" class="gen-range-btn">{{ opt.label }}</button>
              </div>
            </div>
            <div class="gen-match-info">
              <span class="gen-match-text">匹配文章数: <b class="gen-match-count">{{ generatedNewsList.length }}</b></span>
            </div>
          </div>
        </div>
        <button @click="downloadImage" :disabled="isGenerating || generatedNewsList.length === 0" class="gen-export-btn">
          {{ isGenerating ? '生成中...' : '导出图片 (PNG)' }}
        </button>
      </div>
      <div class="generator-preview-area">
        <div id="news-poster" class="poster-canvas" :class="posterCanvasClass">
          <div class="poster-header">
            <h1 class="poster-title">新闻总览</h1>
            <div class="poster-header-line"></div>
            <p class="poster-date-range">{{ dateRangeDisplay }}</p>
            <p class="poster-edition">{{ posterEditionLabel }}</p>
          </div>
          <div class="poster-body" :class="posterBodyClass">
            <div v-for="item in generatedNewsList" :key="item.id" class="poster-item" :class="{ 'poster-item--dense': isDenseRange }">
              <div class="poster-item-dot"></div>
              <div class="poster-item-date">{{ item.date }}</div>
              <h3 class="poster-item-title" :class="{ 'poster-item-title--dense': isDenseRange }">{{ item.title }}</h3>
              <p class="poster-item-summary" :class="{ 'poster-item-summary--dense': isDenseRange }">{{ getPosterSummary(item) }}</p>
            </div>
          </div>
          <div class="poster-footer">
            <div class="poster-footer-inner">
              <div><p class="poster-footer-brand">Haruyuki.cn</p></div>
            </div>
          </div>
        </div>
      </div>
     </div>
  </div>
</template>

<script setup>
import { ref, computed, reactive } from 'vue';
import { toBlob } from 'html-to-image';
import { useMainStore } from '@/stores/main';

const store = useMainStore();

// Generator State
const isGenerating = ref(false);
const genConfig = reactive({
    date: new Date().toISOString().split('T')[0], // Default today
    range: 'week', // day, week, half-month, month, quarter, half-year
    layout: 'classic', // [新增]
});

const rangeOptions = [
    { label: '单日', value: 'day' },
    { label: '周报', value: 'week' },
    { label: '半月刊', value: 'half-month' },
    { label: '月刊', value: 'month' },
    { label: '季度刊', value: 'quarter' },
    { label: '半年刊', value: 'half-year' },
];

const layoutOptions = [
    { label: '经典', value: 'classic' },
    { label: '紧凑', value: 'compact' },
    { label: '图文', value: 'visual' },
    { label: '极简', value: 'minimal' },
    { label: '详细', value: 'detailed' },
];

// ================= Generator Logic =================
const applyRangeOffset = (startDate, range) => {
    if (range === 'week') startDate.setDate(startDate.getDate() - 6);
    else if (range === 'half-month') startDate.setDate(startDate.getDate() - 14);
    else if (range === 'month') startDate.setMonth(startDate.getMonth() - 1);
    else if (range === 'quarter') startDate.setMonth(startDate.getMonth() - 3);
    else if (range === 'half-year') startDate.setMonth(startDate.getMonth() - 6);
};

const rangeLabelMap = {
    day: '单日快报',
    week: '周报',
    'half-month': '半月刊',
    month: '月刊',
    quarter: '季度刊',
    'half-year': '半年刊',
};

const isDenseRange = computed(() => genConfig.range === 'quarter' || genConfig.range === 'half-year');
const posterCanvasClass = computed(() => ({
    'poster-canvas--dense': isDenseRange.value,
    'poster-canvas--quarter': genConfig.range === 'quarter',
    'poster-canvas--half-year': genConfig.range === 'half-year',
}));
const posterBodyClass = computed(() => ({
    'poster-body--dense': isDenseRange.value,
    'poster-body--dense-quarter': genConfig.range === 'quarter',
    'poster-body--dense-half-year': genConfig.range === 'half-year',
}));
const posterEditionLabel = computed(() => rangeLabelMap[genConfig.range] || '周报');
const getPosterSummary = (item) => {
    const text = item.summary || item.preview || '';
    if (!text) return '';
    if (!isDenseRange.value) return text;
    const maxLen = genConfig.range === 'half-year' ? 34 : 54;
    return text.length > maxLen ? `${text.slice(0, maxLen)}...` : text;
};

const generatedNewsList = computed(() => {
    // 1. 确定基准日期 (结束日期)
    const targetDateStr = genConfig.date;
    const targetDate = new Date(targetDateStr);
    targetDate.setHours(23, 59, 59, 999);

    // 2. 确定开始日期
    const startDate = new Date(targetDate);
    startDate.setHours(0, 0, 0, 0);

    applyRangeOffset(startDate, genConfig.range);

    // 3. 数据源
    const source = store.allArticles;
    const result = source.filter(article => {
        if (!article.date) return false;
        if (article.status && article.status !== 'published') return false;

        let rawDate = article.date.toString();
        let dateStr = rawDate
            .replace(/\./g, '-')
            .replace(/年/g, '-')
            .replace(/月/g, '-')
            .replace(/日/g, '');

        const articleDate = new Date(dateStr);
        if (isNaN(articleDate.getTime())) return false;

        articleDate.setHours(12, 0, 0, 0);
        const inRange = articleDate >= startDate && articleDate <= targetDate;
        return inRange;
    }).sort((a, b) => {
        const parse = (d) => {
             const s = d.toString()
                .replace(/\./g, '-')
                .replace(/年/g, '-')
                .replace(/月/g, '-')
                .replace(/日/g, '');
             return new Date(s);
        };
        const dateA = parse(a.date);
        const dateB = parse(b.date);
        return dateB - dateA;
    });

    return result;
});

const getSummaryContent = (item) => {
    if (genConfig.layout === 'compact' || genConfig.layout === 'minimal') return '';
    let text = '';
    if (genConfig.layout === 'detailed') {
       text = item.summary || item.preview || '';
    } else {
       text = item.subtitle || item.summary || item.preview || '';
    }
    if (!text) return '';
    if (genConfig.layout !== 'detailed' && !item.subtitle && !item.summary && text.length > 60) {
        return text.slice(0, 60) + '...';
    }
    return text;
};

const dateRangeText = computed(() => {
    const end = new Date(genConfig.date).toLocaleDateString();
    const d = new Date(genConfig.date);
    applyRangeOffset(d, genConfig.range);
    const start = d.toLocaleDateString();
    return genConfig.range === 'day' ? end : `${start} - ${end}`;
});

const dateRangeDisplay = computed(() => {
    const formatDate = (dateObj) => {
        const y = dateObj.getFullYear();
        const m = String(dateObj.getMonth() + 1).padStart(2, '0');
        const d = String(dateObj.getDate()).padStart(2, '0');
        return `${y}.${m}.${d}`;
    };
    const targetDateStr = genConfig.date;
    const end = new Date(targetDateStr);
    const start = new Date(targetDateStr);

    if (genConfig.range === 'day') return formatDate(end);
    applyRangeOffset(start, genConfig.range);

    return `${formatDate(start)} - ${formatDate(end)}`;
});

const getExportParams = (el) => {
    const rect = el.getBoundingClientRect();
    const width = Math.max(1, rect.width);
    const height = Math.max(1, rect.height);
    const area = width * height;

    // Browser canvas limits are usually the bottleneck, not raw CPU power.
    const preferredRatio = isDenseRange.value ? 1.6 : 2;
    const maxPixels = 16_000_000; // keep output around <=16MP
    const ratioCap = Math.sqrt(maxPixels / area);
    const safeRatio = Math.max(0.9, Math.min(preferredRatio, ratioCap));

    const timeoutMs = Math.min(
        180000,
        Math.max(30000, Math.round(10000 + area * safeRatio * safeRatio / 18000))
    );

    return {
        pixelRatio: Number(safeRatio.toFixed(2)),
        timeoutMs,
    };
};

const exportPosterBlob = async (el, pixelRatio, timeoutMs) => {
    const exportTask = toBlob(el, {
        backgroundColor: '#ffffff',
        pixelRatio,
        // Avoid forced cache busting; it can trigger redundant resource fetches.
        cacheBust: false,
    });
    const timeoutTask = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('export-timeout')), timeoutMs)
    );

    const blob = await Promise.race([exportTask, timeoutTask]);
    if (!blob) throw new Error('export-empty-blob');
    return blob;
};

const downloadImage = async () => {
    const el = document.getElementById('news-poster');
    if (!el) {
        return;
    }

    isGenerating.value = true;
    try {
        // Ensure web fonts are ready before rasterization for best fidelity.
        if (document.fonts?.ready) {
            await document.fonts.ready;
        }

        // Wait two frames to ensure latest layout/class changes are painted.
        await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));

        const { pixelRatio, timeoutMs } = getExportParams(el);
        let blob;

        try {
            blob = await exportPosterBlob(el, pixelRatio, timeoutMs);
        } catch (err) {
            // Auto-retry once with lower resolution for very large posters.
            const retryRatio = Math.max(0.85, Number((pixelRatio * 0.75).toFixed(2)));
            blob = await exportPosterBlob(el, retryRatio, timeoutMs + 30000);
        }

        const link = document.createElement('a');
        link.download = `hibiki-news-${genConfig.date}-${genConfig.range}.png`;
        const url = URL.createObjectURL(blob);
        link.href = url;
        link.click();
        setTimeout(() => URL.revokeObjectURL(url), 3000);
    } catch (err) {
        console.error('Generation failed', err);
        if (err?.message === 'export-timeout') {
            alert('导出超时：内容过多或画布过大，建议缩小时间范围后重试。');
        } else {
            alert('生成失败，请检查控制台');
        }
    } finally {
        isGenerating.value = false;
    }
};
</script>

<style scoped>
/* ==================== Generator Tab (Tab 5) ==================== */
.generator-tab {
  padding: 2rem;
  background-color: var(--sos-bg-subtle);
  flex: 1;
}

.generator-layout {
  display: flex;
  flex-direction: column;
  gap: 2.5rem;
}

@media (min-width: 1280px) {
  .generator-layout {
    flex-direction: row;
  }
}

.generator-sidebar {
  width: 100%;
}

.generator-sidebar > * + * {
  margin-top: 2rem;
}

@media (min-width: 1280px) {
  .generator-sidebar {
    width: 28%;
  }
}

.generator-config-title {
  font-size: 1.125rem;
  line-height: 1.75rem;
  font-weight: 700;
  font-family: "Noto Serif SC", serif;
  margin-bottom: 1rem;
}

.generator-config-box {
  background-color: var(--sos-bg-surface);
  padding: 1.5rem;
  border-radius: 0.75rem;
  border: 1px solid var(--sos-border-default);
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}

.generator-config-box > * + * {
  margin-top: 1.5rem;
}

.gen-date-input {
  width: 100%;
  border-bottom: 2px solid var(--sos-border-default);
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-size: 1.125rem;
  line-height: 1.75rem;
  background-color: transparent;
}

.gen-range-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
}

.gen-range-btn {
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  border: 1px solid;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
}

.gen-match-info {
  padding-top: 1rem;
  border-top: 1px solid var(--sos-bg-muted);
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.gen-match-text {
  color: var(--sos-text-secondary);
}

.gen-match-count {
  color: var(--sos-text-primary);
}

.gen-export-btn {
  width: 100%;
  padding: 1rem;
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  border-radius: 0.75rem;
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1.75rem;
  box-shadow: 0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1);
}

.gen-export-btn:hover {
  background-color: var(--sos-text-primary);
}

.gen-export-btn:disabled {
  opacity: 0.5;
}

/* Generator Preview Area */
.generator-preview-area {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background-color: rgba(229,231,235,0.5);
  border-radius: 0.75rem;
  padding: 2rem;
  border: 1px solid var(--sos-border-default);
  overflow-x: auto;
  overflow-y: auto;
}

@media (min-width: 1280px) {
  .generator-preview-area {
    width: 72%;
  }
}

/* Poster */
.poster-canvas {
  width: 450px;
  min-height: 800px;
  background-color: var(--sos-bg-surface);
  color: var(--sos-text-primary);
  display: flex;
  flex-direction: column;
  position: relative;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  flex-shrink: 0;
}

.poster-canvas--dense {
  min-height: 860px;
}

.poster-canvas--quarter {
  width: 760px;
}

.poster-canvas--half-year {
  width: 900px;
}

.poster-header {
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  padding: 2rem;
  padding-bottom: 1.5rem;
}

.poster-title {
  font-size: 2.25rem;
  line-height: 2.5rem;
  font-weight: 900;
  font-family: "Noto Serif SC", serif;
  line-height: 1;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.poster-header-line {
  width: 3rem;
  height: 0.25rem;
  background-color: var(--sos-bg-surface);
  margin-bottom: 1.5rem;
}

.poster-date-range {
  font-size: 1.25rem;
  line-height: 1.75rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  opacity: 0.9;
}

.poster-edition {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  line-height: 1rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  font-weight: 700;
  opacity: 0.72;
}

.poster-canvas--dense .poster-header {
  padding: 1.5rem;
  padding-bottom: 1rem;
}

.poster-canvas--dense .poster-title {
  margin-top: 0.5rem;
  font-size: 1.875rem;
  line-height: 2.25rem;
}

.poster-canvas--dense .poster-header-line {
  margin-bottom: 0.9rem;
}

.poster-canvas--dense .poster-date-range {
  font-size: 1rem;
  line-height: 1.5rem;
}

.poster-body {
  flex: 1;
  padding: 2rem;
}

.poster-body > * + * {
  margin-top: 1.5rem;
}

.poster-body--dense {
  display: grid;
  gap: 0.7rem 0.9rem;
  align-content: start;
  padding: 1.25rem;
}

.poster-body--dense > * + * {
  margin-top: 0;
}

.poster-body--dense-quarter {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.poster-body--dense-half-year {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.poster-item {
  position: relative;
  padding-left: 1.5rem;
  border-left: 2px solid var(--sos-border-default);
  min-width: 0;
}

.poster-item--dense {
  padding-left: 0.9rem;
  border-left-width: 1px;
  border-left-color: var(--sos-border-strong);
}

.poster-item-dot {
  position: absolute;
  left: -5px;
  top: 0.375rem;
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 9999px;
  background-color: var(--sos-text-primary);
}

.poster-item--dense .poster-item-dot {
  left: -4px;
  top: 0.3rem;
  width: 0.4rem;
  height: 0.4rem;
}

.poster-item-date {
  font-size: 10px;
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  color: var(--sos-text-tertiary);
  letter-spacing: 0.05em;
  margin-bottom: 0.25rem;
}

.poster-item--dense .poster-item-date {
  font-size: 9px;
  margin-bottom: 0.15rem;
}

.poster-item-title {
  font-size: 1.125rem;
  line-height: 1.375;
  font-weight: 700;
  margin-bottom: 0.25rem;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.poster-item-title--dense {
  font-size: 0.9rem;
  line-height: 1.35;
  margin-bottom: 0.15rem;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
}

.poster-item-summary {
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--sos-text-secondary);
  line-height: 1.625;
  font-family: "Noto Sans SC", sans-serif;
  text-align: justify;
  margin-top: 0.25rem;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.poster-item-summary--dense {
  font-size: 0.66rem;
  line-height: 1.35;
  margin-top: 0.1rem;
}

.poster-body--dense-half-year .poster-item-summary--dense {
  -webkit-line-clamp: 1;
}

.poster-body--dense-half-year .poster-item-title--dense {
  font-size: 0.82rem;
}

.poster-footer {
  margin-top: auto;
  padding: 2rem;
  padding-top: 0;
}

.poster-canvas--dense .poster-footer {
  padding: 1rem 1.25rem 1.25rem;
}

.poster-footer-inner {
  border-top: 2px solid var(--sos-text-primary);
  padding-top: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}

.poster-footer-brand {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  font-family: "Noto Serif SC", serif;
}

@media (max-width: 1100px) {
  .poster-canvas--half-year {
    width: 760px;
  }
  .poster-body--dense-half-year {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

/* ==================== Option Active/Inactive ==================== */
.opt-active {
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  border-color: var(--sos-text-primary);
}

.opt-inactive {
  background-color: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
  border-color: var(--sos-border-default);
}

.opt-inactive:hover {
  border-color: var(--sos-text-tertiary);
}

/* ==================== Form Elements ==================== */
.form-label {
  display: block;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  color: var(--sos-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.25rem;
}

.form-label-mb3 {
  display: block;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  color: var(--sos-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.75rem;
}
</style>
