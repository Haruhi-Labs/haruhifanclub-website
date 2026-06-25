<script setup>
import { computed, provide, onMounted } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { SosAvatar } from '@haruhi/ui'
import { useSession } from './useSession.js'
import { CONSOLE_CTX } from './console-context.js'
import './auth.css'

// 个人控制台外壳：带侧边导航的「个人空间」，内容区由嵌套子路由（RouterView）渲染。
// 全局复用——从任一 app 进入都能管理全站内容；视觉跟随该 app 的 site 表达。
const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  basePath: { type: String, default: '/account' }, // 控制台根路径（各子页相对它）
  home: { type: String, default: '/' },
  site: { type: String, default: undefined },
  // 要显示的导航分区 key 子集（与已注册的子路由对应）；不传则显示全部。
  sections: { type: Array, default: undefined },
})

const session = useSession(props.apiBase)
const route = useRoute()
const router = useRouter()
const user = computed(() => session.state.user)

// 下发上下文给子页（免各 app 在每个子路由重复传 props）
provide(CONSOLE_CTX, {
  apiBase: props.apiBase,
  site: props.site,
  basePath: props.basePath,
  loginPath: props.loginPath,
  home: props.home,
})

// 侧边导航项（to 相对 basePath）
const allNavItems = [
  { key: 'overview', label: '概览', to: '' },
  { key: 'artworks', label: '我的作品', to: '/artworks' },
  { key: 'articles', label: '我的文章', to: '/articles' },
  { key: 'exams', label: '我的考试', to: '/exams' },
  { key: 'comments', label: '我的评论', to: '/comments' },
  { key: 'points', label: '积分与兑换', to: '/points' },
  { key: 'profile', label: '资料', to: '/profile' },
  { key: 'settings', label: '账号安全', to: '/settings' },
]
// 仅展示已接入的分区（与各 app 已注册的子路由保持一致），避免导航指向未建页。
const navItems = computed(() =>
  props.sections ? allNavItems.filter((i) => props.sections.includes(i.key)) : allNavItems,
)

function fullPath(to) {
  return props.basePath + to
}
function isActive(to) {
  if (to === '') {
    return route.path === props.basePath || route.path === `${props.basePath}/`
  }
  const target = fullPath(to)
  return route.path === target || route.path.startsWith(`${target}/`)
}

onMounted(async () => {
  if (!session.state.ready) await session.refresh()
  if (!session.state.user) {
    router.push(`${props.loginPath}?redirect=${encodeURIComponent(route.fullPath)}`)
  }
})
</script>

<template>
  <div class="hauth-root sos-scope huc" :data-sos-site="site">
    <div v-if="user" class="huc__shell">
      <aside class="huc__nav">
        <div class="huc__nav-head">
          <SosAvatar :src="user.avatar || undefined" :name="user.nickname || 'U'" size="md" />
          <div class="huc__nav-id">
            <p class="huc__nav-name">{{ user.nickname || '未命名' }}</p>
            <p class="huc__nav-mail">{{ user.email || user.username }}</p>
          </div>
        </div>
        <nav class="huc__nav-list" aria-label="个人中心导航">
          <RouterLink
            v-for="it in navItems"
            :key="it.key"
            :to="fullPath(it.to)"
            class="huc__nav-link"
            :class="{ 'is-active': isActive(it.to) }"
          >
            {{ it.label }}
          </RouterLink>
        </nav>
        <a :href="home" class="huc__nav-back">← 返回站点</a>
      </aside>
      <main class="huc__main">
        <RouterView />
      </main>
    </div>
  </div>
</template>
