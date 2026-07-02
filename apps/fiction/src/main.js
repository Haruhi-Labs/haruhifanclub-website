// 设计系统：tokens + 组件/recipe class（放在应用样式之前，供 style.css 覆盖 token）
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import './style.css'

import { createApp } from 'vue'
import App from './App.vue'
import router from './router'

createApp(App).use(router).mount('#app')
