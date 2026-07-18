<template>
  <div class="advanced-filter-bar">
    <button
      class="advanced-filter-bar__title"
      type="button"
      :aria-expanded="expanded"
      aria-controls="gallery-advanced-filter-controls"
      title="展开或收起高级筛选"
      @click="expanded = !expanded"
    >
      <Funnel :size="17" :stroke-width="2.3" aria-hidden="true" />
      <span>高级筛选</span>
      <ChevronDown
        class="advanced-filter-bar__chevron"
        :class="{ expanded }"
        :size="15"
        aria-hidden="true"
      />
    </button>

    <Transition name="advanced-filter">
      <div v-if="expanded" id="gallery-advanced-filter-controls" class="advanced-filter-bar__controls">
        <div class="filter-track" role="group" aria-label="内容筛选">
          <span
            v-if="content !== 'mix'"
            class="filter-track__slider"
            :class="{ 'pos-0': content === 'haruhi', 'pos-1': content === 'other' }"
          ></span>
          <button
            v-for="option in contentOptions"
            :key="option.value"
            type="button"
            :class="{ active: content === option.value }"
            :aria-pressed="content === option.value"
            @click="emit('update:content', content === option.value ? 'mix' : option.value)"
          >
            {{ option.label }}
          </button>
        </div>

        <div class="filter-track" role="group" aria-label="来源筛选">
          <span
            v-if="effectiveSource !== 'all'"
            class="filter-track__slider"
            :class="{ 'pos-0': effectiveSource === 'network', 'pos-1': effectiveSource === 'personal' }"
          ></span>
          <button
            v-for="option in sourceOptions"
            :key="option.value"
            type="button"
            :class="{ active: effectiveSource === option.value }"
            :disabled="sourceLocked"
            :aria-pressed="effectiveSource === option.value"
            :title="sourceLocked ? '个人分类固定展示个人作品' : undefined"
            @click="toggleSource(option.value)"
          >
            {{ option.label }}
          </button>
        </div>

        <div
          v-if="showTimeRange"
          class="filter-track filter-track--time"
          role="group"
          aria-label="人气时间范围"
        >
          <span
            class="filter-track__slider"
            :class="{
              'pos-0': timeRange === 'week',
              'pos-1': timeRange === 'year',
              'pos-2': timeRange === 'history'
            }"
          ></span>
          <button
            v-for="option in timeOptions"
            :key="option.value"
            type="button"
            :class="{ active: timeRange === option.value }"
            :aria-pressed="timeRange === option.value"
            @click="emit('update:timeRange', option.value)"
          >
            {{ option.label }}
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { ChevronDown, Funnel } from 'lucide-vue-next'

const props = defineProps({
  content: { type: String, default: 'mix' },
  sourceMode: { type: String, default: 'all' },
  sourceLocked: { type: Boolean, default: false },
  showTimeRange: { type: Boolean, default: false },
  timeRange: { type: String, default: 'history' }
})

const emit = defineEmits(['update:content', 'update:sourceMode', 'update:timeRange'])
const expanded = ref(false)
const contentOptions = [
  { value: 'haruhi', label: '凉宫' },
  { value: 'other', label: '其他' }
]
const sourceOptions = [
  { value: 'network', label: '网络' },
  { value: 'personal', label: '个人' }
]
const timeOptions = [
  { value: 'week', label: '一周' },
  { value: 'year', label: '一年' },
  { value: 'history', label: '历史' }
]
const effectiveSource = computed(() => props.sourceLocked ? 'personal' : props.sourceMode)

function toggleSource(value) {
  if (props.sourceLocked) return
  emit('update:sourceMode', props.sourceMode === value ? 'all' : value)
}
</script>

<style scoped>
.advanced-filter-bar {
  display: flex;
  align-items: center;
  gap: 14px;
}

.advanced-filter-bar__title {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 7px;
  color: var(--sos-text-secondary);
  font-size: 14px;
  font-weight: 800;
  white-space: nowrap;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.advanced-filter-bar__title:hover,
.advanced-filter-bar__title[aria-expanded='true'] { color: var(--sos-text-primary); }

.advanced-filter-bar__title:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--sos-accent) 42%, transparent);
  outline-offset: 4px;
  border-radius: 4px;
}

.advanced-filter-bar__chevron { transition: transform 0.22s ease; }
.advanced-filter-bar__chevron.expanded { transform: rotate(180deg); }

.advanced-filter-bar__controls {
  display: flex;
  align-items: center;
  gap: 9px;
}

.advanced-filter-enter-active,
.advanced-filter-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.advanced-filter-enter-from,
.advanced-filter-leave-to { opacity: 0; transform: translateX(-6px); }

.filter-track {
  position: relative;
  display: grid;
  grid-template-columns: repeat(2, minmax(66px, 1fr));
  width: 154px;
  padding: 3px;
  overflow: hidden;
  isolation: isolate;
  background: rgba(255, 255, 255, 0.62);
  border: 1px solid rgb(95, 215, 226);
  border-radius: 999px;
  box-shadow: 0 3px 12px rgba(36, 111, 126, 0.06);
}

.filter-track__slider {
  position: absolute;
  top: 3px;
  bottom: 3px;
  left: 3px;
  z-index: 1;
  width: calc((100% - 6px) / 2);
  pointer-events: none;
  background: rgb(186, 112, 235);
  border-radius: 999px;
  box-shadow: 0 4px 12px rgba(106, 45, 145, 0.18);
  transition: transform 0.25s cubic-bezier(0.34, 1.25, 0.64, 1);
}

.filter-track__slider.pos-0 { transform: translateX(0); }
.filter-track__slider.pos-1 { transform: translateX(100%); }
.filter-track__slider.pos-2 { transform: translateX(200%); }

.filter-track--time {
  grid-template-columns: repeat(3, minmax(56px, 1fr));
  width: 204px;
}

.filter-track--time .filter-track__slider { width: calc((100% - 6px) / 3); }

.filter-track button {
  position: relative;
  z-index: 2;
  min-width: 0;
  min-height: 34px;
  padding: 5px 8px;
  color: rgba(20, 25, 30, 0.62);
  font: inherit;
  font-size: 13px;
  font-weight: 850;
  white-space: nowrap;
  cursor: pointer;
  background: transparent;
  border: 0;
  border-radius: 999px;
}

.filter-track button.active { color: white; text-shadow: 0 1px 2px rgba(0, 0, 0, 0.18); }
.filter-track button:disabled { cursor: not-allowed; }
.filter-track button:disabled:not(.active) { opacity: 0.38; }

@media (max-width: 768px) {
  .advanced-filter-bar { display: contents; }
  .advanced-filter-bar__title {
    width: 36px;
    height: 36px;
    justify-content: center;
    padding: 0;
    color: var(--sos-text-secondary);
    background: var(--sos-bg-surface);
    border: 1px solid var(--sos-border-default);
    border-radius: 50%;
  }

  .advanced-filter-bar__title span,
  .advanced-filter-bar__chevron { display: none; }
  .advanced-filter-bar__controls { flex-wrap: wrap; width: 100%; gap: 8px; }
  .filter-track {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    width: calc(50% - 4px);
  }
  .filter-track--time {
    grid-template-columns: repeat(3, minmax(0, 1fr));
    width: 100%;
  }
}

@media (max-width: 360px) {
  .filter-track button { padding-inline: 4px; font-size: 12px; }
}

@media (prefers-reduced-motion: reduce) {
  .filter-track__slider,
  .advanced-filter-bar__chevron,
  .advanced-filter-enter-active,
  .advanced-filter-leave-active { transition: none; }
}
</style>
