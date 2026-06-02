<script setup lang="ts">
import { ref, reactive, onMounted, computed, nextTick, watch, onBeforeUnmount } from 'vue';
import { useRouter, useRoute, onBeforeRouteLeave } from 'vue-router';
import { useExamStore } from '@/stores/exam';
import { EXAM_DB } from '@/data/mock-exam';
import type { ExamDatabase, LevelConfig } from '@/types/exam';

import ConfigPanel from '@/components/editor/ConfigPanel.vue';
import LevelsPanel from '@/components/editor/LevelsPanel.vue';
import QuestionsPanel from '@/components/editor/QuestionsPanel.vue';
import { api } from '@/services/api';
import { createAuth } from '@haruhi/api-client';

// 统一鉴权：导入试卷需要管理员权限，改为统一 JWT 单点登录（用户名+密码）。
const auth = createAuth('/api');
const hasExamPerm = (user: any): boolean => !!user && (user.isSuperAdmin || (user.apps && user.apps.exam));

const router = useRouter();
const route = useRoute();
const examStore = useExamStore();

const activeTab = ref<'base' | 'levels' | 'questions'>('base');
const isSaving = ref(false);
const isDirty = ref(false);

const mode = ref('create');
const editId = ref('');
const editToken = ref('');

// 用于访问 ConfigPanel 组件实例以触发验证
const configPanelRef = ref<InstanceType<typeof ConfigPanel> | null>(null);

// 状态管理
const currentStatus = ref('creating'); // 默认为创建中
const statusMap: Record<string, string> = {
  creating: '创建中',
  pending: '审核中',
  published: '已发布',
  locked: '已锁定(存在异常，若为误判可联系管理员邮箱haruhiism15532@outlook.com)'
};
const statusText = computed(() => statusMap[currentStatus.value] || currentStatus.value);

// 数据模型
const examData = reactive<ExamDatabase>({
  id: '',
  config: { 
    ...EXAM_DB.config, 
    title: '',    
    paperTitle: '',
    paperSubtitle: '',
    paperMeta: '绝密 ★ 启用前',
    // [新增] 必填项初始化
    author: '',
    contact: '',
    className: '北高一年五班'
  },
  questions: [],
  levels: JSON.parse(JSON.stringify(EXAM_DB.levels))
});

// 监听 paperTitle 自动同步到 exportHeaderTitle
watch(() => examData.config.paperTitle, (newVal) => {
  examData.config.exportHeaderTitle = newVal;
}, { immediate: true });

// 深度监听数据变化，标记为未保存
watch(examData, () => {
  isDirty.value = true;
}, { deep: true });

// 成功弹窗
const successModal = reactive({
  show: false,
  id: '',
  token: '',
  editLink: ''
});

// 导入相关状态
const isImporting = ref(false);
const importFileInput = ref<HTMLInputElement | null>(null);
const importModal = reactive({
  show: false,
  success: false,
  message: '',
  details: '',
  id: '',
  token: '',
  editLink: ''
});

// 管理员登录弹窗状态（统一 JWT：用户名+密码）
const adminModal = reactive<{
  show: boolean;
  user: string;
  pw: string;
  error: string;
  loading: boolean;
  resolve: ((v: boolean) => void) | null;
}>({
  show: false,
  user: '',
  pw: '',
  error: '',
  loading: false,
  resolve: null,
});

// 确保已登录且具备考试平台管理权限：已满足则直接 true；否则弹出登录框。
const ensureAdmin = async (): Promise<boolean> => {
  if (auth.isLoggedIn()) {
    try {
      const user = await auth.me();
      if (hasExamPerm(user)) return true;
    } catch {
      // 会话失效，落入登录流程
    }
    auth.logout();
  }
  return showAdminModal();
};

// 显示登录弹窗，返回是否登录成功
const showAdminModal = (): Promise<boolean> => {
  return new Promise((resolve) => {
    adminModal.show = true;
    adminModal.user = '';
    adminModal.pw = '';
    adminModal.error = '';
    adminModal.loading = false;
    adminModal.resolve = resolve;
  });
};

// 登录校验
const verifyAdmin = async () => {
  if (!adminModal.user.trim() || !adminModal.pw) {
    adminModal.error = '请输入用户名和密码';
    return;
  }
  adminModal.loading = true;
  adminModal.error = '';
  try {
    const user = await auth.login(adminModal.user.trim(), adminModal.pw);
    if (!hasExamPerm(user)) {
      auth.logout();
      adminModal.error = '该账号无考试平台管理权限';
      return;
    }
    adminModal.show = false;
    adminModal.resolve?.(true);
  } catch (e: any) {
    auth.logout();
    adminModal.error = e?.status === 401 ? '用户名或密码错误' : (e?.message || '登录失败');
  } finally {
    adminModal.loading = false;
  }
};

// 取消登录
const cancelAdmin = () => {
  adminModal.show = false;
  adminModal.resolve?.(false);
};

// 浏览器刷新或关闭时的原生弹窗处理
const handleBeforeUnload = (e: BeforeUnloadEvent) => {
  if (isDirty.value) {
    e.preventDefault();
    e.returnValue = ''; // 现代浏览器需要设置这个属性才会弹出提示
  }
};

// 挂载和卸载事件监听
onMounted(async () => {
  window.addEventListener('beforeunload', handleBeforeUnload);

  const qId = route.query.id as string;
  const qToken = route.query.token as string;
  
  if (qId && qToken) {
    mode.value = 'edit';
    editId.value = qId;
    editToken.value = qToken;
    
    // 调用 Store 的验证加载方法
    const loaded = await examStore.verifyAndLoadExam(qId, qToken);
    if (loaded) {
      Object.assign(examData, JSON.parse(JSON.stringify(examStore.paper)));
      currentStatus.value = examStore.paper.status || 'pending';
    } else {
      alert('无法加载试卷：链接无效或试卷不存在');
      router.push('/');
    }
  }

  // 数据初始化完成后，等待一个 tick，重置 dirty 状态
  await nextTick();
  isDirty.value = false;
});

onBeforeUnmount(() => {
  window.removeEventListener('beforeunload', handleBeforeUnload);
});

// 路由守卫：处理应用内跳转（如点击侧边栏返回）
onBeforeRouteLeave((to, from, next) => {
  if (isDirty.value) {
    const answer = window.confirm('您有未保存的更改，离开页面将导致数据丢失，确定要离开吗？');
    if (answer) {
      next();
    } else {
      next(false);
    }
  } else {
    next();
  }
});

// --- 验证逻辑 ---
const validateLevels = (levels: LevelConfig[]) => {
  for (let i = 0; i < levels.length; i++) {
    const lv1 = levels[i];
    // 基础完整性
    if (lv1.min === undefined || lv1.min === null || lv1.max === undefined || lv1.max === null) {
       return `等级 #${i + 1} (${lv1.name}) 分数配置不完整`;
    }
    // 区间合法性
    if (lv1.min > lv1.max) {
      return `等级 #${i + 1} (${lv1.name}) 配置错误：最低分不能高于最高分`;
    }
    // 冲突检测
    for (let j = i + 1; j < levels.length; j++) {
      const lv2 = levels[j];
      if (lv2.min === undefined || lv2.min === null || lv2.max === undefined || lv2.max === null) continue;
      
      // 判定重叠: max(start1, start2) <= min(end1, end2)
      const isOverlap = Math.max(lv1.min, lv2.min) <= Math.min(lv1.max, lv2.max);
      if (isOverlap) {
        return `等级分数冲突：等级「${lv1.name || '#' + (i + 1)}」与「${lv2.name || '#' + (j + 1)}」区间重叠，请修正`;
      }
    }
  }
  return null;
};

const save = async () => {
  // 1. 题目非空校验
  if (examData.questions.length === 0) {
    activeTab.value = 'questions'; // 自动定位到题目Tab
    return alert('提交失败：请至少添加一道题目！');
  }

  // 2. 基础配置非空校验
  const c = examData.config;
  // [关键修改] 增加 author, contact, className 的非空校验
  // @ts-ignore
  const isConfigInvalid = !c.title || !c.paperTitle || !c.paperSubtitle || !c.paperMeta || !c.author || !c.contact || !c.className;
  
  if (isConfigInvalid) {
    activeTab.value = 'base'; // 切换到基础设置Tab
    await nextTick(); // 等待组件挂载
    // 触发组件内部的验证方法（显示红框）
    if (configPanelRef.value) {
      configPanelRef.value.validate();
    }
    return alert('提交失败：基础设置中有未填写的必填项（包括联系方式、署名、班级）！');
  }

  // 3. 等级配置非空校验
  if (examData.levels.length === 0) {
    activeTab.value = 'levels'; // 自动定位到等级Tab
    return alert('提交失败：等级配置不能为空，请至少添加一个等级！');
  }
  
  // 4. 等级配置冲突校验
  const levelError = validateLevels(examData.levels);
  if (levelError) {
    activeTab.value = 'levels'; // 自动定位到等级Tab，让用户看到红框
    return alert(`提交失败：${levelError}`);
  }

  // 5. 等级配置图片非空校验
  for (let i = 0; i < examData.levels.length; i++) {
    const lv = examData.levels[i];
    if (!lv.img || !lv.sketch) {
      activeTab.value = 'levels'; // 自动定位到等级Tab
      const missingField = !lv.img ? '头像图片' : '背景图片';
      return alert(`提交失败，等级配置中未选择头像或背景图片！\n\n等级 #${i + 1}「${lv.name || '未命名'}」缺少${missingField}，请为所有等级配置完整的头像和背景图片。`);
    }
  }

  // 6. 试卷总分与等级覆盖校验
  const totalScore = examData.questions.reduce((sum, q) => sum + (Number(q.score) || 0), 0);
  const maxLevelScore = examData.levels.reduce((max, lv) => Math.max(max, Number(lv.max) || 0), 0);

  // 检查满分是否被某个等级覆盖
  const isTotalScoreCovered = examData.levels.some(lv => {
    const min = Number(lv.min) || 0;
    const max = Number(lv.max) || 0;
    return totalScore >= min && totalScore <= max;
  });

  // 情况 A: 试卷满分未被覆盖 (硬性阻止)
  if (!isTotalScoreCovered) {
    activeTab.value = 'levels';
    if (maxLevelScore < totalScore) {
      return alert(`提交阻止：等级覆盖范围不足！\n\n当前试卷满分为 ${totalScore} 分，但等级配置最高只覆盖到 ${maxLevelScore} 分。\n\n请调整等级配置，确保最高等级的上限至少达到 ${totalScore} 分。`);
    } else {
      return alert(`提交阻止：试卷满分落入等级真空区！\n\n当前试卷满分为 ${totalScore} 分，但没有任何一个等级区间包含此分数。\n这通常是因为等级配置不连续。\n\n请调整等级配置，确保满分能够被正确评级。`);
    }
  }

  // 情况 B: 等级超出满分 (软提醒)
  if (maxLevelScore > totalScore) {
    const confirmSubmit = window.confirm(`等级配置范围提示\n\n当前试卷满分为 ${totalScore} 分，但等级配置最高覆盖到了 ${maxLevelScore} 分。\n\n这意味着部分高分段等级将永远无法触发。是否确认继续提交？`);
    if (!confirmSubmit) {
      activeTab.value = 'levels';
      return; // 用户点击取消，中止提交
    }
  }
  
  isSaving.value = true;
  try {
    const payload = JSON.parse(JSON.stringify(examData));
    
    // 注意：Store 中集成了垃圾回收逻辑，会自动清理本次编辑上传但未被使用的文件
    if (mode.value === 'edit') {
      await examStore.updateExam(editId.value, editToken.value, payload);
      alert('更新成功！试卷已重新提交审核。');
      currentStatus.value = 'pending';
    } else {
      const res = await examStore.saveNewExam(payload);
      // 显示成功弹窗
      successModal.id = res.id;
      successModal.token = res.editToken;
      successModal.editLink = `${window.location.origin}/exam/create?id=${res.id}&token=${res.editToken}`;
      successModal.show = true;
    }
    
    // 保存成功后，重置 dirty 状态
    isDirty.value = false;

  } catch (e: any) {
    alert('操作失败: ' + e.message);
  } finally {
    isSaving.value = false;
  }
};

const copyLink = () => {
  navigator.clipboard.writeText(successModal.editLink);
  alert('链接已复制，请妥善保管！');
};

const goExam = () => {
  router.push(`/exam/${successModal.id}`);
};

// 导入 JSON 文件
const handleImportClick = async () => {
  // 先确保已登录且具备管理权限（统一 JWT）
  const ok = await ensureAdmin();
  if (!ok) return;

  // 权限验证通过后，再触发文件选择
  importFileInput.value?.click();
};

const handleFileImport = async (event: Event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  
  if (!file) return;
  
  // 重置文件输入，允许重复选择同一文件
  target.value = '';
  
  // 检查文件类型
  if (!file.name.endsWith('.json')) {
    alert('请选择 JSON 格式的文件！');
    return;
  }

  // 再次确保登录态有效（用户可能刷新过页面）
  const ok = await ensureAdmin();
  if (!ok) return;

  isImporting.value = true;
  importModal.show = true;
  importModal.success = false;
  importModal.message = '正在解析 JSON 文件...';
  importModal.details = '';
  
  try {
    // 读取文件内容
    const text = await file.text();
    const jsonData = JSON.parse(text);
    
    // 验证基本结构
    if (!jsonData.config || !Array.isArray(jsonData.questions) || !Array.isArray(jsonData.levels)) {
      throw new Error('JSON 格式不正确：缺少 config、questions 或 levels 字段');
    }
    
    // 调用导入 API（统一 JWT 自动鉴权）
    const result = await api.importExam(jsonData as ExamDatabase);

    // 导入成功
    importModal.success = true;
    importModal.message = '导入成功！';
    importModal.details = `已导入 ${result.importedQuestions} 道题目和 ${result.importedLevels} 个等级配置。试卷正在审核中...`;
    importModal.id = result.id;
    importModal.token = result.editToken;
    importModal.editLink = `${window.location.origin}/exam/create?id=${result.id}&token=${result.editToken}`;
    
  } catch (error: any) {
    importModal.success = false;
    importModal.message = '导入失败';
    
    // 如果是 401/403 错误，清除登录态
    if (error.status === 401 || error.status === 403) {
      auth.logout();
    }

    if (error.details && Array.isArray(error.details)) {
      importModal.details = `验证错误：\n${error.details.join('\n')}`;
    } else if (error.message) {
      importModal.details = error.message;
    } else {
      importModal.details = '未知错误，请检查 JSON 文件格式是否正确。';
    }
  } finally {
    isImporting.value = false;
  }
};

const copyImportLink = () => {
  navigator.clipboard.writeText(importModal.editLink);
  alert('链接已复制，请妥善保管！');
};

const goToImportedExam = () => {
  router.push(`/exam/${importModal.id}`);
};

const closeImportModal = () => {
  importModal.show = false;
  if (importModal.success) {
    // 导入成功后可以选择跳转到编辑页面或返回首页
    router.push(`/exam/create?id=${importModal.id}&token=${importModal.token}`);
  }
};
</script>

<template>
  <div class="editor-layout">
    <!-- Desktop Sidebar -->
    <div class="sidebar hidden-mobile">
      <div class="brand" @click="router.push('/')">
        <span class="icon">←</span> 返回
      </div>
      <div class="menu">
        <div class="menu-item" :class="{ active: activeTab === 'base' }" @click="activeTab = 'base'">
          📝 基础设置
        </div>
        <div class="menu-item" :class="{ active: activeTab === 'levels' }" @click="activeTab = 'levels'">
          🏆 等级配置
        </div>
        <div class="menu-item" :class="{ active: activeTab === 'questions' }" @click="activeTab = 'questions'">
          ❓ 题目管理
        </div>
      </div>
    </div>

    <div class="main-content">
      <div class="top-header">
        <div class="header-left-group">
          <div class="header-back-mobile" @click="router.push('/')">←</div>
          <div class="header-title">
            <h2>{{ mode === 'edit' ? '编辑试卷' : '创建新试卷' }}</h2>
            <span class="status-badge" :class="currentStatus">{{ statusText }}</span>
            <span v-if="isDirty" class="unsaved-dot" title="有未保存的更改">●</span>
          </div>
        </div>
        
        <div class="header-actions">
          <button 
            v-if="mode === 'create'" 
            class="ui" 
            :disabled="isImporting || isSaving" 
            @click="handleImportClick"
            style="margin-right: 12px;"
          >
            📥 导入 JSON
          </button>
          <button class="ui green" :disabled="isSaving || isImporting" @click="save">
            {{ isSaving ? '提交中...' : (mode === 'edit' ? '更新' : '提交') }}
          </button>
        </div>
      </div>

      <div class="scroll-area">
        <!-- 隐藏的文件输入框 -->
        <input 
          ref="importFileInput"
          type="file" 
          accept=".json" 
          style="display: none"
          @change="handleFileImport"
        />
        
        <!-- 绑定 ref 以便在 submit 时触发内部验证逻辑 -->
        <ConfigPanel 
          v-if="activeTab === 'base'" 
          ref="configPanelRef"
          :config="examData.config" 
        />
        <LevelsPanel v-if="activeTab === 'levels'" :levels="examData.levels" />
        <QuestionsPanel v-if="activeTab === 'questions'" :questions="examData.questions" />
      </div>

      <!-- Mobile Bottom Nav -->
      <div class="mobile-tab-bar">
        <div class="tab-item" :class="{ active: activeTab === 'base' }" @click="activeTab = 'base'">
          <span class="t-icon">📝</span>
          <span class="t-label">基础</span>
        </div>
        <div class="tab-item" :class="{ active: activeTab === 'levels' }" @click="activeTab = 'levels'">
          <span class="t-icon">🏆</span>
          <span class="t-label">等级</span>
        </div>
        <div class="tab-item" :class="{ active: activeTab === 'questions' }" @click="activeTab = 'questions'">
          <span class="t-icon">❓</span>
          <span class="t-label">题目</span>
        </div>
      </div>
    </div>

    <!-- 成功弹窗 -->
    <div v-if="successModal.show" class="modal-overlay">
      <div class="modal">
        <div class="icon">🎉</div>
        <h3>提交成功！正在审核中</h3>
        <p>您的试卷已进入审核队列。请务必保存下方的<strong>私密编辑链接</strong>，它是您日后修改此试卷的唯一凭证。</p>
        
        <div class="link-box">
          <input type="text" readonly :value="successModal.editLink" />
          <button @click="copyLink">复制</button>
        </div>
        
        <div class="actions">
          <button class="ui ghost" @click="successModal.show = false">关闭</button>
          <button class="ui" @click="goExam">查看试卷</button>
        </div>
      </div>
    </div>

    <!-- 管理员登录弹窗（统一 JWT：用户名+密码） -->
    <div v-if="adminModal.show" class="admin-key-modal-overlay" @click.self="cancelAdmin">
      <div class="admin-key-modal" @click.stop>
        <div class="admin-key-icon">🔒</div>
        <h3>需要管理员权限</h3>
        <p>导入功能仅限管理员使用，请使用统一账号登录以继续。</p>

        <div class="admin-key-input-group">
          <input
            v-model="adminModal.user"
            type="text"
            class="admin-key-input"
            placeholder="用户名"
            @keyup.enter="verifyAdmin"
            autofocus
          />
          <input
            v-model="adminModal.pw"
            type="password"
            class="admin-key-input"
            placeholder="密码"
            @keyup.enter="verifyAdmin"
          />
          <div v-if="adminModal.error" class="admin-key-error">
            {{ adminModal.error }}
          </div>
        </div>

        <div class="admin-key-actions">
          <button class="ui ghost" @click="cancelAdmin">取消</button>
          <button class="ui green" @click="verifyAdmin" :disabled="adminModal.loading">{{ adminModal.loading ? '登录中...' : '登录' }}</button>
        </div>
      </div>
    </div>

    <!-- 导入结果弹窗 -->
    <div v-if="importModal.show" class="modal-overlay">
      <div class="modal">
        <div class="icon">{{ importModal.success ? '✅' : '❌' }}</div>
        <h3>{{ importModal.message }}</h3>
        <p style="white-space: pre-line; text-align: left; max-height: 300px; overflow-y: auto;">
          {{ importModal.details }}
        </p>
        
        <div v-if="importModal.success" class="link-box">
          <input type="text" readonly :value="importModal.editLink" />
          <button @click="copyImportLink">复制</button>
        </div>
        
        <div class="actions">
          <button class="ui ghost" @click="closeImportModal">关闭</button>
          <button v-if="importModal.success" class="ui" @click="goToImportedExam">查看试卷</button>
          <button v-if="importModal.success" class="ui green" @click="closeImportModal">编辑试卷</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 保持原有样式完全不变 */
.editor-layout { display: flex; height: 100vh; background: #f3f4f6; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; overflow: hidden; }
.sidebar { width: 220px; background: #fff; border-right: 1px solid #e5e7eb; display: flex; flex-direction: column; flex-shrink: 0; }
.brand { height: 60px; display: flex; align-items: center; padding: 0 20px; font-weight: 900; color: #374151; cursor: pointer; border-bottom: 1px solid #f3f4f6; }
.brand:hover { background: #f9fafb; }
.icon { margin-right: 8px; font-size: 18px; }
.menu { flex: 1; padding: 20px 10px; display: flex; flex-direction: column; gap: 4px; }
.menu-item { padding: 12px 16px; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 600; color: #6b7280; transition: all 0.2s; }
.menu-item:hover { background: #f3f4f6; color: #111; }
.menu-item.active { background: #e0f2fe; color: #0284c7; }
.main-content { flex: 1; display: flex; flex-direction: column; min-width: 0; position: relative; }
.top-header { height: 60px; background: #fff; border-bottom: 1px solid #e5e7eb; display: flex; align-items: center; justify-content: space-between; padding: 0 30px; flex-shrink: 0; }
.header-left-group { display: flex; align-items: center; gap: 12px; }
.header-back-mobile { display: none; font-size: 20px; padding: 5px; cursor: pointer; color: #666; }
.header-title { display: flex; align-items: center; gap: 12px; }
h2 { font-size: 18px; font-weight: 800; color: #1f2937; margin: 0; white-space: nowrap; }
.status-badge { font-size: 11px; padding: 2px 8px; border-radius: 4px; font-weight: 700; white-space: nowrap; }
.status-badge.creating { background: #e5e7eb; color: #4b5563; }
.status-badge.pending { background: #fef9c3; color: #a16207; }
.status-badge.published { background: #dcfce7; color: #15803d; }
.status-badge.locked { background: #fee2e2; color: #b91c1c; }
.unsaved-dot { color: #f59e0b; font-size: 10px; margin-left: 4px; animation: pulse 2s infinite; }
@keyframes pulse { 0% { opacity: 0.5; } 50% { opacity: 1; } 100% { opacity: 0.5; } }
.scroll-area { flex: 1; overflow-y: auto; padding: 30px; -webkit-overflow-scrolling: touch; }
.mobile-tab-bar { display: none; }
button.ui { border: 0; border-radius: 8px; padding: 10px 20px; font-weight: 700; cursor: pointer; font-size: 14px; white-space: nowrap; }
button.ui.green { background: #16a34a; color: #fff; box-shadow: 0 4px 12px rgba(22, 163, 74, 0.2); transition: transform 0.1s; }
button.ui.green:active { transform: translateY(1px); }
button.ui.green:disabled { background: #86efac; cursor: not-allowed; transform: none; box-shadow: none; }
button.ui.ghost { background: transparent; color: #666; border: 1px solid #e5e7eb; }
.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 100; backdrop-filter: blur(4px); padding: 20px; }
.modal { background: #fff; padding: 30px; border-radius: 16px; width: 480px; max-width: 100%; text-align: center; box-shadow: 0 20px 60px rgba(0,0,0,0.2); animation: popIn 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
.modal .icon { font-size: 48px; margin-bottom: 16px; }
.modal h3 { margin: 0 0 10px 0; font-size: 22px; color: #111; }
.modal p { font-size: 14px; color: #666; line-height: 1.6; margin-bottom: 24px; text-align: left; background: #f9fafb; padding: 12px; border-radius: 8px; }
.link-box { display: flex; gap: 8px; margin-bottom: 24px; }
.link-box input { flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 6px; background: #f9fafb; font-size: 13px; font-family: monospace; color: #333; min-width: 0; }
.link-box button { padding: 0 16px; background: #3b82f6; color: #fff; border: 0; border-radius: 6px; cursor: pointer; font-weight: 600; white-space: nowrap; }
.actions { display: flex; gap: 12px; justify-content: center; }
.actions button.ui { width: 120px; }
@keyframes popIn { from { transform: scale(0.9); opacity: 0; } to { transform: scale(1); opacity: 1; } }
@media (max-width: 768px) {
  .hidden-mobile { display: none !important; }
  .editor-layout { flex-direction: column; }
  .top-header { padding: 0 16px; height: 50px; }
  .header-back-mobile { display: block; margin-right: 4px; }
  h2 { font-size: 16px; max-width: 120px; overflow: hidden; text-overflow: ellipsis; }
  .status-badge { font-size: 10px; padding: 1px 4px; }
  button.ui { padding: 6px 12px; font-size: 13px; }
  .scroll-area { padding: 16px; padding-bottom: 80px; }
  .mobile-tab-bar { display: flex; height: 56px; background: #fff; border-top: 1px solid #e5e7eb; position: absolute; bottom: 0; left: 0; right: 0; justify-content: space-around; align-items: center; z-index: 50; padding-bottom: env(safe-area-inset-bottom); }
  .tab-item { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #9ca3af; gap: 2px; }
  .tab-item.active { color: #16a34a; }
  .tab-item .t-icon { font-size: 18px; }
  .tab-item .t-label { font-size: 10px; font-weight: 600; }
  .modal { padding: 20px; }
  .actions button.ui { width: auto; flex: 1; }
}

/* 管理员密钥弹窗样式 */
.admin-key-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1001;
  padding: 20px;
}

.admin-key-modal {
  background: white;
  border-radius: 16px;
  padding: 32px;
  max-width: 400px;
  width: 100%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  text-align: center;
}

.admin-key-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.admin-key-modal h3 {
  font-size: 24px;
  font-weight: 800;
  margin-bottom: 12px;
  color: #000;
}

.admin-key-modal p {
  font-size: 14px;
  color: #6b7280;
  margin-bottom: 24px;
  line-height: 1.6;
}

.admin-key-input-group {
  margin-bottom: 24px;
}

.admin-key-input {
  width: 100%;
  padding: 12px 16px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 16px;
  transition: border-color 0.2s;
  box-sizing: border-box;
}

.admin-key-input + .admin-key-input {
  margin-top: 12px;
}

.admin-key-input:focus {
  outline: none;
  border-color: #16a34a;
}

.admin-key-error {
  margin-top: 8px;
  color: #ef4444;
  font-size: 14px;
  text-align: left;
}

.admin-key-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.admin-key-actions button.ui {
  padding: 12px 24px;
  font-size: 14px;
}

.admin-key-actions button.ui.green {
  background: #16a34a;
  color: white;
}

.admin-key-actions button.ui.green:hover {
  background: #15803d;
}

.admin-key-actions button.ui.green:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>