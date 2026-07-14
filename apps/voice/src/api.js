// 语音工坊数据层：/api/voice 客户端 + 全站统一账号会话 + 二进制音频请求。
import { createApiClient, getCsrfToken } from '@haruhi/api-client'
import { useSession } from '@haruhi/auth-ui'

export const api = createApiClient('/api/voice')

/** 统一账号会话（与 AccountMenu / 路由守卫共享同一实例）。 */
export const session = useSession('/api')

/** 服务在线状态（后端探活缓存，公开）；含 cooldownSecs 供批量队列自适应间隔。 */
export const getStatus = () => api.get('/status')

/** 角色列表：{ tts: [{name, refs:[…]}], rvc: [{name}] }（后端缓存，公开）。 */
export const getRoles = () => api.get('/roles')

/** 预设参考音频试听地址（GET 公开，浏览器直接作 <audio> src）。 */
export const refUrl = (character, ref) =>
  `/api/voice/ref?character=${encodeURIComponent(character)}&ref=${encodeURIComponent(ref)}`

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

/**
 * 语音合成（预设语气参考）→ wav Blob。
 * payload 直接使用 /hfc/synth 的 snake_case 字段：
 * { character, ref, text, text_lang, speed, how_to_cut, top_k, top_p,
 *   temperature, pause_second, if_freeze, ref_free }
 */
export const synthesize = (payload) =>
  fetchAudio('/tts', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })

/**
 * 语音合成（自定义参考音频）→ wav Blob。
 * formData 字段（snake_case，与后端白名单一致）：character/text/text_lang/
 * prompt_text/prompt_lang/ref_free/speed/how_to_cut/top_k/top_p/temperature/
 * pause_second/if_freeze + ref_audio 文件 + aux_refs 多文件。
 */
export const synthesizeCustom = (formData) =>
  fetchAudio('/tts/custom', { method: 'POST', body: formData })

/**
 * 多句拼接合成 → wav Blob。
 * payload: { items: [{character, ref, text, interval}], text_lang, speed, global_interval }
 */
export const synthesizeBatch = (payload) =>
  fetchAudio('/tts/batch', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })

/**
 * 声线转换 → 音频 Blob（格式由 formData.format 决定）。
 * formData：audio / role / transpose / indexRate / protect / rmsMixRate /
 * filterRadius / resampleSr / format。
 */
export const convert = (formData) =>
  fetchAudio('/rvc', {
    method: 'POST',
    body: formData,
  })
