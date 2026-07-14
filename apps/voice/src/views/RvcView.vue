<script setup>
// 声线转换：单次转换 / 批量转换 两种模式（对应 gradio WebUI 的单次/批量推理 Tab）。
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosTabs } from '@haruhi/ui'
import ServiceBanner from '@/components/ServiceBanner.vue'
import RvcSingle from '@/components/RvcSingle.vue'
import RvcBatch from '@/components/RvcBatch.vue'
import { status } from '@/lib/store'

const route = useRoute()
const router = useRouter()

const TAB_ITEMS = [
  { value: 'single', label: '单次转换' },
  { value: 'batch', label: '批量转换' },
]
const tab = ref(route.query.tab === 'batch' ? 'batch' : 'single')

watch(tab, (v) => {
  router.replace({ query: { ...route.query, tab: v === 'batch' ? 'batch' : undefined } })
})
</script>

<template>
  <div class="vo-page vo-page--work">
    <header class="vo-work-head">
      <h1 class="vo-work-head__title">声线转换</h1>
      <p class="vo-work-head__sub">把你的清唱或语音，变成角色的声线。人声越干净，效果越好。</p>
    </header>

    <ServiceBanner v-if="status.known && !status.rvcOnline" service="声线转换" />

    <SosTabs v-model="tab" :items="TAB_ITEMS" class="vo-mode-tabs" />

    <RvcSingle v-if="tab === 'single'" />
    <RvcBatch v-else />
  </div>
</template>
