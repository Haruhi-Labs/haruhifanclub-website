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
            <!-- Tabs -->
            <div class="tabs-bar">
                <button @click="activeTab = 'activities'" :class="activeTab === 'activities' ? 'tab-active' : 'tab-inactive'" class="tab-btn">活动管理</button>
                <button @click="activeTab = 'prizes'" :class="activeTab === 'prizes' ? 'tab-active' : 'tab-inactive'" class="tab-btn">奖品管理</button>
                <button @click="activeTab = 'pending'" :class="activeTab === 'pending' ? 'tab-active' : 'tab-inactive'" class="tab-btn tab-btn-relative">待审核 <span v-if="pendingCount > 0" class="pending-badge">{{ pendingCount }}</span></button>
                <button @click="activeTab = 'published'" :class="activeTab === 'published' ? 'tab-active' : 'tab-inactive'" class="tab-btn">已发布内容</button>
                <button @click="activeTab = 'points'" :class="activeTab === 'points' ? 'tab-active' : 'tab-inactive'" class="tab-btn">积分管理</button>
                <button @click="activeTab = 'generator'" :class="activeTab === 'generator' ? 'tab-active' : 'tab-inactive'" class="tab-btn tab-btn-flex">新闻总览生成</button>
            </div>

            <!-- ================= Tab 0: Activity Management [NEW] ================= -->
            <div v-if="activeTab === 'activities'" class="tab-content">
                <div class="tab-header">
                    <h3 class="tab-title">活动列表</h3>
                    <button @click="openActivityModal()" class="btn-primary-sm">
                        <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
                        添加活动
                    </button>
                </div>
                <div class="table-wrapper">
                    <table class="data-table">
                        <thead class="table-head">
                            <tr>
                                <th class="th-sort">排序</th>
                                <th class="th-cell">排序ID</th>
                                <th class="th-cell">封面</th>
                                <th class="th-cell">活动名称</th>
                                <th class="th-cell">状态/类型</th>
                                <th class="th-cell">奖励规则</th>
                                <th class="th-cell th-right">操作</th>
                            </tr>
                        </thead>
                        <tbody class="table-body">
                            <tr 
                                v-for="(act, index) in store.activities" 
                                :key="act.id" 
                                class="table-row"
                                draggable="true"
                                @dragstart="onDragStart($event, index, 'activities')"
                                @dragover.prevent="onDragOver($event, index)"
                                @drop="onDrop($event, index, 'activities')"
                                :class="{ 'dragging': draggedItemIndex === index }"
                            >
                                <td class="td-drag">
                                    <svg class="icon-drag" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16"></path></svg>
                                </td>
                                <td class="td-order">#{{ act.displayOrder }}</td>
                                <td class="td-cell">
                                    <div class="thumb-rect">
                                        <img :src="act.image" class="thumb-img">
                                    </div>
                                </td>
                                <td class="td-cell">
                                    <div class="cell-title">{{ act.title }}</div>
                                    <div class="cell-subtitle">{{ act.intro }}</div>
                                </td>
                                <td class="td-cell td-cell-xs">
                                    <div class="cell-col-gap1">
                                        <span class="cell-status-bold" :class="act.status === 'Activate' ? 'text-success' : 'text-muted'">{{ act.status }}</span>
                                        <span class="cell-type">{{ act.type }}</span>
                                    </div>
                                </td>
                                <td class="td-cell td-cell-xs">
                                    <div class="cell-points">+{{ act.pointsPerAction }} PT</div>
                                    <div class="cell-pool">Pool: {{ act.totalPoints }}</div>
                                </td>
                                <td class="td-actions">
                                    <button @click="openActivityModal(act)" class="link-edit">编辑</button>
                                    <button @click="confirmDeleteActivity(act)" class="link-delete">删除</button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- ================= Tab 1: Prize Management ================= -->
            <div v-else-if="activeTab === 'prizes'" class="tab-content">
                <!-- Keep existing prize management code -->
                <div class="tab-header">
                    <h3 class="tab-title">奖品列表</h3>
                    <button @click="openPrizeModal()" class="btn-primary-sm">
                        <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
                        添加奖品
                    </button>
                </div>
                <div class="table-wrapper">
                    <table class="data-table">
                        <thead class="table-head">
                            <tr>
                                <th class="th-sort">排序</th>
                                <th class="th-cell">排序ID</th>
                                <th class="th-cell">图片</th>
                                <th class="th-cell">名称</th>
                                <th class="th-cell">积分/库存</th>
                                <th class="th-cell">属性</th>
                                <th class="th-cell th-right">操作</th>
                            </tr>
                        </thead>
                        <tbody class="table-body">
                            <tr 
                                v-for="(prize, index) in store.prizes" 
                                :key="prize.id" 
                                class="table-row"
                                draggable="true"
                                @dragstart="onDragStart($event, index, 'prizes')"
                                @dragover.prevent="onDragOver($event, index)"
                                @drop="onDrop($event, index, 'prizes')"
                                :class="{ 'dragging': draggedItemIndex === index }"
                            >
                                <td class="td-drag">
                                    <svg class="icon-drag" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16"></path></svg>
                                </td>
                                <td class="td-order">#{{ prize.displayOrder }}</td>
                                <td class="td-cell">
                                    <div class="thumb-square">
                                        <img :src="prize.image" class="thumb-img">
                                    </div>
                                </td>
                                <td class="td-cell td-bold">{{ prize.name }}</td>
                                <td class="td-cell">
                                    <div class="cell-col">
                                        <span class="cell-mono">{{ prize.points }} PT</span>
                                        <span class="cell-stock" :class="prize.stock > 0 ? 'text-success' : 'text-danger'">Stock: {{ prize.stock }}</span>
                                    </div>
                                </td>
                                <td class="td-cell td-cell-xs">
                                    <div class="attr-tags">
                                        <span class="attr-tag-gray">{{ prize.category }}</span>
                                        <span class="attr-tag-color" :style="{ backgroundColor: prize.color || '#999', textShadow: '0 1px 2px rgba(0,0,0,0.3)' }">{{ prize.rarity }}</span>
                                        <span class="attr-tag-outline">{{ prize.size }}</span>
                                    </div>
                                </td>
                                <td class="td-actions">
                                    <button @click="openPrizeModal(prize)" class="link-edit">编辑</button>
                                    <button @click="confirmDeletePrize(prize)" class="link-delete">删除</button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- ================= Tab 2 & 3: Article Management ================= -->
            <div v-else-if="activeTab === 'pending' || activeTab === 'published'" class="article-tab">
                <table class="data-table">
                    <thead class="article-thead">
                        <tr class="article-thead-row">
                            <th class="ath-id">ID / Type</th>
                            <th class="ath-title">标题 & 摘要</th>
                            <th class="ath-author">作者</th>
                            <th class="ath-date">日期</th>
                            <th class="ath-status">状态</th>
                            <th class="ath-actions">操作</th>
                        </tr>
                    </thead>
                    <tbody class="table-body">
                        <tr v-for="article in currentList" :key="article.id" class="article-row group">
                            <td class="atd-id">
                                <div class="article-id">#{{ article.id }}</div>
                                <span v-if="article.type === 'news'" class="type-badge-news">News</span>
                                <span v-else class="type-badge-article">Article</span>
                            </td>
                            <td class="atd-title">
                                <div class="article-title-text">{{ article.title }}</div>
                                <div class="article-summary-text">{{ article.subtitle || article.summary || '无摘要...' }}</div>
                            </td>
                            <td class="atd-author">
                                <div class="author-name">{{ article.author || '凉宫春日应援团' }}</div>
                                <div class="author-participants" v-if="article.participants?.length">
                                    +{{ article.participants.length }} 位参与者
                                </div>
                            </td>
                            <td class="atd-date">
                                {{ article.date }}
                            </td>
                            <td class="atd-status">
                                <div v-if="article.status === 'pending'" class="status-pending">
                                    <span class="status-dot-pending"></span>
                                    Pending
                                </div>
                                <div v-else class="status-published-col">
                                    <span class="status-published">
                                        <span class="status-dot-published"></span>
                                        Published
                                    </span>
                                    <span v-if="article.isPinned" class="pinned-label">
                                        <svg class="icon-xs" fill="currentColor" viewBox="0 0 20 20"><path d="M5 4a2 2 0 012-2h6a2 2 0 012 2v14l-5-2.5L5 18V4z"/></svg>
                                        PINNED ({{ article.pinOrder }})
                                    </span>
                                </div>
                            </td>
                            <td class="atd-actions-cell">
                                <div class="action-btns">
                                    <button @click="openPreview(article)" class="action-btn-preview" title="预览">
                                        <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>
                                    </button>
                                    <div class="action-divider"></div>
                                    <button v-if="article.status === 'pending'" @click="approveArticle(article)" class="action-btn-approve" title="通过审核">通过</button>
                                    <button @click="editArticle(article.id)" class="action-btn-edit" title="编辑">编辑</button>
                                    <button @click="confirmDelete(article)" class="action-btn-delete" title="删除">删除</button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <!-- ================= Tab 4: Points Management ================= -->
            <div v-else-if="activeTab === 'points'" class="points-tab">
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

            <!-- ================= Tab 5: Generator ================= -->
            <div v-else-if="activeTab === 'generator'" class="generator-tab">
                 <!-- Keep existing generator code -->
                 <div class="generator-layout">
                    <div class="generator-sidebar">
                        <div>
                            <h3 class="generator-config-title">生成配置</h3>
                            <div class="generator-config-box">
                                <div>
                                    <label class="form-label">基准日期 (截止日)</label>
                                    <input type="date" v-model="genConfig.date" class="gen-date-input">
                                </div>
                                <div>
                                    <label class="form-label form-label-mb3">时间尺度</label>
                                    <div class="gen-range-grid">
                                        <button v-for="opt in rangeOptions" :key="opt.value" @click="genConfig.range = opt.value" :class="genConfig.range === opt.value ? 'opt-active' : 'opt-inactive'" class="gen-range-btn">{{ opt.label }}</button>
                                    </div>
                                </div>
                                <div class="gen-match-info">
                                    <span class="gen-match-text">匹配文章数: <b class="gen-match-count">{{ generatedNewsList.length }}</b></span>
                                </div>
                            </div>
                        </div>
                        <button @click="downloadImage" :disabled="isGenerating || generatedNewsList.length === 0" class="gen-export-btn">
                            {{ isGenerating ? '生成中...' : '导出图片 (PNG)' }}
                        </button>
                    </div>
                    <div class="generator-preview-area">
                        <div id="news-poster" class="poster-canvas" :class="posterCanvasClass">
                            <div class="poster-header">
                                <h1 class="poster-title">新闻总览</h1>
                                <div class="poster-header-line"></div>
                                <p class="poster-date-range">{{ dateRangeDisplay }}</p>
                                <p class="poster-edition">{{ posterEditionLabel }}</p>
                            </div>
                            <div class="poster-body" :class="posterBodyClass">
                                <div v-for="item in generatedNewsList" :key="item.id" class="poster-item" :class="{ 'poster-item--dense': isDenseRange }">
                                    <div class="poster-item-dot"></div>
                                    <div class="poster-item-date">{{ item.date }}</div>
                                    <h3 class="poster-item-title" :class="{ 'poster-item-title--dense': isDenseRange }">{{ item.title }}</h3>
                                    <p class="poster-item-summary" :class="{ 'poster-item-summary--dense': isDenseRange }">{{ getPosterSummary(item) }}</p>
                                </div>
                            </div>
                            <div class="poster-footer">
                                <div class="poster-footer-inner">
                                    <div><p class="poster-footer-brand">Haruyuki.cn</p></div>
                                </div>
                            </div>
                        </div>
                    </div>
                 </div>
            </div>

        </div>
    </div>

    <Teleport to="body">
      <!-- ================= Activity Edit/Add Modal [NEW] ================= -->
      <Transition name="fade">
          <div v-if="showActivityModal" class="modal-overlay">
              <div class="modal-activity">
                  <h2 class="modal-title">{{ isEditingActivity ? '编辑活动' : '新增活动' }}</h2>
                  <div class="modal-form-space">
                      <div class="form-grid-2col">
                          <div>
                              <label class="form-label">活动标题</label>
                              <input v-model="activityForm.title" type="text" class="form-input form-input-bold">
                          </div>
                          <div>
                              <label class="form-label">类型 (e.g. VIDEO, OFFLINE)</label>
                              <input v-model="activityForm.type" type="text" class="form-input form-input-upper-mono">
                          </div>
                      </div>
                      <div>
                          <label class="form-label">简短介绍</label>
                          <input v-model="activityForm.intro" type="text" class="form-input">
                      </div>
                      <div>
                          <label class="form-label">详细描述 (支持 Markdown 粗体 **text** 和 URL)</label>
                          <textarea v-model="activityForm.detail" rows="6" class="form-textarea"></textarea>
                      </div>
                      <div>
                          <label class="form-label">活动封面图 (上传后自动转Base64)</label>
                          <input type="file" accept="image/*" @change="onActivityImageSelect" class="form-file-input">
                          <div v-if="activityForm.image" class="activity-image-preview">
                              <img :src="activityForm.image" class="thumb-img">
                          </div>
                      </div>
                      <div class="form-grid-3col">
                          <div>
                              <label class="form-label">总奖池积分</label>
                              <input v-model.number="activityForm.totalPoints" type="number" class="form-input form-input-mono">
                          </div>
                          <div>
                              <label class="form-label">单次奖励积分</label>
                              <input v-model.number="activityForm.pointsPerAction" type="number" class="form-input form-input-mono">
                          </div>
                          <div>
                              <label class="form-label">动作名称 (e.g. 参与瓜分)</label>
                              <input v-model="activityForm.actionName" type="text" class="form-input">
                          </div>
                      </div>
                      <div>
                          <label class="form-label">状态</label>
                          <select v-model="activityForm.status" class="form-select">
                              <option value="Activate">Activate (进行中)</option>
                              <option value="Ended">Ended (已结束)</option>
                              <option value="Coming">Coming (即将开始)</option>
                          </select>
                      </div>
                      <div class="modal-footer-btns">
                          <button @click="showActivityModal = false" class="btn-cancel">取消</button>
                          <button @click="submitActivity" :disabled="isSubmitting" class="btn-submit">
                              {{ isSubmitting ? '保存中...' : '保存活动' }}
                          </button>
                      </div>
                  </div>
              </div>
          </div>
      </Transition>

      <!-- ================= Prize Edit/Add Modal ================= -->
      <Transition name="fade">
          <div v-if="showPrizeModal" class="modal-overlay">
              <div class="modal-prize">
                  <!-- Left: Image Uploader & Cropper -->
                  <div class="prize-modal-left">
                      <div v-if="cropperImage" class="cropper-area" 
                           ref="cropperContainer"
                           @mousedown="startDrag" 
                           @touchstart.prevent="startDrag"
                           @wheel.prevent="handleWheel"
                      >
                          <img :src="cropperImage" ref="imageElement" class="cropper-image" :style="{ transform: `translate(${imgState.x}px, ${imgState.y}px) scale(${imgState.scale})` }">
                          <div class="cropper-mask" :style="maskStyle"></div>
                          <div class="cropper-frame" :style="cropFrameStyle"></div>
                      </div>
                      <div v-else class="prize-upload-area">
                          <div class="prize-upload-placeholder">
                              <img v-if="prizeForm.image" :src="prizeForm.image" class="thumb-img">
                              <span v-else class="prize-upload-plus">+</span>
                          </div>
                          <p class="prize-upload-hint">先选择右侧的 Size，再上传图片进行裁切</p>
                          <label class="btn-upload">
                              上传图片 <input type="file" accept="image/*" class="hidden-input" @change="onFileSelect">
                          </label>
                      </div>
                      <div v-if="cropperImage" class="cropper-actions">
                          <button @click="confirmCrop" class="btn-crop-confirm">确认裁切</button>
                          <button @click="cancelCrop" class="btn-crop-cancel">取消</button>
                      </div>
                  </div>
                  <!-- Right: Form Fields -->
                  <div class="prize-modal-right">
                      <h2 class="modal-title">{{ isEditing ? '编辑奖品' : '新增奖品' }}</h2>
                      <div class="modal-form-space">
                          <div>
                              <label class="form-label">名称</label>
                              <input v-model="prizeForm.name" type="text" class="form-input form-input-bold">
                          </div>
                          <div>
                              <label class="form-label">描述</label>
                              <textarea v-model="prizeForm.description" rows="2" class="form-textarea-sm"></textarea>
                          </div>
                          <div class="form-grid-2col-equal">
                              <div>
                                  <label class="form-label">积分</label>
                                  <input v-model.number="prizeForm.points" type="number" class="form-input form-input-mono">
                              </div>
                              <div>
                                  <label class="form-label">库存</label>
                                  <input v-model.number="prizeForm.stock" type="number" class="form-input form-input-mono">
                              </div>
                          </div>
                          <div>
                              <label class="form-label">尺寸 (影响裁切比例)</label>
                              <div class="size-grid">
                                  <button v-for="s in sizeOptions" :key="s.value" @click="changeSize(s.value)" :class="prizeForm.size === s.value ? 'opt-active' : 'opt-inactive'" class="size-btn">{{ s.label }}</button>
                              </div>
                          </div>
                          <div class="form-grid-2col-equal">
                              <div>
                                  <label class="form-label">分类</label>
                                  <select v-model="prizeForm.category" class="form-select">
                                      <option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option>
                                  </select>
                              </div>
                              <div>
                                  <label class="form-label">稀有度</label>
                                  <select v-model="prizeForm.rarity" class="form-select">
                                      <option>UR</option><option>SSR</option><option>SR</option><option>Rare</option><option>Uncommon</option><option>Common</option>
                                  </select>
                              </div>
                          </div>
                          <div>
                              <label class="form-label">主题色</label>
                              <div class="color-picker-row">
                                  <input v-model="prizeForm.color" type="color" class="color-picker">
                                  <input v-model="prizeForm.color" type="text" class="form-input form-input-mono form-input-upper">
                              </div>
                          </div>
                          <div class="modal-footer-btns-sm">
                              <button @click="showPrizeModal = false" class="btn-cancel">取消</button>
                              <button @click="submitPrize" :disabled="isSubmitting" class="btn-submit">{{ isSubmitting ? '提交中...' : '保存' }}</button>
                          </div>
                      </div>
                  </div>
              </div>
          </div>
      </Transition>

      <!-- ================= Preview Overlay (全功能预览) ================= -->
      <Transition name="fade">
          <div v-if="previewArticle" class="preview-overlay animate-fade-in-fast">
            <!-- Preview Header -->
            <div class="preview-header">
                <div class="preview-header-left">
                    <div class="preview-header-info">
                        <span class="preview-mode-label">Preview Mode</span>
                        <h2 class="preview-title serif-font">{{ previewArticle.title }}</h2>
                    </div>
                    
                    <!-- View Switcher -->
                    <div class="preview-switcher">
                        <button 
                            @click="previewMode = 'card'"
                            :class="previewMode === 'card' ? 'preview-tab-active' : 'preview-tab-inactive'"
                            class="preview-tab-btn"
                        >
                            NewsCard
                        </button>
                        <button 
                            @click="previewMode = 'modal'"
                            :class="previewMode === 'modal' ? 'preview-tab-active' : 'preview-tab-inactive'"
                            class="preview-tab-btn"
                        >
                            弹窗详情
                        </button>
                        <button 
                            @click="previewMode = 'page'"
                            :class="previewMode === 'page' ? 'preview-tab-active' : 'preview-tab-inactive'"
                            class="preview-tab-btn"
                        >
                            正文阅读页
                        </button>
                    </div>
                </div>

                <div class="preview-header-right">
                    <span v-if="previewArticle.status === 'pending'" class="preview-pending-badge">当前状态: 待审核</span>
                    <button @click="closePreview" class="preview-close-btn">
                        <svg class="icon-md" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
                    </button>
                </div>
            </div>

            <!-- Preview Body -->
            <div class="preview-body">
                
                <!-- 1. Card Mode -->
                <div v-if="previewMode === 'card'" class="preview-card-mode">
                    <!-- 模拟 Masonry 列宽 -->
                    <div class="preview-card-wrap">
                        <NewsCard :article="previewArticle" class="preview-card-component" />
                        <p class="preview-card-caption">Mobile Card View (Masonry)</p>
                    </div>
                </div>

                <!-- 2. Modal Mode (复刻 DetailModal.vue) -->
                <div v-else-if="previewMode === 'modal'" class="preview-modal-mode">
                    <div class="preview-modal-container">
                        <!-- Modal Header -->
                        <div class="preview-modal-header">
                            <div class="preview-modal-header-label">Preview Mode</div>
                            <div class="preview-modal-close-placeholder"><svg class="icon-md" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg></div>
                        </div>
                        <div class="preview-modal-body">
                            <div class="preview-modal-meta">
                                <div class="preview-modal-tags">
                                    <span v-for="tag in (previewArticle.tags || [])" :key="tag" class="tag-filled">{{ tag }}</span>
                                    <span v-if="previewArticle.type === 'news'" class="tag-outline">News</span>
                                </div>
                                <h2 class="preview-modal-article-title serif-font">{{ previewArticle.title }}</h2>
                                <div class="preview-modal-info-row">
                                    <span>{{ previewArticle.date }}</span>
                                    <span v-if="previewArticle.type !== 'news'" class="preview-modal-author">By {{ previewArticle.author || '凉宫春日应援团' }}</span>
                                </div>
                                <!-- Participants (News) -->
                                <div v-if="previewArticle.type === 'news' && previewArticle.participants?.length" class="preview-modal-participants">
                                    <p class="participants-label">PARTICIPANTS:</p>
                                    <div class="participants-list">
                                        <div v-for="(p, idx) in previewArticle.participants" :key="idx">
                                            <span class="participant-name">{{ p.name }}</span>
                                            <span class="participant-role"> — {{ p.role }} <span v-if="p.project">({{ p.project }})</span></span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div v-if="previewArticle.image" class="preview-modal-image">
                                <img :src="previewArticle.image" class="preview-modal-img">
                            </div>

                            <!-- Renderer (带截断) -->
                            <div class="content-renderer prose prose-lg preview-modal-content">
                                <div v-for="(block, index) in modalBlocks" :key="index">
                                    <p v-if="block.type === 'paragraph'" v-html="formatModalParagraph(block.text)"></p>
                                    <h3 v-else-if="block.type === 'heading'">{{ block.text }}</h3>
                                    <div v-else-if="block.type === 'math'" class="math-block-modal">$$ {{ block.expression }} $$</div>
                                    <div v-else-if="block.type === 'image'" class="image-block-modal"><img :src="block.src" class="image-block-modal-img"></div>
                                </div>
                                <p v-if="isModalTruncated" class="truncation-indicator">......</p>
                            </div>

                            <div class="preview-modal-footer-actions">
                                <button v-if="isModalTruncated" class="btn-read-full">
                                    <span>完整阅读</span>
                                    <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path></svg>
                                </button>
                                <button v-else class="btn-open-page">
                                    <span>在新页面打开</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- 3. Page Mode (复刻 BlogDetailView.vue) -->
                <div v-else-if="previewMode === 'page'" class="preview-page-mode animate-fade-in">
                    <!-- Header -->
                    <div class="page-hero">
                         <!-- Cover -->
                         <div v-if="previewArticle.image" class="page-hero-cover">
                            <img :src="previewArticle.originalImage || previewArticle.image" class="page-hero-img">
                            <div class="page-hero-gradient-main"></div>
                            <div class="page-hero-overlay"></div>
                         </div>
                         <!-- No Cover Fallback -->
                         <div v-else class="page-hero-fallback">
                             <div class="page-hero-pattern" style="filter: grayscale(100%);">
                                 <span v-for="n in 40" :key="n" class="page-hero-pattern-text">{{ n % 2 === 0 ? previewArticle.title : 'HIBIKILOGY' }}</span>
                             </div>
                             <div class="page-hero-gradient-fallback"></div>
                         </div>

                         <!-- Header Content -->
                         <div class="page-hero-content">
                             <div class="page-tags-desktop" style="left: 105px; top: 283px;">
                                 <div class="page-tags-wrap">
                                     <span v-for="tag in previewArticle.tags" :key="tag" class="page-tag-item">
                                         <span class="page-tag-hash">#</span>
                                         <span class="page-tag-text">{{ tag }}</span>
                                     </span>
                                 </div>
                                 <span v-if="previewArticle.type === 'news'" class="page-news-badge">NEWS</span>
                             </div>

                             <div class="page-title-desktop" style="left: 105px; top: 329px; width: 1200px;">
                                 <div>
                                     <h1 class="page-main-title text-shadow">{{ previewArticle.title }}</h1>
                                     <p v-if="previewArticle.subtitle" class="page-subtitle text-shadow">{{ previewArticle.subtitle }}</p>
                                 </div>
                                 <div class="page-author-row text-shadow">
                                     <div class="page-author-info">
                                         <span class="page-author-label">作者</span>
                                         <span class="page-author-name">{{ previewArticle.author || '凉宫春日应援团' }}</span>
                                     </div>
                                     <div class="page-date-info">发表于 {{ previewArticle.date }}</div>
                                 </div>
                             </div>

                             <!-- Mobile Header -->
                             <div class="page-mobile-header">
                                 <div class="page-mobile-tags">
                                     <span v-for="tag in previewArticle.tags" :key="tag" class="page-mobile-tag">#{{ tag }}</span>
                                 </div>
                                 <h1 class="page-mobile-title text-shadow">{{ previewArticle.title }}</h1>
                                 <p class="page-mobile-subtitle text-shadow">{{ previewArticle.subtitle }}</p>
                             </div>
                         </div>
                    </div>

                    <!-- Content Body -->
                     <div class="page-content-container">
                        <div class="page-content-grid">
                            <!-- Main -->
                            <article class="page-article content-renderer prose prose-lg">
                                <div v-if="previewArticle.type === 'news' && previewArticle.participants?.length" class="page-participants-box">
                                    <h2 class="page-participants-heading">参与者信息</h2>
                                    <ul class="page-participants-list">
                                        <li v-for="(p, idx) in previewArticle.participants" :key="idx" class="page-participant-item">
                                            <span class="participant-name">{{ p.name }}</span>
                                            <span class="page-participant-detail">{{ p.role }} <span v-if="p.project">· {{ p.project }}</span></span>
                                        </li>
                                    </ul>
                                </div>

                                <div v-for="(block, index) in previewArticle.content" :key="index" class="page-content-block">
                                    <p v-if="block.type === 'paragraph'" class="article-paragraph page-paragraph" v-html="formatPageParagraph(block.text)"></p>
                                    <h3 v-else-if="block.type === 'heading'" :id="'heading-' + index" class="page-heading">{{ block.text }}</h3>
                                    <div v-else-if="block.type === 'math'" class="page-math-block">
                                        <span class="page-math-expr">$$ {{ block.expression }} $$</span>
                                        <p v-if="block.caption" class="page-math-caption">{{ block.caption }}</p>
                                    </div>
                                    <figure v-else-if="block.type === 'image'" class="page-image-figure">
                                        <div class="page-image-wrap">
                                            <img :src="block.src" class="page-image">
                                        </div>
                                        <figcaption v-if="block.caption" class="page-image-caption">
                                            <span class="caption-dot"></span>{{ block.caption }}
                                        </figcaption>
                                    </figure>
                                </div>
                                <div class="page-fin">
                                    <span class="fin-line"></span><span class="fin-text">Fin</span><span class="fin-line"></span>
                                </div>
                            </article>
                            <!-- Sidebar -->
                            <aside class="page-sidebar">
                                 <div class="page-sidebar-inner">
                                     <h4 class="page-sidebar-title">Catalog</h4>
                                     <nav v-if="toc.length > 0">
                                         <ul class="page-toc-list">
                                             <li v-for="(item, idx) in toc" :key="idx" class="page-toc-item">
                                                 <span class="page-toc-dot"></span>
                                                 <a :href="'#heading-' + item.index" @click.prevent="scrollToHeading(item.index)" class="page-toc-link">{{ item.text }}</a>
                                             </li>
                                         </ul>
                                     </nav>
                                     <p v-else class="page-toc-empty">本文无小标题</p>
                                     <div class="page-sidebar-stats">
                                         <div class="page-sidebar-stats-inner">
                                             <p>字数统计: <span class="page-stat-val">{{ wordCount }}</span> 字</p>
                                             <p>预计阅读: <span class="page-stat-val">{{ Math.ceil(wordCount / 400) }}</span> 分钟</p>
                                         </div>
                                     </div>
                                 </div>
                            </aside>
                        </div>
                    </div>
                </div>
            </div>
          </div>
      </Transition>
    </Teleport>

  </div>
</template>

<script setup>
import { ref, computed, onMounted, reactive, watch, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { toBlob } from 'html-to-image';
import { useMainStore } from '@/stores/main';
import NewsCard from '@/components/NewsCard.vue';
import { createApiClient } from '@haruhi/api-client';

// 统一后端：积分用户列表也走 /api/news + 自动注入 JWT
const newsApi = createApiClient('/api/news');

const store = useMainStore();
const router = useRouter();

const username = ref('');
const password = ref('');
const loginMsg = ref('');
const loginLoading = ref(false);
const activeTab = ref('activities'); // Default to activities

// Points Management State
const pointsSearchId = ref('');
const currentPointsUser = ref(null);
const pointsLoading = ref(false);
const pointsUpdating = ref(false);
const pointChangeAmount = ref('');
const pointChangeReason = ref('');
const pointsUserList = ref([]); // List of users with points

// Fuzzy Search State (New)
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

// Preview State
const previewArticle = ref(null);
const previewMode = ref('card');

// Generator State
const isGenerating = ref(false);
const genConfig = reactive({
    date: new Date().toISOString().split('T')[0], // Default today
    range: 'week', // day, week, half-month, month, quarter, half-year
    layout: 'classic', // [新增]
});

const rangeOptions = [
    { label: '单日', value: 'day' },
    { label: '周报', value: 'week' },
    { label: '半月刊', value: 'half-month' },
    { label: '月刊', value: 'month' },
    { label: '季度刊', value: 'quarter' },
    { label: '半年刊', value: 'half-year' },
];

const layoutOptions = [
    { label: '经典', value: 'classic' },
    { label: '紧凑', value: 'compact' },
    { label: '图文', value: 'visual' },
    { label: '极简', value: 'minimal' },
    { label: '详细', value: 'detailed' }, 
];

// Activity State
const showActivityModal = ref(false);
const isEditingActivity = ref(false);
const activityForm = reactive({
    id: null,
    title: '',
    intro: '',
    detail: '',
    image: '',
    totalPoints: 0,
    actionName: '',
    pointsPerAction: 0,
    status: 'Activate',
    type: 'VIDEO'
});

// Prize State
const showPrizeModal = ref(false);
const isEditing = ref(false);
const isSubmitting = ref(false);
const prizeForm = reactive({ id: null, name: '', description: '', points: 0, stock: 0, category: 'anime', rarity: 'SSR', color: '#ECC94B', size: 'normal', image: '' });
const sizeOptions = [{ label: 'Normal', value: 'normal', ratio: 1, ratioText: '1:1' }, { label: 'Large', value: 'large', ratio: 1, ratioText: '1:1' }, { label: 'Wide', value: 'wide', ratio: 2, ratioText: '2:1' }, { label: 'Tall', value: 'tall', ratio: 0.5, ratioText: '1:2' }];
const categories = [{ id: 'anime', name: '动漫周边' }, { id: 'paint', name: '绘画用具' }, { id: 'computer', name: '电脑配件' }, { id: 'book', name: '书籍' }, { id: 'peripheral', name: '外设' }, { id: 'instrument', name: '乐器' }, { id: 'game', name: '游戏' }];

// Image Cropper State
const cropperImage = ref(null);
const cropperContainer = ref(null);
const imageElement = ref(null);
const imgState = reactive({ x: 0, y: 0, scale: 1 });
const dragStart = { x: 0, y: 0, imgX: 0, imgY: 0 };
const isDragging = ref(false);

// Login（统一 JWT：用户名 + 密码，校验 news 权限）
const login = async () => {
    if (!username.value || !password.value) return;
    loginLoading.value = true;
    loginMsg.value = '';
    try {
        const ok = await store.loginAdmin(username.value.trim(), password.value);
        if (ok) {
            password.value = '';
            store.fetchAdminArticles();
            store.fetchPrizes();
            store.fetchActivities();
            fetchAllPointsUsers();
        } else {
            loginMsg.value = '用户名或密码错误，或该账号无新闻站管理权限';
        }
    } catch (e) {
        loginMsg.value = e?.message || '登录失败';
    } finally {
        loginLoading.value = false;
    }
};

const handleLogout = () => {
    store.logoutAdmin();
    // Reset points state
    currentPointsUser.value = null;
    pointsSearchId.value = '';
    pointsUserList.value = [];
};

// Data
const pendingCount = computed(() => store.adminArticles.filter(a => a.status === 'pending').length);
const publishedCount = computed(() => store.adminArticles.filter(a => (!a.status || a.status === 'published')).length);

const currentList = computed(() => {
    if (activeTab.value === 'pending') {
        return store.adminArticles.filter(a => a.status === 'pending');
    } else {
        return store.adminArticles.filter(a => !a.status || a.status === 'published');
    }
});

// ================= Generator Logic =================
const applyRangeOffset = (startDate, range) => {
    if (range === 'week') startDate.setDate(startDate.getDate() - 6);
    else if (range === 'half-month') startDate.setDate(startDate.getDate() - 14);
    else if (range === 'month') startDate.setMonth(startDate.getMonth() - 1);
    else if (range === 'quarter') startDate.setMonth(startDate.getMonth() - 3);
    else if (range === 'half-year') startDate.setMonth(startDate.getMonth() - 6);
};

const rangeLabelMap = {
    day: '单日快报',
    week: '周报',
    'half-month': '半月刊',
    month: '月刊',
    quarter: '季度刊',
    'half-year': '半年刊',
};

const isDenseRange = computed(() => genConfig.range === 'quarter' || genConfig.range === 'half-year');
const posterCanvasClass = computed(() => ({
    'poster-canvas--dense': isDenseRange.value,
    'poster-canvas--quarter': genConfig.range === 'quarter',
    'poster-canvas--half-year': genConfig.range === 'half-year',
}));
const posterBodyClass = computed(() => ({
    'poster-body--dense': isDenseRange.value,
    'poster-body--dense-quarter': genConfig.range === 'quarter',
    'poster-body--dense-half-year': genConfig.range === 'half-year',
}));
const posterEditionLabel = computed(() => rangeLabelMap[genConfig.range] || '周报');
const getPosterSummary = (item) => {
    const text = item.summary || item.preview || '';
    if (!text) return '';
    if (!isDenseRange.value) return text;
    const maxLen = genConfig.range === 'half-year' ? 34 : 54;
    return text.length > maxLen ? `${text.slice(0, maxLen)}...` : text;
};

const generatedNewsList = computed(() => {
    // 1. 确定基准日期 (结束日期)
    const targetDateStr = genConfig.date; 
    const targetDate = new Date(targetDateStr);
    targetDate.setHours(23, 59, 59, 999);
    
    // 2. 确定开始日期
    const startDate = new Date(targetDate);
    startDate.setHours(0, 0, 0, 0);

    applyRangeOffset(startDate, genConfig.range);
    
    // 3. 数据源
    const source = store.allArticles;
    const result = source.filter(article => {
        if (!article.date) return false;
        if (article.status && article.status !== 'published') return false;

        let rawDate = article.date.toString();
        let dateStr = rawDate
            .replace(/\./g, '-')
            .replace(/年/g, '-')
            .replace(/月/g, '-')
            .replace(/日/g, '');

        const articleDate = new Date(dateStr);
        if (isNaN(articleDate.getTime())) return false;
        
        articleDate.setHours(12, 0, 0, 0);
        const inRange = articleDate >= startDate && articleDate <= targetDate;
        return inRange;
    }).sort((a, b) => {
        const parse = (d) => {
             const s = d.toString()
                .replace(/\./g, '-')
                .replace(/年/g, '-')
                .replace(/月/g, '-')
                .replace(/日/g, '');
             return new Date(s);
        };
        const dateA = parse(a.date);
        const dateB = parse(b.date);
        return dateB - dateA;
    });

    return result;
});

const getSummaryContent = (item) => {
    if (genConfig.layout === 'compact' || genConfig.layout === 'minimal') return '';
    let text = '';
    if (genConfig.layout === 'detailed') {
       text = item.summary || item.preview || '';
    } else {
       text = item.subtitle || item.summary || item.preview || '';
    }
    if (!text) return '';
    if (genConfig.layout !== 'detailed' && !item.subtitle && !item.summary && text.length > 60) {
        return text.slice(0, 60) + '...';
    }
    return text;
};

const dateRangeText = computed(() => {
    const end = new Date(genConfig.date).toLocaleDateString();
    const d = new Date(genConfig.date);
    applyRangeOffset(d, genConfig.range);
    const start = d.toLocaleDateString();
    return genConfig.range === 'day' ? end : `${start} - ${end}`;
});

const dateRangeDisplay = computed(() => {
    const formatDate = (dateObj) => {
        const y = dateObj.getFullYear();
        const m = String(dateObj.getMonth() + 1).padStart(2, '0');
        const d = String(dateObj.getDate()).padStart(2, '0');
        return `${y}.${m}.${d}`;
    };
    const targetDateStr = genConfig.date; 
    const end = new Date(targetDateStr);
    const start = new Date(targetDateStr);

    if (genConfig.range === 'day') return formatDate(end);
    applyRangeOffset(start, genConfig.range);

    return `${formatDate(start)} - ${formatDate(end)}`;
});

const getExportParams = (el) => {
    const rect = el.getBoundingClientRect();
    const width = Math.max(1, rect.width);
    const height = Math.max(1, rect.height);
    const area = width * height;

    // Browser canvas limits are usually the bottleneck, not raw CPU power.
    const preferredRatio = isDenseRange.value ? 1.6 : 2;
    const maxPixels = 16_000_000; // keep output around <=16MP
    const ratioCap = Math.sqrt(maxPixels / area);
    const safeRatio = Math.max(0.9, Math.min(preferredRatio, ratioCap));

    const timeoutMs = Math.min(
        180000,
        Math.max(30000, Math.round(10000 + area * safeRatio * safeRatio / 18000))
    );

    return {
        pixelRatio: Number(safeRatio.toFixed(2)),
        timeoutMs,
    };
};

const exportPosterBlob = async (el, pixelRatio, timeoutMs) => {
    const exportTask = toBlob(el, {
        backgroundColor: '#ffffff',
        pixelRatio,
        // Avoid forced cache busting; it can trigger redundant resource fetches.
        cacheBust: false,
    });
    const timeoutTask = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('export-timeout')), timeoutMs)
    );

    const blob = await Promise.race([exportTask, timeoutTask]);
    if (!blob) throw new Error('export-empty-blob');
    return blob;
};

const downloadImage = async () => {
    const el = document.getElementById('news-poster');
    if (!el) {
        return;
    }

    isGenerating.value = true;
    try {
        // Ensure web fonts are ready before rasterization for best fidelity.
        if (document.fonts?.ready) {
            await document.fonts.ready;
        }

        // Wait two frames to ensure latest layout/class changes are painted.
        await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));

        const { pixelRatio, timeoutMs } = getExportParams(el);
        let blob;

        try {
            blob = await exportPosterBlob(el, pixelRatio, timeoutMs);
        } catch (err) {
            // Auto-retry once with lower resolution for very large posters.
            const retryRatio = Math.max(0.85, Number((pixelRatio * 0.75).toFixed(2)));
            blob = await exportPosterBlob(el, retryRatio, timeoutMs + 30000);
        }

        const link = document.createElement('a');
        link.download = `hibiki-news-${genConfig.date}-${genConfig.range}.png`;
        const url = URL.createObjectURL(blob);
        link.href = url;
        link.click();
        setTimeout(() => URL.revokeObjectURL(url), 3000);
    } catch (err) {
        console.error('Generation failed', err);
        if (err?.message === 'export-timeout') {
            alert('导出超时：内容过多或画布过大，建议缩小时间范围后重试。');
        } else {
            alert('生成失败，请检查控制台');
        }
    } finally {
        isGenerating.value = false;
    }
};

// ================= Activity Actions [NEW] =================

const openActivityModal = (activity = null) => {
    if (activity) {
        isEditingActivity.value = true;
        Object.assign(activityForm, JSON.parse(JSON.stringify(activity)));
    } else {
        isEditingActivity.value = false;
        Object.assign(activityForm, {
            id: null, title: '', intro: '', detail: '', image: '', totalPoints: 0,
            actionName: '', pointsPerAction: 0, status: 'Activate', type: 'VIDEO'
        });
    }
    showActivityModal.value = true;
};

const onActivityImageSelect = (e) => {
    const file = e.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (evt) => {
        activityForm.image = evt.target.result;
    };
    reader.readAsDataURL(file);
};

const submitActivity = async () => {
    isSubmitting.value = true;
    try {
        if (isEditingActivity.value) {
            await store.updateActivity(activityForm.id, activityForm);
        } else {
            await store.addActivity(activityForm);
        }
        showActivityModal.value = false;
    } catch (e) {
        alert('操作失败');
    } finally {
        isSubmitting.value = false;
    }
};

const confirmDeleteActivity = async (activity) => {
    if (confirm(`确认删除活动 "${activity.title}" 吗？`)) {
        await store.deleteActivity(activity.id);
    }
};

// ================= Prize Actions =================

const openPrizeModal = (prize = null) => {
    cropperImage.value = null;
    if (prize) {
        isEditing.value = true;
        Object.assign(prizeForm, JSON.parse(JSON.stringify(prize)));
    } else {
        isEditing.value = false;
        Object.assign(prizeForm, {
            id: null, name: '', description: '', points: 0, stock: 1, category: 'anime',
            rarity: 'SSR', color: '#ECC94B', size: 'normal', image: ''
        });
    }
    showPrizeModal.value = true;
};

const confirmDeletePrize = async (prize) => {
    if (confirm(`确认删除 "${prize.name}" 吗？`)) {
        await store.deletePrize(prize.id);
    }
};

const submitPrize = async () => {
    isSubmitting.value = true;
    try {
        if (isEditing.value) {
            await store.updatePrize(prizeForm.id, prizeForm);
        } else {
            await store.addPrize(prizeForm);
        }
        showPrizeModal.value = false;
    } catch (e) {
        alert('操作失败');
    } finally {
        isSubmitting.value = false;
    }
};

// ================= Image Cropper Logic =================

const getAspectRatio = () => {
    const opt = sizeOptions.find(s => s.value === prizeForm.size);
    return opt ? opt.ratio : 1;
};

const onFileSelect = (e) => {
    const file = e.target.files[0];
    if (!file) return;
    const url = URL.createObjectURL(file);
    cropperImage.value = url;
    imgState.x = 0;
    imgState.y = 0;
    imgState.scale = 1;
    nextTick(() => {
        fitImage();
    });
};

const changeSize = (newSize) => {
    prizeForm.size = newSize;
    if (cropperImage.value) {
        fitImage();
    }
};

const fitImage = () => {
    imgState.x = 0;
    imgState.y = 0;
    imgState.scale = 0.5; 
};

const cropFrameStyle = computed(() => {
    if (!cropperContainer.value) return {};
    const containerW = cropperContainer.value.clientWidth || 300;
    const containerH = cropperContainer.value.clientHeight || 300;
    const ratio = getAspectRatio();
    
    const margin = 40;
    const maxW = containerW - margin;
    const maxH = containerH - margin;
    let w, h;

    if (maxW / maxH > ratio) {
        h = maxH;
        w = h * ratio;
    } else {
        w = maxW;
        h = w / ratio;
    }

    return {
        width: `${w}px`,
        height: `${h}px`,
        left: `${(containerW - w) / 2}px`,
        top: `${(containerH - h) / 2}px`
    };
});

const startDrag = (e) => {
    isDragging.value = true;
    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const clientY = e.touches ? e.touches[0].clientY : e.clientY;
    dragStart.x = clientX;
    dragStart.y = clientY;
    dragStart.imgX = imgState.x;
    dragStart.imgY = imgState.y;
    
    window.addEventListener('mousemove', onDrag);
    window.addEventListener('mouseup', stopDrag);
    window.addEventListener('touchmove', onDrag, { passive: false });
    window.addEventListener('touchend', stopDrag);
};

const onDrag = (e) => {
    if (!isDragging.value) return;
    e.preventDefault();
    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const clientY = e.touches ? e.touches[0].clientY : e.clientY;
    const dx = clientX - dragStart.x;
    const dy = clientY - dragStart.y;
    imgState.x = dragStart.imgX + dx;
    imgState.y = dragStart.imgY + dy;
};

const stopDrag = () => {
    isDragging.value = false;
    window.removeEventListener('mousemove', onDrag);
    window.removeEventListener('mouseup', stopDrag);
    window.removeEventListener('touchmove', onDrag);
    window.removeEventListener('touchend', stopDrag);
};

const handleWheel = (e) => {
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    imgState.scale *= delta;
};

const confirmCrop = () => {
    if (!imageElement.value) return;
    
    const containerW = cropperContainer.value.clientWidth;
    const containerH = cropperContainer.value.clientHeight;
    const ratio = getAspectRatio();
    
    const margin = 40;
    const maxW = containerW - margin;
    const maxH = containerH - margin;
    let cropW, cropH;
    if (maxW / maxH > ratio) {
        cropH = maxH;
        cropW = cropH * ratio;
    } else {
        cropW = maxW;
        cropH = cropW / ratio;
    }
    
    const cropLeft = (containerW - cropW) / 2;
    const cropTop = (containerH - cropH) / 2;

    const outputWidth = 800;
    const outputHeight = outputWidth / ratio;
    
    const canvas = document.createElement('canvas');
    canvas.width = outputWidth;
    canvas.height = outputHeight;
    const ctx = canvas.getContext('2d');
    
    const scaleFactor = outputWidth / cropW;
    
    const imgObj = imageElement.value;
    const renderW = imgObj.naturalWidth * imgState.scale;
    const renderH = imgObj.naturalHeight * imgState.scale;
    
    const containerCenterX = containerW / 2;
    const containerCenterY = containerH / 2;
    
    const imgCenterX = containerCenterX + imgState.x;
    const imgCenterY = containerCenterY + imgState.y;
    
    const imgLeftScreen = imgCenterX - renderW / 2;
    const imgTopScreen = imgCenterY - renderH / 2;
    
    const drawX = (imgLeftScreen - cropLeft) * scaleFactor;
    const drawY = (imgTopScreen - cropTop) * scaleFactor;
    const drawW = renderW * scaleFactor;
    const drawH = renderH * scaleFactor;
    
    ctx.drawImage(imgObj, drawX, drawY, drawW, drawH);
    
    const base64 = canvas.toDataURL('image/webp', 0.8);
    prizeForm.image = base64;
    cropperImage.value = null; 
};

const cancelCrop = () => {
    cropperImage.value = null;
};

// ================= Sorting / Drag and Drop Actions =================
const draggedItemIndex = ref(null);

const onDragStart = (e, index, type) => {
    draggedItemIndex.value = index;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', index);
    e.dataTransfer.setData('type', type);
};

const onDragOver = (e, index) => {
    // Just allow drop
};

const onDrop = async (e, dropIndex, type) => {
    const dragIndex = parseInt(e.dataTransfer.getData('text/plain'), 10);
    const dragType = e.dataTransfer.getData('type');
    
    draggedItemIndex.value = null;

    if (dragType !== type || dragIndex === dropIndex) return;

    // Determine which list to reorder
    let list = [];
    if (type === 'activities') {
        list = [...store.activities];
    } else if (type === 'prizes') {
        list = [...store.prizes];
    }

    // Move item in local copy
    const [movedItem] = list.splice(dragIndex, 1);
    list.splice(dropIndex, 0, movedItem);

    // Extract IDs in new order
    const ids = list.map(item => item.id);

    // Call store action
    if (type === 'activities') {
        // Optimistic update
        store.activities = list; 
        await store.reorderActivities(ids);
    } else if (type === 'prizes') {
        store.prizes = list;
        await store.reorderPrizes(ids);
    }
};

// ================= Points Management Actions (Refactored) =================

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

// Fetch list of all users from API (New Implementation)
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

// Article Actions
const editArticle = (id) => {
    router.push({ path: '/submit', query: { id } });
};

const approveArticle = async (article) => {
    if (confirm(`确认发布文章 "${article.title}" 吗？`)) {
        await store.updateArticle(article.id, { status: 'published' });
    }
};

const confirmDelete = async (article) => {
    const actionName = article.status === 'pending' ? '拒绝' : '删除';
    if (confirm(`确定要${actionName}文章 "${article.title}" 吗？此操作无法撤销。`)) {
        await store.deleteArticle(article.id);
    }
};

// Preview Logic
const openPreview = (article) => {
    previewArticle.value = article;
    previewMode.value = 'card';
    document.body.style.overflow = 'hidden';
};

const closePreview = () => {
    previewArticle.value = null;
    document.body.style.overflow = '';
};

// --- Helper Functions for Rendering ---

// 1. Text Formatters
const escapeHtml = (str) => str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const parseInlineStyles = (html) => html
    .replace(/\*\*(.*?)\*\*/g, '<b>$1</b>')
    .replace(/\*(.*?)\*/g, '<i>$1</i>')
    .replace(/__(.*?)__/g, '<u>$1</u>')
    .replace(/~~(.*?)~~/g, '<s>$1</s>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" class="inline-link">$1</a>');

// For Modal View (Standard <br>)
const formatModalParagraph = (text) => {
  if (!text) return '';
  const escaped = escapeHtml(text.toString());
  const styled = parseInlineStyles(escaped);
  return styled.replace(/\n/g, '<br>'); // Simple break
};

// For Page View (Indented Paragraphs)
const formatPageParagraph = (text) => {
  if (!text) return '';
  const lines = text.toString().split('\n').map(line => line.replace(/^[\s\u3000]+/, ''));
  return lines.map(line => {
    const escaped = escapeHtml(line);
    const styled = parseInlineStyles(escaped);
    return `<span class="para-line">${styled || '&nbsp;'}</span>`;
  }).join('');
};

// For DetailModal Logic (Truncation)
const processContent = (contentBlocks) => {
  const MAX_LENGTH = 150;
  let currentLength = 0;
  let blocks = [];
  let truncated = false;
  if (!contentBlocks) return { blocks: [], truncated: false };

  for (let i = 0; i < contentBlocks.length; i++) {
    const block = contentBlocks[i];
    blocks.push(block);
    if (block.text) currentLength += block.text.length;
    if (currentLength > MAX_LENGTH) {
      if (i < contentBlocks.length - 1) {
        truncated = true;
        while (blocks.length > 0 && blocks[blocks.length - 1].type === 'heading') {
          blocks.pop();
        }
      }
      break;
    }
  }
  if (!truncated && blocks.length < contentBlocks.length) {
    truncated = true;
  }
  return { blocks, truncated };
};
const modalState = computed(() => processContent(previewArticle.value?.content));
const modalBlocks = computed(() => modalState.value.blocks);
const isModalTruncated = computed(() => modalState.value.truncated);

// For Page View (Indented Paragraphs + TOC)
const toc = computed(() => {
  if (!previewArticle.value || !previewArticle.value.content) return [];
  return previewArticle.value.content.map((block, index) => ({ ...block, index })).filter((block) => block.type === 'heading');
});
const wordCount = computed(() => {
  if (!previewArticle.value || !previewArticle.value.content) return 0;
  return previewArticle.value.content.reduce((acc, block) => acc + (block.text ? block.text.length : 0), 0);
});
const scrollToHeading = (index) => {
    const el = document.getElementById(`heading-${index}`);
    if (el) el.scrollIntoView({ behavior: 'smooth' });
};

// 管理态生效后（含异步 JWT 会话恢复完成）加载后台数据
const loadAdminData = () => {
    store.fetchAdminArticles();
    store.fetchPrizes();
    store.fetchActivities();
    // Load initial user list if needed
    fetchAllPointsUsers();
};

onMounted(() => {
    if (store.isAdmin) loadAdminData();
});

// 统一 JWT 会话恢复是异步的：isAdmin 由 false → true 时补拉数据
watch(() => store.isAdmin, (val) => {
    if (val) loadAdminData();
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

/* ==================== Icons ==================== */
.icon-xs {
  width: 0.75rem;
  height: 0.75rem;
}
.icon-sm {
  width: 1rem;
  height: 1rem;
}
.icon-md {
  width: 1.5rem;
  height: 1.5rem;
}
.icon-drag {
  width: 1.25rem;
  height: 1.25rem;
}
.icon-drag:hover {
  color: #000;
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

/* ==================== Tab Content ==================== */
.tab-content {
  padding: 1.5rem;
}
.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}
.tab-title {
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1.75rem;
}

/* ==================== Shared Table Styles ==================== */
.table-wrapper {
  overflow-x: auto;
}
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
.table-row {
  border-bottom: 1px solid #f3f4f6;
  transition: color 150ms, background-color 150ms, border-color 150ms;
}
.table-row:hover {
  background-color: rgba(249,250,251,0.8);
}

/* ==================== Drag State ==================== */
.dragging {
  opacity: 0.5;
}

/* ==================== Table Cells ==================== */
.td-drag {
  padding: 0.75rem;
  padding-left: 1rem;
  cursor: grab;
  color: #9ca3af;
}
.td-drag:active {
  cursor: grabbing;
}
.td-order {
  padding: 0.75rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: #9ca3af;
}
.td-cell {
  padding: 0.75rem;
}
.td-cell-xs {
  font-size: 0.75rem;
  line-height: 1rem;
}
.td-bold {
  font-weight: 700;
}
.td-actions {
  padding: 0.75rem;
  text-align: right;
  padding-right: 1rem;
}

/* ==================== Cell Content ==================== */
.cell-title {
  font-weight: 700;
}
.cell-subtitle {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #9ca3af;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 1;
  max-width: 20rem;
}
.cell-col-gap1 {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.cell-col {
  display: flex;
  flex-direction: column;
}
.cell-status-bold {
  font-weight: 700;
}
.cell-type {
  color: #6b7280;
}
.cell-points {
  color: #9333ea;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-weight: 700;
}
.cell-pool {
  color: #9ca3af;
}
.cell-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}
.cell-stock {
  font-size: 0.75rem;
  line-height: 1rem;
}

/* ==================== Thumbnail ==================== */
.thumb-rect {
  width: 4rem;
  height: 2.5rem;
  border-radius: 0.25rem;
  border: 1px solid #e5e7eb;
  overflow: hidden;
  background-color: #f3f4f6;
}
.thumb-square {
  width: 3rem;
  height: 3rem;
  border-radius: 0.25rem;
  border: 1px solid #e5e7eb;
  overflow: hidden;
  background-color: #f3f4f6;
  position: relative;
}
.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* ==================== Attribute Tags ==================== */
.attr-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}
.attr-tag-gray {
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  background-color: #f3f4f6;
  color: #4b5563;
}
.attr-tag-color {
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  color: #fff;
}
.attr-tag-outline {
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  border: 1px solid #e5e7eb;
}

/* ==================== Action Links ==================== */
.link-edit {
  color: #2563eb;
  margin-right: 0.75rem;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}
.link-edit:hover {
  text-decoration: underline;
}
.link-delete {
  color: #ef4444;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}
.link-delete:hover {
  text-decoration: underline;
}

/* ==================== Utility Colors ==================== */
.text-success { color: #16a34a; }
.text-danger { color: #ef4444; }
.text-muted { color: #9ca3af; }

/* ==================== Article Tab (Tab 2 & 3) ==================== */
.article-tab {
  overflow-x: auto;
  flex: 1;
}
.article-thead {
  background-color: rgba(249,250,251,0.5);
}
.article-thead-row {
  font-size: 0.75rem;
  line-height: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #9ca3af;
}
.ath-id { padding: 1rem 1rem 1rem 2rem; font-weight: 700; }
.ath-title { padding: 1rem; width: 33.333%; font-weight: 700; }
.ath-author { padding: 1rem; font-weight: 700; }
.ath-date { padding: 1rem; font-weight: 700; }
.ath-status { padding: 1rem; font-weight: 700; }
.ath-actions { padding: 1rem 2rem; text-align: right; font-weight: 700; }

.article-row {
  border-bottom: 1px solid #f3f4f6;
  transition: color 150ms, background-color 150ms, border-color 150ms;
}
.article-row:hover {
  background-color: rgba(249,250,251,0.8);
}
.article-row:last-child {
  border-bottom: 0;
}

.atd-id {
  padding: 1.25rem 1rem 1.25rem 2rem;
  vertical-align: top;
}
.article-id {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: #9ca3af;
  font-size: 0.75rem;
  line-height: 1rem;
  margin-bottom: 0.25rem;
}
.type-badge-news {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 10px;
  font-weight: 700;
  background-color: #000;
  color: #fff;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.type-badge-article {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 10px;
  font-weight: 700;
  background-color: #e5e7eb;
  color: #4b5563;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.atd-title {
  padding: 1.25rem 1rem;
  vertical-align: top;
}
.article-title-text {
  font-weight: 700;
  font-size: 1rem;
  line-height: 1.5rem;
  color: #111827;
  margin-bottom: 0.25rem;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 1;
  transition: color 150ms, background-color 150ms, border-color 150ms;
}
.article-row:hover .article-title-text {
  color: #1e3a8a;
}
.article-summary-text {
  color: #9ca3af;
  font-size: 0.75rem;
  line-height: 1rem;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-height: 1.625;
  max-width: 28rem;
}
.atd-author {
  padding: 1.25rem 1rem;
  vertical-align: top;
}
.author-name {
  font-weight: 500;
  color: #111827;
}
.author-participants {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #9ca3af;
  margin-top: 0.125rem;
}
.atd-date {
  padding: 1.25rem 1rem;
  vertical-align: top;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: #6b7280;
  font-size: 0.75rem;
  line-height: 1rem;
}
.atd-status {
  padding: 1.25rem 1rem;
  vertical-align: top;
}
.status-pending {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: #a16207;
  background-color: #fefce8;
  padding: 0.25rem 0.625rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  border: 1px solid #fef9c3;
}
.status-dot-pending {
  width: 0.375rem;
  height: 0.375rem;
  border-radius: 9999px;
  background-color: #eab308;
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
.status-published-col {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  align-items: flex-start;
}
.status-published {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: #15803d;
  background-color: #f0fdf4;
  padding: 0.25rem 0.625rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  border: 1px solid #dcfce7;
}
.status-dot-published {
  width: 0.375rem;
  height: 0.375rem;
  border-radius: 9999px;
  background-color: #22c55e;
}
.pinned-label {
  font-size: 10px;
  font-weight: 700;
  color: #9ca3af;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.atd-actions-cell {
  padding: 1.25rem 2rem;
  vertical-align: top;
  text-align: right;
}
.action-btns {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  opacity: 1;
  transition: opacity 150ms;
}
@media (min-width: 768px) {
  .action-btns {
    opacity: 0;
  }
  .article-row:hover .action-btns {
    opacity: 1;
  }
}
.action-btn-preview {
  padding: 0.5rem;
  color: #9ca3af;
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms, border-color 150ms;
}
.action-btn-preview:hover {
  color: #000;
  background-color: #f3f4f6;
}
.action-divider {
  width: 1px;
  height: 1rem;
  background-color: #e5e7eb;
  margin: 0 0.25rem;
}
.action-btn-approve {
  padding: 0.5rem;
  color: #16a34a;
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}
.action-btn-approve:hover {
  background-color: #f0fdf4;
}
.action-btn-edit {
  padding: 0.5rem;
  color: #2563eb;
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}
.action-btn-edit:hover {
  background-color: #eff6ff;
}
.action-btn-delete {
  padding: 0.5rem;
  color: #f87171;
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}
.action-btn-delete:hover {
  color: #dc2626;
  background-color: #fef2f2;
}

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

/* ==================== Generator Tab (Tab 5) ==================== */
.generator-tab {
  padding: 2rem;
  background-color: #f9fafb;
  flex: 1;
}
.generator-layout {
  display: flex;
  flex-direction: column;
  gap: 2.5rem;
}
@media (min-width: 1280px) {
  .generator-layout {
    flex-direction: row;
  }
}
.generator-sidebar {
  width: 100%;
}
.generator-sidebar > * + * {
  margin-top: 2rem;
}
@media (min-width: 1280px) {
  .generator-sidebar {
    width: 28%;
  }
}
.generator-config-title {
  font-size: 1.125rem;
  line-height: 1.75rem;
  font-weight: 700;
  font-family: "Noto Serif SC", serif;
  margin-bottom: 1rem;
}
.generator-config-box {
  background-color: #fff;
  padding: 1.5rem;
  border-radius: 0.75rem;
  border: 1px solid #e5e7eb;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}
.generator-config-box > * + * {
  margin-top: 1.5rem;
}
.gen-date-input {
  width: 100%;
  border-bottom: 2px solid #e5e7eb;
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-size: 1.125rem;
  line-height: 1.75rem;
  background-color: transparent;
}
.gen-range-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
}
.gen-range-btn {
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  border: 1px solid;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
}
.gen-match-info {
  padding-top: 1rem;
  border-top: 1px solid #f3f4f6;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.875rem;
  line-height: 1.25rem;
}
.gen-match-text {
  color: #6b7280;
}
.gen-match-count {
  color: #000;
}
.gen-export-btn {
  width: 100%;
  padding: 1rem;
  background-color: #000;
  color: #fff;
  border-radius: 0.75rem;
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1.75rem;
  box-shadow: 0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1);
}
.gen-export-btn:hover {
  background-color: #1f2937;
}
.gen-export-btn:disabled {
  opacity: 0.5;
}

/* Generator Preview Area */
.generator-preview-area {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background-color: rgba(229,231,235,0.5);
  border-radius: 0.75rem;
  padding: 2rem;
  border: 1px solid #e5e7eb;
  overflow-x: auto;
  overflow-y: auto;
}
@media (min-width: 1280px) {
  .generator-preview-area {
    width: 72%;
  }
}

/* Poster */
.poster-canvas {
  width: 450px;
  min-height: 800px;
  background-color: #fff;
  color: #000;
  display: flex;
  flex-direction: column;
  position: relative;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  flex-shrink: 0;
}
.poster-canvas--dense {
  min-height: 860px;
}
.poster-canvas--quarter {
  width: 760px;
}
.poster-canvas--half-year {
  width: 900px;
}
.poster-header {
  background-color: #000;
  color: #fff;
  padding: 2rem;
  padding-bottom: 1.5rem;
}
.poster-title {
  font-size: 2.25rem;
  line-height: 2.5rem;
  font-weight: 900;
  font-family: "Noto Serif SC", serif;
  line-height: 1;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}
.poster-header-line {
  width: 3rem;
  height: 0.25rem;
  background-color: #fff;
  margin-bottom: 1.5rem;
}
.poster-date-range {
  font-size: 1.25rem;
  line-height: 1.75rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  opacity: 0.9;
}
.poster-edition {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  line-height: 1rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  font-weight: 700;
  opacity: 0.72;
}
.poster-canvas--dense .poster-header {
  padding: 1.5rem;
  padding-bottom: 1rem;
}
.poster-canvas--dense .poster-title {
  margin-top: 0.5rem;
  font-size: 1.875rem;
  line-height: 2.25rem;
}
.poster-canvas--dense .poster-header-line {
  margin-bottom: 0.9rem;
}
.poster-canvas--dense .poster-date-range {
  font-size: 1rem;
  line-height: 1.5rem;
}
.poster-body {
  flex: 1;
  padding: 2rem;
}
.poster-body > * + * {
  margin-top: 1.5rem;
}
.poster-body--dense {
  display: grid;
  gap: 0.7rem 0.9rem;
  align-content: start;
  padding: 1.25rem;
}
.poster-body--dense > * + * {
  margin-top: 0;
}
.poster-body--dense-quarter {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}
.poster-body--dense-half-year {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.poster-item {
  position: relative;
  padding-left: 1.5rem;
  border-left: 2px solid #e5e7eb;
  min-width: 0;
}
.poster-item--dense {
  padding-left: 0.9rem;
  border-left-width: 1px;
  border-left-color: #d1d5db;
}
.poster-item-dot {
  position: absolute;
  left: -5px;
  top: 0.375rem;
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 9999px;
  background-color: #000;
}
.poster-item--dense .poster-item-dot {
  left: -4px;
  top: 0.3rem;
  width: 0.4rem;
  height: 0.4rem;
}
.poster-item-date {
  font-size: 10px;
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  color: #9ca3af;
  letter-spacing: 0.05em;
  margin-bottom: 0.25rem;
}
.poster-item--dense .poster-item-date {
  font-size: 9px;
  margin-bottom: 0.15rem;
}
.poster-item-title {
  font-size: 1.125rem;
  line-height: 1.375;
  font-weight: 700;
  margin-bottom: 0.25rem;
  overflow-wrap: anywhere;
  word-break: break-word;
}
.poster-item-title--dense {
  font-size: 0.9rem;
  line-height: 1.35;
  margin-bottom: 0.15rem;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
}
.poster-item-summary {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #6b7280;
  line-height: 1.625;
  font-family: "Noto Sans SC", sans-serif;
  text-align: justify;
  margin-top: 0.25rem;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow-wrap: anywhere;
  word-break: break-word;
}
.poster-item-summary--dense {
  font-size: 0.66rem;
  line-height: 1.35;
  margin-top: 0.1rem;
}
.poster-body--dense-half-year .poster-item-summary--dense {
  -webkit-line-clamp: 1;
}
.poster-body--dense-half-year .poster-item-title--dense {
  font-size: 0.82rem;
}
.poster-footer {
  margin-top: auto;
  padding: 2rem;
  padding-top: 0;
}
.poster-canvas--dense .poster-footer {
  padding: 1rem 1.25rem 1.25rem;
}
.poster-footer-inner {
  border-top: 2px solid #000;
  padding-top: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}
.poster-footer-brand {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  font-family: "Noto Serif SC", serif;
}

@media (max-width: 1100px) {
  .poster-canvas--half-year {
    width: 760px;
  }
  .poster-body--dense-half-year {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

/* ==================== Option Active/Inactive ==================== */
.opt-active {
  background-color: #000;
  color: #fff;
  border-color: #000;
}
.opt-inactive {
  background-color: #fff;
  color: #4b5563;
  border-color: #e5e7eb;
}
.opt-inactive:hover {
  border-color: #9ca3af;
}

/* ==================== Buttons ==================== */
.btn-primary-sm {
  padding: 0.5rem 1rem;
  background-color: #000;
  color: #fff;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  transition: all 150ms;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.btn-primary-sm:hover {
  background-color: #1f2937;
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

/* ==================== Modal Overlay ==================== */
.modal-overlay {
  position: fixed;
  top: 0; right: 0; bottom: 0; left: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0,0,0,0.8);
  backdrop-filter: blur(4px);
  padding: 1rem;
}

/* ==================== Activity Modal ==================== */
.modal-activity {
  background-color: #fff;
  border-radius: 0.75rem;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  width: 100%;
  max-width: 56rem;
  max-height: 90vh;
  overflow-y: auto;
  padding: 2rem;
}
.modal-title {
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
  font-family: "Noto Serif SC", serif;
  margin-bottom: 1.5rem;
}
.modal-form-space > * + * {
  margin-top: 1rem;
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
.form-label-mb3 {
  display: block;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.75rem;
}
.form-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.25rem;
}
.form-input-bold {
  font-weight: 700;
}
.form-input-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}
.form-input-upper-mono {
  text-transform: uppercase;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}
.form-input-upper {
  text-transform: uppercase;
}
.form-textarea {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-family: "Noto Sans SC", sans-serif;
}
.form-textarea-sm {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
}
.form-select {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.25rem;
  background-color: #fff;
}
.form-file-input {
  display: block;
  width: 100%;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #6b7280;
}
.form-file-input::file-selector-button {
  margin-right: 1rem;
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  border: 0;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  background-color: #000;
  color: #fff;
}
.form-file-input::file-selector-button:hover {
  background-color: #1f2937;
}
.form-grid-2col {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1.5rem;
}
@media (min-width: 768px) {
  .form-grid-2col {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
.form-grid-2col-equal {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
}
.form-grid-3col {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1rem;
}
@media (min-width: 768px) {
  .form-grid-3col {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

/* Activity Image Preview */
.activity-image-preview {
  margin-top: 0.5rem;
  height: 8rem;
  width: 100%;
  background-color: #f3f4f6;
  border-radius: 0.25rem;
  overflow: hidden;
  position: relative;
  border: 1px solid #e5e7eb;
}

/* ==================== Modal Footer Buttons ==================== */
.modal-footer-btns {
  padding-top: 1.5rem;
  display: flex;
  gap: 0.75rem;
}
.modal-footer-btns-sm {
  padding-top: 1rem;
  display: flex;
  gap: 0.75rem;
}
.btn-cancel {
  flex: 1;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  font-weight: 700;
  color: #6b7280;
}
.btn-cancel:hover {
  background-color: #f9fafb;
}
.btn-submit {
  flex: 1;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  background-color: #000;
  color: #fff;
  border-radius: 0.5rem;
  font-weight: 700;
}
.btn-submit:hover {
  background-color: #1f2937;
}
.btn-submit:disabled {
  opacity: 0.5;
}

/* ==================== Prize Modal ==================== */
.modal-prize {
  background-color: #fff;
  border-radius: 0.75rem;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  width: 100%;
  max-width: 56rem;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
@media (min-width: 768px) {
  .modal-prize {
    flex-direction: row;
  }
}
.prize-modal-left {
  width: 100%;
  background-color: #f3f4f6;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid #e5e7eb;
  position: relative;
}
@media (min-width: 768px) {
  .prize-modal-left {
    width: 50%;
    border-bottom: 0;
    border-right: 1px solid #e5e7eb;
  }
}
.prize-modal-right {
  width: 100%;
  padding: 1.5rem;
  overflow-y: auto;
}
@media (min-width: 768px) {
  .prize-modal-right {
    width: 50%;
  }
}

/* Cropper */
.cropper-area {
  position: relative;
  width: 100%;
  height: 300px;
  overflow: hidden;
  cursor: move;
  touch-action: none;
  display: flex;
  align-items: center;
  justify-content: center;
}
@media (min-width: 768px) {
  .cropper-area {
    height: 100%;
  }
}
.cropper-image {
  position: absolute;
  max-width: none;
  user-select: none;
  pointer-events: none;
  transition: transform 75ms;
  transform-origin: center;
}
.cropper-mask {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  pointer-events: none;
  z-index: 10;
}
.cropper-frame {
  position: absolute;
  z-index: 20;
  border: 2px solid #fff;
  pointer-events: none;
  box-shadow: 0 0 0 9999px rgba(0,0,0,0.6);
}
.cropper-actions {
  position: absolute;
  bottom: 1rem;
  width: 100%;
  padding-left: 1.5rem;
  padding-right: 1.5rem;
  z-index: 40;
  display: flex;
  gap: 0.5rem;
}
.btn-crop-confirm {
  flex: 1;
  background-color: #16a34a;
  color: #fff;
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  border-radius: 0.25rem;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}
.btn-crop-confirm:hover {
  background-color: #15803d;
}
.btn-crop-cancel {
  padding-left: 1rem;
  padding-right: 1rem;
  background-color: #fff;
  color: #374151;
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  border-radius: 0.25rem;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}
.btn-crop-cancel:hover {
  background-color: #f3f4f6;
}

/* Prize Upload */
.prize-upload-area {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  text-align: center;
}
.prize-upload-placeholder {
  width: 10rem;
  height: 10rem;
  background-color: #fff;
  border: 2px dashed #d1d5db;
  border-radius: 0.5rem;
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}
.prize-upload-plus {
  color: #d1d5db;
  font-size: 2.25rem;
  line-height: 2.5rem;
}
.prize-upload-hint {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #6b7280;
  margin-bottom: 1rem;
}
.btn-upload {
  padding: 0.5rem 1.5rem;
  background-color: #000;
  color: #fff;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}
.btn-upload:hover {
  background-color: #1f2937;
}
.hidden-input {
  display: none;
}

/* Size Grid */
.size-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.5rem;
}
.size-btn {
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  border: 1px solid;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  transition: all 150ms;
}

/* Color Picker */
.color-picker-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}
.color-picker {
  width: 2.5rem;
  height: 2.5rem;
  border: none;
  padding: 0;
  background-color: transparent;
  cursor: pointer;
}

/* ==================== Preview Overlay ==================== */
.preview-overlay {
  position: fixed;
  top: 0; right: 0; bottom: 0; left: 0;
  z-index: 60;
  background-color: #f3f4f6;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.preview-header {
  background-color: #fff;
  border-bottom: 1px solid #e5e7eb;
  padding: 0.75rem 1.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  z-index: 10;
  flex-shrink: 0;
}
.preview-header-left {
  display: flex;
  align-items: center;
  gap: 1.5rem;
}
.preview-header-info {
  display: flex;
  flex-direction: column;
}
.preview-mode-label {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
.preview-title {
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1.25;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 28rem;
}
.preview-switcher {
  display: flex;
  background-color: #f3f4f6;
  padding: 0.25rem;
  border-radius: 0.5rem;
}
.preview-tab-btn {
  padding: 0.375rem 1rem;
  border-radius: 0.375rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  transition: all 150ms;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.preview-tab-active {
  background-color: #fff;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  color: #000;
}
.preview-tab-inactive {
  color: #6b7280;
}
.preview-tab-inactive:hover {
  color: #374151;
}
.preview-header-right {
  display: flex;
  align-items: center;
  gap: 1rem;
}
.preview-pending-badge {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  background-color: #fef9c3;
  color: #a16207;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
}
.preview-close-btn {
  padding: 0.5rem;
  border-radius: 9999px;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  color: #6b7280;
}
.preview-close-btn:hover {
  background-color: #f3f4f6;
  color: #000;
}
.preview-body {
  flex: 1;
  overflow-y: auto;
  background-color: #f3f4f6;
  position: relative;
}

/* Card Mode */
.preview-card-mode {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}
.preview-card-wrap {
  width: 380px;
}
.preview-card-component {
  pointer-events: none;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  background-color: #fff;
}
.preview-card-caption {
  text-align: center;
  color: #9ca3af;
  font-size: 0.75rem;
  line-height: 1rem;
  margin-top: 1.5rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

/* Modal Mode */
.preview-modal-mode {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}
@media (min-width: 768px) {
  .preview-modal-mode {
    padding: 2.5rem;
  }
}
.preview-modal-container {
  position: relative;
  background-color: #fff;
  width: 100%;
  max-width: 42rem;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  display: flex;
  flex-direction: column;
  border: 1px solid #000;
  max-height: 90vh;
  overflow-y: auto;
}
.preview-modal-header {
  position: sticky;
  top: 0;
  background-color: #fff;
  border-bottom: 1px solid #f3f4f6;
  padding: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 10;
}
.preview-modal-header-label {
  font-size: 0.75rem;
  line-height: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #9ca3af;
}
.preview-modal-close-placeholder {
  padding: 0.5rem;
  opacity: 0.5;
}
.preview-modal-body {
  padding: 1.5rem;
}
@media (min-width: 768px) {
  .preview-modal-body {
    padding: 2.5rem;
  }
}
.preview-modal-meta {
  margin-bottom: 1.5rem;
}
.preview-modal-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}
.tag-filled {
  display: inline-block;
  background-color: #000;
  color: #fff;
  font-size: 0.75rem;
  line-height: 1rem;
  padding: 0.25rem 0.5rem;
}
.tag-outline {
  display: inline-block;
  border: 1px solid #000;
  color: #000;
  font-size: 0.75rem;
  line-height: 1rem;
  padding: 0.25rem 0.5rem;
  text-transform: uppercase;
}
.preview-modal-article-title {
  font-size: 1.875rem;
  line-height: 2.25rem;
  font-weight: 700;
  margin-bottom: 1rem;
  line-height: 1.25;
}
@media (min-width: 768px) {
  .preview-modal-article-title {
    font-size: 2.25rem;
    line-height: 2.5rem;
  }
}
.preview-modal-info-row {
  display: flex;
  align-items: center;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #6b7280;
  gap: 1rem;
  margin-bottom: 0.5rem;
}
.preview-modal-author {
  color: #000;
  font-weight: 700;
}
.preview-modal-participants {
  font-size: 0.75rem;
  line-height: 1rem;
  background-color: #f9fafb;
  padding: 0.75rem;
  margin-bottom: 1rem;
  border-radius: 0.25rem;
}
.participants-label {
  font-weight: 700;
  color: #9ca3af;
  margin-bottom: 0.25rem;
}
.participants-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.participant-name {
  font-weight: 700;
  color: #1e3a8a;
}
.participant-role {
  color: #6b7280;
}
.preview-modal-image {
  margin-bottom: 2rem;
}
.preview-modal-img {
  width: 100%;
  height: auto;
  transition: all 500ms;
}
.preview-modal-content {
  color: #1f2937;
  font-family: "Noto Serif SC", serif;
  line-height: 2;
  text-align: justify;
}
.math-block-modal {
  margin: 1rem 0;
  padding: 1rem;
  background-color: #f9fafb;
  text-align: center;
}
.image-block-modal {
  margin: 1rem 0;
}
.image-block-modal-img {
  width: 100%;
  max-height: 12rem;
  object-fit: cover;
}
.truncation-indicator {
  color: #9ca3af;
  text-align: center;
  margin-top: 1rem;
}
.preview-modal-footer-actions {
  margin-top: 2.5rem;
  padding-top: 2rem;
  border-top: 1px solid #f3f4f6;
  display: flex;
  justify-content: center;
}
.btn-read-full {
  background-color: #000;
  color: #fff;
  padding: 0.75rem 2rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: not-allowed;
  opacity: 0.8;
}
.btn-open-page {
  border: 1px solid #000;
  padding: 0.75rem 2rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: not-allowed;
  opacity: 0.5;
}

/* ==================== Page Mode (BlogDetail replica) ==================== */
.preview-page-mode {
  background-color: #fff;
  min-height: 100vh;
  font-family: "Noto Sans SC", sans-serif;
  color: #111;
  max-width: 100%;
  overflow-x: hidden;
}
.page-hero {
  position: relative;
  width: 100%;
  height: 600px;
  background-color: #bfbfbf;
  overflow: hidden;
  user-select: none;
}
.page-hero-cover {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  width: 100%;
  height: 100%;
}
.page-hero-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.page-hero-gradient-main {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background: linear-gradient(to bottom, rgba(0,0,0,0.3), transparent, rgba(0,0,0,0.6));
}
.page-hero-overlay {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background-color: rgba(0,0,0,0.1);
}
.page-hero-fallback {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  width: 100%;
  height: 100%;
  background-color: #bfbfbf;
  overflow: hidden;
}
.page-hero-pattern {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  display: flex;
  flex-wrap: wrap;
  align-content: flex-start;
  justify-content: center;
  gap: 2rem;
  opacity: 0.05;
  pointer-events: none;
  transform: rotate(-12deg) scale(1.25);
}
.page-hero-pattern-text {
  font-size: 4.5rem;
  line-height: 1;
  font-weight: 900;
  color: #000;
  white-space: nowrap;
  font-family: "Noto Sans SC", sans-serif;
}
.page-hero-gradient-fallback {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.063), transparent);
}
.page-hero-content {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  top: 0;
  width: 100%;
  max-width: 1600px;
  height: 100%;
  z-index: 10;
}
.page-tags-desktop {
  display: none;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  color: #fff;
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
  position: absolute;
}
@media (min-width: 768px) {
  .page-tags-desktop {
    display: flex;
  }
}
.page-tags-wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}
.page-tag-item {
  display: flex;
  align-items: center;
}
.page-tag-hash {
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 27px;
  line-height: 36px;
  color: rgba(255,255,255,0.5);
  margin-right: 0.25rem;
}
.page-tag-text {
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 27px;
  line-height: 36px;
  color: #fff;
  text-decoration: underline;
  text-decoration-thickness: 2px;
  text-underline-offset: 4px;
}
.page-news-badge {
  margin-left: 0.5rem;
  border: 1px solid rgba(255,255,255,0.6);
  padding: 0.125rem 0.5rem;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.1em;
}
.page-title-desktop {
  display: none;
  flex-direction: column;
  color: #fff;
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
  position: absolute;
}
@media (min-width: 768px) {
  .page-title-desktop {
    display: flex;
  }
}
.page-main-title {
  font-family: "Noto Serif SC", serif;
  font-weight: 900;
  font-size: 80px;
  line-height: 90px;
  letter-spacing: 0;
  margin-bottom: 0;
  color: #fff;
}
.page-subtitle {
  margin-top: 6px;
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 27px;
  line-height: 45px;
  color: #fff;
}
.page-author-row {
  margin-top: 1rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  column-gap: 1rem;
  row-gap: 0.25rem;
}
.page-author-info {
  font-family: "Noto Sans SC", sans-serif;
  font-size: 24px;
  line-height: 45px;
  color: #fff;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.page-author-label {
  font-weight: 400;
  opacity: 0.8;
}
.page-author-name {
  font-weight: 700;
}
.page-date-info {
  font-family: "Noto Sans SC", sans-serif;
  font-size: 21px;
  line-height: 45px;
  color: rgba(255,255,255,0.9);
}

/* Mobile Header */
.page-mobile-header {
  position: absolute;
  left: 1rem;
  right: 1rem;
  bottom: 2.5rem;
  color: #fff;
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
}
@media (min-width: 768px) {
  .page-mobile-header {
    display: none;
  }
}
.page-mobile-tags {
  margin-bottom: 0.5rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}
.page-mobile-tag {
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #fff;
}
.page-mobile-title {
  font-family: "Noto Serif SC", serif;
  font-weight: 900;
  font-size: 1.5rem;
  line-height: 1.375;
}
.page-mobile-subtitle {
  margin-top: 0.25rem;
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

/* Page Content Body */
.page-content-container {
  max-width: 1600px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1rem;
  padding-right: 1rem;
  position: relative;
  margin-top: 61px;
}
@media (min-width: 768px) {
  .page-content-container {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}
@media (min-width: 1024px) {
  .page-content-container {
    padding-left: 0;
    padding-right: 0;
  }
}
.page-content-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 3rem;
  padding-bottom: 5rem;
}
@media (min-width: 1024px) {
  .page-content-grid {
    grid-template-columns: repeat(12, minmax(0, 1fr));
    padding-left: 105px;
    padding-right: 2.5rem;
  }
}
.page-article {
  padding-top: 0;
  padding-bottom: 3rem;
  max-width: none;
}
@media (min-width: 768px) {
  .page-article {
    padding-top: 0;
    padding-bottom: 5rem;
  }
}
@media (min-width: 1024px) {
  .page-article {
    grid-column: span 9;
  }
}

/* Prose overrides for page article */
.page-article :deep(h1),
.page-article :deep(h2),
.page-article :deep(h3),
.page-article :deep(h4),
.page-article :deep(h5),
.page-article :deep(h6) {
  font-family: "Noto Serif SC", serif;
}
.page-article :deep(p) {
  color: #333;
}
.page-article :deep(img) {
  border-radius: 0.125rem;
}

/* Page Participants */
.page-participants-box {
  margin-bottom: 2.5rem;
  border-radius: 0.5rem;
  border: 1px solid #f3f4f6;
  background-color: rgba(249,250,251,0.8);
  padding: 1.25rem 1.5rem;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}
.page-participants-heading {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  letter-spacing: 0.25em;
  color: #6b7280;
  text-transform: uppercase;
  margin-bottom: 0.75rem;
}
.page-participants-list {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #374151;
}
.page-participants-list > * + * {
  margin-top: 0.5rem;
}
.page-participant-item {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  column-gap: 0.5rem;
  row-gap: 0.125rem;
}
.page-participant-detail {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #6b7280;
}

/* Page Content Blocks */
.page-content-block {
  margin-bottom: 2rem;
}
.page-paragraph {
  line-height: 1.625;
  text-align: justify;
  font-size: 22px;
  letter-spacing: 0.025em;
  color: #1f2937;
}
@media (min-width: 768px) {
  .page-paragraph {
    font-size: 23px;
  }
}
.page-heading {
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
  margin-top: 4rem;
  margin-bottom: 1.5rem;
  padding-bottom: 1rem;
  border-bottom: 2px solid #e5e5e5;
  color: #000;
  position: relative;
}
.page-math-block {
  margin: 2.5rem 0;
  padding: 2rem;
  background-color: #f9f9f9;
  border-left: 4px solid #999;
  text-align: center;
  overflow-x: auto;
}
.page-math-expr {
  font-family: "Noto Serif SC", serif;
  font-size: 1.25rem;
  line-height: 1.75rem;
}
.page-math-caption {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #6b7280;
  margin-top: 1rem;
  font-style: normal;
  font-family: "Noto Sans SC", sans-serif;
}
.page-image-figure {
  margin: 3rem 0;
}
.page-image-wrap {
  position: relative;
  overflow: hidden;
}
.page-image {
  width: 100%;
  height: auto;
  display: block;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  border: 1px solid #f3f4f6;
}
.page-image-caption {
  text-align: center;
  font-size: 0.75rem;
  line-height: 1rem;
  color: #6b7280;
  margin-top: 0.75rem;
  font-family: "Noto Sans SC", sans-serif;
  letter-spacing: 0.025em;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 0.25rem;
}
.caption-dot {
  width: 0.25rem;
  height: 0.25rem;
  background-color: #9ca3af;
  border-radius: 9999px;
}

/* Fin marker */
.page-fin {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  margin: 5rem 0;
  opacity: 0.3;
}
.fin-line {
  height: 1px;
  width: 3rem;
  background-color: #000;
}
.fin-text {
  font-family: "Noto Serif SC", serif;
  font-style: italic;
  font-size: 1.125rem;
  line-height: 1.75rem;
}

/* Sidebar */
.page-sidebar {
  display: none;
  padding-left: 2rem;
  border-left: 1px solid #f3f4f6;
  background-color: rgba(255,255,255,0.5);
}
@media (min-width: 1024px) {
  .page-sidebar {
    display: block;
    grid-column: span 3;
  }
}
.page-sidebar-inner {
  position: sticky;
  top: 8rem;
  padding-top: 3rem;
  padding-bottom: 3rem;
}
.page-sidebar-title {
  font-weight: 700;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.2em;
  color: #999;
  margin-bottom: 2rem;
}
.page-toc-list {
  border-left: 2px solid #f3f4f6;
  margin-left: 0.25rem;
  padding-left: 1.25rem;
  position: relative;
}
.page-toc-list > * + * {
  margin-top: 1.25rem;
}
.page-toc-item {
  position: relative;
}
.page-toc-dot {
  position: absolute;
  left: -23px;
  top: 6px;
  width: 6px;
  height: 6px;
  border-radius: 9999px;
  background-color: #d1d5db;
  transition: color 150ms, background-color 150ms, border-color 150ms;
}
.page-toc-item:hover .page-toc-dot {
  background-color: #000;
}
.page-toc-link {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #6b7280;
  transition: all 150ms;
  display: block;
  line-height: 1.625;
  font-family: "Noto Sans SC", sans-serif;
}
.page-toc-link:hover {
  color: #000;
  font-weight: 700;
}
.page-toc-empty {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #d1d5db;
  font-style: italic;
  padding-left: 1.5rem;
}
.page-sidebar-stats {
  margin-top: 4rem;
  padding-top: 2rem;
  border-top: 1px solid #f3f4f6;
}
.page-sidebar-stats-inner {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #9ca3af;
  font-family: "Noto Sans SC", sans-serif;
}
.page-sidebar-stats-inner > * + * {
  margin-top: 0.5rem;
}
.page-stat-val {
  color: #000;
}

/* ==================== Existing Preserved Styles ==================== */

/* Scoped styles for typography in preview */
:deep(b) { font-weight: bold; }
:deep(i) { font-style: italic; }
:deep(u) { text-decoration: underline; }
:deep(s) { text-decoration: line-through; }
:deep(.inline-link) {
  color: #2563eb;
  word-break: break-all;
}
:deep(.inline-link:hover) {
  text-decoration: underline;
}

/* Keep the old preview-overlay entrance timing (200ms) */
.animate-fade-in-fast {
  animation: anim-fade-in 0.2s ease both;
}

/* Text Shadow for Page Header */
.text-shadow {
  text-shadow: 0px 6px 22.5px rgba(0, 0, 0, 0.3);
}

/* Article Paragraph Indentation for Page View */
.article-paragraph {
  margin: 0;
}
.article-paragraph :deep(.para-line) {
  display: block;
  text-indent: 2em;
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(0,0,0,0.05);
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0,0,0,0.2);
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(0,0,0,0.4);
}

.bg-checkerboard {
  background-image: 
    linear-gradient(45deg, #ccc 25%, transparent 25%), 
    linear-gradient(-45deg, #ccc 25%, transparent 25%), 
    linear-gradient(45deg, transparent 75%, #ccc 75%), 
    linear-gradient(-45deg, transparent 75%, #ccc 75%);
  background-size: 20px 20px;
  background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
}

/* Fade transition for Vue <Transition name="fade"> */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
