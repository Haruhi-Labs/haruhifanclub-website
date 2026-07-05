// 语音工坊数据层：/api/voice 客户端 + 全站统一账号会话 + 二进制音频请求。
import { createApiClient, getCsrfToken } from '@haruhi/api-client'
import { useSession } from '@haruhi/auth-ui'

export const api = createApiClient('/api/voice')

/** 统一账号会话（与 AccountMenu / 路由守卫共享同一实例）。 */
export const session = useSession('/api')

/** 服务在线状态（后端探活缓存，公开）。 */
export const getStatus = () => api.get('/status')

/** 角色列表：{ tts: [{name, refs:[…]}], rvc: [{name}] }（后端缓存，公开）。 */
export const getRoles = () => api.get('/roles')

/**
 * 音频类请求不能走 createApiClient（其一律按文本解析响应），
 * 用原生 fetch 携带会话 cookie + CSRF 头，成功时返回 Blob。
 * 失败时抛 Error，附 status 供界面按 401/429/503 分流提示。
 */
async function fetchAudio(path, init) {
  const res = await fetch(`/api/voice${path}`, {
    ...init,
    credentials: 'include',
    headers: {
      ...(init.headers || {}),
      'X-CSRF-Token': getCsrfToken() || '',
    },
  })
  if (!res.ok) {
    let message = `请求失败（${res.status}）`
    try {
      const data = await res.json()
      if (data?.error) message = data.error
    } catch {
      /* 非 JSON 错误体，用默认文案 */
    }
    const err = new Error(message)
    err.status = res.status
    throw err
  }
  return res.blob()
}

/** 语音合成：payload = { character, ref, text, textLang, speed } → wav Blob。 */
export const synthesize = ({ character, ref, text, textLang, speed }) =>
  fetchAudio('/tts', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ character, ref, text, text_lang: textLang, speed }),
  })

/** 声线转换：formData 含 audio / role / transpose / indexRate / protect → wav Blob。 */
export const convert = (formData) =>
  fetchAudio('/rvc', {
    method: 'POST',
    body: formData,
  })
