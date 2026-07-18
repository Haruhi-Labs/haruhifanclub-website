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

    <section id="gallery-latest" class="curated-section curated-section--latest">
      <header class="curated-section__header">
        <div>
          <span>NEW ARRIVALS / 02</span>
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
          <span>POPULAR / 03</span>
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
import { computed } from 'vue'
import { ArrowUpRight, RefreshCw } from 'lucide-vue-next'
import ArtworkPreviewFrame from './ArtworkPreviewFrame.vue'
import JustifiedArtworkGrid from './JustifiedArtworkGrid.vue'

const props = defineProps({
  sections: { type: Object, required: true },
  loading: { type: Object, required: true },
  personalized: { type: Boolean, default: false },
})

const emit = defineEmits(['open', 'refresh', 'viewAll'])

const recommended = computed(() => props.sections.recommended || [])
const popular = computed(() => props.sections.popular || [])
const latest = computed(() => props.sections.latest || [])
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
  .curated-refresh {
    width: auto;
    min-height: 32px;
    gap: 5px;
    padding: 4px 0;
    background: transparent;
    border: 0;
    border-radius: 0;
  }
  .curated-refresh svg { display: block; flex: 0 0 auto; }
  .curated-refresh span { position: static; width: auto; height: auto; overflow: visible; clip: auto; }
  .curated-feature { margin-top: 12px; }
  .curated-section { padding-top: 58px; }
  .curated-section__header { margin-bottom: 12px; padding-bottom: 11px; }
  .curated-section__header h2 { font-size: 20px; }
  .curated-section__header span { margin-bottom: 3px; font-size: 9px; }
}

@media (prefers-reduced-motion: reduce) {
  .spinning { animation: none; }
}
</style>
