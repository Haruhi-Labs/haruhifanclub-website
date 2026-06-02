<template>
  <!-- 
    Vue 3 支持多根节点，为了代码清晰，我们将主容器和弹窗分开。
    主容器保持原样。
  -->
  <div class="page-container">
    <div class="header">
      <div class="title-group">
        <h1>特别授权查询</h1>
        <p class="subtitle">仅展示对应援团/社团有额外授权许可的个人作品</p>
      </div>
      <div class="decoration">
        <svg width="60" height="60" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.2)" stroke-width="1.5">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
        </svg>
      </div>
    </div>

    <div v-if="loading" class="loading-state">
      数据加载中...
    </div>

    <div v-else-if="list.length === 0" class="empty-state">
      暂无包含特别授权的作品
    </div>

    <div v-else class="license-table">
      <!-- 表头 -->
      <div class="row header-row">
        <div class="col-img">预览</div>
        <div class="col-info">作品信息</div>
        <div class="col-auth">作者</div>
        <div class="col-lic">授权详情</div>
      </div>

      <!-- 列表内容 -->
      <div 
        v-for="item in list" 
        :key="item.id" 
        class="row item-row"
        @click="openItem(item)"
        role="button"
        tabindex="0"
      >
        <div class="col-img">
          <div class="thumb-box">
            <img :src="item.image_url" loading="lazy" alt="" />
          </div>
        </div>
        
        <div class="col-info">
          <div class="art-title">{{ item.title }}</div>
          <div class="art-desc">{{ truncate(item.description, 30) }}</div>
        </div>

        <div class="col-auth">
          <div class="auth-name">{{ item.uploader_name || '匿名' }}</div>
          <div class="auth-uid">{{ item.uploader_uid }}</div>
        </div>

        <div class="col-lic">
          <div class="lic-tags">
            <span v-for="tag in getGroupLicenses(item)" :key="tag" class="lic-tag">
              {{ tag }}
            </span>
          </div>
        </div>
        
        <!-- 悬停时的箭头提示 -->
        <div class="arrow-hint">➔</div>
      </div>
    </div>
  </div>

  <!-- 
    核心修复：
    使用 <Teleport to="body"> 将弹窗挂载到 body 标签下。
    这样可以跳过 .page-container 的 backdrop-filter 产生的包含块限制，
    确保 position: fixed 能相对于整个浏览器窗口定位。
  -->
  <Teleport to="body">
    <ArtworkModal
      v-model="modalOpen"
      :item="activeItem"
      @close="activeItem = null"
    />
  </Teleport>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { api } from '../services/api.js'
import ArtworkModal from '../components/ArtworkModal.vue'

const list = ref([])
const loading = ref(true)
const modalOpen = ref(false)
const activeItem = ref(null)

// 获取数据：只拉取 source_type='personal' 的作品，然后在前端筛选有 GROUP 授权的
async function loadData() {
  loading.value = true
  try {
    const res = await api.artworksList({ 
      source_type: 'personal', 
      status: 'approved',
      pageSize: 100 
    })
    
    const rawList = res.data || []
    
    // 筛选逻辑
    list.value = rawList.filter(item => {
      const licenses = Array.isArray(item.licenses) ? item.licenses : []
      return licenses.some(l => l.startsWith('GROUP:'))
    })

  } catch (e) {
    console.error('Load failed', e)
  } finally {
    loading.value = false
  }
}

function getGroupLicenses(item) {
  const licenses = Array.isArray(item.licenses) ? item.licenses : []
  return licenses
    .filter(l => l.startsWith('GROUP:'))
    .map(l => l.replace('GROUP:', ''))
}

function truncate(str, len) {
  if (!str) return ''
  return str.length > len ? str.slice(0, len) + '...' : str
}

function openItem(item) {
  activeItem.value = item
  modalOpen.value = true
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.page-container {
  max-width: 1200px;
  margin: 40px auto;
  padding: 40px; 
  min-height: 80vh;
  
  /* 注意：这个 backdrop-filter 是导致之前弹窗定位错误的原因。
   它创建了一个新的 Containing Block。
   现在的解决方案是把弹窗移出去（Teleport），所以这个样式可以保留不动，
   继续维持页面的毛玻璃美观效果。
  */
  background: rgba(30, 41, 59, 0.75);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 1px solid rgba(255,255,255,0.1);
}

.title-group h1 {
  font-size: 32px;
  font-weight: 900;
  color: #fff;
  margin: 0 0 8px 0;
  letter-spacing: 1px;
}

.subtitle {
  color: rgba(255,255,255,0.7);
  font-size: 14px;
}

/* --- 列表样式 --- */
.license-table {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.row {
  display: grid;
  grid-template-columns: 80px 1fr 150px 2fr; 
  align-items: center;
  gap: 20px;
  padding: 12px 20px;
  border-radius: 12px;
  transition: all 0.2s ease;
}

.header-row {
  background: rgba(0,0,0,0.2);
  font-weight: 700;
  color: rgba(255,255,255,0.6);
  font-size: 13px;
  padding: 10px 20px;
}

.item-row {
  background: rgba(255,255,255,0.08);
  border: 1px solid rgba(255,255,255,0.05);
  cursor: pointer;
  position: relative;
  overflow: hidden;
  height: 84px;
}

.item-row:hover {
  background: rgba(255,255,255,0.15);
  transform: translateX(4px);
  border-color: rgba(255,255,255,0.3);
}

.col-img {
  display: flex;
  align-items: center;
}
.thumb-box {
  width: 60px;
  height: 60px;
  border-radius: 8px;
  overflow: hidden;
  background: #000;
  border: 1px solid rgba(255,255,255,0.1);
}
.thumb-box img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.col-info {
  overflow: hidden;
}
.art-title {
  font-size: 16px;
  font-weight: 700;
  color: #fff;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.art-desc {
  font-size: 12px;
  color: rgba(255,255,255,0.6);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.auth-name {
  font-weight: 600;
  color: #a5b4fc;
}
.auth-uid {
  font-size: 12px;
  color: rgba(255,255,255,0.4);
  font-family: monospace;
}

.col-lic {
  display: flex;
  align-items: center;
}
.lic-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-height: 60px;
  overflow: hidden;
}
.lic-tag {
  background: rgba(16, 185, 129, 0.2);
  border: 1px solid rgba(16, 185, 129, 0.4);
  color: #6ee7b7;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  line-height: 1.2;
}

.arrow-hint {
  position: absolute;
  right: 20px;
  opacity: 0;
  color: rgba(255,255,255,0.5);
  font-size: 20px;
  transition: all 0.2s;
}
.item-row:hover .arrow-hint {
  opacity: 1;
  transform: translateX(5px);
}

.loading-state, .empty-state {
  text-align: center;
  padding: 60px;
  color: rgba(255,255,255,0.6);
  font-size: 16px;
  background: rgba(0,0,0,0.2);
  border-radius: 12px;
}

@media (max-width: 768px) {
  .page-container {
    margin: 80px 12px 30px;
    padding: 20px 12px; 
  }

  .row {
    grid-template-columns: 60px 1fr;
    grid-template-areas: 
      "img info"
      "img lic";
    gap: 8px 12px;
    height: auto;
    padding: 16px;
  }
  
  .header-row { display: none; }
  
  .col-img { grid-area: img; }
  .col-info { grid-area: info; }
  .col-lic { grid-area: lic; }
  
  .col-auth { display: none; }
  
  .lic-tags {
    flex-wrap: nowrap;
    overflow-x: auto;
    padding-bottom: 4px;
  }
  .lic-tag {
    white-space: nowrap;
  }
  
  .arrow-hint { display: none; }
}
</style>