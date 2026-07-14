import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 春日语音工坊前端，部署于子路径 /voice/，dev 代理到统一后端 17777。
export default defineConfig({
  base: '/voice/',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 5211,
    proxy: {
      '/api': { target: 'http://127.0.0.1:17777', changeOrigin: true },
    },
  },
})
