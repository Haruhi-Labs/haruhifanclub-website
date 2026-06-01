import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 后端统一监听 17777；dev 下代理 /api 与 /uploads。
export default defineConfig({
  base: '/console/',
  plugins: [vue()],
  server: {
    port: 5200,
    proxy: {
      '/api': { target: 'http://127.0.0.1:17777', changeOrigin: true },
      '/uploads': { target: 'http://127.0.0.1:17777', changeOrigin: true },
    },
  },
})
