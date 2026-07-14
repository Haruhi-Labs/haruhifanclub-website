<script setup>
// RVC 参数控件组（单次/批量共用）：变调 + 导出格式 + 折叠高级参数。
// 参数集合与 gradio WebUI 一致：检索特征占比 / 清辅音保护 / 音量包络融合 /
// harvest 中值滤波半径 / 后处理重采样。音高提取算法固定 rmvpe（WebUI 亦仅此一项）。
import { ref } from 'vue'
import { SosCheckbox, SosField } from '@haruhi/ui'
import { RVC_FORMATS } from '@/lib/options'

const params = defineModel({
  type: Object,
  default: () => ({
    transpose: 0,
    format: 'wav',
    indexRate: 0.75,
    protect: 0.33,
    rmsMixRate: 0.25,
    filterRadius: 3,
    resampleSr: 0,
  }),
})

const showAdvanced = ref(false)
</script>

<template>
  <div class="vo-rvc-params">
    <SosField :label="`变调 ${params.transpose > 0 ? '+' : ''}${params.transpose} 半音`" help="男声转女声一般 +12，女声转男声 -12，同性 0">
      <input v-model.number="params.transpose" type="range" class="vo-range" min="-12" max="12" step="1" />
    </SosField>

    <SosField label="导出格式">
      <div class="vo-format-group" role="radiogroup">
        <SosCheckbox
          v-for="f in RVC_FORMATS"
          :key="f"
          v-model="params.format"
          type="radio"
          :value="f"
          name="rvc-format"
        >
          {{ f }}
        </SosCheckbox>
      </div>
    </SosField>

    <button type="button" class="vo-advanced-toggle" @click="showAdvanced = !showAdvanced">
      {{ showAdvanced ? '收起高级参数 ▲' : '高级参数（检索 / 保护 / 包络 / 滤波 / 重采样） ▼' }}
    </button>
    <template v-if="showAdvanced">
      <div class="vo-panel__row">
        <SosField :label="`检索特征占比 ${Number(params.indexRate).toFixed(2)}`" help="越高越贴角色音色，过高可能失真" class="vo-panel__field">
          <input v-model.number="params.indexRate" type="range" class="vo-range" min="0" max="1" step="0.05" />
        </SosField>
        <SosField :label="`清辅音/呼吸保护 ${Number(params.protect).toFixed(2)}`" help="防电音撕裂，0.5 = 关闭保护" class="vo-panel__field">
          <input v-model.number="params.protect" type="range" class="vo-range" min="0" max="0.5" step="0.01" />
        </SosField>
      </div>
      <div class="vo-panel__row">
        <SosField :label="`音量包络融合 ${Number(params.rmsMixRate).toFixed(2)}`" help="0=保留输入的响度起伏，1=完全用输出包络" class="vo-panel__field">
          <input v-model.number="params.rmsMixRate" type="range" class="vo-range" min="0" max="1" step="0.05" />
        </SosField>
        <SosField :label="`中值滤波半径 ${params.filterRadius}`" help="≥3 时对音高做中值滤波，可减轻哑音" class="vo-panel__field">
          <input v-model.number="params.filterRadius" type="range" class="vo-range" min="0" max="7" step="1" />
        </SosField>
      </div>
      <SosField :label="`后处理重采样 ${params.resampleSr === 0 ? '关闭' : params.resampleSr + ' Hz'}`" help="0 = 不重采样">
        <input v-model.number="params.resampleSr" type="range" class="vo-range" min="0" max="48000" step="1000" />
      </SosField>
      <p class="vo-rvc-f0note">音高提取算法：rmvpe（当前最佳，固定使用）</p>
    </template>
  </div>
</template>
