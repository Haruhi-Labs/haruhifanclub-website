import { fileURLToPath, URL } from 'node:url'
import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'

function readOnlyProxyGuard(enabled) {
  return {
    name: 'art-read-only-proxy-guard',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const method = String(req.method || 'GET').toUpperCase()
        const requestUrl = String(req.url || '')
        const isApiRequest = requestUrl.startsWith('/api/')
        // 作品详情 GET 会记录浏览与任务进度，预览模式也必须拦截。
        const isStatefulArtworkRead = /^\/api\/art\/artworks\/\d+(?:\?|$)/.test(requestUrl)
        const isSafeMethod = ['GET', 'HEAD', 'OPTIONS'].includes(method) && !isStatefulArtworkRead
        if (!enabled || !isApiRequest || isSafeMethod) {
          next()
          return
        }

        res.statusCode = 405
        res.setHeader('Content-Type', 'application/json; charset=utf-8')
        res.end(JSON.stringify({
          ok: false,
          message: '真实数据预览模式为只读，未向正式站写入数据',
        }))
      })
    },
  }
}

// 绘画部画廊前端，部署于子路径 /art/，dev 代理到统一后端 17777。
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const proxyTarget = env.VITE_ART_PROXY_TARGET || 'http://127.0.0.1:17777'
  const proxyReadOnly = env.VITE_ART_PROXY_READ_ONLY === '1'

  return {
    base: '/art/',
    plugins: [vue(), readOnlyProxyGuard(proxyReadOnly)],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
      },
    },
    server: {
      port: 5201,
      proxy: {
        '/api': { target: proxyTarget, changeOrigin: true },
        '/uploads': { target: proxyTarget, changeOrigin: true },
      },
    },
  }
})
