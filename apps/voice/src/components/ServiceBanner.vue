<script setup>
// 服务离线横幅：语音服务跑在团员本地机器上（经 frp 上服务器），停机属正常状态。
import { SosNotice } from '@haruhi/ui'

defineProps({
  /** 离线的服务名，如「语音合成」 */
  service: { type: String, required: true },
})
</script>

<template>
  <SosNotice tone="warning" class="vo-banner">
    <strong>{{ service }}服务当前离线</strong> ——
    本功能由团员本地算力驱动，机器不在线时暂不能提交；页面可正常浏览，请稍后再来试试。
  </SosNotice>
</template>
