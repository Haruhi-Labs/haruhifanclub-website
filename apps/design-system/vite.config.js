import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// SOS / Parallel Design System 文档页，部署于子路径 /design-system/。
export default defineConfig({
  base: '/design-system/',
  plugins: [vue()],
  server: {
    port: 5206,
  },
})
