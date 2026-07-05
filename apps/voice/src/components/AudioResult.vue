<script setup>
// 合成/转换结果：播放器 + 下载。blob URL 随组件卸载或结果更换时释放。
import { ref, watch, onBeforeUnmount } from 'vue'

const props = defineProps({
  /** 结果音频 Blob；为 null 时不渲染 */
  blob: { type: Object, default: null },
  /** 下载文件名（不含扩展名） */
  filename: { type: String, default: '春日语音工坊' },
})

const url = ref('')

watch(
  () => props.blob,
  (blob) => {
    if (url.value) URL.revokeObjectURL(url.value)
    url.value = blob ? URL.createObjectURL(blob) : ''
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  if (url.value) URL.revokeObjectURL(url.value)
})
</script>

<template>
  <div v-if="url" class="vo-result">
    <p class="vo-result__title">
      <span class="vo-result__dot" aria-hidden="true"></span>生成完成
    </p>
    <audio class="vo-result__player" :src="url" controls preload="metadata"></audio>
    <a class="sos-button sos-button--secondary vo-result__download" :href="url" :download="`${filename}.wav`">
      下载 WAV
    </a>
  </div>
</template>
