<template>
  <div class="gallery-page">
    <GalleryCatalog @open="openItem" />
  </div>
</template>

<script setup>
import { useRouter } from 'vue-router'
import GalleryCatalog from '../components/GalleryCatalog.vue'

defineOptions({ name: 'GalleryView' })

const router = useRouter()

function openItem(item) {
  router.push({ name: 'artwork-detail', params: { id: item.id } })
}
</script>
