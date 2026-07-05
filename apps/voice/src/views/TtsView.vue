<script setup>
// 语音合成：选角色 → 选语气参考 → 输入文本 → 合成 → 播放/下载。
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosField, SosSelect, SosSpinner } from '@haruhi/ui'
import ServiceBanner from '@/components/ServiceBanner.vue'
import AudioResult from '@/components/AudioResult.vue'
import { session, synthesize } from '@/api'
import { ensureRoles, refreshStatus, roles, status } from '@/lib/store'

const route = useRoute()
const router = useRouter()

const TEXT_MAX = 500

// 语种：中文标签 → TTS_infer_pack 代号
const LANGS = [
  { label: '日文', value: 'all_ja' },
  { label: '中文', value: 'all_zh' },
  { label: '中英混合', value: 'zh' },
  { label: '日英混合', value: 'ja' },
  { label: '英文', value: 'en' },
  { label: '多语种混合', value: 'auto' },
]

const character = ref('')
const refName = ref('')
const text = ref('')
const textLang = ref('all_ja')
const speed = ref(1.0)

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
    result.value = await synthesize({
      character: character.value,
      ref: refName.value,
      text: text.value,
      textLang: textLang.value,
      speed: Number(speed.value),
    })
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

const canSubmit = computed(
  () =>
    !busy.value &&
    status.ttsOnline &&
    character.value &&
    refName.value &&
    text.value.trim().length > 0 &&
    textCount.value <= TEXT_MAX,
)
</script>

<template>
  <div class="vo-page vo-page--work">
    <header class="vo-work-head">
      <h1 class="vo-work-head__title">语音合成</h1>
      <p class="vo-work-head__sub">让角色替你说出这句台词。选好角色与语气，输入文本即可。</p>
    </header>

    <ServiceBanner v-if="status.known && !status.ttsOnline" service="语音合成" />

    <div class="vo-workbench">
      <section class="vo-panel">
        <div class="vo-panel__row">
          <SosField label="角色" class="vo-panel__field">
            <SosSelect v-model="character" :options="characterOptions" :disabled="!roles.tts.length" />
          </SosField>
          <SosField label="语气参考" help="决定这句话的情绪与语气" class="vo-panel__field">
            <SosSelect v-model="refName" :options="refOptions" :disabled="!currentRefs.length" />
          </SosField>
        </div>

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
          <SosField label="文本语种" class="vo-panel__field">
            <SosSelect v-model="textLang" :options="LANGS" />
          </SosField>
          <SosField :label="`语速 ×${Number(speed).toFixed(2)}`" class="vo-panel__field">
            <input v-model.number="speed" type="range" class="vo-range" min="0.6" max="1.65" step="0.05" />
          </SosField>
        </div>

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
          <li>「语气参考」取自原作台词片段，直接决定情绪——换个语气常有惊喜。</li>
          <li>长文本会自动切句合成，耗时更久；500 字以内、一次一段效果最稳。</li>
          <li>同一时间只有一条合成通道，忙时请稍等片刻再提交。</li>
        </ul>
      </aside>
    </div>
  </div>
</template>
