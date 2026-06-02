<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useExamStore } from '@/stores/exam';
import { api } from '@/services/api';
import type { ExamDatabase } from '@/types/exam';
import { createAuth } from '@haruhi/api-client';

// 统一鉴权：导入试卷需要管理员权限，改为统一 JWT 单点登录（用户名+密码）。
const auth = createAuth('/api');
const hasExamPerm = (user: any): boolean => !!user && (user.isSuperAdmin || (user.apps && user.apps.exam));

const router = useRouter();
const route = useRoute();
const examStore = useExamStore();
const exams = ref<any[]>([]);
const loading = ref(true);
const highlightId = ref('');

// 分页相关状态
const currentPage = ref(1);
const totalPages = ref(1);
const total = ref(0);
const pageSize = 9;

// 搜索相关状态
const searchKeyword = ref('');
const searchInput = ref<HTMLInputElement | null>(null);
const isSearching = ref(false);

// 导入相关状态
const importFileInput = ref<HTMLInputElement | null>(null);
const isImporting = ref(false);
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

// --- 背景图逻辑 Start ---
const bgImage = ref('');

//在此处定义两套背景图列表
const desktopImages = [
  'desktop-1.webp',
  'desktop-2.webp',
  'desktop-3.webp',
  'desktop-4.webp',
];

const mobileImages = [
  'mobile-1.webp',
  'mobile-2.webp',
  'mobile-3.webp',
];

const initBackground = () => {
  const isMobile = window.innerWidth <= 768;
  const targetList = isMobile ? mobileImages : desktopImages;

  if (targetList.length === 0) return;

  // 随机选择列表中的一张
  const randomIdx = Math.floor(Math.random() * targetList.length);
  const fileName = targetList[randomIdx];
  
  // Vite 动态引入静态资源的方式
  try {
    bgImage.value = new URL(`../assets/images/${fileName}`, import.meta.url).href;
  } catch (e) {
    console.warn('Background image not found:', fileName);
  }
};
// --- 背景图逻辑 End ---

// 加载试卷列表
const loadExams = async (page: number = 1, search: string = '') => {
  loading.value = true;
  try {
    const result = await examStore.listExams(page, pageSize, search);
    
    exams.value = result.data || [];
    currentPage.value = result.pagination.page;
    totalPages.value = result.pagination.totalPages;
    total.value = result.pagination.total;
    
    // 如果有高亮ID（且不是官方haruhi），且高亮项在当前页，将其移动到列表第一位
    // 搜索模式下不显示官方试卷，也不处理高亮
    if (!search && highlightId.value && highlightId.value !== 'haruhi') {
      const idx = exams.value.findIndex((e: any) => e.id === highlightId.value);
      if (idx > -1) {
        const [target] = exams.value.splice(idx, 1);
        exams.value.unshift(target); // 插入到头部
        
        // 自动滚动到高亮位置
        setTimeout(() => {
          const el = document.querySelector('.card.highlight');
          if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }, 300);
      }
    }
    
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
};

onMounted(async () => {
  initBackground(); // 初始化背景

  // 获取 URL 中的 highlight、page 和 search 参数
  highlightId.value = route.query.highlight as string || '';
  const pageFromQuery = parseInt(route.query.page as string) || 1;
  currentPage.value = pageFromQuery;
  searchKeyword.value = (route.query.search as string) || '';

  // 加载列表
  await loadExams(currentPage.value, searchKeyword.value);
});

// 切换页码
const changePage = async (page: number) => {
  if (page < 1 || page > totalPages.value || page === currentPage.value) return;
  
  currentPage.value = page;
  
  // 更新 URL 参数（保留 highlight 和 search）
  const query: any = {};
  if (highlightId.value) query.highlight = highlightId.value;
  if (searchKeyword.value) query.search = searchKeyword.value;
  if (page > 1) query.page = page;
  router.push({ query });
  
  // 滚动到顶部
  window.scrollTo({ top: 0, behavior: 'smooth' });
  
  await loadExams(page, searchKeyword.value);
};

// 执行搜索
const performSearch = async () => {
  isSearching.value = true;
  currentPage.value = 1; // 搜索时重置到第一页
  
  // 更新 URL 参数
  const query: any = {};
  if (highlightId.value) query.highlight = highlightId.value;
  if (searchKeyword.value.trim()) {
    query.search = searchKeyword.value.trim();
  }
  router.push({ query });
  
  // 滚动到顶部
  window.scrollTo({ top: 0, behavior: 'smooth' });
  
  await loadExams(1, searchKeyword.value.trim());
  isSearching.value = false;
};

// 清除搜索
const clearSearch = async () => {
  searchKeyword.value = '';
  currentPage.value = 1;
  
  // 更新 URL 参数（移除 search）
  const query: any = {};
  if (highlightId.value) query.highlight = highlightId.value;
  router.push({ query });
  
  // 滚动到顶部
  window.scrollTo({ top: 0, behavior: 'smooth' });
  
  await loadExams(1, '');
};

// 搜索框回车事件
const handleSearchKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    performSearch();
  }
};

// 防抖函数
const debounce = (fn: Function, delay: number) => {
  let timeoutId: any;
  return (...args: any[]) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
};

// 监听搜索关键词变化，实现即时搜索
watch(searchKeyword, debounce((newVal: string) => {
  performSearch();
}, 500));

const goHaruhi = () => router.push('/haruhi');
const goExam = (id: string) => router.push(`/exam/${id}`);
const goCreate = () => router.push('/create');

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
  
  // 重置文件输入
  target.value = '';
  
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
    const text = await file.text();
    const jsonData = JSON.parse(text);
    
    if (!jsonData.config || !Array.isArray(jsonData.questions) || !Array.isArray(jsonData.levels)) {
      throw new Error('JSON 格式不正确：缺少 config、questions 或 levels 字段');
    }
    
    const result = await api.importExam(jsonData as ExamDatabase);

    importModal.success = true;
    importModal.message = '导入成功！';
    importModal.details = `已导入 ${result.importedQuestions} 道题目和 ${result.importedLevels} 个等级配置。试卷正在审核中...`;
    importModal.id = result.id;
    importModal.token = result.editToken;
    importModal.editLink = `${window.location.origin}/exam/create?id=${result.id}&token=${result.editToken}`;
    
    // 刷新试卷列表（重新加载当前页）
    await loadExams(currentPage.value, searchKeyword.value);
    
  } catch (error: any) {
    importModal.success = false;
    importModal.message = '导入失败';
    
    // 如果是权限错误，清除登录态
    if (error.status === 401 || error.status === 403) {
      auth.logout();
      importModal.details = '登录已过期或无权限，请重新登录后再试。';
    } else if (error.details && Array.isArray(error.details)) {
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

const goToEditImportedExam = () => {
  router.push(`/exam/create?id=${importModal.id}&token=${importModal.token}`);
};

const closeImportModal = () => {
  importModal.show = false;
};
</script>

<template>
  <div class="home-wrapper">
    <!-- 背景层：独立于内容，应用高斯模糊 -->
    <div class="bg-layer" :style="{ backgroundImage: `url(${bgImage})` }"></div>
    
    <!-- 内容层：相对定位，确保在背景之上 -->
    <div class="home-container">
      <div class="header">
        <h1>试卷中心</h1>
        <p>选择一张喜欢的试卷开始挑战～<br>或是创建一份属于自己的试卷！</p>
      </div>

      <div class="actions">
        <input 
          ref="importFileInput"
          type="file" 
          accept=".json" 
          style="display: none"
          @change="handleFileImport"
        />
        <button 
          class="ui" 
          :disabled="isImporting" 
          @click="handleImportClick"
          style="margin-right: 12px;margin-bottom: 12px;height: 68px;"
        >
          📥 {{ isImporting ? '导入中...' : '导入 JSON' }}
        </button>
        <button class="ui green" @click="goCreate" :disabled="isImporting">
          <span style="font-size:1.4em; margin-right:6px; font-weight: 900;">+</span> 创建新试卷
        </button>
      </div>

      <!-- 搜索框 -->
      <div class="search-container">
        <div class="search-box">
          <input
            ref="searchInput"
            v-model="searchKeyword"
            type="text"
            class="search-input"
            placeholder="搜索试卷名称或描述"
            @keydown="handleSearchKeydown"
            :disabled="isSearching || loading"
          />
          <button
            class="search-btn"
            @click="performSearch"
            :disabled="isSearching || loading"
          >
            <span v-if="!isSearching">🔍</span>
            <span v-else>...</span>
          </button>
        </div>
        <div v-if="searchKeyword" class="search-info">
          搜索"{{ searchKeyword }}"，找到 {{ total }} 套试卷
        </div>
      </div>

      <div class="grid">
        <!-- 官方置顶（仅在第一页且无搜索时显示） -->
        <div 
          v-if="currentPage === 1 && !searchKeyword"
          class="card special" 
          :class="{ highlight: highlightId === 'haruhi' }"
          @click="goHaruhi"
        >
          <div v-if="highlightId === 'haruhi'" class="highlight-badge">👇 扫码直达</div>
          <div class="tag official">官方</div>
          <h3>SOS团入团测试</h3>
          <p>测试你是否具有成为团员的资格！</p>
          <div class="meta">满分 100 · 4 栏布局</div>
        </div>
     
        <!-- 用户上传 -->
        <div v-if="loading" class="card loading">
          加载中...
        </div>
        
        <div v-for="ex in exams" :key="ex.id" 
             class="card" 
             :class="{ highlight: highlightId === ex.id }"
             @click="goExam(ex.id)">
          <div v-if="highlightId === ex.id" class="highlight-badge">👇 扫码直达</div>
          
          <!-- [修改] 优先显示 Config 中的 author，否则显示默认"用户" -->
          <div class="tag user">{{ ex.config?.author || '用户' }}</div>
          
          <h3>{{ ex.config?.title || '未命名试卷' }}</h3>
          <p>{{ ex.config?.paperSubtitle || '暂无描述' }}</p>
          <div class="meta">包含 {{ ex.questions?.length || 0 }} 道题目</div>
        </div>
      </div>

      <!-- 分页组件 -->
      <div v-if="totalPages > 1" class="pagination">
        <button 
          class="pagination-btn" 
          :disabled="currentPage === 1" 
          @click="changePage(currentPage - 1)"
        >
          ← 上一页
        </button>
        
        <div class="pagination-pages">
          <button
            v-for="page in totalPages"
            :key="page"
            class="pagination-page"
            :class="{ active: currentPage === page }"
            @click="changePage(page)"
          >
            {{ page }}
          </button>
        </div>
        
        <button 
          class="pagination-btn" 
          :disabled="currentPage === totalPages" 
          @click="changePage(currentPage + 1)"
        >
          下一页 →
        </button>
      </div>

      <!-- 分页信息 -->
      <div v-if="total > 0" class="pagination-info">
        <span v-if="searchKeyword">搜索到</span>
        <span v-else>共</span>
        {{ total }} 套试卷，第 {{ currentPage }} / {{ totalPages }} 页
      </div>
      
      <!-- 无搜索结果提示 -->
      <div v-if="!loading && searchKeyword && total === 0" class="no-results">
        <div class="no-results-icon">搜索</div>
        <h3>未找到相关试卷</h3>
        <p>请尝试使用其他关键词搜索，或<a href="javascript:void(0)" @click="clearSearch">清除搜索</a>查看所有试卷。</p>
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
    <div v-if="importModal.show" class="import-modal-overlay" @click.self="closeImportModal">
      <div class="import-modal" @click.stop>
        <div class="import-icon">{{ importModal.success ? '✅' : '❌' }}</div>
        <h3>{{ importModal.message }}</h3>
        <p style="white-space: pre-line; text-align: left; max-height: 300px; overflow-y: auto; margin: 20px 0;">
          {{ importModal.details }}
        </p>
        
        <div v-if="importModal.success" class="import-link-box">
          <input type="text" readonly :value="importModal.editLink" />
          <button @click="copyImportLink">复制</button>
        </div>
        
        <div class="import-actions">
          <button class="ui ghost" @click="closeImportModal">关闭</button>
          <button v-if="importModal.success" class="ui" @click="goToImportedExam">查看试卷</button>
          <button v-if="importModal.success" class="ui green" @click="goToEditImportedExam">编辑试卷</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 新增：背景层样式 
  position: fixed 确保背景不随页面滚动（视差效果）
*/
.bg-layer {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0; /* 最底层 */
  
  background-position: center top;
  background-repeat: no-repeat;
  
  /* 高斯模糊 8px */
  filter: blur(8px);
  /* 放大防止白边 */
  transform: scale(1.1);
  pointer-events: none; /* 确保不影响点击 */
  
  /* 默认底色 */
  background-color: #f5f5f7; 
}

/* 新增：泛白滤镜遮罩
  使用伪元素在图片上方盖一层半透明白色
*/
.bg-layer::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  /* 调整最后一个数值(0.4)来控制泛白程度，越大越白 */
  background: rgba(255, 255, 255, 0.4); 
  z-index: 1;
}

/* 桌面端：高度铺满，宽度自适应，两侧留渐变过渡 */
@media (min-width: 769px) {
  .bg-layer {
    background-size: auto 100%; /* 高度100%，宽度保持比例 */
    /* 可以在图片下层垫一个径向渐变，让两侧空白处更自然 */
    background-image: var(--bg-url), radial-gradient(circle at center, rgba(255,255,255,0) 0%, #f5f5f7 100%);
  }
}

/* 移动端：全屏铺满 (cover) */
@media (max-width: 768px) {
  .bg-layer {
    background-size: cover;
    background-position: center;
  }
}

.home-wrapper {
  position: relative;
  min-height: 100vh;
}

.home-container {
  position: relative;
  z-index: 1; /* 内容层级高于背景 */
  padding: 60px 20px; /* 增加顶部 padding */
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  max-width: 100%;
}

.header {
  text-align: center;
  margin-bottom: 50px;
}
.header h1 {
  font-size: 48px; /* 加大标题 */
  font-weight: 900; /* 最粗字重 */
  margin-bottom: 16px;
  color: #000000; /* 纯黑，高对比度 */
  letter-spacing: -1.5px; /* 收紧字间距，更现代 */
}
.header p {
  font-size: 18px; /* 加大副标题 */
  font-weight: 600; /* 加粗 */
  color: #333333; /* 深灰，高对比度 */
  line-height: 1.6;
}
.actions {
  text-align: center;
  margin-bottom: 30px;
}

/* 搜索容器样式 */
.search-container {
  max-width: 600px;
  margin: 0 auto 40px;
  width: 100%;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background: white;
  border-radius: 12px;
  padding: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: box-shadow 0.2s;
}

.search-box:focus-within {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  padding: 12px 16px;
  font-size: 16px;
  font-weight: 500;
  color: #374151;
  background: transparent;
}

.search-input::placeholder {
  color: #9ca3af;
}

.search-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.search-btn {
  width: 44px;
  height: 44px;
  border: none;
  border-radius: 8px;
  background: #16a34a;
  color: white;
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.search-btn:hover:not(:disabled) {
  background: #15803d;
  transform: scale(1.05);
}

.search-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.search-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.search-clear-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  background: #f3f4f6;
  color: #6b7280;
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.search-clear-btn:hover:not(:disabled) {
  background: #e5e7eb;
  color: #374151;
}

.search-clear-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.search-info {
  margin-top: 12px;
  text-align: center;
  color: #6b7280;
  font-size: 14px;
  font-weight: 500;
}
button.ui {
  border: 0;
  border-radius: 99px;
  padding: 16px 32px; /* 加大按钮 */
  font-weight: 800; /* 加粗按钮文字 */
  font-size: 18px; /* 加大按钮文字 */
  cursor: pointer;
  transition: transform 0.1s;
}
button.ui.green {
  background: #16a34a;
  color: #fff;
  box-shadow: 0 6px 16px rgba(22, 163, 74, 0.4);
}
button.ui:active { transform: scale(0.96); }

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); /* 卡片最小宽度稍作增加 */
  gap: 24px;
  max-width: 1200px;
  margin: 0 auto;
}
.card {
  /* 卡片背景保持不透明，确保内容清晰 */
  background: rgba(255, 255, 255, 0.95); /* 更不透明一些，增强文字对比 */
  backdrop-filter: blur(10px); 
  border-radius: 20px; /* 圆角加大 */
  padding: 28px; /* 内边距加大 */
  box-shadow: 0 4px 20px rgba(0,0,0,0.08);
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s, border-color 0.2s;
  position: relative;
  overflow: hidden;
  border: 2px solid rgba(255,255,255,0.8);
}
.card:hover {
  transform: translateY(-4px);
  box-shadow: 0 16px 40px rgba(0,0,0,0.15);
  background: #fff;
}

/* 高亮样式 */
.card.highlight {
  border-color: #16a34a;
  background: #f0fdf4;
  box-shadow: 0 8px 30px rgba(22, 163, 74, 0.25);
  transform: translateY(-4px);
}
.highlight-badge {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  background: #16a34a;
  color: #fff;
  font-size: 13px;
  font-weight: 800;
  text-align: center;
  padding: 6px;
  z-index: 2;
}

.card .tag {
  position: absolute;
  top: 20px;
  right: 20px;
  font-size: 11px;
  font-weight: 900;
  padding: 6px 10px;
  border-radius: 8px;
  text-transform: uppercase;
}
/* 调整有 badge 时的 tag 位置 */
.card.highlight .tag {
  top: 36px; 
}

.card .tag.official { background: #e0f2fe; color: #0369a1; /* 更加深的蓝色 */ }
.card .tag.user { background: #f3f4f6; color: #4b5563; /* 更深的灰色 */ }
.card.special {
  background: linear-gradient(135deg, rgba(255, 251, 235, 0.98) 0%, rgba(255, 255, 255, 0.98) 100%);
  border: 2px solid #fbbf24; /* 加粗边框 */
}
/* 官方卡片高亮时覆盖默认边框 */
.card.special.highlight {
  border: 2px solid #16a34a; 
}

.card.special .tag {
  background: #fcd34d;
  color: #78350f;
}

.card h3 { 
  margin: 0 80px 10px 0; 
  font-size: 22px; /* 加大卡片标题 */
  font-weight: 800; /* 加粗 */
  color: #000; /* 纯黑 */
  line-height: 1.3;
  word-wrap: break-word;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card p { 
  margin: 0 0 20px 0; 
  font-size: 16px; /* 加大正文 */
  font-weight: 500; /* 中等字重 */
  color: #374151; /* 深灰 */
  line-height: 1.6; 
}
.card .meta { 
  font-size: 13px; 
  color: #6b7280; /* 加深颜色 */
  font-weight: 700; 
}
.card.loading { display: flex; align-items: center; justify-content: center; color: #999; font-weight: 600; min-height: 160px; }

/* 移动端适配调整 */
@media (max-width: 768px) {
  .header h1 {
    font-size: 36px; /* 移动端稍小一点但依然很大 */
  }
  .header p {
    font-size: 16px;
  }
  .card h3 {
    font-size: 20px;
  }
  
  .actions {
    margin-bottom: 20px;
    flex-direction: column;
    gap: 12px;
  }
  
  .actions button {
    width: 100%;
    max-width: 300px;
  }
  
  .search-container {
    margin-bottom: 30px;
    padding: 0 10px;
  }
  
  .search-box {
    border-radius: 10px;
    padding: 3px;
  }
  
  .search-input {
    padding: 10px 12px;
    font-size: 15px;
  }
  
  .search-btn {
    width: 40px;
    height: 40px;
    font-size: 18px;
  }
  
  .search-clear-btn {
    width: 32px;
    height: 32px;
    font-size: 16px;
  }
  
  .search-info {
    font-size: 13px;
    margin-top: 10px;
  }
}

/* 无搜索结果样式 */
.no-results {
  text-align: center;
  padding: 60px 20px;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 16px;
  margin: 40px auto;
  max-width: 500px;
}

.no-results-icon {
  font-size: 64px;
  margin-bottom: 20px;
}

.no-results h3 {
  font-size: 24px;
  font-weight: 800;
  color: #374151;
  margin-bottom: 12px;
}

.no-results p {
  font-size: 16px;
  color: #6b7280;
  line-height: 1.6;
}

.no-results a {
  color: #16a34a;
  text-decoration: none;
  font-weight: 600;
}

.no-results a:hover {
  text-decoration: underline;
}

/* 分页组件样式 */
.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  margin: 40px 0 20px;
  flex-wrap: wrap;
}

.pagination-btn {
  padding: 10px 20px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: white;
  color: #374151;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.pagination-btn:hover:not(:disabled) {
  background: #f3f4f6;
  border-color: #d1d5db;
}

.pagination-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pagination-pages {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.pagination-page {
  min-width: 40px;
  height: 40px;
  padding: 0 12px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: white;
  color: #374151;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.pagination-page:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
}

.pagination-page.active {
  background: #16a34a;
  color: white;
  border-color: #16a34a;
}

.pagination-page.active:hover {
  background: #15803d;
  border-color: #15803d;
}

.pagination-info {
  text-align: center;
  color: #6b7280;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 40px;
}

/* 移动端分页适配 */
@media (max-width: 768px) {
  .pagination {
    gap: 8px;
  }
  
  .pagination-btn {
    padding: 8px 16px;
    font-size: 14px;
  }
  
  .pagination-page {
    min-width: 36px;
    height: 36px;
    padding: 0 10px;
    font-size: 14px;
  }
  
  .pagination-info {
    font-size: 13px;
  }
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

.admin-key-actions button.ui.green:disabled {
  opacity: 0.6;
  cursor: not-allowed;
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

.admin-key-actions button {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.admin-key-actions button.ui.ghost {
  background: #f3f4f6;
  color: #374151;
}

.admin-key-actions button.ui.ghost:hover {
  background: #e5e7eb;
}

.admin-key-actions button.ui.green {
  background: #16a34a;
  color: white;
}

.admin-key-actions button.ui.green:hover {
  background: #15803d;
}

/* 导入弹窗样式 */
.import-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.import-modal {
  background: white;
  border-radius: 16px;
  padding: 32px;
  max-width: 500px;
  width: 100%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  text-align: center;
}

.import-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.import-modal h3 {
  font-size: 24px;
  font-weight: 800;
  margin-bottom: 16px;
  color: #000;
}

.import-link-box {
  margin: 20px 0;
  display: flex;
  gap: 8px;
}

.import-link-box input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  font-size: 14px;
}

.import-link-box button {
  padding: 10px 20px;
  background: #f3f4f6;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s;
}

.import-link-box button:hover {
  background: #e5e7eb;
}

.import-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-top: 24px;
}

.import-actions button {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.import-actions button.ui.ghost {
  background: #f3f4f6;
  color: #374151;
}

.import-actions button.ui.ghost:hover {
  background: #e5e7eb;
}

.import-actions button.ui {
  background: #3b82f6;
  color: white;
}

.import-actions button.ui:hover {
  background: #2563eb;
}

.import-actions button.ui.green {
  background: #16a34a;
}

.import-actions button.ui.green:hover {
  background: #15803d;
}
</style>