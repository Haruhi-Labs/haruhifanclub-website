import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 同人小说站前端，部署于子路径 /fiction/，dev 代理到统一后端 17777。
export default defineConfig({
  base: '/fiction/',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 5210,
    proxy: {
      '/api': { target: 'http://127.0.0.1:17777', changeOrigin: true },
      '/uploads': { target: 'http://127.0.0.1:17777', changeOrigin: true },
    },
  },
})
