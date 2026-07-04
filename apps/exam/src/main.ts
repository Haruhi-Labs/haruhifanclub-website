import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { installRouterMeta } from '@haruhi/seo'
import router from '@/router' // 引入路由
import App from './App.vue'

// 引入全局样式
import '@/assets/styles/base.scss'
import '@/assets/styles/paper.scss'
// 设计系统：tokens + 基础组件/recipe class（exam 表达由根节点 data-sos-site 决定）。
// 放在站点样式之后，让 .sos-* 组件类在同特异性时胜出。
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'

installRouterMeta(router, { siteName: '春日试卷中心', defaultTitle: '春日试卷中心 · 凉宫春日应援团' })

const app = createApp(App)

app.use(createPinia())
app.use(router) // 使用路由

app.mount('#app')