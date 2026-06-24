import { createApp } from 'vue'
import { createPinia } from 'pinia'
// 浅层接入设计系统：只引入 token 层（配色/间距等语义变量），
// 不引入组件层；表达模式由根节点 data-sos-site="news" 提供。
import '@haruhi/design-system/tokens.css'
import App from './App.vue'
import router from './router'
import './style.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
