// WebAuthn ceremony 辅助：base64url ↔ ArrayBuffer，并把 navigator.credentials 的
// 结果序列化成后端 webauthn-rs 期望的 JSON 形状。仅在浏览器 + 安全上下文（HTTPS/localhost）可用。

export function isPasskeySupported() {
  return (
    typeof window !== 'undefined' &&
    typeof window.PublicKeyCredential !== 'undefined' &&
    typeof navigator !== 'undefined' &&
    !!navigator.credentials
  )
}

// 条件式 UI（在用户名/邮箱输入框的自动填充里直接列出本站通行密钥）是否可用
export async function isConditionalUiAvailable() {
  try {
    return (
      isPasskeySupported() &&
      typeof window.PublicKeyCredential.isConditionalMediationAvailable === 'function' &&
      (await window.PublicKeyCredential.isConditionalMediationAvailable())
    )
  } catch {
    return false
  }
}

function b64urlToBuf(s) {
  const pad = s.length % 4 === 0 ? '' : '='.repeat(4 - (s.length % 4))
  const b64 = (s + pad).replace(/-/g, '+').replace(/_/g, '/')
  const bin = atob(b64)
  const buf = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i += 1) buf[i] = bin.charCodeAt(i)
  return buf.buffer
}

function bufToB64url(buf) {
  const bytes = new Uint8Array(buf)
  let bin = ''
  for (let i = 0; i < bytes.length; i += 1) bin += String.fromCharCode(bytes[i])
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

// 注册：把后端 options.publicKey 内的 base64url 字段转 ArrayBuffer →
// navigator.credentials.create → 序列化新凭据（attestation）。
export async function createCredential(publicKey) {
  const pk = { ...publicKey, challenge: b64urlToBuf(publicKey.challenge) }
  pk.user = { ...publicKey.user, id: b64urlToBuf(publicKey.user.id) }
  if (Array.isArray(publicKey.excludeCredentials)) {
    pk.excludeCredentials = publicKey.excludeCredentials.map((c) => ({
      ...c,
      id: b64urlToBuf(c.id),
    }))
  }
  const cred = await navigator.credentials.create({ publicKey: pk })
  if (!cred) throw new Error('未完成添加：操作被取消')
  const r = cred.response
  return {
    id: cred.id,
    rawId: bufToB64url(cred.rawId),
    type: cred.type,
    response: {
      attestationObject: bufToB64url(r.attestationObject),
      clientDataJSON: bufToB64url(r.clientDataJSON),
      transports: typeof r.getTransports === 'function' ? r.getTransports() : undefined,
    },
  }
}

// 登录：把后端 options.publicKey 内的 base64url 字段转 ArrayBuffer →
// navigator.credentials.get → 序列化断言（assertion）。
// conditional=true 走「条件式自动填充」，由浏览器在自动填充列表中触发（autoFill）。
export async function getCredential(publicKey, { conditional = false, signal } = {}) {
  const pk = { ...publicKey, challenge: b64urlToBuf(publicKey.challenge) }
  if (Array.isArray(publicKey.allowCredentials)) {
    pk.allowCredentials = publicKey.allowCredentials.map((c) => ({ ...c, id: b64urlToBuf(c.id) }))
  }
  const opts = { publicKey: pk }
  if (signal) opts.signal = signal
  if (conditional) opts.mediation = 'conditional'
  const cred = await navigator.credentials.get(opts)
  if (!cred) throw new Error('未完成：操作被取消')
  const r = cred.response
  return {
    id: cred.id,
    rawId: bufToB64url(cred.rawId),
    type: cred.type,
    response: {
      authenticatorData: bufToB64url(r.authenticatorData),
      clientDataJSON: bufToB64url(r.clientDataJSON),
      signature: bufToB64url(r.signature),
      userHandle: r.userHandle ? bufToB64url(r.userHandle) : null,
    },
  }
}
