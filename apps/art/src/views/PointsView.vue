<template>
  <div class="container-card">
    
    <!-- 头部区域 -->
    <div class="header-section">
      <h1>积分快捷查询</h1>
      <p class="subtitle">本页面查询和展示的积分情况皆是在此画廊获得的绘画部积分，而非应援团积分。<br>10 绘画部积分=1 应援团积分 </p>
    </div>

    <!-- 1. 查询区域 -->
    <div class="search-wrapper-outer" ref="searchRef">
      <div class="search-section panel">
        <div class="search-bar">
          <div class="input-wrapper">
            <input 
              type="text" 
              class="input search-input" 
              placeholder="输入创作者唯一ID搜索..." 
              v-model="query"
              @input="onInput"
              @focus="showSuggestions = true"
            />
            <button class="btn btn--accent" @click="doSearch(query)" :disabled="!query.trim()">
              查询
            </button>
            
            <!-- 模糊匹配下拉框 -->
            <div v-if="showSuggestions && suggestions.length > 0" class="suggestions-dropdown">
              <div 
                v-for="s in suggestions" 
                :key="s.uid" 
                class="suggestion-item"
                @click="selectSuggestion(s)"
              >
                <img 
                  :src="s.avatar_url || defaultAvatar" 
                  @error="e => e.target.src = defaultAvatar"
                  class="s-avatar" 
                  alt=""
                />
                <span class="s-uid">{{ s.uid }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 查询结果展示 -->
        <div v-if="searchResult" class="result-box">
          <div class="user-card">
            <div class="u-info">
              <img 
                :src="searchResult.creator?.avatar_url || defaultAvatar" 
                @error="e => e.target.src = defaultAvatar"
                class="u-avatar" 
                alt=""
              />
              <div class="u-text">
                <div class="u-name">{{ searchResult.creator?.uid || query }}</div>
                <div class="u-badge">创作者</div>
              </div>
            </div>
            <div class="u-score">
              <div class="label">当前积分</div>
              <div class="val">{{ searchResult.total }}</div>
            </div>
          </div>

          <div class="history-list">
            <h3>变更记录 (最近50条)</h3>
            <div v-if="searchResult.history.length === 0" class="empty-hint">暂无记录</div>
            <ul v-else>
              <li v-for="(h, i) in searchResult.history" :key="i" class="h-item">
                <div class="h-left">
                  <div class="h-note">{{ h.note }}</div>
                  <div class="h-sub" v-if="h.artwork_title">作品: {{ h.artwork_title }}</div>
                  <div class="h-date">{{ new Date(h.created_at).toLocaleString() }}</div>
                </div>
                <div class="h-right" :class="h.points > 0 ? 'plus' : 'minus'">
                  {{ h.points > 0 ? '+' : '' }}{{ h.points }}
                </div>
              </li>
            </ul>
          </div>
        </div>
        
        <div v-if="notFound" class="not-found">未找到该用户的积分记录</div>
      </div>
    </div>

    <!-- 2. 排行榜区域 -->
    <div class="leaderboard-section">
      <h2 class="lb-title">
        <span class="icon">🏆</span> 画廊积分榜
      </h2>
      
      <div class="grid-table">
        <div class="row header">
          <div class="cell rank">排名</div>
          <div class="cell user">创作者</div>
          <div class="cell score">总积分</div>
        </div>

        <div v-if="loadingLb" class="loading">加载排行榜中...</div>
        
        <div 
          v-else 
          v-for="(item, idx) in leaderboard" 
          :key="item.uid" 
          class="row item"
          :class="{'top-3': idx < 3}"
        >
          <div class="cell rank">
            <span v-if="idx === 0" class="medal gold">1</span>
            <span v-else-if="idx === 1" class="medal silver">2</span>
            <span v-else-if="idx === 2" class="medal bronze">3</span>
            <span v-else class="medal-plain">{{ idx + 1 }}</span>
          </div>
          <div class="cell user" @click="fillSearch(item.uid)">
            <img 
              :src="item.avatar_url || defaultAvatar" 
              @error="e => e.target.src = defaultAvatar"
              class="lb-avatar" 
              alt=""
            />
            <span class="lb-uid">{{ item.uid }}</span>
          </div>
          <div class="cell score">{{ item.total }}</div>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { api } from '../services/api.js'

// 默认头像 (SVG Base64)
const defaultAvatar = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='%23cbd5e1'%3E%3Cpath d='M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z'/%3E%3C/svg%3E"

// --- 搜索逻辑 ---
const query = ref('')
const showSuggestions = ref(false)
const suggestions = ref([])
const searchResult = ref(null)
const notFound = ref(false)
const searchRef = ref(null) // 滚动锚点
let debounceTimer = null

function onInput() {
  notFound.value = false
  searchResult.value = null
  showSuggestions.value = true
  
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(async () => {
    if (!query.value.trim()) {
      suggestions.value = []
      return
    }
    try {
      const res = await api.searchCreators(query.value)
      suggestions.value = res.data || []
    } catch (e) {
      console.error(e)
    }
  }, 300)
}

function selectSuggestion(s) {
  query.value = s.uid
  showSuggestions.value = false
  doSearch(s.uid)
}

async function doSearch(uid) {
  if (!uid) return
  showSuggestions.value = false
  searchResult.value = null
  notFound.value = false
  
  try {
    const res = await api.pointsHistory(uid)
    if (res.total === 0 && res.history.length === 0) {
      notFound.value = true
    } else {
      searchResult.value = res
    }
  } catch (e) {
    notFound.value = true
  }
}

function fillSearch(uid) {
  query.value = uid
  doSearch(uid)
  // 平滑滚动到搜索框
  if (searchRef.value) {
    searchRef.value.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}

// --- 排行榜逻辑 ---
const leaderboard = ref([])
const loadingLb = ref(false)

async function loadLeaderboard() {
  loadingLb.value = true
  try {
    const res = await api.pointsLeaderboard(1)
    leaderboard.value = res.data || []
  } finally {
    loadingLb.value = false
  }
}

onMounted(() => {
  loadLeaderboard()
  // 点击外部关闭下拉建议
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.input-wrapper')) {
      showSuggestions.value = false
    }
  })
})
</script>

<style scoped>
/* 使用全局样式变量 */
.header-section {
  text-align: center;
  margin-bottom: 30px;
}
.header-section h1 {
  font-size: 32px;
  margin: 0 0 8px;
  color: var(--text);
  font-weight: 950;
  letter-spacing: -0.5px;
}
.subtitle {
  color: var(--muted);
  font-size: 15px;
}

/* --- 布局容器 --- */
.search-wrapper-outer {
  max-width: 800px; /* 与排行榜对齐 */
  margin: 0 auto 40px;
}

.leaderboard-section {
  max-width: 800px; /* 与搜索框对齐 */
  margin: 0 auto;
}

/* --- 搜索部分 --- */
.search-section {
  padding: 30px;
  position: relative;
  /* 使用全局 panel 样式 */
}

.search-bar {
  /* 输入框宽度适配容器，略微收缩 */
  width: 100%;
  margin: 0 auto 20px;
}

.input-wrapper {
  position: relative;
  display: flex;
  gap: 10px;
}

.search-input {
  height: 48px; /* 稍微高一点 */
  font-size: 16px;
}

.suggestions-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: #fff;
  border: 1px solid var(--line);
  border-radius: 12px;
  box-shadow: var(--shadow-2);
  margin-top: 6px;
  z-index: 50;
  max-height: 300px;
  overflow-y: auto;
}

.suggestion-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  cursor: pointer;
  transition: background 0.2s;
  border-bottom: 1px solid rgba(0,0,0,0.03);
}
.suggestion-item:last-child { border-bottom: none; }
.suggestion-item:hover { background: var(--bg-0); }

.s-avatar {
  width: 32px; height: 32px;
  border-radius: 50%;
  object-fit: cover;
  background: #f1f5f9;
}
.s-uid { font-weight: 700; color: var(--text); }

/* --- 结果卡片 --- */
.result-box {
  animation: fadeIn 0.4s ease;
}

.user-card {
  background: linear-gradient(135deg, var(--bg-0), #fff);
  border: 1px solid var(--line);
  border-radius: 20px;
  padding: 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
  box-shadow: var(--shadow-1);
}

.u-info { display: flex; align-items: center; gap: 16px; }
.u-avatar {
  width: 64px; height: 64px;
  border-radius: 50%;
  border: 3px solid #fff;
  box-shadow: 0 4px 10px rgba(0,0,0,0.1);
  background: #f1f5f9;
  object-fit: cover;
}
.u-name { font-size: 20px; font-weight: 800; color: var(--text); }
.u-badge {
  display: inline-block;
  font-size: 12px;
  background: var(--accent);
  color: #fff;
  padding: 2px 8px;
  border-radius: 99px;
  margin-top: 4px;
}

.u-score { text-align: right; }
.u-score .label { font-size: 13px; color: var(--muted); font-weight: 700; text-transform: uppercase; }
.u-score .val { font-size: 36px; font-weight: 900; color: var(--accent-2); line-height: 1; margin-top: 4px; }

/* --- 历史记录 --- */
.history-list h3 { font-size: 16px; margin-bottom: 12px; color: var(--muted); }
.history-list ul { list-style: none; padding: 0; margin: 0; }

.h-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px dashed var(--line);
}
.h-item:last-child { border-bottom: none; }

.h-left { flex: 1; }
.h-note { font-weight: 700; font-size: 15px; color: var(--text); }
.h-sub { font-size: 13px; color: var(--muted); margin-top: 2px; }
.h-date { font-size: 12px; color: var(--muted); opacity: 0.7; margin-top: 2px; }

.h-right {
  font-weight: 850;
  font-size: 16px;
}
.h-right.plus { color: #10b981; }
.h-right.minus { color: #ef4444; }

.not-found, .empty-hint {
  text-align: center;
  padding: 40px;
  color: var(--muted);
  font-weight: 600;
}

/* --- 排行榜部分 --- */
.lb-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 24px;
  font-weight: 900;
  margin-bottom: 20px;
  color: var(--text);
}

.grid-table {
  background: rgba(255,255,255,0.85);
  border-radius: 16px;
  border: 1px solid var(--line);
  overflow: hidden;
  backdrop-filter: blur(10px);
  box-shadow: var(--shadow-1);
}

.row {
  display: grid;
  grid-template-columns: 60px 1fr 100px;
  padding: 14px 20px;
  align-items: center;
}

.header {
  background: rgba(255,255,255,0.5);
  border-bottom: 1px solid var(--line);
  font-weight: 800;
  color: var(--muted);
  font-size: 13px;
}

.item {
  border-bottom: 1px solid rgba(0,0,0,0.03);
  transition: background 0.2s;
}
.item:last-child { border-bottom: none; }
.item:hover { background: rgba(255,255,255,0.6); }

/* 列样式 */
.cell.rank { text-align: center; }
.cell.user { display: flex; align-items: center; gap: 12px; cursor: pointer; }
.cell.score { text-align: right; font-weight: 850; color: var(--accent-2); font-size: 16px; }

.lb-avatar { 
  width: 36px; height: 36px; 
  border-radius: 50%; 
  background: #f1f5f9; 
  object-fit: cover; 
}
.lb-uid { font-weight: 750; color: var(--text); }

/* 奖牌样式 */
.medal {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 28px; height: 28px;
  border-radius: 50%;
  color: #fff;
  font-weight: 900;
  font-size: 14px;
  text-shadow: 0 1px 2px rgba(0,0,0,0.2);
}
.gold { background: linear-gradient(135deg, #ffd700, #f59e0b); box-shadow: 0 2px 6px rgba(245, 158, 11, 0.4); }
.silver { background: linear-gradient(135deg, #e5e7eb, #9ca3af); box-shadow: 0 2px 6px rgba(156, 163, 175, 0.4); color: #4b5563; text-shadow: none; }
.bronze { background: linear-gradient(135deg, #d97706, #b45309); box-shadow: 0 2px 6px rgba(180, 83, 9, 0.4); }

.medal-plain {
  font-weight: 700;
  color: var(--muted);
  opacity: 0.6;
}

.loading {
  padding: 40px;
  text-align: center;
  color: var(--muted);
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

@media (max-width: 600px) {
  .search-section { padding: 20px; }
  .user-card { flex-direction: column; text-align: center; gap: 16px; }
  .u-info { flex-direction: column; gap: 8px; }
  .u-score { text-align: center; width: 100%; border-top: 1px dashed var(--line); paddingTop: 16px; }
}
</style>