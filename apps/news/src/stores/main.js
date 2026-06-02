import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { createApiClient, createAdminAuth } from '@haruhi/api-client';

// 统一后端约定：
// - 模块 API 统一前缀 /api/news（旧的 ${API_BASE}/xxx → /api/news/xxx）
// - 管理员鉴权改为统一 JWT 单点登录：登录走 /api/auth，业务请求由 api-client 自动注入 Authorization: Bearer <jwt>
// - 图片字段库里已是绝对路径 /uploads/news/<md5>.<ext>，前端直接使用，无需再拼前缀
// - 删除旧的 X-Admin-Token / VITE_ADMIN_TOKEN / VITE_ADMIN_PASSWORD / VITE_API_BASE 逻辑
const api = createApiClient('/api/news');
// 管理员鉴权收敛到共享的 createAdminAuth（登录/会话恢复/登出/news 权限校验统一在此）
const admin = createAdminAuth('news');

export const useMainStore = defineStore('main', () => {
    // --- State ---
    const dbArticles = ref([]);
    const adminArticles = ref([]);
    const prizes = ref([]);
    const activities = ref([]); // [新增] 活动列表
    const loading = ref(false);

    // 管理员态：基于统一 JWT 校验通过 news 权限后置位（会话恢复见 restoreAdmin）
    const isAdmin = ref(false);

    const selectedArticle = ref(null);
    const isSearchOpen = ref(false);
    const searchQuery = ref('');

    // --- Actions ---

    const logoutAdmin = () => {
        admin.logout();
        isAdmin.value = false;
        adminArticles.value = [];
    };

    // --- Activities Actions [新增] ---
    const fetchActivities = async () => {
        try {
            const result = await api.get('/activities');
            if (result.message === 'success') {
                activities.value = result.data || [];
            }
        } catch (error) {
            console.error('[Store] Failed to fetch activities:', error);
        }
    };

    const addActivity = async (data) => {
        if (!isAdmin.value) return false;
        try {
            await api.post('/admin/activities', data);
            await fetchActivities();
            return true;
        } catch (error) {
            console.error('Failed to add activity:', error);
            return false;
        }
    };

    const updateActivity = async (id, data) => {
        if (!isAdmin.value) return false;
        try {
            await api.put(`/admin/activities/${id}`, data);
            await fetchActivities();
            return true;
        } catch (error) {
            console.error('Failed to update activity:', error);
            return false;
        }
    };

    const deleteActivity = async (id) => {
        if (!isAdmin.value) return false;
        try {
            await api.del(`/admin/activities/${id}`);
            await fetchActivities();
            return true;
        } catch (error) {
            console.error('Failed to delete activity:', error);
            return false;
        }
    };

    const reorderActivities = async (ids) => {
        if (!isAdmin.value) return false;
        try {
            await api.post('/admin/activities/reorder', { ids });
            // Optimistically or refresh. Let's refresh to be sure.
            await fetchActivities();
            return true;
        } catch (error) {
            console.error('Failed to reorder activities:', error);
            return false;
        }
    };

    // --- Prizes Actions ---
    const fetchPrizes = async () => {
        try {
            const result = await api.get('/prizes');
            if (result.message === 'success') {
                prizes.value = result.data || [];
            }
        } catch (error) {
            console.error('[Store] Failed to fetch prizes:', error);
        }
    };

    const addPrize = async (prizeData) => {
        if (!isAdmin.value) return false;
        try {
            await api.post('/admin/prizes', prizeData);
            await fetchPrizes();
            return true;
        } catch (error) {
            return false;
        }
    };

    const updatePrize = async (id, prizeData) => {
        if (!isAdmin.value) return false;
        try {
            await api.put(`/admin/prizes/${id}`, prizeData);
            await fetchPrizes();
            return true;
        } catch (error) {
            return false;
        }
    };

    const deletePrize = async (id) => {
        if (!isAdmin.value) return false;
        try {
            await api.del(`/admin/prizes/${id}`);
            await fetchPrizes();
            return true;
        } catch (error) {
            return false;
        }
    };

    const reorderPrizes = async (ids) => {
        if (!isAdmin.value) return false;
        try {
            await api.post('/admin/prizes/reorder', { ids });
            await fetchPrizes();
            return true;
        } catch (error) {
            return false;
        }
    };

    // --- Article Actions ---
    const fetchAdminArticles = async () => {
        if (!isAdmin.value) return;
        try {
            const result = await api.get('/admin/articles');
            if (result.message === 'success') {
                adminArticles.value = result.data;
            }
        } catch (error) {
            // 401/403：JWT 失效或无权限，退出管理态
            if (error?.status === 401 || error?.status === 403) {
                logoutAdmin();
            }
            console.error('[Store] Failed to fetch admin articles:', error);
        }
    };

    const allArticles = computed(() => isAdmin.value ? adminArticles.value : dbArticles.value);

    const allTags = computed(() => {
        const tagCounts = {};
        allArticles.value.forEach(article => {
            if (article.tags && Array.isArray(article.tags)) {
                article.tags.forEach(tag => {
                    const cleanTag = tag.trim();
                    if (cleanTag) tagCounts[cleanTag] = (tagCounts[cleanTag] || 0) + 1;
                });
            }
        });
        return Object.entries(tagCounts).sort((a, b) => b[1] - a[1]).map(entry => entry[0]);
    });

    watch(isAdmin, (newVal) => {
        if (newVal) fetchAdminArticles();
        else adminArticles.value = [];
    });

    // 统一 JWT 登录：用户名 + 密码 → /api/auth/login，由 createAdminAuth 校验 news 权限。
    // 永不抛错，返回 { ok, user?, error? }；调用点把它当 truthy/falsy（ok）用即可。
    const loginAdmin = async (username, password) => {
        const r = await admin.login(username, password);
        isAdmin.value = r.ok;
        return r;
    };

    // 会话恢复：页面加载时若本地已有 JWT，则校验是否仍具 news 权限。
    const restoreAdmin = async () => {
        const user = await admin.restore();
        isAdmin.value = !!user;
        return isAdmin.value;
    };

    const fetchArticles = async () => {
        loading.value = true;
        try {
            const result = await api.get('/articles');
            if (result.message === 'success') dbArticles.value = result.data || [];
        } catch (error) {
            console.error('[Store] Failed to fetch articles:', error);
        } finally {
            loading.value = false;
        }
    };

    const fetchArticleById = async (id) => {
        try {
            const result = await api.get(`/articles/${id}`);
            return result.data;
        } catch (error) {
            return null;
        }
    };

    const addArticle = async (articleData) => {
        const { id, ...payload } = articleData;
        try {
            const result = await api.post('/articles', payload);
            await fetchArticles();
            if (isAdmin.value) await fetchAdminArticles();
            return result;
        } catch (error) {
            return false;
        }
    };

    const updateArticle = async (id, articleData) => {
        try {
            await api.put(`/articles/${id}`, articleData);
            await fetchArticles();
            await fetchAdminArticles();
            return true;
        } catch (error) {
            return false;
        }
    };

    const deleteArticle = async (id) => {
        try {
            await api.del(`/articles/${id}`);
            await fetchArticles();
            await fetchAdminArticles();
            return true;
        } catch (error) {
            return false;
        }
    };

    // --- Points Actions ---
    const fetchUserPoints = async (userId) => {
        try {
            const result = await api.get(`/points/${userId}`);
            return result.data;
        } catch (error) {
            return null;
        }
    };

    const searchUsers = async (query) => {
        try {
            const result = await api.get(`/points/search?q=${encodeURIComponent(query)}`);
            return result.data || [];
        } catch (error) {
            return [];
        }
    };

    const updateUserPoints = async (userId, change, reason) => {
        if (!isAdmin.value) return false;
        try {
            const result = await api.post('/points/update', { id: userId, change, reason });
            return result.data;
        } catch (error) {
            return false;
        }
    };

    // --- Modal Logic ---
    const openModal = async (article) => {
        if (!article) return;
        selectedArticle.value = article;
        document.body.style.overflow = 'hidden';
        if (article.isContentPreview) {
            const full = await fetchArticleById(article.id);
            if (full) {
                const idx = dbArticles.value.findIndex(a => a.id === full.id);
                if (idx !== -1) dbArticles.value[idx] = full;
                else dbArticles.value.push(full);
                if (selectedArticle.value && selectedArticle.value.id === full.id) selectedArticle.value = full;
            }
        }
    };

    const closeModal = () => {
        selectedArticle.value = null;
        document.body.style.overflow = '';
    };

    const toggleSearch = () => {
        isSearchOpen.value = !isSearchOpen.value;
        if (isSearchOpen.value) searchQuery.value = '';
    };

    fetchArticles();
    fetchPrizes();
    fetchActivities(); // Initial load
    // 恢复管理员会话（异步，成功后 watch(isAdmin) 会自动拉取后台数据）
    restoreAdmin();

    return {
        allArticles,
        adminArticles,
        prizes,
        activities, // Export activities
        allTags,
        selectedArticle,
        isSearchOpen,
        searchQuery,
        loading,
        isAdmin,
        loginAdmin,
        logoutAdmin,
        restoreAdmin,
        fetchArticles,
        fetchAdminArticles,
        fetchPrizes,
        addPrize,
        updatePrize,
        deletePrize,
        fetchActivities, // Export
        addActivity, // Export
        updateActivity, // Export
        deleteActivity, // Export
        reorderActivities,
        fetchArticleById,
        addArticle,
        updateArticle,
        deleteArticle,
        fetchUserPoints,
        searchUsers,
        updateUserPoints,
        openModal,
        closeModal,
        toggleSearch,
        reorderPrizes
    };
});
