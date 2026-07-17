<template>
  <div class="curated-gallery">
    <header class="curated-intro">
      <div>
        <span class="curated-intro__index">CURATED / 01</span>
        <h1>{{ personalized ? '为你精选' : '今日精选' }}</h1>
      </div>
      <button
        class="curated-refresh"
        type="button"
        :disabled="loading.recommended"
        data-sfx="click"
        @click="emit('refresh')"
      >
        <RefreshCw
          :size="17"
          :stroke-width="2.2"
          :class="{ spinning: loading.recommended }"
          aria-hidden="true"
        />
        <span>换一批</span>
      </button>
    </header>

    <JustifiedArtworkGrid
      class="curated-feature"
      :items="recommended"
      :target-row-height="350"
      :mobile-target-row-height="155"
      :gap="14"
      :balanced-row-count="2"
      :min-row-height="380"
      :max-row-height="450"
      :mobile-min-row-height="175"
      :mobile-max-row-height="215"
      aria-label="推荐作品"
    >
      <template #default="{ item, index }">
        <ArtworkPreviewFrame
          :item="item"
          :position="index"
          show-label
          image-fit="cover"
          source="home-recommended"
          @open="emit('open', $event)"
        />
      </template>
    </JustifiedArtworkGrid>

    <section id="gallery-personal" class="curated-section curated-section--personal">
      <header class="curated-section__header">
        <div>
          <span>ARTISTS / 02</span>
          <h2>创作者展位</h2>
        </div>
        <button type="button" data-sfx="click" @click="openCatalog('personal')">
          查看全部 <ArrowUpRight :size="16" aria-hidden="true" />
        </button>
      </header>
      <div
        ref="creatorStage"
        class="creator-stage"
        @mouseenter="extendCreatorRotation"
        @focusin="extendCreatorRotation"
      >
        <Transition name="creator-batch" mode="out-in">
          <div :key="creatorBatchKey" class="curated-creators">
            <div
              v-for="(row, rowIndex) in creatorRows"
              :key="`${rowIndex}:${row.map(group => group.uid).join('|')}`"
              class="creator-row"
            >
              <article
                v-for="group in row"
                :key="group.uid"
                class="creator-exhibit"
                :style="creatorExhibitStyle(group)"
              >
                <header>
                  <button
                    class="creator-exhibit__identity"
                    type="button"
                    :aria-label="`在新标签页查看创作者 ${group.name} 的个人主页`"
                    @click="emit('creator', { uid: group.uid, name: group.name })"
                  >
                    <span class="creator-exhibit__avatar">
                      <img v-if="group.avatar" :src="group.avatar" alt="" />
                      <UserRound v-else :size="18" aria-hidden="true" />
                    </span>
                    <span class="creator-exhibit__name">
                      <strong>{{ group.name }}</strong>
                      <small>{{ group.items.length }} 件展出</small>
                    </span>
                  </button>
                </header>
                <JustifiedArtworkGrid
                  class="creator-exhibit__works"
                  :items="exhibitWorks(group)"
                  :target-row-height="320"
                  :mobile-target-row-height="240"
                  :gap="10"
                  :mobile-gap="8"
                  :mobile-pairs="false"
                  :max-row-height="360"
                  :mobile-max-row-height="280"
                  center-incomplete-rows
                  :aria-label="`${group.name} 的展出作品`"
                >
                  <template #default="{ item, index }">
                    <ArtworkPreviewFrame
                      :item="item"
                      :position="index"
                      show-label
                      source="home-personal"
                      @open="emit('open', $event)"
                    />
                  </template>
                </JustifiedArtworkGrid>
              </article>
            </div>
          </div>
        </Transition>
      </div>
    </section>

    <section id="gallery-latest" class="curated-section curated-section--latest">
      <header class="curated-section__header">
        <div>
          <span>NEW ARRIVALS / 03</span>
          <h2>最新发布</h2>
        </div>
        <button type="button" data-sfx="click" @click="openCatalog('latest')">
          查看全部 <ArrowUpRight :size="16" aria-hidden="true" />
        </button>
      </header>
      <JustifiedArtworkGrid
        class="curated-new-grid"
        :items="latest"
        :target-row-height="350"
        :mobile-target-row-height="155"
        :gap="14"
        :balanced-row-count="2"
        :min-row-height="380"
        :max-row-height="450"
        :mobile-min-row-height="175"
        :mobile-max-row-height="215"
      >
        <template #default="{ item, index }">
          <ArtworkPreviewFrame
            :item="item"
            :position="index"
            show-label
            image-fit="cover"
            source="home-latest"
            @open="emit('open', $event)"
          />
        </template>
      </JustifiedArtworkGrid>
    </section>

    <section id="gallery-popular" class="curated-section curated-section--popular">
      <header class="curated-section__header">
        <div>
          <span>POPULAR / 04</span>
          <h2>本周人气</h2>
        </div>
        <button type="button" data-sfx="click" @click="openCatalog('popular', 'week')">
          查看全部 <ArrowUpRight :size="16" aria-hidden="true" />
        </button>
      </header>
      <JustifiedArtworkGrid
        class="curated-ranking"
        :items="popular"
        :target-row-height="350"
        :mobile-target-row-height="155"
        :gap="14"
        :balanced-row-count="2"
        :min-row-height="380"
        :max-row-height="450"
        :mobile-min-row-height="175"
        :mobile-max-row-height="215"
      >
        <template #default="{ item, index }">
          <ArtworkPreviewFrame
            :item="item"
            :position="index"
            show-label
            image-fit="cover"
            source="home-popular-week"
            @open="emit('open', $event)"
          />
        </template>
      </JustifiedArtworkGrid>
    </section>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ArrowUpRight, RefreshCw, UserRound } from 'lucide-vue-next'
import ArtworkPreviewFrame from './ArtworkPreviewFrame.vue'
import JustifiedArtworkGrid from './JustifiedArtworkGrid.vue'

const props = defineProps({
  sections: { type: Object, required: true },
  creatorExhibits: { type: Array, default: () => [] },
  loading: { type: Object, required: true },
  personalized: { type: Boolean, default: false },
})

const emit = defineEmits(['open', 'refresh', 'viewAll', 'creator'])

const recommended = computed(() => props.sections.recommended || [])
const popular = computed(() => props.sections.popular || [])
const latest = computed(() => props.sections.latest || [])
const CREATOR_ROTATION_MS = 12_000
const CREATOR_HOVER_EXTENSION_MS = 6_000
const CREATOR_ROW_GAP = 42
const CREATOR_TARGET_MEDIA_HEIGHT = 250
const creatorStage = ref(null)
const creatorStageWidth = ref(0)
const creatorMobile = ref(false)
const creatorStart = ref(0)
const creatorCycle = ref(0)
let creatorTimer = 0
let creatorDeadline = 0
let creatorExtendedCycle = -1
let creatorMedia = null
let creatorResizeObserver = null

function circularCreatorGroups(count) {
  const all = props.creatorExhibits || []
  if (!all.length) return []
  return Array.from({ length: count }, (_, index) => all[(creatorStart.value + index) % all.length])
}

const creatorBatchKey = computed(() => (
  `${creatorCycle.value}:${creatorGroups.value.map(group => group.uid).join('|')}`
))

function exhibitWorks(group) {
  const items = Array.isArray(group?.items) ? group.items : []
  if (items.length <= 3) return items
  const offset = (creatorCycle.value * 3) % items.length
  return Array.from({ length: 3 }, (_, index) => items[(offset + index) % items.length])
}

function artworkRatio(item) {
  const width = Number(item?.image_width || item?.width || item?.images?.[0]?.width)
  const height = Number(item?.image_height || item?.height || item?.images?.[0]?.height)
  return width > 0 && height > 0 ? width / height : 1
}

function creatorExhibitDemand(group) {
  const items = exhibitWorks(group)
  const ratioSum = items.reduce((sum, item) => sum + artworkRatio(item), 0)
  const innerGaps = Math.max(0, items.length - 1) * 10
  return Math.min(1100, Math.max(220, ratioSum * CREATOR_TARGET_MEDIA_HEIGHT + innerGaps))
}

const creatorLayout = computed(() => {
  const total = props.creatorExhibits.length
  if (!total) return { groups: [], rows: [] }
  if (creatorMobile.value) {
    const groups = circularCreatorGroups(Math.min(2, total))
    return { groups, rows: groups.map(group => [group]) }
  }
  if (total === 1) {
    const groups = circularCreatorGroups(1)
    return { groups, rows: [groups] }
  }

  const width = creatorStageWidth.value || 1200
  const minCount = Math.min(2, total)
  const maxCount = Math.min(5, total)
  let best = null
  for (let count = minCount; count <= maxCount; count += 1) {
    const groups = circularCreatorGroups(count)
    for (let split = 1; split < count; split += 1) {
      const rows = [groups.slice(0, split), groups.slice(split)]
      const rowDemands = rows.map(row => (
        row.reduce((sum, group) => sum + creatorExhibitDemand(group), 0)
          + CREATOR_ROW_GAP * Math.max(0, row.length - 1)
      ))
      const scales = rowDemands.map(demand => width / Math.max(1, demand))
      const scaleCost = scales.reduce((sum, scale) => sum + Math.abs(Math.log(scale)), 0)
      const balanceCost = Math.abs(Math.log(scales[0] / scales[1]))
      const singletonCost = rows.reduce((sum, row) => {
        if (row.length !== 1) return sum
        return sum + (exhibitWorks(row[0]).length === 1 ? 1.35 : 0.08)
      }, 0)
      const crowdingCost = rows.reduce(
        (sum, row) => sum + Math.max(0, row.length - 3) * 0.35,
        0,
      )
      const score = scaleCost + balanceCost * 0.7 + singletonCost + crowdingCost - count * 0.02
      if (!best || score < best.score) best = { groups, rows, score }
    }
  }
  return best || { groups: circularCreatorGroups(minCount), rows: [] }
})

const creatorGroups = computed(() => creatorLayout.value.groups)
const creatorRows = computed(() => creatorLayout.value.rows)

function creatorExhibitStyle(group) {
  const demand = Math.round(creatorExhibitDemand(group))
  return {
    '--creator-grow': String(demand),
    '--creator-basis': `${demand}px`,
  }
}

function syncCreatorLayoutMode() {
  creatorMobile.value = Boolean(creatorMedia?.matches)
  const total = props.creatorExhibits.length
  if (total) creatorStart.value %= total
}

function resetCreatorPosition() {
  const total = props.creatorExhibits.length
  if (!total) {
    creatorStart.value = 0
    return
  }
  const elapsedBatches = Math.floor(Date.now() / CREATOR_ROTATION_MS)
  creatorStart.value = (elapsedBatches * (creatorMobile.value ? 2 : 4)) % total
}

function canRotateCreators() {
  const total = props.creatorExhibits.length
  if (!total) return false
  const hasMoreWorks = props.creatorExhibits.some(group => (group.items?.length || 0) > 3)
  return total > creatorGroups.value.length || hasMoreWorks
}

function scheduleCreatorRotation(delay = CREATOR_ROTATION_MS) {
  window.clearTimeout(creatorTimer)
  creatorDeadline = Date.now() + delay
  creatorTimer = window.setTimeout(rotateCreators, delay)
}

function rotateCreators() {
  if (document.hidden || !canRotateCreators()) {
    scheduleCreatorRotation()
    return
  }
  const total = props.creatorExhibits.length
  if (total > creatorGroups.value.length) {
    creatorStart.value = (creatorStart.value + Math.max(1, creatorGroups.value.length)) % total
  }
  creatorCycle.value += 1
  creatorExtendedCycle = -1
  scheduleCreatorRotation()
}

function extendCreatorRotation() {
  if (!canRotateCreators() || creatorExtendedCycle === creatorCycle.value) return
  creatorExtendedCycle = creatorCycle.value
  const remaining = Math.max(0, creatorDeadline - Date.now())
  scheduleCreatorRotation(remaining + CREATOR_HOVER_EXTENSION_MS)
}

watch(() => props.creatorExhibits, resetCreatorPosition)

onMounted(() => {
  creatorMedia = window.matchMedia('(max-width: 640px)')
  syncCreatorLayoutMode()
  resetCreatorPosition()
  creatorMedia.addEventListener?.('change', syncCreatorLayoutMode)
  creatorResizeObserver = new ResizeObserver(entries => {
    creatorStageWidth.value = entries[0]?.contentRect?.width || creatorStage.value?.clientWidth || 0
  })
  creatorResizeObserver.observe(creatorStage.value)
  scheduleCreatorRotation()
})

onBeforeUnmount(() => {
  window.clearTimeout(creatorTimer)
  creatorMedia?.removeEventListener?.('change', syncCreatorLayoutMode)
  creatorResizeObserver?.disconnect()
})

function openCatalog(category, range) {
  emit('viewAll', { category, range })
}
</script>

<style scoped>
.curated-gallery {
  width: min(1480px, calc(100% - 80px));
  margin: 0 auto;
  padding: 8px 0 120px;
}

.curated-intro {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) auto;
  align-items: end;
  gap: 28px;
  padding: 10px 0 22px;
  border-bottom: 1px solid var(--sos-border-default);
}

.curated-intro__index,
.curated-section__header span {
  display: block;
  margin-bottom: 6px;
  color: var(--sos-text-tertiary);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0;
}

.curated-intro h1,
.curated-section__header h2 {
  margin: 0;
  color: var(--sos-text-primary);
  font-weight: 900;
  letter-spacing: 0;
}

.curated-intro h1 { font-size: 32px; }

.curated-refresh {
  display: inline-flex;
  align-items: center;
  justify-self: end;
  gap: 7px;
  min-height: 38px;
  padding: 0 13px;
  color: var(--sos-text-secondary);
  font: inherit;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-default);
  border-radius: 999px;
}

.curated-refresh:hover:not(:disabled) { color: var(--sos-accent); border-color: var(--sos-accent); }
.curated-refresh:disabled { cursor: wait; opacity: 0.55; }

.curated-feature {
  margin-top: 20px;
}

.curated-section { padding-top: 96px; scroll-margin-top: 90px; }

.curated-section__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--sos-border-default);
}

.curated-section__header h2 { font-size: 25px; }

.curated-section__header button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--sos-text-secondary);
  font-size: 13px;
  font-weight: 750;
  text-decoration: none;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.curated-section__header button:hover { color: var(--sos-accent); }

.curated-creators {
  display: grid;
  gap: 34px;
}

.creator-row {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  gap: 42px;
}

.creator-stage { min-height: 1px; }
.creator-batch-enter-active,
.creator-batch-leave-active { transition: opacity 0.32s ease, transform 0.32s ease; }
.creator-batch-enter-from { opacity: 0; transform: translateY(6px); }
.creator-batch-leave-to { opacity: 0; transform: translateY(-4px); }

.creator-exhibit {
  min-width: 0;
  flex: var(--creator-grow) 1 var(--creator-basis);
}

.creator-exhibit > header {
  margin-bottom: 12px;
}

.creator-exhibit__identity {
  display: inline-flex;
  max-width: 100%;
  align-items: center;
  gap: 10px;
  padding: 0;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.creator-exhibit__identity:focus-visible {
  outline: 3px solid color-mix(in srgb, var(--sos-focus) 44%, transparent);
  outline-offset: 4px;
  border-radius: 6px;
}

.creator-exhibit__avatar {
  display: grid;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  place-items: center;
  overflow: hidden;
  color: var(--sos-text-secondary);
  background: var(--sos-bg-muted);
  border-radius: 50%;
}

.creator-exhibit__avatar img { width: 100%; height: 100%; object-fit: cover; }
.creator-exhibit__name { min-width: 0; }
.creator-exhibit__name strong,
.creator-exhibit__name small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.creator-exhibit header strong { color: var(--sos-text-primary); font-size: 14px; }
.creator-exhibit header small { margin-top: 2px; color: var(--sos-text-tertiary); font-size: 11px; }
.creator-exhibit__identity:hover strong { color: var(--sos-accent); }
.creator-exhibit__identity:hover .creator-exhibit__avatar {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--sos-accent) 34%, transparent);
}

.creator-exhibit__works {
  width: 100%;
}

.spinning { animation: curated-spin 0.8s linear infinite; }
@keyframes curated-spin { to { transform: rotate(360deg); } }

@media (max-width: 900px) {
  .curated-intro { grid-template-columns: 1fr auto; }
}

@media (max-width: 640px) {
  .curated-gallery { width: calc(100% - 28px); padding-top: 0; }
  .curated-intro { gap: 12px; padding-bottom: 14px; }
  .curated-intro h1 { font-size: 24px; }
  .curated-intro__index { font-size: 10px; }
  .curated-refresh { width: 38px; padding: 0; border-radius: 50%; }
  .curated-refresh span { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
  .curated-feature { margin-top: 12px; }
  .curated-section { padding-top: 58px; }
  .curated-section__header { margin-bottom: 12px; padding-bottom: 11px; }
  .curated-section__header h2 { font-size: 20px; }
  .curated-section__header span { margin-bottom: 3px; font-size: 9px; }
  .curated-creators { gap: 28px; }
  .creator-row { display: grid; grid-template-columns: 1fr; gap: 28px; }
  .creator-exhibit { width: 100%; min-width: 0; max-width: none; }
}

@media (prefers-reduced-motion: reduce) {
  .spinning { animation: none; }
  .creator-batch-enter-active,
  .creator-batch-leave-active { transition: none; }
}
</style>
