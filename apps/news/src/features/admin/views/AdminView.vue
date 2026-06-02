<template>
  <div class="admin-root">

    <!-- ================= Login Section ================= -->
    <div v-if="!store.isAdmin" class="login-wrapper animate-slide-up">
        <div class="login-card">
            <div class="login-top-bar"></div>
            <div class="login-header">
                <div class="login-avatar">H</div>
                <h2 class="login-title">管理后台</h2>
                <p class="login-subtitle">Admin Dashboard</p>
            </div>

            <input
                type="text"
                v-model="username"
                @keyup.enter="login"
                placeholder="Username"
                class="login-input"
                autocomplete="username"
            >
            <input
                type="password"
                v-model="password"
                @keyup.enter="login"
                placeholder="Password"
                class="login-input"
                autocomplete="current-password"
            >
            <p v-if="loginMsg" class="login-msg">{{ loginMsg }}</p>
            <button @click="login" :disabled="loginLoading" class="login-btn">
                <span>{{ loginLoading ? '登录中…' : '进入系统' }}</span>
                <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"></path></svg>
            </button>
        </div>
    </div>

    <!-- ================= Admin Dashboard ================= -->
    <div v-else class="dashboard-container animate-slide-up">

        <!-- Top Bar -->
        <div class="top-bar">
            <div>
                <h1 class="top-bar-title serif-font">控制台</h1>
                <p class="top-bar-subtitle">欢迎回来，管理员。今天是 {{ new Date().toLocaleDateString() }}</p>
            </div>
            <div class="top-bar-actions">
                 <router-link to="/" class="btn-back">
                    返回首页
                 </router-link>
                 <button @click="handleLogout" class="btn-logout">
                    退出登录
                 </button>
                 <router-link to="/submit" class="btn-new-content">
                    <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
                    新建内容
                 </router-link>
            </div>
        </div>

        <!-- Dashboard Stats -->
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">活动总数</div>
                <div class="stat-value stat-blue">{{ store.activities.length }}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">奖品总数</div>
                <div class="stat-value stat-purple">{{ store.prizes.length }}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">已发布文章</div>
                <div class="stat-value stat-green">{{ publishedCount }}</div>
            </div>
             <div class="stat-card">
                <div class="stat-label">新闻快讯</div>
                <div class="stat-value">{{ store.adminArticles.filter(a => a.type === 'news').length }}</div>
            </div>
        </div>

        <!-- Content Area -->
        <div class="content-area">
            <!-- Tabs (RBAC 子作用域门控：只渲染当前用户有权的 tab) -->
            <div class="tabs-bar">
                <button
                    v-for="tab in visibleTabs"
                    :key="tab.key"
                    @click="activeTab = tab.key"
                    :class="[activeTab === tab.key ? 'tab-active' : 'tab-inactive', tab.btnClass]"
                    class="tab-btn"
                >
                    {{ tab.label }}
                    <span v-if="tab.key === 'pending' && pendingCount > 0" class="pending-badge">{{ pendingCount }}</span>
                </button>
            </div>

            <!-- 当前 tab 组件（懒加载） -->
            <component v-if="activeComponent" :is="activeComponent" :key="activeTab" :mode="activeMode" />
        </div>
    </div>

  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch, defineAsyncComponent } from 'vue';
import { useMainStore } from '@/stores/main';
import { createAdminAuth, hasScope } from '@haruhi/api-client';

const store = useMainStore();

// 后台壳自行持有一个 createAdminAuth('news')，用于在登录/会话恢复后拿当前用户对象（含 apps）算可见 tab。
const admin = createAdminAuth('news');
const currentUser = ref(null);

const username = ref('');
const password = ref('');
const loginMsg = ref('');
const loginLoading = ref(false);

// ================= Tab 组件懒加载注册表 =================
// 每个 tab：key / label / 需要的 RBAC 子作用域 / 异步组件 / 额外按钮样式 class / 传给组件的 mode
const ActivityAdmin = defineAsyncComponent(() => import('@/features/activity/admin/ActivityAdmin.vue'));
const PrizeAdmin = defineAsyncComponent(() => import('@/features/store/admin/PrizeAdmin.vue'));
const ArticleAdmin = defineAsyncComponent(() => import('@/features/blog/admin/ArticleAdmin.vue'));
const GeneratorAdmin = defineAsyncComponent(() => import('@/features/blog/admin/GeneratorAdmin.vue'));
const PointsAdmin = defineAsyncComponent(() => import('@/features/points/admin/PointsAdmin.vue'));

const TABS = [
    { key: 'activities', label: '活动管理', scope: 'news.activity', comp: ActivityAdmin },
    { key: 'prizes', label: '奖品管理', scope: 'news.store', comp: PrizeAdmin },
    { key: 'pending', label: '待审核', scope: 'news.blog', comp: ArticleAdmin, mode: 'pending', btnClass: 'tab-btn-relative' },
    { key: 'published', label: '已发布内容', scope: 'news.blog', comp: ArticleAdmin, mode: 'published' },
    { key: 'points', label: '积分管理', scope: 'news.points', comp: PointsAdmin },
    { key: 'generator', label: '新闻总览生成', scope: 'news.blog', comp: GeneratorAdmin, btnClass: 'tab-btn-flex' },
];

// 当前登录用户有权看到的 tab（hasScope 含父级继承：有 news 或超管 → 全部可见）
const visibleTabs = computed(() => TABS.filter((t) => hasScope(currentUser.value, t.scope)));

const activeTab = ref(null);

// 当可见 tab 变化时，保证 activeTab 始终落在一个可见 tab 上（默认第一个可见）
watch(visibleTabs, (tabs) => {
    if (!tabs.length) { activeTab.value = null; return; }
    if (!tabs.some((t) => t.key === activeTab.value)) {
        activeTab.value = tabs[0].key;
    }
}, { immediate: true });

const activeTabDef = computed(() => TABS.find((t) => t.key === activeTab.value) || null);
const activeComponent = computed(() => activeTabDef.value?.comp || null);
const activeMode = computed(() => activeTabDef.value?.mode);

// 拉取当前用户对象（含 apps），用于 RBAC tab 门控
const refreshUser = async () => {
    try {
        currentUser.value = await admin.me();
    } catch {
        currentUser.value = null;
    }
};

// Login（统一 JWT：用户名 + 密码，校验 news 权限）
const login = async () => {
    if (!username.value || !password.value) return;
    loginLoading.value = true;
    loginMsg.value = '';
    try {
        const r = await store.loginAdmin(username.value.trim(), password.value);
        if (r.ok) {
            password.value = '';
            currentUser.value = r.user || null;
            if (!currentUser.value) await refreshUser();
            store.fetchAdminArticles();
            store.fetchPrizes();
            store.fetchActivities();
        } else {
            loginMsg.value = r.error || '用户名或密码错误，或该账号无新闻站管理权限';
        }
    } catch (e) {
        loginMsg.value = e?.message || '登录失败';
    } finally {
        loginLoading.value = false;
    }
};

const handleLogout = () => {
    store.logoutAdmin();
    currentUser.value = null;
};

// Stats / Tab badge data
const pendingCount = computed(() => store.adminArticles.filter(a => a.status === 'pending').length);
const publishedCount = computed(() => store.adminArticles.filter(a => (!a.status || a.status === 'published')).length);

// 管理态生效后（含异步 JWT 会话恢复完成）加载后台数据
const loadAdminData = () => {
    store.fetchAdminArticles();
    store.fetchPrizes();
    store.fetchActivities();
};

onMounted(() => {
    if (store.isAdmin) {
        loadAdminData();
        refreshUser();
    }
});

// 统一 JWT 会话恢复是异步的：isAdmin 由 false → true 时补拉数据 + 刷新用户
watch(() => store.isAdmin, (val) => {
    if (val) {
        loadAdminData();
        if (!currentUser.value) refreshUser();
    } else {
        currentUser.value = null;
    }
});
</script>

<style scoped>
/* ==================== Base / Root ==================== */
.admin-root {
  min-height: 100vh;
  background-color: rgba(249,250,251,0.5);
}

/* ==================== Login Section ==================== */
.login-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 80vh;
}

.login-card {
  background-color: #fff;
  padding: 2.5rem;
  border-radius: 0.75rem;
  border: 1px solid #e5e7eb;
  box-shadow: 0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1);
  max-width: 24rem;
  width: 100%;
  text-align: center;
  position: relative;
  overflow: hidden;
}

.login-top-bar {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 0.25rem;
  background-color: #000;
}

.login-header {
  margin-bottom: 1.5rem;
}

.login-avatar {
  width: 4rem;
  height: 4rem;
  background-color: #000;
  color: #fff;
  border-radius: 9999px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-left: auto;
  margin-right: auto;
  font-size: 1.5rem;
  line-height: 2rem;
  font-family: "Noto Serif SC", serif;
  font-weight: 900;
  margin-bottom: 1rem;
}

.login-title {
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
  font-family: "Noto Serif SC", serif;
}

.login-subtitle {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #9ca3af;
  margin-top: 0.25rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.login-input {
  width: 100%;
  background-color: #f9fafb;
  border: 1px solid #e5e7eb;
  padding: 0.75rem;
  border-radius: 0.5rem;
  margin-bottom: 1rem;
  outline: none;
  text-align: center;
  transition: all 150ms;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.login-input:focus {
  border-color: #000;
  box-shadow: 0 0 0 1px #000;
}

.login-msg {
  color: #dc2626;
  font-size: 0.85rem;
  text-align: center;
  margin: -0.25rem 0 0.75rem;
}

.login-btn[disabled] {
  opacity: 0.6;
  cursor: not-allowed;
}

.login-btn {
  width: 100%;
  background-color: #000;
  color: #fff;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  border-radius: 0.5rem;
  font-weight: 700;
  transition: transform 150ms;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.login-btn:hover {
  background-color: #1f2937;
}

.login-btn:active {
  transform: scale(0.95);
}

.icon-sm {
  width: 1rem;
  height: 1rem;
}

/* ==================== Dashboard Container ==================== */
.dashboard-container {
  max-width: 1600px;
  margin-left: auto;
  margin-right: auto;
  padding: 1rem;
}

@media (min-width: 768px) {
  .dashboard-container {
    padding: 2rem;
  }
}

/* ==================== Top Bar ==================== */
.top-bar {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 2rem;
  gap: 1rem;
}

@media (min-width: 768px) {
  .top-bar {
    flex-direction: row;
  }
}

.top-bar-title {
  font-size: 1.875rem;
  line-height: 2.25rem;
  font-weight: 900;
  color: #000;
  margin-bottom: 0.25rem;
}

.top-bar-subtitle {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #6b7280;
  font-family: "Noto Sans SC", sans-serif;
}

.top-bar-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.btn-back {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  color: #6b7280;
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms;
}

.btn-back:hover {
  color: #000;
  background-color: #fff;
}

.btn-logout {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  background-color: #fff;
  border: 1px solid #e5e7eb;
  color: #ef4444;
  border-radius: 0.5rem;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  transition: all 150ms;
}

.btn-logout:hover {
  border-color: #000;
  color: #dc2626;
}

.btn-new-content {
  padding: 0.5rem 1.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  background-color: #000;
  color: #fff;
  border-radius: 0.5rem;
  box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1);
  transition: transform 150ms;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-new-content:hover {
  background-color: #1f2937;
}

.btn-new-content:active {
  transform: scale(0.95);
}

/* ==================== Stats Grid ==================== */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

@media (min-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}

.stat-card {
  background-color: #fff;
  padding: 1.5rem;
  border-radius: 0.75rem;
  border: 1px solid #f3f4f6;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  transition: box-shadow 150ms;
}

.stat-card:hover {
  box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1);
}

.stat-label {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.stat-value {
  font-size: 1.875rem;
  line-height: 2.25rem;
  font-weight: 900;
}

.stat-blue { color: #2563eb; }

.stat-purple { color: #9333ea; }

.stat-green { color: #16a34a; }

/* ==================== Content Area ==================== */
.content-area {
  background-color: #fff;
  border-radius: 1rem;
  border: 1px solid #e5e7eb;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  overflow: hidden;
  min-height: 600px;
  display: flex;
  flex-direction: column;
}

/* ==================== Tabs ==================== */
.tabs-bar {
  display: flex;
  border-bottom: 1px solid #f3f4f6;
  padding-left: 1.5rem;
  padding-right: 1.5rem;
  padding-top: 0.5rem;
  overflow-x: auto;
  flex-shrink: 0;
}

.tab-btn {
  padding-bottom: 1rem;
  padding-top: 1rem;
  padding-left: 1rem;
  padding-right: 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  border-bottom: 2px solid transparent;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  white-space: nowrap;
}

.tab-btn-relative {
  position: relative;
}

.tab-btn-flex {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.tab-active {
  color: #000;
  border-bottom-color: #000;
}

.tab-inactive {
  color: #9ca3af;
  border-bottom-color: transparent;
}

.tab-inactive:hover {
  color: #4b5563;
}

.pending-badge {
  margin-left: 0.5rem;
  background-color: #eab308;
  color: #fff;
  font-size: 10px;
  padding: 0.125rem 0.375rem;
  border-radius: 9999px;
}
</style>
