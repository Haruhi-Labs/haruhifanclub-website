import { createApp } from 'vue'
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import '@haruhi/design-system/bridges.css'
import App from './App.vue'
import router from './router'

const app = createApp(App)
app.use(router)
app.mount('#app')
