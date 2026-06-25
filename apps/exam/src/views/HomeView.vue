<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useExamStore } from '@/stores/exam';
import { api } from '@/services/api';
import type { ExamDatabase } from '@/types/exam';
import { useExamAdmin } from '@/composables/useExamAdmin';

// 统一鉴权：导入试卷需要管理员权限，走共享 createAdminAuth('exam')。
const { admin, adminModal, ensureAdmin, verifyAdmin, cancelAdmin } = useExamAdmin();

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
      admin.logout();
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

      <!-- 搜索框：统一搜索规范 .sos-search -->
      <div class="search-container">
        <div class="sos-search sos-search--lg exam-search">
          <span class="sos-search__icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="7" /><path d="M21 21l-4.3-4.3" />
            </svg>
          </span>
          <input
            ref="searchInput"
            v-model="searchKeyword"
            type="search"
            class="sos-search__input"
            placeholder="搜索试卷名称或描述"
            aria-label="搜索试卷"
            @keydown="handleSearchKeydown"
            :disabled="isSearching || loading"
          />
          <button
            class="sos-search__submit"
            aria-label="搜索"
            :disabled="isSearching || loading"
            @click="performSearch"
          >
            <svg v-if="!isSearching" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="7" /><path d="M21 21l-4.3-4.3" />
            </svg>
            <span v-else>…</span>
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
  background-color: var(--sos-bg-page); 
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
    background-image: var(--bg-url), radial-gradient(circle at center, rgba(255,255,255,0) 0%, var(--sos-bg-page) 100%);
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
  color: var(--sos-text-primary); /* 纯黑，高对比度 */
  letter-spacing: -1.5px; /* 收紧字间距，更现代 */
}
.header p {
  font-size: 18px; /* 加大副标题 */
  font-weight: 600; /* 加粗 */
  color: var(--sos-text-secondary); /* 深灰，高对比度 */
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
  background: var(--sos-bg-surface);
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
  color: var(--sos-text-secondary);
  background: transparent;
}

.search-input::placeholder {
  color: var(--sos-text-disabled);
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
  background: var(--sos-accent);
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
  background: var(--sos-accent-hover);
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
  background: var(--sos-bg-muted);
  color: var(--sos-text-tertiary);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.search-clear-btn:hover:not(:disabled) {
  background: var(--sos-border-default);
  color: var(--sos-text-secondary);
}

.search-clear-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.search-info {
  margin-top: 12px;
  text-align: center;
  color: var(--sos-text-tertiary);
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
  background: var(--sos-accent);
  color: var(--sos-accent-contrast);
  box-shadow: var(--sos-shadow-card);
}
button.ui.green:hover:not(:disabled) {
  background: var(--sos-accent-hover);
}
/* 默认 .ui 按钮：米白承载面次按钮（藏蓝描边） */
button.ui {
  background: var(--sos-bg-surface);
  color: var(--sos-accent-2);
  box-shadow: var(--sos-shadow-xs);
  border: 1px solid var(--sos-border-default);
}
button.ui:hover:not(:disabled) {
  border-color: var(--sos-accent-2);
}
button.ui:disabled {
  opacity: 0.55;
  cursor: not-allowed;
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
  background: color-mix(in srgb, var(--sos-bg-surface) 92%, transparent); /* 毛玻璃米白，增强文字对比 */
  backdrop-filter: blur(10px); 
  border-radius: 20px; /* 圆角加大 */
  padding: 28px; /* 内边距加大 */
  box-shadow: 0 4px 20px rgba(0,0,0,0.08);
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s, border-color 0.2s;
  position: relative;
  overflow: hidden;
  border: 2px solid var(--sos-border-subtle);
}
.card:hover {
  transform: translateY(-4px);
  box-shadow: var(--sos-hover-shadow);
  background: var(--sos-bg-surface);
}

/* 高亮样式 */
.card.highlight {
  border-color: var(--sos-accent);
  background: var(--sos-accent-soft);
  box-shadow: 0 8px 30px color-mix(in srgb, var(--sos-accent) 22%, transparent);
  transform: translateY(-4px);
}
.highlight-badge {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  background: var(--sos-accent);
  color: var(--sos-accent-contrast);
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

.card .tag.official {
  background: color-mix(in srgb, var(--sos-accent-2) 14%, var(--sos-bg-surface));
  color: var(--sos-accent-2); /* 学生藏蓝 */
}
.card .tag.user { background: var(--sos-bg-muted); color: var(--sos-text-secondary); }
.card.special {
  background: linear-gradient(135deg, var(--sos-accent-soft) 0%, var(--sos-bg-surface) 100%);
  border: 2px solid var(--sos-signal); /* 金色信号描边 */
}
/* 官方卡片高亮时覆盖默认边框 */
.card.special.highlight {
  border: 2px solid var(--sos-accent); 
}

.card.special .tag {
  background: var(--sos-signal);
  color: var(--sos-text-primary);
}

.card h3 { 
  margin: 0 80px 10px 0; 
  font-size: 22px; /* 加大卡片标题 */
  font-weight: 800; /* 加粗 */
  color: var(--sos-text-primary); /* 纯黑 */
  line-height: 1.3;
  word-wrap: break-word;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card p { 
  margin: 0 0 20px 0; 
  font-size: 16px; /* 加大正文 */
  font-weight: 500; /* 中等字重 */
  color: var(--sos-text-secondary); /* 深灰 */
  line-height: 1.6; 
}
.card .meta { 
  font-size: 13px; 
  color: var(--sos-text-tertiary); /* 加深颜色 */
  font-weight: 700; 
}
.card.loading { display: flex; align-items: center; justify-content: center; color: var(--sos-text-tertiary); font-weight: 600; min-height: 160px; }

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
  background: color-mix(in srgb, var(--sos-bg-surface) 92%, transparent);
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
  color: var(--sos-text-secondary);
  margin-bottom: 12px;
}

.no-results p {
  font-size: 16px;
  color: var(--sos-text-tertiary);
  line-height: 1.6;
}

.no-results a {
  color: var(--sos-accent);
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
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.pagination-btn:hover:not(:disabled) {
  background: var(--sos-bg-muted);
  border-color: var(--sos-border-strong);
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
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.pagination-page:hover {
  background: var(--sos-bg-muted);
  border-color: var(--sos-border-strong);
}

.pagination-page.active {
  background: var(--sos-accent);
  color: white;
  border-color: var(--sos-accent);
}

.pagination-page.active:hover {
  background: var(--sos-accent-hover);
  border-color: var(--sos-accent-hover);
}

.pagination-info {
  text-align: center;
  color: var(--sos-text-tertiary);
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
  background: var(--sos-bg-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1001;
  padding: 20px;
}

.admin-key-modal {
  background: var(--sos-bg-surface);
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
  color: var(--sos-text-primary);
}

.admin-key-modal p {
  font-size: 14px;
  color: var(--sos-text-tertiary);
  margin-bottom: 24px;
  line-height: 1.6;
}

.admin-key-input-group {
  margin-bottom: 24px;
}

.admin-key-input {
  width: 100%;
  padding: 12px 16px;
  border: 2px solid var(--sos-border-default);
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
  border-color: var(--sos-accent);
}

.admin-key-actions button.ui.green:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.admin-key-error {
  margin-top: 8px;
  color: var(--sos-danger);
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
  background: var(--sos-bg-muted);
  color: var(--sos-text-secondary);
}

.admin-key-actions button.ui.ghost:hover {
  background: var(--sos-border-default);
}

.admin-key-actions button.ui.green {
  background: var(--sos-accent);
  color: white;
}

.admin-key-actions button.ui.green:hover {
  background: var(--sos-accent-hover);
}

/* 导入弹窗样式 */
.import-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--sos-bg-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.import-modal {
  background: var(--sos-bg-surface);
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
  color: var(--sos-text-primary);
}

.import-link-box {
  margin: 20px 0;
  display: flex;
  gap: 8px;
}

.import-link-box input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  font-size: 14px;
}

.import-link-box button {
  padding: 10px 20px;
  background: var(--sos-bg-muted);
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s;
}

.import-link-box button:hover {
  background: var(--sos-border-default);
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
  background: var(--sos-bg-muted);
  color: var(--sos-text-secondary);
}

.import-actions button.ui.ghost:hover {
  background: var(--sos-border-default);
}

.import-actions button.ui {
  background: var(--sos-accent-2);
  color: white;
}

.import-actions button.ui:hover {
  background: var(--sos-link-hover);
}

.import-actions button.ui.green {
  background: var(--sos-accent);
}

.import-actions button.ui.green:hover {
  background: var(--sos-accent-hover);
}
</style>