import { createApp } from 'vue'
import { createPinia } from 'pinia'
// 深度接入设计系统：tokens + components/recipes（含 .sos-article-card 团报卡）。
// 表达模式由根节点 data-sos-site="news" 提供，组件层只消费 token，不改编辑部气质。
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import { installRouterMeta } from '@haruhi/seo'
import App from './App.vue'
import router from './router'
import './style.css'

// 路由级 title/noindex（详情页由视图内 usePageMeta 在数据加载后覆盖）
installRouterMeta(router, { siteName: '春日团报', defaultTitle: '春日团报 · 凉宫春日应援团' })

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
