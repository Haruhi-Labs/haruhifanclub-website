// 全站共享的角色列表与服务在线状态。
// 角色列表只拉一次（后端有缓存）；状态定时轮询 + 提交前强刷（离线时禁提交）。
import { reactive } from 'vue'
import { getRoles, getStatus } from '@/api'

export const roles = reactive({
  loading: false,
  loaded: false,
  error: '',
  /** [{ name, refs: [string] }] —— GPT-SoVITS characters.json 的角色与预设语气 */
  tts: [],
  /** [{ name }] —— RVC roles.json 的角色 */
  rvc: [],
})

let rolesInflight = null

/** 首次调用触发拉取；重复调用复用同一请求 / 已加载则直接返回。 */
export function ensureRoles() {
  if (roles.loaded) return Promise.resolve()
  if (rolesInflight) return rolesInflight
  roles.loading = true
  roles.error = ''
  rolesInflight = getRoles()
    .then((d) => {
      roles.tts = Array.isArray(d?.tts) ? d.tts : []
      roles.rvc = Array.isArray(d?.rvc) ? d.rvc : []
      roles.loaded = true
    })
    .catch((e) => {
      roles.error = e?.message || '角色列表加载失败'
    })
    .finally(() => {
      roles.loading = false
      rolesInflight = null
    })
  return rolesInflight
}

export const status = reactive({
  known: false,
  ttsOnline: false,
  rvcOnline: false,
  checkedAt: null,
  /** 服务端配置的每用户提交冷却（秒），批量队列按它自适应间隔 */
  cooldownSecs: 10,
})

/** 拉一次在线状态（页面轮询与提交前都走这里）。失败按全离线处理。 */
export async function refreshStatus() {
  try {
    const d = await getStatus()
    status.ttsOnline = !!d?.tts?.online
    status.rvcOnline = !!d?.rvc?.online
    status.checkedAt = d?.checkedAt || null
    if (Number.isFinite(d?.cooldownSecs)) status.cooldownSecs = d.cooldownSecs
  } catch {
    status.ttsOnline = false
    status.rvcOnline = false
  }
  status.known = true
}

let pollTimer = null

/** 启动状态轮询（60s 一次；组件卸载不停——全站生命周期共享一个轮询即可）。 */
export function startStatusPolling() {
  if (pollTimer) return
  refreshStatus()
  pollTimer = setInterval(refreshStatus, 60_000)
}
