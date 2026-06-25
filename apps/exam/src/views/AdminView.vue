<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { createApiClient } from '@haruhi/api-client';
import { useExamAdmin } from '@/composables/useExamAdmin';

// 统一鉴权：登录/会话恢复/登出走共享 createAdminAuth('exam')；后台数据请求由 createApiClient 自动带 JWT。
const { admin } = useExamAdmin();
const examApi = createApiClient('/api/exam');

const stats = ref({ siteVisits: 0, totalExams: 0, totalExamViews: 0 });
const exams = ref<any[]>([]);
const loading = ref(false);

const isAuthenticated = ref(false);
const loginUser = ref('');
const inputPw = ref('');
const loginMsg = ref('');
const loginLoading = ref(false);

// 登录：用户名 + 密码 → JWT，并校验是否有考试平台管理权限
const login = async () => {
  if (!loginUser.value.trim() || !inputPw.value) return;
  loginLoading.value = true;
  loginMsg.value = '';
  try {
    const r = await admin.login(loginUser.value.trim(), inputPw.value);
    if (!r.ok) {
      loginMsg.value = r.error || '登录失败';
      return;
    }
    isAuthenticated.value = true;
    inputPw.value = '';
    loadData();
  } finally {
    loginLoading.value = false;
  }
};

const logout = () => {
  admin.logout();
  isAuthenticated.value = false;
  exams.value = [];
};

const loadData = async () => {
  if (!isAuthenticated.value) return;
  loading.value = true;
  try {
    stats.value = await examApi.get('/admin/stats');
    exams.value = await examApi.get('/admin/list');
  } catch (e: any) {
    if (e?.status === 401 || e?.status === 403) {
      alert('登录已过期或无权限，请重新登录');
      logout();
    } else {
      console.error(e);
    }
  } finally {
    loading.value = false;
  }
};

const changeStatus = async (id: string, status: string) => {
  try {
    await examApi.post(`/admin/exams/${id}/status`, { status });
    loadData();
  } catch (e) {
    // error
  }
};

const deleteExam = async (id: string) => {
  if (!confirm('确定删除吗？操作不可恢复。')) return;
  try {
    await examApi.del(`/admin/exams/${id}`);
    loadData();
  } catch (e) {
    // error
  }
};

const goEdit = (id: string, token: string) => {
  window.open(`/exam/create?id=${id}&token=${token}`, '_blank');
};

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-';
  return new Date(dateStr).toLocaleString();
};

// 会话恢复：若已有有效 JWT 且仍具备权限，直接进入后台
onMounted(async () => {
  const user = await admin.restore();
  if (user) {
    isAuthenticated.value = true;
    loadData();
  }
});
</script>

<template>
  <div class="admin-container">
    
    <div v-if="!isAuthenticated" class="login-layer">
      <div class="login-card">
        <div class="login-icon">🔒</div>
        <h2>控制台访问受限</h2>
        <p>请使用统一账号登录以继续操作</p>
        <div class="login-form">
          <input
            type="text"
            v-model="loginUser"
            placeholder="用户名"
            @keyup.enter="login"
            autofocus
          />
          <input
            type="password"
            v-model="inputPw"
            placeholder="密码"
            @keyup.enter="login"
          />
          <button class="ui green block" @click="login" :disabled="loginLoading">
            {{ loginLoading ? '登录中...' : '登录' }}
          </button>
        </div>
        <div v-if="loginMsg" class="login-msg">{{ loginMsg }}</div>
      </div>
    </div>

    <template v-else>
      <div class="header">
        <div class="header-row">
          <h1>📊 试炼场控制台</h1>
          <button class="ui small ghost" @click="logout">退出登录</button>
        </div>
        <div class="stats-bar">
          <div class="stat-item">
            <span class="label">全站总访问</span>
            <span class="val">{{ stats.siteVisits }}</span>
          </div>
          <div class="stat-item">
            <span class="label">试卷总数</span>
            <span class="val">{{ stats.totalExams }}</span>
          </div>
          <div class="stat-item">
            <span class="label">试卷总阅览</span>
            <span class="val">{{ stats.totalExamViews }}</span>
          </div>
        </div>
      </div>

      <div class="table-card">
        <div class="toolbar">
          <h3>试卷管理列表</h3>
          <button class="ui small" @click="loadData">刷新</button>
        </div>

        <div class="table-scroll">
          <table>
            <thead>
              <tr>
                <th class="col-status">状态</th>
                <th class="col-title">标题/ID</th>
                <!-- [新增] 联系方式列 -->
                <th class="col-contact">联系方式</th>
                <th class="col-heat">热度</th>
                <th class="col-reason">AI 审核意见</th>
                <th class="col-date">创建时间</th>
                <th class="col-action">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="ex in exams" :key="ex.id">
                <td>
                  <span class="badge" :class="ex.status">
                    {{ ex.status === 'published' ? '已发布' : ex.status === 'locked' ? '已锁定' : '审核中' }}
                  </span>
                </td>
                <td>
                  <div class="ex-title" :title="ex.title">{{ ex.title }}</div>
                  <div class="ex-id">{{ ex.id }}</div>
                </td>
                <!-- [新增] 联系方式展示 -->
                <td class="cell-nowrap">{{ ex.config?.contact || '-' }}</td>
                <td>{{ ex.visit_count }}</td>
                <td class="cell-reason">
                  <div class="reason-text">{{ ex.ai_reason || '-' }}</div>
                </td>
                <td class="cell-nowrap">{{ formatDate(ex.created_at) }}</td>
                <td>
                  <div class="actions">
                    <button @click="goEdit(ex.id, ex.edit_token)" class="btn-text blue">编辑</button>
                    <button v-if="ex.status !== 'published'" @click="changeStatus(ex.id, 'published')" class="btn-text green">通过</button>
                    <button v-if="ex.status !== 'locked'" @click="changeStatus(ex.id, 'locked')" class="btn-text orange">锁定</button>
                    <button @click="deleteExam(ex.id)" class="btn-text red">删除</button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
          <div v-if="exams.length === 0 && !loading" class="empty-state">
            暂无数据
          </div>
        </div>
      </div>
    </template>

  </div>
</template>

<style scoped>
.admin-container { 
  max-width: 1400px; 
  margin: 0 auto; 
  padding: 40px 20px; 
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; 
  background: var(--sos-bg-page);
  min-height: 100vh; 
}

.login-layer { display: flex; align-items: center; justify-content: center; height: 60vh; }
.login-card { 
  background: var(--sos-bg-surface);
  padding: 40px; 
  border-radius: 16px; 
  box-shadow: 0 20px 40px rgba(0,0,0,0.08); 
  text-align: center; 
  width: 100%; 
  max-width: 380px;
}
.login-icon { font-size: 48px; margin-bottom: 16px; }
.login-card h2 { margin: 0 0 8px; color: var(--sos-text-primary); }
.login-card p { color: var(--sos-text-tertiary); margin-bottom: 24px; font-size: 14px; }
.login-form { display: flex; flex-direction: column; gap: 12px; }
.login-form input {
  padding: 12px 14px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  outline: none;
  transition: all 0.2s;
  font-size: 14px;
}
.login-form input:focus { border-color: var(--sos-accent); box-shadow: var(--sos-ring); }
.ui.green.block { width: 100%; }
.ui.green:disabled { opacity: 0.6; cursor: not-allowed; }
.login-msg { margin-top: 14px; color: var(--sos-danger); font-size: 13px; }

.header { margin-bottom: 30px; }
.header-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.header h1 { font-size: 24px; font-weight: 700; color: var(--sos-text-primary); margin: 0; }

.stats-bar { display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; }
.stat-item { 
  background: var(--sos-bg-surface);
  padding: 24px; 
  border-radius: 12px; 
  box-shadow: 0 2px 10px rgba(0,0,0,0.03); 
  display: flex; 
  flex-direction: column; 
}
.stat-item .label { font-size: 13px; color: var(--sos-text-tertiary); margin-bottom: 8px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; }
.stat-item .val { font-size: 32px; font-weight: 800; color: var(--sos-text-primary); line-height: 1; }

.table-card { background: #fff; border-radius: 12px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05); overflow: hidden; border: 1px solid var(--sos-border-default); }
.toolbar { padding: 20px 24px; border-bottom: 1px solid var(--sos-bg-muted); display: flex; justify-content: space-between; align-items: center; }
.toolbar h3 { margin: 0; font-size: 16px; font-weight: 600; color: var(--sos-text-secondary); }

.ui.small { padding: 8px 14px; font-size: 13px; border: 1px solid var(--sos-border-default); background: var(--sos-bg-surface); color: var(--sos-accent-2); border-radius: 6px; cursor: pointer; transition: all 0.2s; }
.ui.small:hover { background: var(--sos-bg-subtle); border-color: var(--sos-border-strong); }
.ui.green { background: var(--sos-accent); color: var(--sos-accent-contrast); border: none; padding: 10px 20px; border-radius: 8px; cursor: pointer; font-weight: 600; white-space: nowrap; }
.ui.green:hover { background: var(--sos-accent-hover); }
.ui.ghost { background: transparent; border: 1px solid var(--sos-border-strong); color: var(--sos-text-secondary); }
.ui.ghost:hover { background: var(--sos-bg-muted); }

.table-scroll { overflow-x: auto; width: 100%; }
table { width: 100%; border-collapse: collapse; font-size: 14px; white-space: normal; }

th { 
  text-align: left; 
  padding: 14px 24px; 
  background: var(--sos-bg-subtle); 
  color: var(--sos-text-secondary); 
  font-weight: 600; 
  font-size: 12px; 
  text-transform: uppercase; 
  border-bottom: 1px solid var(--sos-border-default);
}

.col-status { width: 100px; min-width: 100px; }
.col-title { min-width: 260px; }
.col-contact { width: 180px; min-width: 180px; } /* 新增列宽 */
.col-heat { width: 80px; }
.col-reason { min-width: 320px; }
.col-date { width: 180px; min-width: 180px; }
.col-action { width: 200px; min-width: 200px; }

td { 
  padding: 16px 24px; 
  border-bottom: 1px solid var(--sos-bg-muted); 
  color: var(--sos-text-secondary); 
  vertical-align: top; 
}
tr:last-child td { border-bottom: none; }
tr:hover td { background-color: var(--sos-bg-subtle); }

.ex-title { font-weight: 600; color: var(--sos-text-primary); margin-bottom: 4px; line-height: 1.4; font-size: 15px; }
.ex-id { font-size: 12px; color: var(--sos-text-tertiary); font-family: monospace; }

.cell-reason { max-width: 500px; }
.reason-text { color: var(--sos-text-secondary); line-height: 1.6; font-size: 13px; background: var(--sos-bg-subtle); padding: 8px 12px; border-radius: 6px; border: 1px solid var(--sos-bg-muted); }
.cell-nowrap { white-space: nowrap; color: var(--sos-text-tertiary); font-size: 13px; }

.badge { padding: 4px 10px; border-radius: 99px; font-size: 12px; font-weight: 600; display: inline-flex; align-items: center; white-space: nowrap; }
.badge.published { background: var(--sos-success-soft); color: var(--sos-success); }
.badge.locked { background: var(--sos-danger-soft); color: var(--sos-danger); }
.badge.pending { background: color-mix(in srgb, var(--sos-signal) 20%, var(--sos-bg-surface)); color: var(--sos-warning); }

.actions { display: flex; gap: 16px; align-items: center; }
.btn-text { background: none; border: none; cursor: pointer; font-size: 13px; font-weight: 500; padding: 0; transition: opacity 0.2s; white-space: nowrap; }
.btn-text:hover { text-decoration: underline; opacity: 0.8; }
.btn-text.green { color: var(--sos-success); }
.btn-text.orange { color: var(--sos-warning); }
.btn-text.red { color: var(--sos-danger); }
.btn-text.blue { color: var(--sos-accent-2); }

.empty-state { text-align:center; padding: 60px; color:var(--sos-text-tertiary); font-size: 14px; }

@media (max-width: 768px) {
  .stats-bar { grid-template-columns: 1fr; gap: 12px; }
  .header-row { flex-direction: column; gap: 12px; align-items: flex-start; }
  .login-card { padding: 30px 20px; }
}
</style>