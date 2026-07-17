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

  <ArtworkModal
    :model-value="modalOpen"
    :item="activeItem"
    @update:model-value="value => !value && closeModal()"
    @tag="searchTag"
    @author="openAuthor"
    @close="closeModal"
  />
</template>

<script setup>
import { nextTick, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useGalleryStore } from '../stores/galleryStore.js'
import ArtworkModal from '../components/ArtworkModal.vue'
import CuratedGalleryHome from '../components/CuratedGalleryHome.vue'
import GalleryCatalog from '../components/GalleryCatalog.vue'

const route = useRoute()
const router = useRouter()
const store = useGalleryStore()

const modalOpen = ref(false)
const activeItem = ref(null)

function openItem(item) {
  router.push({ query: { ...route.query, artwork: item.id } })
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

function closeModal() {
  const nextQuery = { ...route.query }
  delete nextQuery.artwork
  router.push({ query: nextQuery })
}

function searchTag(tag) {
  router.push({ name: 'gallery-search', query: { q: tag, field: 'tag' } })
}

function openAuthor(author) {
  if (!author?.uid) return
  router.push({ name: 'adventurer-profile', params: { uid: author.uid } })
}

function openCreatorPage(author) {
  if (!author?.uid) return
  const target = router.resolve({
    name: 'adventurer-profile',
    params: { uid: author.uid },
    query: { from: 'gallery' },
  })
  window.open(target.href, '_blank', 'noopener,noreferrer')
}

async function syncArtwork(id) {
  if (!id) {
    modalOpen.value = false
    activeItem.value = null
    return
  }

  const item = await store.fetchArtworkById(id)
  if (String(route.query.artwork || '') !== String(id)) return
  if (item) {
    activeItem.value = item
    modalOpen.value = true
  }
}

watch(() => route.query.artwork, syncArtwork)

onMounted(() => {
  store.loadSections()
  syncArtwork(route.query.artwork)
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
