import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

// 统一后端约定：后端监听 17777；dev 下代理 /api 与 /uploads。
// 部署在子路径 /exam/ 下；@ 指向 src。
export default defineConfig({
  base: '/exam/',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern-compiler',
      },
    },
  },
  server: {
    port: 5202,
    proxy: {
      '/api': { target: 'http://127.0.0.1:17777', changeOrigin: true },
      '/uploads': { target: 'http://127.0.0.1:17777', changeOrigin: true },
    },
  },
})
