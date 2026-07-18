import { fileURLToPath, URL } from 'node:url'
import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'

// 绘画部画廊前端，部署于子路径 /art/，dev 代理到统一后端 17777。
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const proxyTarget = env.VITE_ART_PROXY_TARGET || 'http://127.0.0.1:17777'

  return {
    base: '/art/',
    plugins: [vue()],
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
