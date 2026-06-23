import { defineConfig } from 'vite'

// SOS / Parallel Design System 文档页，部署于子路径 /design-system/。
export default defineConfig({
  base: '/design-system/',
  server: {
    port: 5206,
  },
})
