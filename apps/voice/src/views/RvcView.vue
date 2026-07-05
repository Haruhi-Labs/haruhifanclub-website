<script setup>
// 声线转换：选角色 → 上传音频 → 变调/高级参数 → 转换 → 播放/下载。
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosField, SosSelect, SosSpinner } from '@haruhi/ui'
import ServiceBanner from '@/components/ServiceBanner.vue'
import AudioResult from '@/components/AudioResult.vue'
import { convert, session } from '@/api'
import { ensureRoles, refreshStatus, roles, status } from '@/lib/store'

const route = useRoute()
const router = useRouter()

const UPLOAD_MAX = 50 * 1024 * 1024

const role = ref('')
const file = ref(null)
const transpose = ref(0)
const showAdvanced = ref(false)
const indexRate = ref(0.75)
const protect = ref(0.33)

const busy = ref(false)
const error = ref('')
const result = ref(null)
const dragOver = ref(false)

ensureRoles().then(() => {
  if (!role.value && roles.rvc.length) role.value = roles.rvc[0].name
})

const roleOptions = computed(() => roles.rvc.map((r) => ({ label: r.name, value: r.name })))
const loggedIn = computed(() => !!session.state.user)

function pickFile(f) {
  error.value = ''
  if (!f) return
  if (f.size > UPLOAD_MAX) {
    error.value = '音频文件过大（上限 50MB），请先压缩或裁剪'
    return
  }
  file.value = f
}

function onFileInput(e) {
  pickFile(e.target.files?.[0])
  e.target.value = '' // 允许重选同一文件
}

function onDrop(e) {
  dragOver.value = false
  pickFile(e.dataTransfer?.files?.[0])
}

const fileLabel = computed(() => {
  if (!file.value) return ''
  const mb = (file.value.size / 1024 / 1024).toFixed(1)
  return `${file.value.name}（${mb} MB）`
})

async function submit() {
  if (!loggedIn.value) {
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (busy.value || !file.value) return
  error.value = ''

  await refreshStatus()
  if (!status.rvcOnline) return

  const fd = new FormData()
  fd.append('role', role.value)
  fd.append('transpose', String(transpose.value))
  fd.append('indexRate', String(indexRate.value))
  fd.append('protect', String(protect.value))
  fd.append('audio', file.value, file.value.name)

  busy.value = true
  try {
    result.value = await convert(fd)
  } catch (e) {
    if (e?.status === 401) {
      router.push({ name: 'login', query: { redirect: route.fullPath } })
      return
    }
    if (e?.status === 503) {
      status.rvcOnline = false
      return
    }
    error.value = e?.message || '转换失败，请稍后再试'
  } finally {
    busy.value = false
  }
}

const canSubmit = computed(() => !busy.value && status.rvcOnline && role.value && !!file.value)
</script>

<template>
  <div class="vo-page vo-page--work">
    <header class="vo-work-head">
      <h1 class="vo-work-head__title">声线转换</h1>
      <p class="vo-work-head__sub">把你的清唱或语音，变成角色的声线。人声越干净，效果越好。</p>
    </header>

    <ServiceBanner v-if="status.known && !status.rvcOnline" service="声线转换" />

    <div class="vo-workbench">
      <section class="vo-panel">
        <SosField label="目标角色" class="vo-panel__field">
          <SosSelect v-model="role" :options="roleOptions" :disabled="!roles.rvc.length" />
        </SosField>

        <SosField label="源音频" help="支持 wav / mp3 / flac / m4a 等，上限 50MB；请使用无伴奏的干声">
          <label
            class="vo-drop"
            :class="{ 'is-over': dragOver, 'has-file': !!file }"
            @dragover.prevent="dragOver = true"
            @dragleave="dragOver = false"
            @drop.prevent="onDrop"
          >
            <input type="file" accept="audio/*,.wav,.mp3,.flac,.m4a,.ogg,.aac" class="vo-drop__input" @change="onFileInput" />
            <svg class="vo-drop__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M12 16V4m0 0 4 4m-4-4L8 8M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" />
            </svg>
            <span v-if="file" class="vo-drop__name">{{ fileLabel }}</span>
            <span v-else class="vo-drop__hint">点击选择或拖拽音频到这里</span>
          </label>
        </SosField>

        <SosField :label="`变调 ${transpose > 0 ? '+' : ''}${transpose} 半音`" help="男声转女声一般 +12，女声转男声 -12，同性 0">
          <input v-model.number="transpose" type="range" class="vo-range" min="-12" max="12" step="1" />
        </SosField>

        <button type="button" class="vo-advanced-toggle" @click="showAdvanced = !showAdvanced">
          {{ showAdvanced ? '收起高级参数 ▲' : '高级参数 ▼' }}
        </button>
        <div v-if="showAdvanced" class="vo-panel__row">
          <SosField :label="`检索特征占比 ${Number(indexRate).toFixed(2)}`" help="越高越贴角色音色，过高可能失真" class="vo-panel__field">
            <input v-model.number="indexRate" type="range" class="vo-range" min="0" max="1" step="0.05" />
          </SosField>
          <SosField :label="`咬字保护 ${Number(protect).toFixed(2)}`" help="保护清辅音与呼吸声，0.5 = 关闭" class="vo-panel__field">
            <input v-model.number="protect" type="range" class="vo-range" min="0" max="0.5" step="0.01" />
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
            <SosSpinner v-if="busy" label="转换中" />
            <template v-if="busy">转换中…（整曲可能需要数分钟，请勿关闭页面）</template>
            <template v-else-if="!loggedIn">登录后转换</template>
            <template v-else>开始转换</template>
          </button>
          <span v-if="!loggedIn" class="vo-panel__hint">发起转换需要应援团统一账号</span>
        </div>

        <AudioResult :blob="result" :filename="`春日语音工坊_${role || 'rvc'}`" />
      </section>

      <aside class="vo-tips">
        <h2 class="vo-tips__title">小贴士</h2>
        <ul class="vo-tips__list">
          <li>输入请用<strong>干声</strong>（无伴奏无混响）；带伴奏的歌请先分离人声。</li>
          <li>整曲转换是长任务，可能要几分钟；同一时间只处理一个任务，忙时会提示稍后再试。</li>
          <li>转换结果保留你的唱腔与节奏，只替换音色——唱得越准，成品越好。</li>
          <li>音域差异大时调整「变调」；电音感明显时调低「检索特征占比」。</li>
        </ul>
      </aside>
    </div>
  </div>
</template>
