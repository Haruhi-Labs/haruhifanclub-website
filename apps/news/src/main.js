import { createApp } from 'vue'
import { createPinia } from 'pinia'
// 深度接入设计系统：tokens + components/recipes（含 .sos-article-card 团报卡）。
// 表达模式由根节点 data-sos-site="news" 提供，组件层只消费 token，不改编辑部气质。
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import App from './App.vue'
import router from './router'
import './style.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
