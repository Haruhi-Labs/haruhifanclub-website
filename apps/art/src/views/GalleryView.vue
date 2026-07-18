<template>
  <div class="gallery-page">
    <div v-if="store.sectionsError" class="gallery-error-box">{{ store.sectionsError }}</div>
    <CuratedGalleryHome
      :sections="store.sections"
      :creator-exhibits="store.creatorExhibits"
      :loading="store.sectionsLoading"
      :personalized="store.recommendationsPersonalized"
      @open="openItem"
      @refresh="store.refreshRecommendations()"
      @view-all="openCatalog"
      @creator="openCreatorPage"
    />
    <GalleryCatalog @open="openItem" />
  </div>
</template>

<script setup>
import { nextTick, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useGalleryStore } from '../stores/galleryStore.js'
import CuratedGalleryHome from '../components/CuratedGalleryHome.vue'
import GalleryCatalog from '../components/GalleryCatalog.vue'

const route = useRoute()
const router = useRouter()
const store = useGalleryStore()

function openItem(item) {
  router.push({ name: 'artwork-detail', params: { id: item.id } })
}

async function openCatalog({ category, range }) {
  await router.push({
    name: 'gallery',
    query: {
      category,
      range: category === 'popular' && range !== 'history' ? range : undefined,
    },
    hash: '#gallery-catalog',
  })
  await nextTick()
  window.requestAnimationFrame(() => {
    document.querySelector('#gallery-catalog')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  })
}

function openCreatorPage(author) {
  if (!author?.uid) return
  router.push({
    name: 'adventurer-profile',
    params: { uid: author.uid },
    query: { from: 'gallery' },
  })
}

onMounted(() => {
  store.loadSections()
  if (route.hash === '#gallery-catalog') {
    nextTick(() => document.querySelector('#gallery-catalog')?.scrollIntoView({ block: 'start' }))
  }
})
</script>

<style scoped>
.gallery-error-box {
  width: min(1480px, calc(100% - 80px));
  margin: 0 auto 16px;
  padding: 12px;
  color: #a31621;
  background: #fff0f1;
  border: 1px solid #ffc8cd;
  border-radius: 8px;
}

@media (max-width: 768px) {
  .gallery-error-box { width: calc(100% - 28px); }
}
</style>
