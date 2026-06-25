// @haruhi/auth-ui —— 全站共享账号 UI（可换肤 Vue 组件）+ 无头登录态组合式。
// 组件用 vue + vue-router（均为各 app 已有的 peer 依赖，单例复用）。
// 主题：在外层对 .hauth-root 设 CSS 变量（如 --hauth-accent）即可换肤。
export { useSession } from './useSession.js'
export { useUserHub } from './useUserHub.js'
export { useConsoleContext } from './console-context.js'

export { default as LoginView } from './LoginView.vue'
export { default as ResetPasswordView } from './ResetPasswordView.vue'
export { default as VerifyEmailView } from './VerifyEmailView.vue'
export { default as ProfileView } from './ProfileView.vue'
export { default as SettingsView } from './SettingsView.vue'
export { default as AccountMenu } from './AccountMenu.vue'

// 个人控制台（带侧边导航的「个人空间」外壳 + 各子页）
export { default as UserConsoleLayout } from './UserConsoleLayout.vue'
export { default as OverviewView } from './OverviewView.vue'
export { default as MyArtworksView } from './MyArtworksView.vue'
export { default as MyArticlesView } from './MyArticlesView.vue'
export { default as MyExamsView } from './MyExamsView.vue'
export { default as MyCommentsView } from './MyCommentsView.vue'
export { default as PointsView } from './PointsView.vue'
