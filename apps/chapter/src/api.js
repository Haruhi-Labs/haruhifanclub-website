import { createApiClient } from '@haruhi/api-client'
import { useSession } from '@haruhi/auth-ui'

export const api = createApiClient('/api/chapter')
export const session = useSession('/api')

export function uploadBranchMedia(branchId, file) {
  const form = new FormData()
  form.append('file', file)
  return api.postForm(`/admin/branches/${branchId}/media`, form)
}
