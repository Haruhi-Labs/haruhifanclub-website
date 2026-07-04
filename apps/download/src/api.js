// 资源站数据层：封装 /api/download 客户端 + 全站统一账号会话。
import { createApiClient } from '@haruhi/api-client'
import { useSession } from '@haruhi/auth-ui'

export const api = createApiClient('/api/download')

/** 统一账号会话（与 AccountMenu / 路由守卫共享同一实例）。 */
export const session = useSession('/api')

/** 拉取整棵资源索引（后端内存缓存，服务端定时同步语雀知识库）。 */
export const getIndex = () => api.get('/index')
