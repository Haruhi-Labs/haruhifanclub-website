<template>
  <section class="container-card admin-panel">
    <!-- 顶部 -->
    <div class="head-row">
      <div class="head-left">
        <div class="h1">管理后台 (AI Enhanced)</div>
        <div class="sub" v-if="authed">已登录 | Security: Backend Env</div>
      </div>
      <button v-if="authed" class="btn-ghost sm danger" @click="logout">退出登录</button>
    </div>

    <!-- 登录页（统一 JWT 单点登录：用户名 + 密码） -->
    <div v-if="!authed" class="auth-box">
      <div class="form2">
        <input class="input" v-model="loginUser" type="text" placeholder="请输入用户名" @keydown.enter="checkPw" />
        <input class="input" v-model="inputPw" type="password" placeholder="请输入密码" @keydown.enter="checkPw" />
        <button class="btn" @click="checkPw" :disabled="loading">
          {{ loading ? '验证中...' : '进入系统' }}
        </button>
      </div>
      <div class="msg" v-if="msg">{{ msg }}</div>
    </div>

    <!-- 主面板 -->
    <div v-else class="panel-layout">
      <!-- 左侧/顶部 一级导航 -->
      <nav class="main-nav">
        <button :class="['nav-item', mainTab==='images' && 'active']" @click="mainTab='images'">
          📷 图片管理
        </button>
        <button :class="['nav-item', mainTab==='comments' && 'active']" @click="mainTab='comments'">
          💬 评论管理
        </button>
        <button :class="['nav-item', mainTab==='creators' && 'active']" @click="mainTab='creators'">
          🎨 创作者
        </button>
      </nav>

      <!-- 内容区域 -->
      <main class="content-area">

        <!-- ================= 图片管理 ================= -->
        <div v-if="mainTab==='images'" class="tab-content">
          <div class="sub-tabs">
            <button :class="['sub-tab', imageTab==='audit' && 'on']" @click="imageTab='audit'">审核管理 ({{ adminStore.pending.length }})</button>
            <button :class="['sub-tab', imageTab==='list' && 'on']" @click="imageTab='list'">作品列表 (已过审)</button>
          </div>

          <!-- 子页：审核管理 -->
          <div v-if="imageTab==='audit'" class="sub-view">
            <div class="toolbar">
              <span class="tip">待审核或被AI拦截的内容</span>
              <button class="btn-ghost sm" @click="adminStore.loadPending">刷新队列</button>
            </div>

            <div v-if="adminStore.pending.length" class="card-grid">
              <article class="manage-card" v-for="it in adminStore.pending" :key="it.id">
                <div class="m-thumb" @click="openPreview(it)" style="cursor: pointer;" title="点击预览">
                  <img :src="it.image_url" loading="lazy" />
                  <div class="status-badge" :class="it.status">{{ it.status === 'flagged' ? 'AI 锁定' : '待审核' }}</div>
                </div>
                <div class="m-body">
                  <div class="m-row title-row">
                    <div class="m-title">{{ it.title }}</div>
                    <button class="btn-text" @click="startEdit(it)">编辑</button>
                  </div>
                  
                  <div class="ai-box" v-if="it.status === 'flagged'">
                    <span class="ai-label">🤖 拦截原因:</span> {{ it.ai_reason || '未知风险' }}
                  </div>

                  <div class="m-info">
                    <span class="tag">{{ it.content_type }}</span>
                    <span class="tag">{{ it.source_type }}</span>
                    <span class="u-name">UP: {{ it.uploader_name || '匿名' }} ({{ it.uploader_uid || '无UID' }})</span>
                  </div>
                  <div class="m-desc">{{ it.description }}</div>
                  
                  <!-- 编辑器 -->
                  <div v-if="editingId === it.id" class="inline-editor expanded">
                    <div class="editor-row">
                      <input v-model="editForm.title" class="input sm" placeholder="标题">
                      <input v-model="editForm.tags" class="input sm" placeholder="标签 (空格分隔)">
                    </div>
                    <textarea v-model="editForm.description" class="textarea sm" placeholder="描述"></textarea>
                    
                    <div class="editor-row">
                      <select v-model="editForm.source_type" class="select sm">
                        <option value="personal">个人作品</option>
                        <option value="network">网络收集</option>
                      </select>
                      <select v-model="editForm.content_type" class="select sm">
                        <option value="haruhi">凉宫内容</option>
                        <option value="other">非凉宫内容</option>
                      </select>
                    </div>

                    <div class="editor-row">
                      <input v-model="editForm.uploader_name" class="input sm" placeholder="上传者名称">
                      <input v-model="editForm.uploader_uid" class="input sm" placeholder="上传者 UID">
                    </div>
                    
                    <div class="editor-row">
                      <input v-model="editForm.origin_url" class="input sm" placeholder="来源链接 (URL)" style="flex:1">
                    </div>

                    <div class="editor-licenses">
                      <div class="lic-group">
                        <div class="lic-title">公开授权</div>
                        <label v-for="opt in NET_LICENSE_OPTIONS" :key="opt" class="chk-item">
                          <input type="checkbox" :value="opt" v-model="editForm.netLicenses"> {{ opt }}
                        </label>
                      </div>
                      <div class="lic-group">
                        <div class="lic-title">社团授权</div>
                        <label v-for="opt in GROUP_LICENSE_OPTIONS" :key="opt" class="chk-item">
                          <input type="checkbox" :value="opt" v-model="editForm.groupLicenses"> {{ opt }}
                        </label>
                      </div>
                    </div>

                    <div class="btns">
                      <button class="btn sm" @click="saveEdit(it)">💾 保存修改</button>
                      <button class="btn-ghost sm" @click="editingId=null">取消</button>
                    </div>
                  </div>

                  <div class="m-actions">
                    <textarea class="textarea note-input" v-model="notes[it.id]" placeholder="审核备注..."></textarea>
                    <div class="btn-group">
                      <button class="btn success" @click="approve(it)">通过</button>
                      <button class="btn danger" @click="hardDelete(it)">彻底删除</button>
                    </div>
                  </div>
                </div>
              </article>
            </div>
            <div v-else class="empty-state">🎉 暂无待审核内容</div>
          </div>

          <!-- 子页：作品列表 (已过审) -->
          <div v-if="imageTab==='list'" class="sub-view">
            <div class="toolbar filter-bar">
              <div class="filters">
                <select v-model="artListFilter.content" @change="loadApprovedList" class="select">
                  <option value="all">全部分类</option>
                  <option value="haruhi">凉宫内容</option>
                  <option value="other">非凉宫内容</option>
                </select>
                <select v-model="artListFilter.source" @change="loadApprovedList" class="select">
                  <option value="all">全部来源</option>
                  <option value="personal">个人作品</option>
                  <option value="network">网络收集</option>
                </select>
                <input v-model="artListFilter.q" @keydown.enter="loadApprovedList" class="input sm search-input" placeholder="搜索标题/描述/UID..." />
                <button class="btn sm" @click="loadApprovedList">查询</button>
              </div>
            </div>

            <div class="card-grid">
              <article class="manage-card" v-for="it in approvedList" :key="it.id">
                <div class="m-thumb" @click="openPreview(it)" style="cursor: pointer;" title="点击预览">
                  <img :src="it.image_url" loading="lazy" />
                  <div class="status-badge approved">已发布</div>
                </div>
                <div class="m-body">
                   <div class="m-row title-row">
                    <div class="m-title">{{ it.title }}</div>
                    <button class="btn-text" @click="startEdit(it)">编辑</button>
                  </div>
                  <div class="m-info">
                    <span class="tag">{{ it.content_type }}</span>
                    <span class="tag">{{ it.source_type }}</span>
                    <span class="u-name">UID: {{ it.uploader_uid }}</span>
                  </div>
                  <div class="m-desc">{{ it.description }}</div>
                  
                  <div v-if="editingId === it.id" class="inline-editor expanded">
                    <div class="editor-row">
                      <input v-model="editForm.title" class="input sm" placeholder="标题">
                      <input v-model="editForm.tags" class="input sm" placeholder="标签 (空格分隔)">
                    </div>
                    <textarea v-model="editForm.description" class="textarea sm" placeholder="描述"></textarea>

                    <div class="editor-row">
                      <select v-model="editForm.source_type" class="select sm">
                        <option value="personal">个人作品</option>
                        <option value="network">网络收集</option>
                      </select>
                      <select v-model="editForm.content_type" class="select sm">
                        <option value="haruhi">凉宫内容</option>
                        <option value="other">非凉宫内容</option>
                      </select>
                    </div>

                    <div class="editor-row">
                      <input v-model="editForm.uploader_name" class="input sm" placeholder="上传者名称">
                      <input v-model="editForm.uploader_uid" class="input sm" placeholder="上传者 UID">
                    </div>

                    <div class="editor-row">
                      <input v-model="editForm.origin_url" class="input sm" placeholder="来源链接 (URL)" style="flex:1">
                    </div>
                    
                    <div class="editor-licenses">
                      <div class="lic-group">
                        <div class="lic-title">公开授权</div>
                        <label v-for="opt in NET_LICENSE_OPTIONS" :key="opt" class="chk-item">
                          <input type="checkbox" :value="opt" v-model="editForm.netLicenses"> {{ opt }}
                        </label>
                      </div>
                      <div class="lic-group">
                        <div class="lic-title">社团授权</div>
                        <label v-for="opt in GROUP_LICENSE_OPTIONS" :key="opt" class="chk-item">
                          <input type="checkbox" :value="opt" v-model="editForm.groupLicenses"> {{ opt }}
                        </label>
                      </div>
                    </div>

                    <div class="btns">
                      <button class="btn sm" @click="saveEdit(it)">💾 保存修改</button>
                      <button class="btn-ghost sm" @click="editingId=null">取消</button>
                    </div>
                  </div>

                  <div class="m-actions right">
                    <button class="btn-ghost warn sm" @click="lockArtwork(it)">🔒 锁定 (退回审核)</button>
                    <button class="btn-ghost danger sm" @click="hardDelete(it, 'list')">🗑 删除</button>
                  </div>
                </div>
              </article>
            </div>
            <div class="pagination">
              <button class="btn-ghost sm" :disabled="artListPage<=1" @click="changePage(-1)">上一页</button>
              <span>第 {{ artListPage }} 页</span>
              <button class="btn-ghost sm" @click="changePage(1)">下一页</button>
            </div>
          </div>
        </div>

        <!-- ================= 评论管理 ================= -->
        <div v-if="mainTab==='comments'" class="tab-content">
          <div class="sub-tabs">
            <button :class="['sub-tab', commentTab==='pending' && 'on']" @click="switchCommentTab('pending')">待复核 (AI锁定)</button>
            <button :class="['sub-tab', commentTab==='all' && 'on']" @click="switchCommentTab('all')">全部评论</button>
          </div>

          <div class="toolbar" v-if="commentTab==='all'">
            <input v-model="commentSearch" class="input sm comment-search" placeholder="搜索评论内容..." />
          </div>

          <div class="comment-table desktop-only">
            <div class="c-row header">
              <div class="col-user">用户</div>
              <div class="col-content">内容</div>
              <div class="col-status">状态</div>
              <div class="col-action">操作</div>
            </div>
            <div class="c-row" v-for="c in filteredComments" :key="c.id">
              <div class="col-user">
                <div class="u-name">{{ c.user_name }}</div>
                <div class="u-time">{{ new Date(c.created_at).toLocaleString() }}</div>
              </div>
              <div class="col-content">
                <div class="body-text">{{ c.body }}</div>
                <div v-if="c.ai_reason" class="ai-reason-mini">🤖 {{ c.ai_reason }}</div>
              </div>
              <div class="col-status">
                <span :class="['badge-mini', c.status]">{{ c.status === 'flagged' ? 'AI锁定' : (c.status==='hidden'?'隐藏':'正常') }}</span>
              </div>
              <div class="col-action actions-flex">
                <button v-if="c.status !== 'public'" class="btn-mini success" @click="updateComment(c, 'public')">通过</button>
                <button v-if="c.status === 'public'" class="btn-mini warn" @click="updateComment(c, 'flagged')">锁定</button>
                <button class="btn-mini danger" @click="deleteComment(c)">删除</button>
              </div>
            </div>
            <div v-if="filteredComments.length === 0" class="empty-row">无数据</div>
          </div>

          <div class="comment-list-mobile">
            <article class="comment-card" v-for="c in filteredComments" :key="`mobile-${c.id}`">
              <div class="comment-card-top">
                <div>
                  <div class="u-name">{{ c.user_name }}</div>
                  <div class="u-time">{{ new Date(c.created_at).toLocaleString() }}</div>
                </div>
                <span :class="['badge-mini', c.status]">{{ c.status === 'flagged' ? 'AI锁定' : (c.status==='hidden'?'隐藏':'正常') }}</span>
              </div>
              <div class="body-text">{{ c.body }}</div>
              <div v-if="c.ai_reason" class="ai-reason-mini">🤖 {{ c.ai_reason }}</div>
              <div class="comment-card-actions">
                <button v-if="c.status !== 'public'" class="btn-mini success" @click="updateComment(c, 'public')">通过</button>
                <button v-if="c.status === 'public'" class="btn-mini warn" @click="updateComment(c, 'flagged')">锁定</button>
                <button class="btn-mini danger" @click="deleteComment(c)">删除</button>
              </div>
            </article>
            <div v-if="filteredComments.length === 0" class="empty-row">无数据</div>
          </div>
        </div>

        <!-- ================= 创作者管理 (优化后) ================= -->
        <div v-if="mainTab==='creators'" class="tab-content">
          <div class="two-col-layout">
            <!-- 左列：列表 -->
            <div class="col-left" :class="{ 'mobile-hidden': selectedCreator }">
              <div class="toolbar tight">
                <input v-model="creatorSearch" class="input sm" placeholder="搜索 UID..." />
                <div class="add-row">
                   <input v-model="newCreatorUid" placeholder="新增 UID" class="input sm" />
                   <button class="btn sm" @click="addCreator">+</button>
                </div>
              </div>

              <div class="creator-list-v">
                <div 
                  v-for="c in filteredCreators" :key="c.uid" 
                  class="creator-item" 
                  :class="{ active: selectedCreator?.uid === c.uid }"
                  @click="selectCreator(c)"
                >
                  <img :src="c.avatar_url || '/api/art/placeholder/40/40'" class="c-avatar sm" />
                  <div class="c-info-mini">
                    <div class="c-uid">{{ c.uid }}</div>
                    <div class="c-sub">
                      <span>{{ new Date(c.created_at).toLocaleDateString() }}</span>
                      <span v-if="c.qq" class="qq-badge">QQ</span>
                    </div>
                  </div>
                  <div class="c-arr">›</div>
                </div>
                <div v-if="filteredCreators.length === 0" class="empty-ph">暂无数据</div>
              </div>
            </div>

            <!-- 右列：详情与编辑 -->
            <div class="col-right" :class="{ 'mobile-visible': selectedCreator }">
              <div v-if="selectedCreator" class="creator-detail-panel">
                <div class="panel-header">
                  <div class="ph-title">编辑创作者: {{ selectedCreator.uid }}</div>
                  <div class="panel-actions">
                    <button class="btn-ghost sm mobile-only" @click="selectedCreator=null">返回列表</button>
                    <button class="btn-ghost danger sm" @click="deleteCreator">删除账号</button>
                  </div>
                </div>

                <div class="edit-form">
                   <!-- 头像设置 -->
                   <div class="form-group">
                     <label>头像配置</label>
                     <div class="avatar-uploader">
                       <img :src="previewAvatar || selectedCreator.avatar_url || '/api/art/placeholder/80/80'" class="avatar-preview" />
                       <div class="au-actions">
                         <input type="file" ref="fileInput" accept="image/*" @change="handleFileChange" style="display:none" />
                         <button class="btn-ghost sm" @click="$refs.fileInput.click()">选择本地图片</button>
                         <div class="tip-text">支持 jpg, png, webp</div>
                       </div>
                     </div>
                   </div>

                   <!-- 名称/UID 修改 -->
                   <div class="form-group">
                     <label>UID (名称)</label>
                     <div class="row">
                       <input v-model="editCreatorForm.uid" class="input" placeholder="输入新的 UID" />
                     </div>
                     <div class="tip-text warn">⚠️ 修改 UID 会同步更新该作者所有的投稿记录和积分记录，请谨慎操作。</div>
                   </div>

                   <!-- QQ号 修改 -->
                   <div class="form-group">
                     <label>关联 QQ 号</label>
                     <div class="row">
                       <input v-model="editCreatorForm.qq" class="input" placeholder="输入关联的QQ号码" />
                     </div>
                     <div class="tip-text">用于身份核实或联系，仅后台可见。</div>
                   </div>
                   
                   <div class="form-actions">
                     <button class="btn" @click="updateCreator" :disabled="!isCreatorModified">保存更改</button>
                     <span v-if="saveMsg" class="save-msg">{{ saveMsg }}</span>
                   </div>
                </div>

                <div class="divider"></div>

                <!-- 积分管理 -->
                <div class="points-section">
                  <div class="label-lg">积分管理</div>
                  
                  <div class="points-action-row">
                    <div class="quick-points">
                      <button v-for="v in [10, 20, 50, -10, -50]" :key="v" 
                        class="chip-btn" 
                        :class="pointsForm.amount === v && 'active'"
                        @click="pointsForm.amount = v">
                        {{ v > 0 ? '+' + v : v }}
                      </button>
                    </div>
                    <div class="pa-form">
                      <div class="input-group">
                        <span class="input-prefix">分值</span>
                        <input type="number" v-model.number="pointsForm.amount" class="input points-num sm" />
                      </div>
                      <input v-model="pointsForm.reason" class="input sm" placeholder="变更原因 (必填)" style="flex:1" />
                      <button class="btn sm" @click="grantPoints" :disabled="!pointsForm.reason">执行</button>
                    </div>
                  </div>

                  <div class="ph-list compact">
                    <div class="ph-row head">
                      <span>时间</span>
                      <span>变动</span>
                      <span>原因</span>
                    </div>
                    <div class="ph-scroll-area">
                       <div class="ph-row" v-for="(log, idx) in creatorLogs" :key="idx">
                        <span class="ph-time">{{ new Date(log.granted_at).toLocaleDateString() }}</span>
                        <span class="ph-val" :class="log.points > 0 ? 'pos' : 'neg'">{{ log.points > 0 ? '+' : '' }}{{ log.points }}</span>
                        <span class="ph-reason">{{ log.note || log.artwork_title }}</span>
                      </div>
                      <div v-if="!creatorLogs.length" class="empty-ph">暂无积分记录</div>
                    </div>
                  </div>
                </div>

              </div>
              <div v-else class="empty-select">
                <div class="icon">🎨</div>
                <div>请在左侧选择一个创作者进行管理</div>
              </div>
            </div>
          </div>
        </div>

      </main>
    </div>

    <!-- 作品预览弹窗 -->
    <ArtworkModal v-model="showPreview" :item="previewItem" />

  </section>
</template>

<script setup>
import { onMounted, ref, computed, watch } from 'vue'
import { useAdminStore } from '../stores/adminStore.js'
import { api } from '../services/api.js'
import ArtworkModal from '../components/ArtworkModal.vue'
import { createAdminAuth } from '@haruhi/api-client'

// 统一鉴权：登录/会话恢复/登出/art 权限校验全部收敛到共享 createAdminAuth；
// 其余后台请求由 services/api.js 自动带 JWT
const admin = createAdminAuth('art')

const adminStore = useAdminStore()
const authed = ref(false)
const loginUser = ref('')
const inputPw = ref('')
const msg = ref('')
const loading = ref(false)

// 导航状态
const mainTab = ref('images') // images, comments, creators
const imageTab = ref('audit') // audit, list
const commentTab = ref('pending') // pending, all

// --- 图片管理数据 ---
const approvedList = ref([])
const artListPage = ref(1)
const artListFilter = ref({ content: 'all', source: 'all', q: '' })
const editingId = ref(null)
const editForm = ref({ 
  title: '', description: '', tags: '', 
  uploader_name: '', uploader_uid: '', 
  source_type: 'personal', content_type: 'haruhi', origin_url: '',
  netLicenses: [], groupLicenses: []
})

const NET_LICENSE_OPTIONS = [
  '可在b站、小红书等社交媒体转载',
  '允许用于视频等个人创作',
  '允许用于制作无料发放'
]

const GROUP_LICENSE_OPTIONS = [
  '允许应援团社交媒体官方账号转载',
  '允许用于应援团官方视频/游戏等创作企划',
  '允许制作无料周边发放',
  '允许制作贩售周边'
]
const notes = ref({}) // 审核备注
const showPreview = ref(false)
const previewItem = ref(null)

// --- 评论管理数据 ---
const comments = ref([])
const commentSearch = ref('')

// --- 创作者管理数据 ---
const creators = ref([])
const creatorSearch = ref('')
const newCreatorUid = ref('')
const selectedCreator = ref(null)
const editCreatorForm = ref({ uid: '', qq: '', file: null })
const previewAvatar = ref(null)
const saveMsg = ref('')
const creatorLogs = ref([])
const pointsForm = ref({ amount: 10, reason: '' })

// 计算属性：过滤后的创作者
const filteredCreators = computed(() => {
  if (!creatorSearch.value) return creators.value
  const q = creatorSearch.value.toLowerCase()
  return creators.value.filter(c => c.uid.toLowerCase().includes(q))
})

// 计算是否有修改
const isCreatorModified = computed(() => {
  if (!selectedCreator.value) return false
  const uidChanged = editCreatorForm.value.uid !== selectedCreator.value.uid
  const qqChanged = editCreatorForm.value.qq !== (selectedCreator.value.qq || '')
  const fileChanged = !!editCreatorForm.value.file
  return uidChanged || qqChanged || fileChanged
})

// 计算属性：过滤后的评论
const filteredComments = computed(() => {
  if (commentTab.value === 'pending') return comments.value 
  if (!commentSearch.value) return comments.value
  const q = commentSearch.value.toLowerCase()
  return comments.value.filter(c => 
    c.body.toLowerCase().includes(q) || 
    c.user_name.toLowerCase().includes(q)
  )
})

// --- 核心方法 ---

async function checkPw() {
  if (!loginUser.value || !inputPw.value) return
  loading.value = true
  msg.value = ''

  try {
    // 共享鉴权：login 内部已校验 art 权限，永不抛错，返回 { ok, user?, error? }
    const r = await admin.login(loginUser.value.trim(), inputPw.value)
    if (!r.ok) {
      msg.value = r.error
      return
    }
    authed.value = true
    inputPw.value = ''
    init()
  } finally {
    loading.value = false
  }
}

function logout() {
  admin.logout()
  authed.value = false
  loginUser.value = ''
  inputPw.value = ''
  msg.value = ''
}

function init() {
  adminStore.loadPending()
  loadApprovedList()
}

// ---------------- 图片管理 ----------------

async function loadApprovedList() {
  try {
    const res = await api.artworksList({
      status: 'approved',
      content_type: artListFilter.value.content,
      source_type: artListFilter.value.source,
      q: artListFilter.value.q,
      page: artListPage.value,
      pageSize: 20
    })
    approvedList.value = res.data || []
  } catch(e) { console.error(e) }
}

function changePage(delta) {
  artListPage.value += delta
  loadApprovedList()
}

function openPreview(it) {
  previewItem.value = it
  showPreview.value = true
}

async function approve(it) {
  await adminStore.approveArtwork(it, notes.value[it.id])
}

async function hardDelete(it, from = 'audit') {
  if (!confirm(`⚠️ 警告：正在从数据库中永久删除作品 "${it.title}"。\n此操作不可恢复！是否继续？`)) return
  await api.adminDeleteArtwork(it.id)
  if (from === 'audit') adminStore.removeItem(it.id)
  else if (from === 'list') loadApprovedList()
}

async function lockArtwork(it) {
  if (!confirm('锁定后该作品将下架并进入审核队列，确认？')) return
  await api.adminUpdateArtworkStatus(it.id, 'flagged')
  loadApprovedList()
  adminStore.loadPending() 
}

function startEdit(it) {
  editingId.value = it.id
  editForm.value = {
    title: it.title,
    description: it.description,
    tags: Array.isArray(it.tags) ? it.tags.join(' ') : '',
    uploader_name: it.uploader_name || '',
    uploader_uid: it.uploader_uid || '',
    source_type: it.source_type || 'personal',
    content_type: it.content_type || 'haruhi',
    origin_url: it.origin_url || '',
    netLicenses: [],
    groupLicenses: []
  }
  
  if (Array.isArray(it.licenses)) {
    it.licenses.forEach(l => {
      if (l.startsWith('NET:')) editForm.value.netLicenses.push(l.replace('NET:', ''))
      if (l.startsWith('GROUP:')) editForm.value.groupLicenses.push(l.replace('GROUP:', ''))
    })
  }
}
async function saveEdit(it) {
  const payload = {
    ...editForm.value,
    licenses: JSON.stringify([
      ...editForm.value.netLicenses.map(x => `NET:${x}`),
      ...editForm.value.groupLicenses.map(x => `GROUP:${x}`)
    ])
  }
  await api.adminUpdateArtworkDetails(it.id, payload)
  
  // 更新本地数据
  it.title = editForm.value.title
  it.description = editForm.value.description
  it.uploader_name = editForm.value.uploader_name
  it.uploader_uid = editForm.value.uploader_uid
  it.source_type = editForm.value.source_type
  it.content_type = editForm.value.content_type
  it.origin_url = editForm.value.origin_url
  it.licenses = [
      ...editForm.value.netLicenses.map(x => `NET:${x}`),
      ...editForm.value.groupLicenses.map(x => `GROUP:${x}`)
  ]
  editingId.value = null
}

// ---------------- 评论管理 ----------------

async function switchCommentTab(t) {
  commentTab.value = t
  commentSearch.value = ''
  const statusParam = t === 'pending' ? 'flagged' : 'all'
  const res = await api.adminListComments(statusParam)
  comments.value = res.data
}

async function updateComment(c, status) {
  await api.adminUpdateCommentStatus(c.id, status)
  switchCommentTab(commentTab.value)
}

async function deleteComment(c) {
  if (!confirm('确认删除评论？')) return
  await api.adminDeleteComment(c.id)
  comments.value = comments.value.filter(x => x.id !== c.id)
}

// ---------------- 创作者管理 (新逻辑) ----------------

async function loadCreators() {
  const res = await api.adminCreators()
  creators.value = res.data
  // 如果当前选中的创作者还在列表中，刷新它
  if (selectedCreator.value) {
    const fresh = creators.value.find(c => c.uid === selectedCreator.value.uid)
    if (fresh) {
      // 保持选中，但可能数据已更新，暂不强制覆盖表单，避免打断输入
    } 
  }
}

async function addCreator() {
  if (!newCreatorUid.value) return
  await api.adminAddCreator(newCreatorUid.value)
  newCreatorUid.value = ''
  await loadCreators()
}

async function selectCreator(c) {
  selectedCreator.value = c
  // 绑定数据到表单，处理 QQ 可能为空的情况
  editCreatorForm.value = { uid: c.uid, qq: c.qq || '', file: null }
  previewAvatar.value = null
  saveMsg.value = ''
  pointsForm.value = { amount: 10, reason: '' }
  
  // 加载积分
  const res = await api.adminPointsLedger()
  creatorLogs.value = (res.data || []).filter(l => l.uid === c.uid)
}

function handleFileChange(e) {
  const file = e.target.files[0]
  if (!file) return
  editCreatorForm.value.file = file
  previewAvatar.value = URL.createObjectURL(file)
}

async function updateCreator() {
  if (!selectedCreator.value) return
  
  try {
    const formData = new FormData()
    formData.append('new_uid', editCreatorForm.value.uid)
    formData.append('qq', editCreatorForm.value.qq) // 添加 QQ
    if (editCreatorForm.value.file) {
      formData.append('avatar', editCreatorForm.value.file)
    }

    const res = await api.adminUpdateCreator(selectedCreator.value.uid, formData)
    
    saveMsg.value = '保存成功！'
    setTimeout(() => saveMsg.value = '', 2000)

    // 重新加载列表
    await loadCreators()
    
    // 定位到新的UID (如果改名了)
    const newUid = editCreatorForm.value.uid
    const newObj = creators.value.find(c => c.uid === newUid)
    if (newObj) {
      selectCreator(newObj)
    }
  } catch(e) {
    console.error(e)
    alert('保存失败: ' + (e.message || '未知错误'))
  }
}

async function deleteCreator() {
  if (!selectedCreator.value) return
  const uid = selectedCreator.value.uid
  if (!confirm(`⚠️ 严重警告：\n你确定要删除创作者 "${uid}" 吗？\n该操作不可恢复！`)) return

  try {
    await api.adminDeleteCreator(uid)
    selectedCreator.value = null
    await loadCreators()
  } catch(e) {
    alert('删除失败')
  }
}

async function grantPoints() {
  if (!selectedCreator.value || !pointsForm.value.reason) return
  try {
    await api.adminGrantPoints({
      uid: selectedCreator.value.uid,
      artwork_id: 0,
      points: pointsForm.value.amount,
      note: pointsForm.value.reason
    })
    // 刷新记录
    const res = await api.adminPointsLedger()
    creatorLogs.value = (res.data || []).filter(l => l.uid === selectedCreator.value.uid)
    pointsForm.value.reason = ''
  } catch(e) {
    alert('操作失败')
  }
}

// 监听 Tab 切换加载数据
watch(mainTab, (v) => {
  if (v === 'images' && imageTab.value === 'list') loadApprovedList()
  if (v === 'comments') switchCommentTab('pending')
  if (v === 'creators') loadCreators()
})

onMounted(async () => {
  // 共享鉴权：会话恢复（校验 token 有效性 + art 权限），失败返回 null 并已登出
  loading.value = true
  try {
    const user = await admin.restore()
    if (user) {
      authed.value = true
      init()
    }
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
/* 全局样式 */
.admin-panel {
  min-height: 80vh;
  display: flex;
  flex-direction: column;
}

/* 顶部 Header */
.head-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #f3f4f6;
}
.head-left { display: flex; flex-direction: column; gap: 4px; }
.h1 { font-size: 24px; font-weight: 900; color: #1f2937; }
.sub { font-size: 13px; color: #6b7280; font-family: monospace; }

.auth-box { padding: 40px; display: flex; flex-direction: column; align-items: center; gap: 10px; }
.form2 { display: flex; gap: 10px; }
.msg { color: #dc2626; font-size: 13px; margin-top: 10px; }

/* 布局 */
.panel-layout { display: flex; flex: 1; margin-top: 0px; }
.main-nav {
  width: 140px;
  border-right: 1px solid #e5e7eb;
  padding: 20px 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.nav-item {
  text-align: left;
  padding: 12px 15px;
  background: transparent;
  border: none;
  cursor: pointer;
  font-weight: 600;
  color: #4b5563;
  border-left: 3px solid transparent;
  transition: all 0.2s;
}
.nav-item:hover { background: #f3f4f6; color: #111; }
.nav-item.active { background: #eff6ff; color: #2563eb; border-left-color: #2563eb; }

.content-area { flex: 1; padding: 20px 30px; background: #f9fafb; }

/* 子Tab */
.sub-tabs { display: flex; gap: 20px; border-bottom: 1px solid #e5e7eb; margin-bottom: 20px; }
.sub-tab {
  padding: 10px 0;
  background: transparent;
  border: none;
  font-size: 15px;
  font-weight: 600;
  color: #9ca3af;
  cursor: pointer;
  border-bottom: 2px solid transparent;
}
.sub-tab:hover { color: #4b5563; }
.sub-tab.on { color: #111; border-bottom-color: #111; }

/* 工具栏 */
.toolbar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 10px; }
.toolbar.tight { margin-bottom: 10px; flex-direction: column; align-items: stretch; gap: 8px; }
.filter-bar { background: #fff; padding: 12px; border-radius: 8px; border: 1px solid #e5e7eb; }
.filters { display: flex; gap: 10px; align-items: center; flex: 1; }
.comment-search { width: 240px; }
.tip { font-size: 13px; color: #6b7280; }
.add-row { display: flex; gap: 8px; }

/* 两列式布局 */
.two-col-layout { display: flex; height: calc(100vh - 200px); min-height: 500px; gap: 20px; }
.col-left { width: 280px; display: flex; flex-direction: column; border-right: 1px solid #e5e7eb; padding-right: 16px; }
.col-right { flex: 1; overflow-y: auto; background: #fff; border-radius: 8px; border: 1px solid #e5e7eb; padding: 0; }

/* 创作者列表 */
.creator-list-v { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 4px; padding-right: 4px; }
.creator-item {
  display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: 8px;
  cursor: pointer; transition: all 0.15s; border: 1px solid transparent;
}
.creator-item:hover { background: #fff; border-color: #e5e7eb; }
.creator-item.active { background: #eff6ff; border-color: #bfdbfe; }
.c-avatar.sm { width: 36px; height: 36px; border-radius: 50%; object-fit: cover; background: #eee; }
.c-info-mini { flex: 1; overflow: hidden; }
.c-uid { font-weight: 600; font-size: 14px; color: #1f2937; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.c-sub { font-size: 11px; color: #9ca3af; display: flex; gap: 6px; align-items: center; }
.qq-badge { background: #dbeafe; color: #1e40af; padding: 0 4px; border-radius: 3px; font-size: 10px; font-weight: bold; }
.c-arr { color: #d1d5db; font-size: 18px; }

/* 创作者详情面板 */
.creator-detail-panel { display: flex; flex-direction: column; height: 100%; }
.panel-header { padding: 16px 24px; border-bottom: 1px solid #e5e7eb; display: flex; justify-content: space-between; align-items: center; background: #f9fafb; }
.ph-title { font-size: 18px; font-weight: bold; color: #111; }
.panel-actions { display: flex; align-items: center; gap: 8px; }
.empty-select { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #9ca3af; gap: 10px; }
.empty-select .icon { font-size: 40px; opacity: 0.5; }

.edit-form { padding: 24px; display: flex; flex-direction: column; gap: 20px; }
.form-group { display: flex; flex-direction: column; gap: 8px; }
.form-group label { font-size: 13px; font-weight: 600; color: #374151; }
.avatar-uploader { display: flex; align-items: center; gap: 20px; }
.avatar-preview { width: 80px; height: 80px; border-radius: 50%; object-fit: cover; border: 1px solid #e5e7eb; }
.au-actions { display: flex; flex-direction: column; gap: 6px; }
.tip-text { font-size: 12px; color: #9ca3af; }
.tip-text.warn { color: #d97706; margin-top: 4px; }
.form-actions { display: flex; align-items: center; gap: 10px; margin-top: 10px; }
.save-msg { color: #059669; font-size: 13px; font-weight: 600; }
.divider { height: 1px; background: #e5e7eb; margin: 0 24px; }

/* 详情页内的积分部分 */
.points-section { padding: 24px; }
.label-lg { font-size: 16px; font-weight: bold; color: #1f2937; margin-bottom: 16px; }
.points-action-row { margin-bottom: 16px; }

.ph-list.compact { border: 1px solid #e5e7eb; border-radius: 8px; overflow: hidden; }
.ph-scroll-area { max-height: 200px; overflow-y: auto; }
.ph-row { padding: 8px 12px; font-size: 12px; }

/* 卡片通用 */
.card-grid { display: grid; gap: 16px; }
.manage-card {
  display: flex;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}
.m-thumb { width: 140px; height: 140px; position: relative; flex-shrink: 0; }
.m-thumb img { width: 100%; height: 100%; object-fit: cover; }

/* 状态标 */
.status-badge {
  position: absolute;
  top: 0; left: 0; right: 0;
  padding: 4px;
  text-align: center;
  font-size: 11px;
  font-weight: bold;
  color: #fff;
  background: rgba(0,0,0,0.6);
}
.status-badge.flagged { background: #dc2626; }
.status-badge.approved { background: #059669; }
.status-badge.pending { background: #d97706; }

.m-body { flex: 1; padding: 16px; display: flex; flex-direction: column; gap: 8px; }
.m-row { display: flex; justify-content: space-between; align-items: center; }
.m-title { font-weight: bold; font-size: 16px; color: #111; }

.m-info { display: flex; gap: 8px; align-items: center; font-size: 12px; color: #6b7280; flex-wrap: wrap; }
.tag { background: #f3f4f6; padding: 2px 8px; border-radius: 4px; }
.m-info .u-name { margin-left: auto; }

.m-desc { font-size: 13px; color: #4b5563; line-height: 1.4; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }

.ai-box { background: #fef2f2; border: 1px solid #fee2e2; padding: 6px 10px; border-radius: 4px; font-size: 12px; color: #b91c1c; }

/* 编辑器 */
.inline-editor { padding: 12px; background: #f9fafb; border: 1px dashed #d1d5db; border-radius: 6px; display: flex; flex-direction: column; gap: 8px; margin: 8px 0; }
.btns { display: flex; gap: 8px; }

/* 操作栏 */
.m-actions { margin-top: auto; padding-top: 10px; display: flex; gap: 10px; align-items: center; }
.m-actions.right { justify-content: flex-end; }
.note-input { flex: 1; height: 34px; font-size: 13px; }
.btn-group { display: flex; gap: 8px; }

/* 评论表格 */
.comment-table { border: 1px solid #e5e7eb; border-radius: 8px; background: #fff; overflow: hidden; }
.c-row { display: flex; padding: 12px 16px; border-bottom: 1px solid #f3f4f6; align-items: flex-start; gap: 12px; }
.c-row.header { background: #f9fafb; font-weight: bold; font-size: 13px; color: #6b7280; border-bottom: 1px solid #e5e7eb; }
.col-user { width: 140px; font-size: 13px; }
.col-content { flex: 1; font-size: 13px; }
.col-status { width: 90px; text-align: center; }
.col-action { width: 160px; display: flex; gap: 6px; justify-content: flex-end; }
.desktop-only { display: block; }
.mobile-only { display: none !important; }
.comment-list-mobile { display: none; }

.u-name { font-weight: bold; color: #111; }
.u-time { font-size: 11px; color: #9ca3af; margin-top: 2px; }
.body-text { color: #374151; line-height: 1.4; }
.ai-reason-mini { font-size: 11px; color: #dc2626; margin-top: 4px; font-weight: 500; }

.badge-mini { font-size: 11px; padding: 3px 8px; border-radius: 4px; background: #f3f4f6; color: #6b7280; }
.badge-mini.flagged { background: #fee2e2; color: #dc2626; }
.badge-mini.public { background: #d1fae5; color: #059669; }

.quick-points { display: flex; gap: 8px; margin-bottom: 12px; flex-wrap: wrap; }
.chip-btn {
  padding: 4px 12px; border-radius: 20px; border: 1px solid #e5e7eb; background: #fff; 
  cursor: pointer; font-size: 13px; font-weight: 500; color: #4b5563; transition: all 0.2s;
}
.chip-btn:hover { border-color: #d1d5db; background: #f3f4f6; }
.chip-btn.active { background: #1f2937; color: #fff; border-color: #1f2937; }

.pa-form { display: flex; gap: 12px; align-items: stretch; }
.input-group { position: relative; width: 100px; flex-shrink: 0; }
.input-prefix { position: absolute; left: 10px; top: 50%; transform: translateY(-50%); font-size: 12px; color: #9ca3af; pointer-events: none; }
.points-num { padding-left: 40px !important; text-align: center; font-weight: bold; }

/* 按钮与输入框通用 */
.btn { 
  padding: 0 16px; height: 38px; border-radius: 8px; font-size: 14px; font-weight: 600; 
  cursor: pointer; border: none; background: #111; color: #fff; transition: all 0.2s; 
  display: inline-flex; align-items: center; justify-content: center;
}
.btn:hover:not(:disabled) { background: #333; transform: translateY(-1px); box-shadow: 0 2px 5px rgba(0,0,0,0.1); }
.btn:active:not(:disabled) { transform: translateY(0); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; background: #666; }
.btn.sm { height: 32px; font-size: 12px; padding: 0 12px; }
.btn.lg { height: 42px; font-size: 14px; }
.btn.success { background: #059669; }
.btn.danger { background: #dc2626; }

.btn-ghost { 
  height: 38px; padding: 0 16px; background: transparent; border: 1px solid #d1d5db; 
  color: #374151; border-radius: 8px; cursor: pointer; transition: all 0.2s;
  display: inline-flex; align-items: center; justify-content: center;
}
.btn-ghost:hover { background: #f3f4f6; border-color: #9ca3af; color: #111; }
.btn-ghost.sm { height: 32px; font-size: 12px; padding: 0 12px; }
.btn-ghost.warn { color: #d97706; border-color: #fbbf24; }
.btn-ghost.danger { color: #dc2626; border-color: #fca5a5; }

.btn-text { background: transparent; border: none; color: #2563eb; cursor: pointer; font-size: 13px; text-decoration: underline; padding: 0; }
.btn-mini { padding: 4px 10px; font-size: 11px; border-radius: 6px; border: none; cursor: pointer; color: #fff; background: #6b7280; font-weight: 600; }
.btn-mini.success { background: #059669; }
.btn-mini.warn { background: #d97706; }
.btn-mini.danger { background: #dc2626; }

.input, .textarea, .select { 
  border: 1px solid #d1d5db; border-radius: 8px; padding: 10px 12px; outline: none; 
  font-size: 14px; width: 100%; transition: all 0.2s; background: #fff;
}
.input.sm, .select.sm { padding: 6px 10px; font-size: 13px; }
.input:focus, .textarea:focus { border-color: #2563eb; box-shadow: 0 0 0 3px rgba(37,99,235,0.1); }
.textarea { resize: vertical; min-height: 80px; }

/* 积分记录 */
.ph-row { display: flex; padding: 10px 16px; border-bottom: 1px solid #f3f4f6; font-size: 13px; }
.ph-row:last-child { border-bottom: none; }
.ph-row.head { background: #f9fafb; font-weight: bold; color: #6b7280; border-bottom: 1px solid #e5e7eb; }
.ph-time { width: 100px; color: #9ca3af; }
.ph-val { width: 70px; font-weight: bold; }
.ph-val.pos { color: #059669; }
.ph-val.neg { color: #dc2626; }
.ph-reason { flex: 1; color: #374151; }
.empty-ph { padding: 24px; text-align: center; color: #9ca3af; font-size: 13px; }

/* Expanded Editor Styles */
.inline-editor.expanded { gap: 12px; }
.editor-row { display: flex; gap: 8px; }
.editor-licenses { display: flex; flex-direction: column; gap: 8px; background: #fff; padding: 10px; border: 1px solid #e5e7eb; border-radius: 6px; }
.lic-group { display: flex; flex-direction: column; gap: 4px; }
.lic-title { font-size: 11px; font-weight: bold; color: #6b7280; text-transform: uppercase; letter-spacing: 0.5px; }
.chk-item { font-size: 12px; display: flex; align-items: center; gap: 6px; color: #374151; cursor: pointer; }
.chk-item:hover { color: #111; }

@media (max-width: 1024px) {
  .content-area { padding: 16px; }
  .filters { flex-wrap: wrap; }
  .two-col-layout { gap: 12px; min-height: 0; height: auto; }
  .col-left { width: 240px; }
}

@media (max-width: 768px) {
  .head-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
    margin-bottom: 14px;
  }
  .h1 { font-size: 20px; }

  .auth-box {
    padding: 16px 0 8px;
    align-items: stretch;
  }
  .form2 {
    flex-direction: column;
    width: 100%;
  }
  .form2 .btn { width: 100%; }

  .panel-layout { display: block; }
  .main-nav {
    width: auto;
    border-right: none;
    border-bottom: 1px solid #e5e7eb;
    padding: 0 0 10px;
    margin-bottom: 12px;
    flex-direction: row;
    gap: 8px;
    overflow-x: auto;
    position: sticky;
    top: 0;
    z-index: 8;
    background: #f9fafb;
  }
  .nav-item {
    white-space: nowrap;
    border-left: none;
    border-bottom: 3px solid transparent;
    border-radius: 8px;
    padding: 10px 12px;
    font-size: 14px;
  }
  .nav-item.active {
    border-left-color: transparent;
    border-bottom-color: #2563eb;
  }
  .content-area { padding: 12px; }

  .sub-tabs {
    gap: 12px;
    overflow-x: auto;
    margin-bottom: 14px;
  }
  .sub-tab {
    white-space: nowrap;
    font-size: 14px;
  }

  .toolbar {
    align-items: stretch;
    gap: 8px;
  }
  .tip { font-size: 12px; }
  .comment-search { width: 100%; }
  .filter-bar { padding: 10px; }
  .filters {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .filters .btn,
  .filters .select,
  .filters .input {
    width: 100%;
  }

  .card-grid { gap: 12px; }
  .manage-card { flex-direction: column; }
  .m-thumb { width: 100%; height: 190px; }
  .m-body { padding: 12px; gap: 10px; }
  .title-row { gap: 8px; align-items: flex-start; }
  .m-title { font-size: 15px; line-height: 1.35; }
  .m-actions {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .m-actions.right { justify-content: flex-start; }
  .m-actions .btn,
  .m-actions .btn-ghost {
    width: 100%;
  }
  .btn-group {
    display: grid;
    gap: 8px;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .note-input {
    min-height: 74px;
    height: auto;
  }
  .editor-row {
    flex-direction: column;
    gap: 8px;
  }
  .btns {
    display: grid;
    gap: 8px;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .desktop-only { display: none !important; }
  .mobile-only { display: inline-flex !important; }
  .comment-list-mobile {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .comment-card {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 10px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .comment-card-top {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 10px;
  }
  .comment-card-actions {
    display: grid;
    gap: 8px;
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
  .comment-card-actions .btn-mini {
    width: 100%;
    min-height: 32px;
  }

  .two-col-layout {
    display: block;
    height: auto;
    min-height: 0;
  }
  .col-left {
    width: 100%;
    border-right: none;
    padding-right: 0;
  }
  .col-left.mobile-hidden { display: none; }
  .creator-list-v {
    max-height: calc(100vh - 300px);
    padding-right: 0;
  }
  .col-right {
    display: none;
    min-height: 0;
  }
  .col-right.mobile-visible { display: block; }
  .panel-header {
    padding: 12px;
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .ph-title { font-size: 16px; }
  .panel-actions {
    width: 100%;
  }
  .panel-actions .btn-ghost {
    flex: 1;
  }
  .edit-form,
  .points-section {
    padding: 12px;
    gap: 14px;
  }
  .divider { margin: 0 12px; }
  .avatar-uploader {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  .au-actions {
    width: 100%;
    align-items: stretch;
  }
  .au-actions .btn-ghost { width: 100%; }
  .form-actions {
    flex-direction: column;
    align-items: stretch;
    margin-top: 4px;
  }
  .form-actions .btn { width: 100%; }
  .pa-form {
    flex-direction: column;
    gap: 8px;
  }
  .input-group { width: 100%; }
  .points-num {
    text-align: left;
    padding-left: 50px !important;
  }
  .ph-scroll-area {
    max-height: none;
  }
  .ph-row {
    padding: 8px 10px;
    gap: 8px;
    align-items: center;
  }
  .ph-time {
    width: 84px;
    flex-shrink: 0;
  }
  .ph-val {
    width: 56px;
    flex-shrink: 0;
  }
}

</style>
