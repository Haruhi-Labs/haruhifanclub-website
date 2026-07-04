// 设计系统：tokens + 组件/recipe class（放在应用样式之前，供 style.css 覆盖 token）
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import './style.css'

import { createApp } from 'vue'
import { installRouterMeta } from '@haruhi/seo'
import App from './App.vue'
import router from './router'

installRouterMeta(router, {
  siteName: '春日资源站',
  defaultTitle: '凉宫春日资源站 · 凉宫春日应援团',
})

createApp(App).use(router).mount('#app')
