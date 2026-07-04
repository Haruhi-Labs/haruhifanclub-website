import { createApp } from 'vue'
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import '@haruhi/design-system/bridges.css'
import { installRouterMeta } from '@haruhi/seo'
import App from './App.vue'
import router from './router'

installRouterMeta(router, { siteName: '春日商城', defaultTitle: '春日商城 · 凉宫春日应援团' })

const app = createApp(App)
app.use(router)
app.mount('#app')
