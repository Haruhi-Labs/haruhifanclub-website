import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from '@/router' // 引入路由
import App from './App.vue'

// 引入全局样式
import '@/assets/styles/base.scss'
import '@/assets/styles/paper.scss'

const app = createApp(App)

app.use(createPinia())
app.use(router) // 使用路由

app.mount('#app')