import { createApiClient } from '@haruhi/api-client'
import { useSession } from '@haruhi/auth-ui'

export const api = createApiClient('/api/chapter')
export const session = useSession('/api')

export function uploadBranchMedia(branchId, file) {
  const form = new FormData()
  form.append('file', file)
  return api.postForm(`/admin/branches/${branchId}/media`, form)
}

function safeUrl(value, allowedSchemes) {
  const candidate = String(value || '').trim()
  const scheme = candidate.match(/^([a-z][a-z0-9+.-]*):/i)?.[1]?.toLowerCase()
  return scheme && allowedSchemes.includes(scheme) ? candidate : ''
}

export function safeExternalUrl(value) {
  return safeUrl(value, ['http', 'https'])
}

export function safeContactUrl(value) {
  return safeUrl(value, ['http', 'https', 'mailto', 'tel'])
}
