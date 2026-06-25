import { createApp } from 'vue'
import App from './App.vue'
// 设计系统：tokens（暗色桥接）+ components（统一页脚等 class 契约）。
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import './style.css'
import { router } from './router'

createApp(App).use(router).mount('#app')
