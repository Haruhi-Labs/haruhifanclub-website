<template>
  <div
    :class="['app-container', 'sos-scope', { 'sidebar-open': sidebarOpen }]"
    data-sos-site="shop"
    data-sos-density="compact"
  >
    <aside class="sidebar">
        <div class="sidebar-header">
            <h1 class="sidebar-title"><i class="fa fa-star" style="margin-right: 0.5rem;"></i>SOS团支部</h1>
        </div>
        <nav class="nav-menu">
            <div class="nav-label">控制台</div>
            <router-link to="/admin/dashboard" class="nav-item" active-class="active" @click="closeSidebarOnMobile">
                <i class="fa fa-tachometer-alt" style="width: 1.5rem;"></i> 总览看板
            </router-link>
            <router-link to="/admin/orders" class="nav-item" active-class="active" @click="closeSidebarOnMobile">
                <i class="fa fa-list-alt" style="width: 1.5rem;"></i> 订单管理
            </router-link>
            <router-link to="/admin/messages" class="nav-item" active-class="active" @click="closeSidebarOnMobile">
                <i class="fa fa-comments" style="width: 1.5rem;"></i> 留言管理
            </router-link>
            <!-- [新增] 商品库入口 -->
            <router-link to="/admin/products" class="nav-item" active-class="active" @click="closeSidebarOnMobile">
                <i class="fa fa-box" style="width: 1.5rem;"></i> 商品库
            </router-link>
            <router-link to="/admin/coupons" class="nav-item" active-class="active" @click="closeSidebarOnMobile">
                <i class="fa fa-ticket-alt" style="width: 1.5rem;"></i> 优惠券
            </router-link>
            <router-link to="/admin/stats" class="nav-item" active-class="active" @click="closeSidebarOnMobile">
                <i class="fa fa-chart-bar" style="width: 1.5rem;"></i> 数据统计
            </router-link>
            
            <div class="nav-label">系统</div>
            <router-link to="/admin/settings" class="nav-item" active-class="active" @click="closeSidebarOnMobile">
                <i class="fa fa-cog" style="width: 1.5rem;"></i> 设置
            </router-link>
        </nav>
        <div class="sidebar-footer">北高文艺部内部系统 v1.1</div>
    </aside>
    <div v-if="sidebarOpen" class="mobile-sidebar-backdrop" @click="sidebarOpen = false"></div>

    <main class="main-wrapper">
        <header class="top-header">
            <div class="header-left">
                <button class="sidebar-toggle" type="button" aria-label="打开导航菜单" @click="toggleSidebar">
                    <i class="fa fa-bars"></i>
                </button>
                <h2 class="page-heading">{{ pageTitle }}</h2>
            </div>
            <div class="user-profile">
                <span class="user-info">
                    管理员: <strong>{{ adminName }}</strong>
                    <small v-if="adminRoleLabel">（{{ adminRoleLabel }}）</small>
                </span>
                <button class="logout-btn" :disabled="logoutBusy" @click="logout">
                    <i class="fa fa-sign-out-alt"></i>
                </button>
            </div>
        </header>
        <div class="content-scroll">
            <router-view></router-view>
        </div>
    </main>
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useShopStore } from '@/stores/shopStore'
import '@/assets/admin.css'

const route = useRoute()
const router = useRouter()
const store = useShopStore()
const sidebarOpen = ref(false)
const logoutBusy = ref(false)
const pageTitleMap = {
    'admin-dashboard': '总览看板',
    'admin-orders': '订单管理',
    'admin-messages': '留言管理',
    'admin-products': '商品库',
    'admin-coupons': '优惠券管理',
    'admin-stats': '数据统计',
    'admin-settings': '系统设置'
}

const pageTitle = computed(() => pageTitleMap[route.name] || '管理后台')
const adminUser = computed(() => store.state.adminUser)
const adminName = computed(() => {
    const user = adminUser.value
    return user?.nickname || user?.displayName || user?.username || '管理员'
})
const adminRoleLabel = computed(() => {
    const user = adminUser.value
    if (!user) return ''
    if (user.isSuperAdmin) return '超级管理员'
    return user.apps?.shop?.roleName || '商城管理'
})

const toggleSidebar = () => {
    sidebarOpen.value = !sidebarOpen.value
}

const closeSidebarOnMobile = () => {
    if (typeof window === 'undefined') return
    if (window.innerWidth < 1024) sidebarOpen.value = false
}

watch(
    () => route.fullPath,
    () => {
        closeSidebarOnMobile()
    }
)

onMounted(() => {
    if (!store.state.adminUser) {
        store.loadAdminUser()
    }
})

const logout = async () => {
    if (logoutBusy.value) return
    logoutBusy.value = true
    try {
        await store.adminLogout()
        await router.replace('/admin/login')
    } finally {
        logoutBusy.value = false
    }
}
</script>
