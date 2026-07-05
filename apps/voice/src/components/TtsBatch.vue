<script setup>
// 多句拼接：多角色多语气的台词列表 → 一次合成拼接成完整音频。
// 对应 gradio WebUI「多句拼接 (Batch)」Tab：添加表单 + 可编辑列表 + 撤销/清空 + 全局设置。
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosField, SosSelect, SosSpinner } from '@haruhi/ui'
import AudioResult from '@/components/AudioResult.vue'
import { session, synthesizeBatch } from '@/api'
import { ensureRoles, refreshStatus, roles, status } from '@/lib/store'
import { LANGS } from '@/lib/options'

const route = useRoute()
const router = useRouter()

const MAX_ITEMS = 60
const MAX_CHARS = 4000

// 添加表单
const addChar = ref('')
const addRef = ref('')
const addText = ref('')
const addInterval = ref(0.2)

// 列表：[{character, ref, text, interval}]
const items = ref([])

// 全局设置（与 gradio Batch Tab 一致）
const globalInterval = ref(0.1)
const textLang = ref('all_ja')
const speed = ref(1.0)

const busy = ref(false)
const error = ref('')
const result = ref(null)

ensureRoles().then(() => {
  if (!addChar.value && roles.tts.length) addChar.value = roles.tts[0].name
})

const addRefs = computed(() => {
  const c = roles.tts.find((x) => x.name === addChar.value)
  return c?.refs || []
})
watch(
  addRefs,
  (refs) => {
    if (!refs.includes(addRef.value)) addRef.value = refs[0] || ''
  },
  { immediate: true },
)

const characterOptions = computed(() => roles.tts.map((c) => ({ label: c.name, value: c.name })))
const addRefOptions = computed(() => addRefs.value.map((r) => ({ label: r, value: r })))
const loggedIn = computed(() => !!session.state.user)
const totalChars = computed(() => items.value.reduce((n, it) => n + it.text.length, 0))

/** 行内编辑时某行角色变了，语气不在新角色的列表里则回退到第一条 */
function refsOf(charName) {
  const c = roles.tts.find((x) => x.name === charName)
  return c?.refs || []
}
function onRowCharChange(row) {
  const refs = refsOf(row.character)
  if (!refs.includes(row.ref)) row.ref = refs[0] || ''
}

function addItem() {
  error.value = ''
  if (!addText.value.trim()) {
    error.value = '本句文本不能为空'
    return
  }
  if (items.value.length >= MAX_ITEMS) {
    error.value = `句子已达上限（${MAX_ITEMS} 句）`
    return
  }
  items.value.push({
    character: addChar.value,
    ref: addRef.value,
    text: addText.value.trim(),
    interval: Number(addInterval.value),
  })
  addText.value = ''
}

function removeItem(i) {
  items.value.splice(i, 1)
}
function undoLast() {
  items.value.pop()
}
function clearAll() {
  items.value = []
}

async function submit() {
  if (!loggedIn.value) {
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (busy.value || !items.value.length) return
  error.value = ''

  await refreshStatus()
  if (!status.ttsOnline) return

  busy.value = true
  try {
    result.value = await synthesizeBatch({
      items: items.value.map((it) => ({
        character: it.character,
        ref: it.ref,
        text: it.text,
        interval: Number(it.interval) || 0,
      })),
      text_lang: textLang.value,
      speed: Number(speed.value),
      global_interval: Number(globalInterval.value),
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
    error.value = e?.message || '拼接合成失败，请稍后再试'
  } finally {
    busy.value = false
  }
}

const canSubmit = computed(
  () =>
    !busy.value &&
    status.ttsOnline &&
    items.value.length > 0 &&
    totalChars.value <= MAX_CHARS &&
    items.value.every((it) => it.text.trim() && it.character && it.ref),
)
</script>

<template>
  <div class="vo-workbench vo-workbench--wide">
    <section class="vo-panel">
      <h2 class="vo-panel__subhead">1 · 添加句子</h2>
      <div class="vo-panel__row">
        <SosField label="角色" class="vo-panel__field">
          <SosSelect v-model="addChar" :options="characterOptions" :disabled="!roles.tts.length" />
        </SosField>
        <SosField label="语气参考" class="vo-panel__field">
          <SosSelect v-model="addRef" :options="addRefOptions" :disabled="!addRefs.length" />
        </SosField>
      </div>
      <SosField label="本句文本">
        <textarea v-model="addText" class="sos-textarea" rows="2" placeholder="输入这句话的内容…" @keydown.enter.exact.prevent="addItem"></textarea>
      </SosField>
      <div class="vo-panel__row">
        <SosField :label="`本句后停顿 ${Number(addInterval).toFixed(1)} 秒`" class="vo-panel__field">
          <input v-model.number="addInterval" type="range" class="vo-range" min="0" max="2" step="0.1" />
        </SosField>
        <div class="vo-panel__field vo-batch-addwrap">
          <button type="button" class="sos-button sos-button--secondary" @click="addItem">添加到列表</button>
        </div>
      </div>

      <h2 class="vo-panel__subhead">
        2 · 台词列表
        <span class="vo-batch-meta">{{ items.length }}/{{ MAX_ITEMS }} 句 · {{ totalChars }}/{{ MAX_CHARS }} 字</span>
      </h2>
      <p v-if="!items.length" class="vo-batch-empty">还没有句子——在上方添加，或多角色对话就换着角色加。</p>
      <ol v-else class="vo-batch-list">
        <li v-for="(row, i) in items" :key="i" class="vo-batch-row">
          <span class="vo-batch-row__n">{{ i + 1 }}</span>
          <div class="vo-batch-row__fields">
            <div class="vo-batch-row__selects">
              <SosSelect v-model="row.character" :options="characterOptions" @update:modelValue="onRowCharChange(row)" />
              <SosSelect v-model="row.ref" :options="refsOf(row.character).map((r) => ({ label: r, value: r }))" />
              <label class="vo-batch-row__interval">
                停顿
                <input v-model.number="row.interval" type="number" min="0" max="2" step="0.1" class="sos-input" />
                秒
              </label>
            </div>
            <textarea v-model="row.text" class="sos-textarea vo-batch-row__text" rows="1"></textarea>
          </div>
          <button type="button" class="vo-auxlist__rm" @click="removeItem(i)" aria-label="删除本句">✕</button>
        </li>
      </ol>
      <div v-if="items.length" class="vo-batch-listactions">
        <button type="button" class="sos-button sos-button--ghost" @click="undoLast">撤销上一条</button>
        <button type="button" class="sos-button sos-button--ghost" @click="clearAll">清空列表</button>
      </div>

      <h2 class="vo-panel__subhead">3 · 全局设置与合成</h2>
      <div class="vo-panel__row vo-panel__row--three">
        <SosField :label="`全局额外间隔 ${Number(globalInterval).toFixed(1)} 秒`" class="vo-panel__field">
          <input v-model.number="globalInterval" type="range" class="vo-range" min="0" max="2" step="0.1" />
        </SosField>
        <SosField label="合成语种" class="vo-panel__field">
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
          <SosSpinner v-if="busy" label="拼接合成中" />
          <template v-if="busy">拼接合成中…（{{ items.length }} 句，可能需要几分钟）</template>
          <template v-else-if="!loggedIn">登录后合成</template>
          <template v-else>开始多句拼接合成</template>
        </button>
      </div>

      <AudioResult :blob="result" filename="春日语音工坊_多句拼接" />
    </section>

    <aside class="vo-tips">
      <h2 class="vo-tips__title">小贴士</h2>
      <ul class="vo-tips__list">
        <li>不同句可以用<strong>不同角色与语气</strong>——直接拼出一段对话或小剧场。</li>
        <li>「本句后停顿」控制这句话之后的间隔；「全局额外间隔」在每句后再统一加。</li>
        <li>列表里的角色、语气、文本、停顿都可以直接改。</li>
        <li>句子越多耗时越久（每句约 5~20 秒），提交后请耐心等待。</li>
      </ul>
    </aside>
  </div>
</template>
