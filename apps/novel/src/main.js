import './assets/main.css'
// 设计系统：tokens + 基础组件/recipe class（library 表达由根节点 data-sos-site 决定）。
// 放在 main.css 之后引入，让 .sos-* 组件类在与 Tailwind 工具类同特异性时胜出。
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'

import { createApp } from 'vue'
import App from './App.vue'
import router from './router'

const app = createApp(App)

app.use(router)

app.mount('#app')
