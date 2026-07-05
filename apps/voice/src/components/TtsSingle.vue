<script setup>
// 单句合成工作台：预设语气（含试听）/ 自定义参考音频 两种参考来源 + 全量高级参数。
// 控件集合与 gradio WebUI 高级模式一一对应。
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosCheckbox, SosField, SosSelect, SosSpinner, SosTabs } from '@haruhi/ui'
import AudioResult from '@/components/AudioResult.vue'
import { refUrl, session, synthesize, synthesizeCustom } from '@/api'
import { ensureRoles, refreshStatus, roles, status } from '@/lib/store'
import { CUTS, LANGS } from '@/lib/options'

const route = useRoute()
const router = useRouter()

const TEXT_MAX = 500
const REF_MAX = 20 * 1024 * 1024
const AUX_MAX = 4

const REF_MODES = [
  { value: 'preset', label: '预设语气' },
  { value: 'custom', label: '自定义参考' },
]

const character = ref('')
const refMode = ref('preset')
const refName = ref('')

// 自定义参考
const customRef = ref(null) // File
const promptText = ref('')
const promptLang = ref('all_ja')
const refFree = ref(false)
const auxRefs = ref([]) // File[]

const text = ref('')
const textLang = ref('all_ja')
const speed = ref(1.0)

// 高级参数（默认值与 WebUI 相同）
const showAdvanced = ref(false)
const howToCut = ref('cut1')
const pauseSecond = ref(0.3)
const ifFreeze = ref(false)
const topK = ref(15)
const topP = ref(1.0)
const temperature = ref(1.0)

const busy = ref(false)
const error = ref('')
const result = ref(null)

ensureRoles().then(() => {
  if (!character.value && roles.tts.length) character.value = roles.tts[0].name
})

const currentRefs = computed(() => {
  const c = roles.tts.find((x) => x.name === character.value)
  return c?.refs || []
})

// 切角色时联动重置语气参考（默认第一条）
watch(
  currentRefs,
  (refs) => {
    if (!refs.includes(refName.value)) refName.value = refs[0] || ''
  },
  { immediate: true },
)

const characterOptions = computed(() => roles.tts.map((c) => ({ label: c.name, value: c.name })))
const refOptions = computed(() => currentRefs.value.map((r) => ({ label: r, value: r })))
const loggedIn = computed(() => !!session.state.user)
const textCount = computed(() => text.value.length)

/** 预设语气试听地址（随角色/语气联动） */
const previewUrl = computed(() =>
  refMode.value === 'preset' && character.value && refName.value
    ? refUrl(character.value, refName.value)
    : '',
)

function onCustomRef(e) {
  error.value = ''
  const f = e.target.files?.[0]
  e.target.value = ''
  if (!f) return
  if (f.size > REF_MAX) {
    error.value = '参考音频过大（上限 20MB）；请用 3~10 秒的干净人声'
    return
  }
  customRef.value = f
}

function onAuxRefs(e) {
  error.value = ''
  const files = Array.from(e.target.files || [])
  e.target.value = ''
  for (const f of files) {
    if (auxRefs.value.length >= AUX_MAX) break
    if (f.size > REF_MAX) {
      error.value = `多参考音频「${f.name}」过大（单个上限 20MB），已跳过`
      continue
    }
    auxRefs.value.push(f)
  }
}

function removeAux(i) {
  auxRefs.value.splice(i, 1)
}

function baseParams() {
  return {
    text: text.value,
    text_lang: textLang.value,
    speed: Number(speed.value),
    how_to_cut: howToCut.value,
    top_k: Number(topK.value),
    top_p: Number(topP.value),
    temperature: Number(temperature.value),
    pause_second: Number(pauseSecond.value),
    if_freeze: ifFreeze.value,
  }
}

async function submit() {
  if (!loggedIn.value) {
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (busy.value) return
  error.value = ''

  // 提交前强刷一次状态，避免拿 60s 前的旧「在线」
  await refreshStatus()
  if (!status.ttsOnline) return

  busy.value = true
  try {
    if (refMode.value === 'preset') {
      result.value = await synthesize({
        character: character.value,
        ref: refName.value,
        ref_free: refFree.value,
        ...baseParams(),
      })
    } else {
      const fd = new FormData()
      fd.append('character', character.value)
      fd.append('prompt_text', promptText.value)
      fd.append('prompt_lang', promptLang.value)
      fd.append('ref_free', String(refFree.value))
      const params = baseParams()
      for (const [k, v] of Object.entries(params)) fd.append(k, String(v))
      fd.append('ref_audio', customRef.value, customRef.value.name)
      for (const f of auxRefs.value) fd.append('aux_refs', f, f.name)
      result.value = await synthesizeCustom(fd)
    }
  } catch (e) {
    if (e?.status === 401) {
      router.push({ name: 'login', query: { redirect: route.fullPath } })
      return
    }
    if (e?.status === 503) {
      status.ttsOnline = false
      return
    }
    error.value = e?.message || '合成失败，请稍后再试'
  } finally {
    busy.value = false
  }
}

const canSubmit = computed(() => {
  if (busy.value || !status.ttsOnline || !character.value) return false
  if (!text.value.trim() || textCount.value > TEXT_MAX) return false
  if (refMode.value === 'preset') return !!refName.value
  return !!customRef.value && (refFree.value || promptText.value.trim().length > 0)
})
</script>

<template>
  <div class="vo-workbench">
    <section class="vo-panel">
      <div class="vo-panel__row">
        <SosField label="角色" class="vo-panel__field">
          <SosSelect v-model="character" :options="characterOptions" :disabled="!roles.tts.length" />
        </SosField>
        <SosField label="参考来源" class="vo-panel__field">
          <SosTabs v-model="refMode" :items="REF_MODES" class="vo-refmode" />
        </SosField>
      </div>

      <template v-if="refMode === 'preset'">
        <SosField label="语气参考" help="取自原作台词片段，直接决定这句话的情绪与语气">
          <SosSelect v-model="refName" :options="refOptions" :disabled="!currentRefs.length" />
        </SosField>
        <div v-if="previewUrl" class="vo-preview">
          <span class="vo-preview__label">试听语气</span>
          <audio class="vo-preview__player" :src="previewUrl" controls preload="none"></audio>
        </div>
      </template>

      <template v-else>
        <SosField label="参考音频" help="3~10 秒干净人声；角色模型仍由上方「角色」决定">
          <label class="vo-drop vo-drop--slim" :class="{ 'has-file': !!customRef }">
            <input type="file" accept="audio/*,.wav,.mp3,.flac,.m4a,.ogg,.aac" class="vo-drop__input" @change="onCustomRef" />
            <span v-if="customRef" class="vo-drop__name">{{ customRef.name }}（{{ (customRef.size / 1024 / 1024).toFixed(1) }} MB）</span>
            <span v-else class="vo-drop__hint">点击选择参考音频</span>
          </label>
        </SosField>
        <SosCheckbox v-model="refFree">开启无参考文本模式（不建议使用）</SosCheckbox>
        <div class="vo-panel__row" v-if="!refFree">
          <SosField label="参考音频的文本" class="vo-panel__field" help="参考音频里说的原话">
            <textarea v-model="promptText" class="sos-textarea" rows="2"></textarea>
          </SosField>
          <SosField label="参考音频的语种" class="vo-panel__field">
            <SosSelect v-model="promptLang" :options="LANGS" />
          </SosField>
        </div>
        <SosField label="多参考音频（可选，融合音色）" :help="`最多 ${AUX_MAX} 个，取多段参考的平均音色`">
          <label class="vo-drop vo-drop--slim">
            <input type="file" multiple accept="audio/*,.wav,.mp3,.flac,.m4a,.ogg,.aac" class="vo-drop__input" @change="onAuxRefs" />
            <span class="vo-drop__hint">点击添加（已选 {{ auxRefs.length }}/{{ AUX_MAX }}）</span>
          </label>
          <ul v-if="auxRefs.length" class="vo-auxlist">
            <li v-for="(f, i) in auxRefs" :key="f.name + i">
              <span class="vo-auxlist__name">{{ f.name }}</span>
              <button type="button" class="vo-auxlist__rm" @click="removeAux(i)" aria-label="移除">✕</button>
            </li>
          </ul>
        </SosField>
      </template>

      <SosField label="台词文本" :error="textCount > TEXT_MAX ? `超出 ${textCount - TEXT_MAX} 字` : ''">
        <textarea
          v-model="text"
          class="sos-textarea vo-panel__text"
          rows="5"
          :maxlength="TEXT_MAX + 50"
          placeholder="ただの人間には興味ありません。この中に宇宙人、未来人、異世界人、超能力者がいたら、あたしのところに来なさい。以上。"
        ></textarea>
      </SosField>
      <p class="vo-panel__count" :class="{ 'is-over': textCount > TEXT_MAX }">
        {{ textCount }} / {{ TEXT_MAX }}
      </p>

      <div class="vo-panel__row">
        <SosField label="合成语种" class="vo-panel__field">
          <SosSelect v-model="textLang" :options="LANGS" />
        </SosField>
        <SosField :label="`语速 ×${Number(speed).toFixed(2)}`" class="vo-panel__field">
          <input v-model.number="speed" type="range" class="vo-range" min="0.6" max="1.65" step="0.05" />
        </SosField>
      </div>

      <button type="button" class="vo-advanced-toggle" @click="showAdvanced = !showAdvanced">
        {{ showAdvanced ? '收起高级参数 ▲' : '高级参数（切句 / 停顿 / GPT 采样） ▼' }}
      </button>
      <template v-if="showAdvanced">
        <div class="vo-panel__row">
          <SosField label="切句方式" class="vo-panel__field">
            <SosSelect v-model="howToCut" :options="CUTS" />
          </SosField>
          <SosField :label="`句间停顿 ${Number(pauseSecond).toFixed(2)} 秒`" class="vo-panel__field">
            <input v-model.number="pauseSecond" type="range" class="vo-range" min="0.1" max="0.5" step="0.01" />
          </SosField>
        </div>
        <SosCheckbox v-model="ifFreeze">锁定语气随机性，仅调整上句音色</SosCheckbox>
        <div class="vo-panel__row vo-panel__row--three">
          <SosField :label="`top_k ${topK}`" class="vo-panel__field">
            <input v-model.number="topK" type="range" class="vo-range" min="1" max="100" step="1" />
          </SosField>
          <SosField :label="`top_p ${Number(topP).toFixed(2)}`" class="vo-panel__field">
            <input v-model.number="topP" type="range" class="vo-range" min="0" max="1" step="0.05" />
          </SosField>
          <SosField :label="`temperature ${Number(temperature).toFixed(2)}`" class="vo-panel__field">
            <input v-model.number="temperature" type="range" class="vo-range" min="0" max="1" step="0.05" />
          </SosField>
        </div>
      </template>

      <p v-if="error" class="vo-error" role="alert">{{ error }}</p>

      <div class="vo-panel__actions">
        <button
          type="button"
          class="sos-button sos-button--primary vo-submit"
          :disabled="loggedIn && !canSubmit"
          :aria-busy="busy || undefined"
          @click="submit"
        >
          <SosSpinner v-if="busy" label="合成中" />
          <template v-if="busy">合成中…（约需十几秒到一分钟）</template>
          <template v-else-if="!loggedIn">登录后合成</template>
          <template v-else>开始合成</template>
        </button>
        <span v-if="!loggedIn" class="vo-panel__hint">发起合成需要应援团统一账号</span>
      </div>

      <AudioResult :blob="result" :filename="`春日语音工坊_${character || 'tts'}`" />
    </section>

    <aside class="vo-tips">
      <h2 class="vo-tips__title">小贴士</h2>
      <ul class="vo-tips__list">
        <li>角色模型按日语台词训练，<strong>日文文本效果最佳</strong>；中文也可一试。</li>
        <li>「语气参考」可先试听再合成——换个语气常有惊喜。</li>
        <li>「自定义参考」用你自己的 3~10 秒音频定语气；须填参考音频的原话文本。</li>
        <li>长文本按「切句方式」自动分段；500 字以内、一次一段效果最稳。</li>
        <li>top_k / top_p / temperature 控制发挥度：越低越稳、越高越有变化。</li>
      </ul>
    </aside>
  </div>
</template>
