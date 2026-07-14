<script setup>
// 语音合成：单句合成 / 多句拼接 两种模式（对应 gradio WebUI 的高级模式与 Batch Tab）。
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosTabs } from '@haruhi/ui'
import ServiceBanner from '@/components/ServiceBanner.vue'
import TtsSingle from '@/components/TtsSingle.vue'
import TtsBatch from '@/components/TtsBatch.vue'
import { status } from '@/lib/store'

const route = useRoute()
const router = useRouter()

const TAB_ITEMS = [
  { value: 'single', label: '单句合成' },
  { value: 'batch', label: '多句拼接' },
]
const tab = ref(route.query.tab === 'batch' ? 'batch' : 'single')

// tab 状态进 URL，可分享/回退
watch(tab, (v) => {
  router.replace({ query: { ...route.query, tab: v === 'batch' ? 'batch' : undefined } })
})
</script>

<template>
  <div class="vo-page vo-page--work">
    <header class="vo-work-head">
      <h1 class="vo-work-head__title">语音合成</h1>
      <p class="vo-work-head__sub">让角色替你说出这句台词——单句精调，或多句多角色拼成一段对话。</p>
    </header>

    <ServiceBanner v-if="status.known && !status.ttsOnline" service="语音合成" />

    <SosTabs v-model="tab" :items="TAB_ITEMS" class="vo-mode-tabs" />

    <TtsSingle v-if="tab === 'single'" />
    <TtsBatch v-else />
  </div>
</template>
