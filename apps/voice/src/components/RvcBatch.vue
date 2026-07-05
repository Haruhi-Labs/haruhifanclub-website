<script setup>
// 批量转换：多文件队列，逐个串行转换（尊重服务端单并发与冷却），逐个下载。
// 对应 gradio WebUI「批量推理」Tab（web 版输出直接回到浏览器，无需服务器输出文件夹）。
import { computed, onBeforeUnmount, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosField, SosSelect, SosSpinner } from '@haruhi/ui'
import RvcParams from '@/components/RvcParams.vue'
import { convert, session } from '@/api'
import { ensureRoles, refreshStatus, roles, status } from '@/lib/store'

const route = useRoute()
const router = useRouter()

const UPLOAD_MAX = 50 * 1024 * 1024
const QUEUE_MAX = 20

const role = ref('')
const params = ref({
  transpose: 0,
  format: 'wav',
  indexRate: 0.75,
  protect: 0.33,
  rmsMixRate: 0.25,
  filterRadius: 3,
  resampleSr: 0,
})

/** 队列行：{ file, state: 'pending'|'running'|'done'|'error', url, message } */
const queue = ref([])
const running = ref(false)
const error = ref('')
let cancelled = false

ensureRoles().then(() => {
  if (!role.value && roles.rvc.length) role.value = roles.rvc[0].name
})

const roleOptions = computed(() => roles.rvc.map((r) => ({ label: r.name, value: r.name })))
const loggedIn = computed(() => !!session.state.user)
const doneCount = computed(() => queue.value.filter((r) => r.state === 'done').length)

function onFiles(e) {
  error.value = ''
  const files = Array.from(e.target.files || [])
  e.target.value = ''
  for (const f of files) {
    if (queue.value.length >= QUEUE_MAX) {
      error.value = `队列已满（上限 ${QUEUE_MAX} 个文件）`
      break
    }
    if (f.size > UPLOAD_MAX) {
      error.value = `「${f.name}」过大（上限 50MB），已跳过`
      continue
    }
    queue.value.push({ file: f, state: 'pending', url: '', message: '' })
  }
}

function removeRow(i) {
  const row = queue.value[i]
  if (row.state === 'running') return
  if (row.url) URL.revokeObjectURL(row.url)
  queue.value.splice(i, 1)
}

function clearQueue() {
  if (running.value) return
  queue.value.forEach((r) => r.url && URL.revokeObjectURL(r.url))
  queue.value = []
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms))

async function convertRow(row) {
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
  fd.append('audio', row.file, row.file.name)
  return convert(fd)
}

async function start() {
  if (!loggedIn.value) {
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (running.value || !queue.value.some((r) => r.state === 'pending')) return
  error.value = ''
  cancelled = false
  running.value = true

  try {
    for (const row of queue.value) {
      if (cancelled) break
      if (row.state !== 'pending') continue

      await refreshStatus()
      if (!status.rvcOnline) {
        row.state = 'error'
        row.message = '服务离线'
        break
      }

      row.state = 'running'
      try {
        const blob = await convertRow(row)
        row.url = URL.createObjectURL(blob)
        row.state = 'done'
      } catch (e) {
        if (e?.status === 429) {
          // 撞上冷却/占用：等一轮冷却后重试一次
          row.message = '排队等待中…'
          await sleep((status.cooldownSecs + 2) * 1000)
          try {
            const blob = await convertRow(row)
            row.url = URL.createObjectURL(blob)
            row.state = 'done'
            row.message = ''
          } catch (e2) {
            row.state = 'error'
            row.message = e2?.message || '转换失败'
          }
        } else if (e?.status === 401) {
          router.push({ name: 'login', query: { redirect: route.fullPath } })
          return
        } else {
          row.state = 'error'
          row.message = e?.message || '转换失败'
          if (e?.status === 503) {
            status.rvcOnline = false
            break
          }
        }
      }

      // 两个任务之间等一轮冷却（服务端 cooldownSecs），避免整队 429
      const hasNext = queue.value.some((r) => r.state === 'pending')
      if (hasNext && !cancelled) await sleep((status.cooldownSecs + 1) * 1000)
    }
  } finally {
    running.value = false
  }
}

function stop() {
  cancelled = true
}

onBeforeUnmount(() => {
  cancelled = true
  queue.value.forEach((r) => r.url && URL.revokeObjectURL(r.url))
})

const STATE_LABEL = {
  pending: '等待中',
  running: '转换中…',
  done: '完成',
  error: '失败',
}
</script>

<template>
  <div class="vo-workbench">
    <section class="vo-panel">
      <SosField label="目标角色" class="vo-panel__field">
        <SosSelect v-model="role" :options="roleOptions" :disabled="!roles.rvc.length" />
      </SosField>

      <SosField label="批量添加音频" :help="`每个 ≤50MB，队列上限 ${QUEUE_MAX} 个；全部使用下方同一套参数`">
        <label class="vo-drop vo-drop--slim">
          <input type="file" multiple accept="audio/*,.wav,.mp3,.flac,.m4a,.ogg,.aac" class="vo-drop__input" @change="onFiles" />
          <span class="vo-drop__hint">点击选择多个音频文件（已加入 {{ queue.length }} 个）</span>
        </label>
      </SosField>

      <ol v-if="queue.length" class="vo-queue">
        <li v-for="(row, i) in queue" :key="row.file.name + i" class="vo-queue__row" :data-state="row.state">
          <span class="vo-queue__name">{{ row.file.name }}</span>
          <span class="vo-queue__meta">{{ (row.file.size / 1024 / 1024).toFixed(1) }} MB</span>
          <span class="vo-queue__state">
            <SosSpinner v-if="row.state === 'running'" label="转换中" />
            {{ row.message || STATE_LABEL[row.state] }}
          </span>
          <a
            v-if="row.state === 'done'"
            class="sos-button sos-button--secondary vo-queue__dl"
            :href="row.url"
            :download="`春日语音工坊_${row.file.name.replace(/\.[^.]+$/, '')}.${params.format}`"
          >下载</a>
          <button
            v-else-if="row.state !== 'running'"
            type="button"
            class="vo-auxlist__rm"
            aria-label="移除"
            @click="removeRow(i)"
          >✕</button>
        </li>
      </ol>

      <RvcParams v-model="params" />

      <p v-if="error" class="vo-error" role="alert">{{ error }}</p>

      <div class="vo-panel__actions">
        <button
          type="button"
          class="sos-button sos-button--primary vo-submit"
          :disabled="loggedIn && (running || !queue.some((r) => r.state === 'pending') || !status.rvcOnline)"
          :aria-busy="running || undefined"
          @click="running ? stop() : start()"
        >
          <SosSpinner v-if="running" label="批量转换中" />
          <template v-if="running">批量转换中（{{ doneCount }}/{{ queue.length }}）… 点击停止</template>
          <template v-else-if="!loggedIn">登录后转换</template>
          <template v-else>开始批量转换</template>
        </button>
        <button v-if="queue.length && !running" type="button" class="sos-button sos-button--ghost" @click="clearQueue">
          清空队列
        </button>
      </div>
    </section>

    <aside class="vo-tips">
      <h2 class="vo-tips__title">小贴士</h2>
      <ul class="vo-tips__list">
        <li>批量任务逐个串行转换，任务间会按服务端冷却自动等待，无需盯着。</li>
        <li>完成的文件<strong>逐个点「下载」保存</strong>——离开页面后结果会释放。</li>
        <li>队列共用同一套角色与参数；需要不同参数请分批提交。</li>
        <li>某个文件失败不影响后续文件继续转换。</li>
      </ul>
    </aside>
  </div>
</template>
