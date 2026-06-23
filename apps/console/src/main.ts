import { createApp } from 'vue'
import App from './App.vue'
// 设计系统 token：控制台经 data-sos-theme=dark 桥接到 DS 暗色语义层。
import '@haruhi/design-system/tokens.css'
import './style.css'
import { router } from './router'

createApp(App).use(router).mount('#app')
