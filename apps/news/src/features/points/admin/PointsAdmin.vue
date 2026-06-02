<template>
  <div class="points-tab">
    <!-- Keep existing points management -->
    <div class="points-sidebar">
      <div class="points-search-bar">
        <div class="points-search-group">
          <div class="points-search-input-wrap">
            <svg class="search-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>
            <input v-model="pointsSearchId" @input="handlePointsInput" @keyup.enter="handlePointsSearch" type="text" class="points-search-input" placeholder="搜索用户 ID...">
          </div>
          <div v-if="suggestions.length > 0" class="suggestions-dropdown">
            <div v-for="(suggestion, index) in suggestions" :key="index" @click="selectSuggestion(suggestion)" class="suggestion-item">{{ suggestion }}</div>
          </div>
        </div>
      </div>
      <div class="user-list">
        <div v-if="pointsUserList.length === 0 && !pointsLoading" class="user-list-empty">
          <p class="user-list-empty-text">列表为空</p>
          <button @click="fetchAllPointsUsers" class="user-list-load-btn">加载所有用户</button>
        </div>
        <div v-for="user in pointsUserList" :key="user.id" @click="selectUserFromList(user)" class="user-card" :class="currentPointsUser?.id === user.id ? 'user-card-active' : 'user-card-inactive'">
          <div class="user-card-top">
            <span class="user-card-id">{{ user.id }}</span>
            <span class="user-card-label">User</span>
          </div>
          <div class="user-card-bottom">
            <span class="user-card-pts-label">现有积分</span>
            <span class="user-card-pts-value">{{ user.total }}</span>
          </div>
        </div>
      </div>
    </div>
    <div class="points-main">
      <div v-if="currentPointsUser" class="points-detail animate-slide-up-sm">
        <div class="points-detail-header">
             <div>
              <h2 class="points-detail-title">用户详情</h2>
              <p class="points-detail-id">{{ currentPointsUser.id }}</p>
             </div>
             <div class="points-detail-total-wrap">
                 <div class="points-total-label">Total Points</div>
                 <div class="points-total-value">{{ currentPointsUser.total }}</div>
             </div>
        </div>
        <div class="points-adjust-box">
          <h3 class="points-adjust-title">调整积分</h3>
          <div class="points-adjust-grid">
            <div class="points-adjust-amount">
              <label class="form-label">变动数额</label>
              <input v-model.number="pointChangeAmount" type="number" placeholder="+/- Amount" class="form-input form-input-mono">
            </div>
            <div class="points-adjust-reason">
              <label class="form-label">变动原因</label>
              <input v-model="pointChangeReason" type="text" placeholder="Reason" class="form-input">
            </div>
          </div>
          <div class="points-adjust-submit">
            <button @click="submitPointsUpdate" :disabled="pointsUpdating || !pointChangeAmount" class="btn-primary-disabled">
              {{ pointsUpdating ? '处理中...' : '确认修改' }}
            </button>
          </div>
        </div>
        <div>
          <h3 class="points-history-title">历史记录</h3>
          <table class="data-table">
            <thead class="table-head">
              <tr><th class="th-sort">日期</th><th class="th-cell">原因</th><th class="th-cell th-right">变动</th></tr>
            </thead>
            <tbody class="table-body">
              <tr v-for="(record, idx) in (currentPointsUser.history || [])" :key="idx" class="history-row">
                <td class="htd-date">{{ record.date }}</td>
                <td class="htd-reason">{{ record.reason || '-' }}</td>
                <td class="htd-change" :class="String(record.change || '').startsWith('+') ? 'text-success' : 'text-danger'">{{ record.change || '0' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
      <div v-else class="points-empty">
        <p class="points-empty-text">请选择一个用户</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { useMainStore } from '@/stores/main';
import { createApiClient } from '@haruhi/api-client';

// 统一后端：积分用户列表也走 /api/news + 自动注入 JWT
const newsApi = createApiClient('/api/news');

const store = useMainStore();

// Points Management State
const pointsSearchId = ref('');
const currentPointsUser = ref(null);
const pointsLoading = ref(false);
const pointsUpdating = ref(false);
const pointChangeAmount = ref('');
const pointChangeReason = ref('');
const pointsUserList = ref([]); // List of users with points

// Fuzzy Search State
const showSuggestions = ref(false);
const suggestions = ref([]);
// Debounce helper
const debounce = (fn, delay) => {
    let timeoutId;
    return (...args) => {
        if (timeoutId) clearTimeout(timeoutId);
        timeoutId = setTimeout(() => {
            fn(...args);
        }, delay);
    };
};

const handleLogout = () => {
    store.logoutAdmin();
    // Reset points state
    currentPointsUser.value = null;
    pointsSearchId.value = '';
    pointsUserList.value = [];
};

// ================= Points Management Actions =================

// Helper to update local list (History behavior)
const updateLocalList = (user) => {
    const existingIndex = pointsUserList.value.findIndex(u => u.id === user.id);
    if (existingIndex !== -1) {
        // Move to top if exists
        pointsUserList.value.splice(existingIndex, 1);
        pointsUserList.value.unshift(user);
    } else {
        // Add new to top
        pointsUserList.value.unshift(user);
    }
};

// Fetch list of all users from API
const fetchAllPointsUsers = async () => {
    pointsLoading.value = true;
    try {
        const result = await newsApi.get('/admin/points/users');
        // Assuming response is { message: "success", data: [...] }
        pointsUserList.value = result.data || [];
    } catch (e) {
        console.error("Error fetching user list:", e);
        // 401/403：JWT 失效或无权限，退出管理态
        if (e?.status === 401 || e?.status === 403) handleLogout();
    } finally {
        pointsLoading.value = false;
    }
};

const buildPointsUser = (userId, data, fallback = {}) => ({
    ...fallback,
    ...data,
    id: userId,
    total: Number(data?.total ?? fallback?.total ?? 0),
    history: Array.isArray(data?.history) ? data.history : (Array.isArray(fallback?.history) ? fallback.history : [])
});

// 1. Search Logic
const handlePointsSearch = async () => {
    showSuggestions.value = false; // Hide suggestions
    const userId = pointsSearchId.value.trim();
    if (!userId) return;

    pointsLoading.value = true;
    try {
        const data = await store.fetchUserPoints(userId);
        if (data) {
            const userWithId = buildPointsUser(userId, data);
            currentPointsUser.value = userWithId;

            // Add to local history list if not fully loaded
            updateLocalList(userWithId);

            // Clear form
            pointChangeAmount.value = '';
            pointChangeReason.value = '';
        } else {
            alert('未找到用户或加载失败');
        }
    } finally {
        pointsLoading.value = false;
    }
};

// 2. Fuzzy Suggestions Logic
const fetchSuggestions = async () => {
    if (!pointsSearchId.value || pointsSearchId.value.trim().length < 1) {
        suggestions.value = [];
        return;
    }

    if (typeof store.searchUsers === 'function') {
        try {
            const results = await store.searchUsers(pointsSearchId.value);
            suggestions.value = Array.isArray(results) ? results : [];
        } catch (error) {
            console.error(error);
            suggestions.value = [];
        }
    }
};

const debouncedFetch = debounce(fetchSuggestions, 300);

const handlePointsInput = () => {
    showSuggestions.value = true;
    debouncedFetch();
};

const handlePointsFocus = () => {
    if (pointsSearchId.value) {
        showSuggestions.value = true;
        debouncedFetch();
    }
};

const handlePointsBlur = () => {
    setTimeout(() => {
        showSuggestions.value = false;
    }, 200);
};

const selectSuggestion = (suggestion) => {
    pointsSearchId.value = suggestion;
    showSuggestions.value = false;
    handlePointsSearch();
};

const selectUserFromList = (user) => {
    pointsSearchId.value = user.id;
    pointChangeAmount.value = '';
    pointChangeReason.value = '';
    loadUserDetail(user);
};

const loadUserDetail = async (user) => {
    if (!user?.id) return;

    pointsLoading.value = true;
    try {
        const data = await store.fetchUserPoints(user.id);
        if (!data) {
            currentPointsUser.value = buildPointsUser(user.id, null, user);
            return;
        }
        const userWithId = buildPointsUser(user.id, data, user);
        currentPointsUser.value = userWithId;
        // 仅用于右侧详情展示，避免点击查看时改变左侧列表排序
    } finally {
        pointsLoading.value = false;
    }
};

// 3. Update Logic
const submitPointsUpdate = async () => {
    if (!currentPointsUser.value || !pointChangeAmount.value) return;

    // CRITICAL: Check ID existence
    if (!currentPointsUser.value.id) {
        alert("错误：当前用户ID丢失，请重新搜索");
        return;
    }

    pointsUpdating.value = true;
    try {
        const updatedUser = await store.updateUserPoints(
            currentPointsUser.value.id,
            pointChangeAmount.value,
            pointChangeReason.value
        );

        if (updatedUser) {
            // Ensure ID persists in the returned object or merge it
            const userWithId = { ...updatedUser, id: currentPointsUser.value.id };

            // Update current view
            currentPointsUser.value = userWithId;
            // Update list view
            updateLocalList(userWithId);

            pointChangeAmount.value = '';
            pointChangeReason.value = '';
            alert('修改成功');
        } else {
            alert('修改失败');
        }
    } catch (e) {
        console.error("Update failed", e);
        alert('修改出错: ' + e.message);
    } finally {
        pointsUpdating.value = false;
    }
};

// 进入积分 tab 即加载用户列表（原后台壳登录/恢复后调 fetchAllPointsUsers）
onMounted(() => {
    fetchAllPointsUsers();
});
</script>

<style scoped>
.data-table {
  width: 100%;
  text-align: left;
  border-collapse: collapse;
}

.table-head {
  background-color: #f9fafb;
  font-size: 0.75rem;
  line-height: 1rem;
  text-transform: uppercase;
  color: #6b7280;
}

.table-body {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.th-sort {
  padding: 0.75rem;
  padding-left: 1rem;
  font-weight: 700;
  width: 3rem;
}

.th-cell {
  padding: 0.75rem;
  font-weight: 700;
}

.th-right {
  text-align: right;
  padding-right: 1rem;
}

/* ==================== Utility Colors ==================== */
.text-success { color: #16a34a; }

.text-danger { color: #ef4444; }

/* ==================== Points Tab (Tab 4) ==================== */
.points-tab {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 600px;
  overflow: hidden;
}

@media (min-width: 768px) {
  .points-tab {
    flex-direction: row;
  }
}

.points-sidebar {
  width: 100%;
  border-right: 1px solid #f3f4f6;
  display: flex;
  flex-direction: column;
  background-color: rgba(249,250,251,0.5);
}

@media (min-width: 768px) {
  .points-sidebar {
    width: 33.333%;
  }
}

.points-search-bar {
  padding: 1rem;
  border-bottom: 1px solid #f3f4f6;
  background-color: #fff;
  position: sticky;
  top: 0;
  z-index: 20;
}

.points-search-group {
  position: relative;
  width: 100%;
}

.points-search-input-wrap {
  position: relative;
  display: flex;
  align-items: center;
  background-color: #fff;
  border: 1px solid #d1d5db;
  border-radius: 0.5rem;
  transition: all 150ms;
}

.points-search-input-wrap:focus-within {
  border-color: #000;
  box-shadow: 0 0 0 1px #000;
}

.search-icon {
  width: 1rem;
  height: 1rem;
  margin-left: 0.75rem;
  color: #9ca3af;
}

.points-search-input {
  width: 100%;
  font-size: 0.875rem;
  line-height: 1.25rem;
  padding: 0.75rem;
  outline: none;
  background-color: transparent;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.suggestions-dropdown {
  position: absolute;
  left: 0;
  right: 0;
  top: 100%;
  margin-top: 0.5rem;
  background-color: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  box-shadow: 0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1);
  z-index: 50;
  overflow: hidden;
  max-height: 15rem;
  overflow-y: auto;
}

.suggestion-item {
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  cursor: pointer;
  border-bottom: 1px solid #f9fafb;
}

.suggestion-item:hover {
  background-color: #f9fafb;
}

.suggestion-item:last-child {
  border-bottom: 0;
}

.user-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.user-list > * + * {
  margin-top: 0.5rem;
}

.user-list-empty {
  text-align: center;
  padding-top: 2.5rem;
  padding-bottom: 2.5rem;
  color: #9ca3af;
  font-size: 0.75rem;
  line-height: 1rem;
  padding-left: 1rem;
  padding-right: 1rem;
}

.user-list-empty-text {
  margin-bottom: 0.5rem;
}

.user-list-load-btn {
  margin-top: 0.5rem;
  color: #2563eb;
}

.user-list-load-btn:hover {
  text-decoration: underline;
}

.user-card {
  padding: 0.75rem;
  border-radius: 0.5rem;
  border: 1px solid;
  cursor: pointer;
  transition: all 150ms;
}

.user-card:hover {
  box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1);
}

.user-card-active {
  background-color: #000;
  color: #fff;
  border-color: #000;
}

.user-card-inactive {
  background-color: #fff;
  border-color: #e5e7eb;
  color: #374151;
}

.user-card-inactive:hover {
  border-color: #d1d5db;
}

.user-card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.25rem;
}

.user-card-id {
  font-weight: 700;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.user-card-label {
  font-size: 0.75rem;
  line-height: 1rem;
  opacity: 0.7;
}

.user-card-bottom {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}

.user-card-pts-label {
  font-size: 0.75rem;
  line-height: 1rem;
  opacity: 0.7;
}

.user-card-pts-value {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1;
}

/* Points Main Panel */
.points-main {
  width: 100%;
  padding: 1.5rem;
  overflow-y: auto;
  background-color: #fff;
  position: relative;
}

@media (min-width: 768px) {
  .points-main {
    width: 66.666%;
    padding: 2.5rem;
  }
}

.points-detail {
  max-width: 42rem;
  margin-left: auto;
  margin-right: auto;
}

.points-detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid #f3f4f6;
}

.points-detail-title {
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
  font-family: "Noto Serif SC", serif;
  margin-bottom: 0.25rem;
}

.points-detail-id {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: #6b7280;
}

.points-detail-total-wrap {
  text-align: right;
}

.points-total-label {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.25rem;
}

.points-total-value {
  font-size: 2.25rem;
  line-height: 2.5rem;
  font-weight: 900;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: #9333ea;
}

.points-adjust-box {
  background-color: #f9fafb;
  border: 1px solid #e5e7eb;
  padding: 1.5rem;
  border-radius: 0.75rem;
  margin-bottom: 2rem;
}

.points-adjust-title {
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1.75rem;
  margin-bottom: 1rem;
}

.points-adjust-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1rem;
  margin-bottom: 1rem;
}

@media (min-width: 768px) {
  .points-adjust-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

.points-adjust-amount {
  grid-column: span 1;
}

@media (min-width: 768px) {
  .points-adjust-amount {
    grid-column: span 1;
  }
  .points-adjust-reason {
    grid-column: span 2;
  }
}

.points-adjust-submit {
  display: flex;
  justify-content: flex-end;
}

.points-history-title {
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1.75rem;
  margin-bottom: 1rem;
}

/* History Table */
.history-row {
  border-top: 1px solid #f3f4f6;
}

.history-row:hover {
  background-color: #f9fafb;
}

.htd-date {
  padding: 0.75rem;
  padding-left: 1rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: #6b7280;
  font-size: 0.75rem;
  line-height: 1rem;
}

.htd-reason {
  padding: 0.75rem;
  font-weight: 500;
}

.htd-change {
  padding: 0.75rem;
  text-align: right;
  padding-right: 1rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-weight: 700;
}

.points-empty {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #d1d5db;
}

.points-empty-text {
  font-size: 1.125rem;
  line-height: 1.75rem;
  font-weight: 700;
}

.btn-primary-disabled {
  background-color: #000;
  color: #fff;
  padding: 0.5rem 1.5rem;
  border-radius: 0.5rem;
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.btn-primary-disabled:hover {
  background-color: #1f2937;
}

.btn-primary-disabled:disabled {
  opacity: 0.5;
}

/* ==================== Form Elements ==================== */
.form-label {
  display: block;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.25rem;
}

.form-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.25rem;
}

.form-input-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}
</style>
