<script setup>
// 单次转换工作台：上传 / 麦克风录音 两种来源 + 全量参数（RvcParams）。
import { computed, onBeforeUnmount, ref, reactive } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosField, SosSelect, SosSpinner, SosTabs } from '@haruhi/ui'
import AudioResult from '@/components/AudioResult.vue'
import RvcParams from '@/components/RvcParams.vue'
import { convert, session } from '@/api'
import { ensureRoles, refreshStatus, roles, status } from '@/lib/store'

const route = useRoute()
const router = useRouter()

const UPLOAD_MAX = 50 * 1024 * 1024

const SRC_MODES = [
  { value: 'upload', label: '上传文件' },
  { value: 'record', label: '麦克风录音' },
]

const role = ref('')
const srcMode = ref('upload')
const file = ref(null)
const params = ref({
  transpose: 0,
  format: 'wav',
  indexRate: 0.75,
  protect: 0.33,
  rmsMixRate: 0.25,
  filterRadius: 3,
  resampleSr: 0,
})

const busy = ref(false)
const error = ref('')
const result = ref(null)
const dragOver = ref(false)

// 录音状态
const rec = reactive({ recording: false, seconds: 0, previewUrl: '' })
let recorder = null
let recChunks = []
let recTimer = null

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
  setResultFile(f)
}

function setResultFile(f) {
  if (rec.previewUrl) URL.revokeObjectURL(rec.previewUrl)
  rec.previewUrl = ''
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

async function startRecording() {
  error.value = ''
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
    recorder = new MediaRecorder(stream)
    recChunks = []
    recorder.ondataavailable = (e) => e.data.size && recChunks.push(e.data)
    recorder.onstop = () => {
      const type = recorder.mimeType || 'audio/webm'
      const ext = type.includes('mp4') ? 'm4a' : type.includes('ogg') ? 'ogg' : 'webm'
      const blob = new Blob(recChunks, { type })
      const f = new File([blob], `麦克风录音.${ext}`, { type })
      setResultFile(f)
      rec.previewUrl = URL.createObjectURL(blob)
      stream.getTracks().forEach((t) => t.stop())
    }
    recorder.start()
    rec.recording = true
    rec.seconds = 0
    recTimer = setInterval(() => {
      rec.seconds += 1
      if (rec.seconds >= 300) stopRecording() // 录音上限 5 分钟
    }, 1000)
  } catch {
    error.value = '无法访问麦克风，请检查浏览器权限'
  }
}

function stopRecording() {
  if (recorder && rec.recording) recorder.stop()
  rec.recording = false
  clearInterval(recTimer)
}

onBeforeUnmount(() => {
  stopRecording()
  if (rec.previewUrl) URL.revokeObjectURL(rec.previewUrl)
})

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

  const p = params.value
  const fd = new FormData()
  fd.append('role', role.value)
  fd.append('transpose', String(p.transpose))
  fd.append('indexRate', String(p.indexRate))
  fd.append('protect', String(p.protect))
  fd.append('rmsMixRate', String(p.rmsMixRate))
  fd.append('filterRadius', String(p.filterRadius))
  fd.append('resampleSr', String(p.resampleSr))
  fd.append('format', p.format)
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
  <div class="vo-workbench">
    <section class="vo-panel">
      <div class="vo-panel__row">
        <SosField label="目标角色" class="vo-panel__field">
          <SosSelect v-model="role" :options="roleOptions" :disabled="!roles.rvc.length" />
        </SosField>
        <SosField label="音频来源" class="vo-panel__field">
          <SosTabs v-model="srcMode" :items="SRC_MODES" class="vo-refmode" />
        </SosField>
      </div>

      <SosField v-if="srcMode === 'upload'" label="源音频" help="支持 wav / mp3 / flac / m4a 等，上限 50MB；请使用无伴奏的干声">
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

      <SosField v-else label="麦克风录音" help="靠近麦克风、安静环境效果更好；录音上限 5 分钟">
        <div class="vo-recorder">
          <button
            type="button"
            class="sos-button"
            :class="rec.recording ? 'sos-button--danger' : 'sos-button--secondary'"
            @click="rec.recording ? stopRecording() : startRecording()"
          >
            <span v-if="rec.recording" class="vo-recorder__dot" aria-hidden="true"></span>
            {{ rec.recording ? `停止录音（${rec.seconds}s）` : '开始录音' }}
          </button>
          <audio v-if="rec.previewUrl" class="vo-recorder__preview" :src="rec.previewUrl" controls></audio>
          <span v-else-if="file && !rec.recording" class="vo-drop__name">{{ fileLabel }}</span>
        </div>
      </SosField>

      <RvcParams v-model="params" />

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

      <AudioResult :blob="result" :filename="`春日语音工坊_${role || 'rvc'}`" :ext="params.format" />
    </section>

    <aside class="vo-tips">
      <h2 class="vo-tips__title">小贴士</h2>
      <ul class="vo-tips__list">
        <li>输入请用<strong>干声</strong>（无伴奏无混响）；带伴奏的歌请先分离人声。</li>
        <li>整曲转换是长任务，可能要几分钟；同一时间只处理一个任务。</li>
        <li>转换结果保留你的唱腔与节奏，只替换音色——唱得越准，成品越好。</li>
        <li>音域差异大时调整「变调」；电音感明显时调低「检索特征占比」。</li>
      </ul>
    </aside>
  </div>
</template>
