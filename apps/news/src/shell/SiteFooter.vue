<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { SosSiteFooter, defaultSiteFooterGroups } from '@haruhi/ui'

const route = useRoute()
const show = computed(() => !route.name?.includes('editor') && !route.meta.hideNavbar)
const logoUrl = `${import.meta.env.BASE_URL}haruhi-logo-192.png`
const bottomLinks = [
  {
    label: '皖ICP备2025089290号-1',
    href: 'https://beian.miit.gov.cn/#/Integrated/index',
  },
]
</script>

<template>
  <SosSiteFooter
    v-if="show"
    :logo-src="logoUrl"
    :groups="defaultSiteFooterGroups"
    :bottom-links="bottomLinks"
  />
</template>
