import { createApp } from 'vue'
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import '@haruhi/design-system/bridges.css'
import { installRouterMeta } from '@haruhi/seo'
import App from './App.vue'
import router from './router'
import './style.css'

installRouterMeta(router, {
  siteName: '凉宫春日应援团',
  defaultTitle: '地方支部 · 凉宫春日应援团',
})

createApp(App).use(router).mount('#app')
