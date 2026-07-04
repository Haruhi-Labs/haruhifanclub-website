import { createApp } from 'vue'
import { createPinia } from 'pinia'
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import '@haruhi/design-system/bridges.css'
import './style.css'
import { installRouterMeta } from '@haruhi/seo'
import App from './App.vue'
import router from './router'

installRouterMeta(router, { siteName: '春日画廊', defaultTitle: '春日画廊 · 凉宫春日应援团' })

const app = createApp(App)

// ✅ 关键修复：必须先注册 Pinia，再注册 Router
// 这样当 Router 加载组件触发 setup() 时，Store 才能找到活跃的 Pinia 实例
app.use(createPinia())
app.use(router)

app.mount('#app')
