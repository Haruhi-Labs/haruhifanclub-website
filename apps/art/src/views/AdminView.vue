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
        <input
          class="input"
          v-model="loginUser"
          type="text"
          placeholder="请输入用户名"
          @keydown.enter="checkPw"
        />
        <input
          class="input"
          v-model="inputPw"
          type="password"
          placeholder="请输入密码"
          @keydown.enter="checkPw"
        />
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
        <button :class="['nav-item', mainTab === 'images' && 'active']" @click="mainTab = 'images'">
          📷 图片管理
        </button>
        <button
          :class="['nav-item', mainTab === 'comments' && 'active']"
          @click="mainTab = 'comments'"
        >
          💬 评论管理
        </button>
        <button :class="['nav-item', mainTab === 'guild' && 'active']" @click="mainTab = 'guild'">
          ⚔️ 公会系统
        </button>
        <button
          :class="['nav-item', mainTab === 'announcements' && 'active']"
          @click="mainTab = 'announcements'"
        >
          📢 公告
        </button>
      </nav>

      <!-- 内容区域 -->
      <main class="content-area">
        <!-- ================= 图片管理 ================= -->
        <div v-if="mainTab === 'images'" class="tab-content">
          <div class="sub-tabs">
            <button :class="['sub-tab', imageTab === 'audit' && 'on']" @click="imageTab = 'audit'">
              审核管理 ({{ adminStore.pending.length }})
            </button>
            <button :class="['sub-tab', imageTab === 'list' && 'on']" @click="imageTab = 'list'">
              作品列表 (已过审)
            </button>
          </div>

          <!-- 子页：审核管理 -->
          <div v-if="imageTab === 'audit'" class="sub-view">
            <div class="toolbar">
              <span class="tip">待审核或被AI拦截的内容</span>
              <button class="btn-ghost sm" @click="adminStore.loadPending">刷新队列</button>
            </div>

            <div v-if="adminStore.pending.length" class="card-grid">
              <article class="manage-card has-trash" v-for="it in adminStore.pending" :key="it.id">
                <div
                  class="m-thumb"
                  @click="openPreview(it)"
                  style="cursor: pointer"
                  title="点击预览"
                >
                  <img :src="it.image_url" loading="lazy" />
                  <div class="status-badge" :class="it.status">
                    {{ it.status === 'flagged' ? 'AI 锁定' : '待审核' }}
                  </div>
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
                    <span class="u-name"
                      >UP: {{ it.uploader_display_name || it.uploader_name || '匿名' }}</span
                    >
                  </div>
                  <div class="m-desc">{{ it.description }}</div>

                  <!-- 编辑器 -->
                  <div v-if="editingId === it.id" class="inline-editor expanded">
                    <div class="editor-row">
                      <input v-model="editForm.title" class="input sm" placeholder="标题" />
                      <input
                        v-model="editForm.tags"
                        class="input sm"
                        placeholder="标签 (空格分隔)"
                      />
                    </div>
                    <textarea
                      v-model="editForm.description"
                      class="textarea sm"
                      placeholder="描述"
                    ></textarea>

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
                      <input
                        v-model="editForm.origin_url"
                        class="input sm"
                        placeholder="来源链接 (URL)"
                        style="flex: 1"
                      />
                    </div>

                    <div class="editor-licenses">
                      <div class="lic-group">
                        <div class="lic-title">公开授权</div>
                        <label v-for="opt in NET_LICENSE_OPTIONS" :key="opt" class="chk-item">
                          <input type="checkbox" :value="opt" v-model="editForm.netLicenses" />
                          {{ opt }}
                        </label>
                      </div>
                      <div class="lic-group">
                        <div class="lic-title">社团授权</div>
                        <label v-for="opt in GROUP_LICENSE_OPTIONS" :key="opt" class="chk-item">
                          <input type="checkbox" :value="opt" v-model="editForm.groupLicenses" />
                          {{ opt }}
                        </label>
                      </div>
                    </div>

                    <div class="btns">
                      <button class="btn sm" @click="saveEdit(it)">💾 保存修改</button>
                      <button class="btn-ghost sm" @click="editingId = null">取消</button>
                    </div>
                  </div>

                  <div class="m-actions">
                    <textarea
                      class="textarea note-input"
                      v-model="notes[it.id]"
                      placeholder="审核备注..."
                    ></textarea>
                    <div class="btn-group">
                      <button class="btn success" @click="approve(it)">通过</button>
                      <button
                        class="trash-btn"
                        type="button"
                        title="彻底删除"
                        aria-label="彻底删除作品"
                        @click="hardDelete(it)"
                      >
                        🗑
                      </button>
                    </div>
                  </div>
                </div>
              </article>
            </div>
            <div v-else class="empty-state">🎉 暂无待审核内容</div>
          </div>

          <!-- 子页：作品列表 (已过审) -->
          <div v-if="imageTab === 'list'" class="sub-view">
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
                <input
                  v-model="artListFilter.q"
                  @keydown.enter="loadApprovedList"
                  class="input sm search-input"
                  placeholder="搜索标题/描述/UID..."
                />
                <button class="btn sm" @click="loadApprovedList">查询</button>
              </div>
            </div>

            <div class="card-grid">
              <article class="manage-card has-trash" v-for="it in approvedList" :key="it.id">
                <div
                  class="m-thumb"
                  @click="openPreview(it)"
                  style="cursor: pointer"
                  title="点击预览"
                >
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
                    <span class="u-name"
                      >UP: {{ it.uploader_display_name || it.uploader_name || '—' }}</span
                    >
                  </div>
                  <div class="m-desc">{{ it.description }}</div>

                  <div v-if="editingId === it.id" class="inline-editor expanded">
                    <div class="editor-row">
                      <input v-model="editForm.title" class="input sm" placeholder="标题" />
                      <input
                        v-model="editForm.tags"
                        class="input sm"
                        placeholder="标签 (空格分隔)"
                      />
                    </div>
                    <textarea
                      v-model="editForm.description"
                      class="textarea sm"
                      placeholder="描述"
                    ></textarea>

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
                      <input
                        v-model="editForm.origin_url"
                        class="input sm"
                        placeholder="来源链接 (URL)"
                        style="flex: 1"
                      />
                    </div>

                    <div class="editor-licenses">
                      <div class="lic-group">
                        <div class="lic-title">公开授权</div>
                        <label v-for="opt in NET_LICENSE_OPTIONS" :key="opt" class="chk-item">
                          <input type="checkbox" :value="opt" v-model="editForm.netLicenses" />
                          {{ opt }}
                        </label>
                      </div>
                      <div class="lic-group">
                        <div class="lic-title">社团授权</div>
                        <label v-for="opt in GROUP_LICENSE_OPTIONS" :key="opt" class="chk-item">
                          <input type="checkbox" :value="opt" v-model="editForm.groupLicenses" />
                          {{ opt }}
                        </label>
                      </div>
                    </div>

                    <div class="btns">
                      <button class="btn sm" @click="saveEdit(it)">💾 保存修改</button>
                      <button class="btn-ghost sm" @click="editingId = null">取消</button>
                    </div>
                  </div>

                  <div class="m-actions right">
                    <button class="btn-ghost warn sm" @click="lockArtwork(it)">
                      🔒 锁定 (退回审核)
                    </button>
                    <button
                      class="trash-btn"
                      type="button"
                      title="删除"
                      aria-label="删除作品"
                      @click="hardDelete(it, 'list')"
                    >
                      🗑
                    </button>
                  </div>
                </div>
              </article>
            </div>
            <div class="pagination">
              <button class="btn-ghost sm" :disabled="artListPage <= 1" @click="changePage(-1)">
                上一页
              </button>
              <span>第 {{ artListPage }} 页</span>
              <button class="btn-ghost sm" @click="changePage(1)">下一页</button>
            </div>
          </div>
        </div>

        <!-- ================= 评论管理 ================= -->
        <div v-if="mainTab === 'comments'" class="tab-content">
          <div class="sub-tabs">
            <button
              :class="['sub-tab', commentTab === 'pending' && 'on']"
              @click="switchCommentTab('pending')"
            >
              待复核 (AI锁定)
            </button>
            <button
              :class="['sub-tab', commentTab === 'all' && 'on']"
              @click="switchCommentTab('all')"
            >
              全部评论
            </button>
          </div>

          <div class="toolbar" v-if="commentTab === 'all'">
            <input
              v-model="commentSearch"
              class="input sm comment-search"
              placeholder="搜索评论内容..."
            />
          </div>

          <div class="comment-table desktop-only">
            <div class="c-row header">
              <div class="col-user">用户</div>
              <div class="col-content">内容</div>
              <div class="col-status">状态</div>
              <div class="col-action">操作</div>
            </div>
            <div class="c-row has-trash" v-for="c in filteredComments" :key="c.id">
              <div class="col-user">
                <div class="u-name">{{ c.user_name }}</div>
                <div class="u-time">{{ new Date(c.created_at).toLocaleString() }}</div>
              </div>
              <div class="col-content">
                <div class="body-text">{{ c.body }}</div>
                <div v-if="c.ai_reason" class="ai-reason-mini">🤖 {{ c.ai_reason }}</div>
              </div>
              <div class="col-status">
                <span :class="['badge-mini', c.status]">{{
                  c.status === 'flagged' ? 'AI锁定' : c.status === 'hidden' ? '隐藏' : '正常'
                }}</span>
              </div>
              <div class="col-action actions-flex">
                <button
                  v-if="c.status !== 'public'"
                  class="btn-mini success"
                  @click="updateComment(c, 'public')"
                >
                  通过
                </button>
                <button
                  v-if="c.status === 'public'"
                  class="btn-mini warn"
                  @click="updateComment(c, 'flagged')"
                >
                  锁定
                </button>
                <button
                  class="trash-btn"
                  type="button"
                  title="删除"
                  aria-label="删除评论"
                  @click="deleteComment(c)"
                >
                  🗑
                </button>
              </div>
            </div>
            <div v-if="filteredComments.length === 0" class="empty-row">无数据</div>
          </div>

          <div class="comment-list-mobile">
            <article
              class="comment-card has-trash"
              v-for="c in filteredComments"
              :key="`mobile-${c.id}`"
            >
              <div class="comment-card-top">
                <div>
                  <div class="u-name">{{ c.user_name }}</div>
                  <div class="u-time">{{ new Date(c.created_at).toLocaleString() }}</div>
                </div>
                <span :class="['badge-mini', c.status]">{{
                  c.status === 'flagged' ? 'AI锁定' : c.status === 'hidden' ? '隐藏' : '正常'
                }}</span>
              </div>
              <div class="body-text">{{ c.body }}</div>
              <div v-if="c.ai_reason" class="ai-reason-mini">🤖 {{ c.ai_reason }}</div>
              <div class="comment-card-actions">
                <button
                  v-if="c.status !== 'public'"
                  class="btn-mini success"
                  @click="updateComment(c, 'public')"
                >
                  通过
                </button>
                <button
                  v-if="c.status === 'public'"
                  class="btn-mini warn"
                  @click="updateComment(c, 'flagged')"
                >
                  锁定
                </button>
                <button
                  class="trash-btn"
                  type="button"
                  title="删除"
                  aria-label="删除评论"
                  @click="deleteComment(c)"
                >
                  🗑
                </button>
              </div>
            </article>
            <div v-if="filteredComments.length === 0" class="empty-row">无数据</div>
          </div>
        </div>

        <!-- ================= 公会系统 ================= -->
        <div v-if="mainTab === 'guild'" class="tab-content guild-admin">
          <div class="sub-tabs">
            <button
              :class="['sub-tab', guildTab === 'quests' && 'on']"
              @click="switchGuildTab('quests')"
            >
              委托管理
            </button>
            <button
              :class="['sub-tab', guildTab === 'questClaims' && 'on']"
              @click="switchGuildTab('questClaims')"
            >
              委托验收
            </button>
            <button
              :class="['sub-tab', guildTab === 'rewards' && 'on']"
              @click="switchGuildTab('rewards')"
            >
              商品管理
            </button>
            <button
              :class="['sub-tab', guildTab === 'redemptions' && 'on']"
              @click="switchGuildTab('redemptions')"
            >
              兑换审核
            </button>
            <button
              :class="['sub-tab', guildTab === 'budget' && 'on']"
              @click="switchGuildTab('budget')"
            >
              预算管理
            </button>
            <button
              :class="['sub-tab', guildTab === 'rewardSettings' && 'on']"
              @click="switchGuildTab('rewardSettings')"
            >
              奖励规则
            </button>
            <button
              :class="['sub-tab', guildTab === 'ratings' && 'on']"
              @click="switchGuildTab('ratings')"
            >
              评级审核
            </button>
            <button
              :class="['sub-tab', guildTab === 'creators' && 'on']"
              @click="switchGuildTab('creators')"
            >
              创作者档案
            </button>
            <button
              :class="['sub-tab', guildTab === 'profiles' && 'on']"
              @click="switchGuildTab('profiles')"
            >
              访问许可
            </button>
          </div>

          <div v-if="guildMsg" class="guild-msg">{{ guildMsg }}</div>

          <div v-if="guildTab === 'creators'" class="guild-creator-page">
            <div class="creator-production-overview">
              <div class="creator-production-card creator-production-card--window">
                <span>统计窗口</span>
                <b>{{ creatorStatsWindowLabel }}</b>
                <div class="creator-window-switch" role="group" aria-label="统计窗口">
                  <button
                    v-for="option in creatorStatsWindowOptions"
                    :key="option.value"
                    type="button"
                    :class="['creator-window-btn', creatorStatsWindow === option.value && 'on']"
                    @click="setCreatorStatsWindow(option.value)"
                  >
                    {{ option.label }}
                  </button>
                </div>
                <em>生产端真实数据</em>
              </div>
              <div class="creator-production-card">
                <span>画作合计</span>
                <b>{{ formatWhole(creatorStatsOverall.artworksTotal) }} 张</b>
                <em
                  >{{ creatorStatsAverageLabel }}
                  {{ formatAverage(creatorStatsOverall.avgArtworksPerMonth) }} 张/月</em
                >
              </div>
              <div class="creator-production-card">
                <span>金币合计</span>
                <b>{{ formatWhole(creatorStatsOverall.coinsTotal) }}G</b>
                <em
                  >{{ creatorStatsAverageLabel }}
                  {{ formatAverage(creatorStatsOverall.avgCoinsPerMonth) }}G/月</em
                >
              </div>
            </div>

            <div class="two-col-layout guild-creator-layout">
              <div class="col-left" :class="{ 'mobile-hidden': selectedCreator }">
                <div class="toolbar tight">
                  <input
                    v-model="creatorSearch"
                    class="input sm"
                    placeholder="搜索 UID、昵称、联系方式..."
                  />
                  <div class="add-row">
                    <input v-model="newCreatorUid" placeholder="新增 UID" class="input sm" />
                    <button class="btn sm" @click="addCreator">+</button>
                  </div>
                </div>

                <div class="creator-list-v">
                  <div
                    v-for="c in filteredCreators"
                    :key="c.uid"
                    class="creator-item creator-item--guild"
                    :class="{ active: selectedCreator?.uid === c.uid }"
                    @click="selectCreator(c)"
                  >
                    <img :src="c.avatar_url || '/api/art/placeholder/40/40'" class="c-avatar sm" />
                    <div class="c-info-mini">
                      <div class="c-uid">{{ c.name || c.uid }}</div>
                      <div class="c-sub">
                        <span>{{ c.rating || 'F' }}级 / Lv{{ c.level || 1 }}</span>
                        <span>{{ c.accessShortLabel || '档案0' }}</span>
                        <span v-if="creatorContact(c)" class="qq-badge">{{
                          creatorContactLabel(c)
                        }}</span>
                      </div>
                    </div>
                    <div class="c-arr">›</div>
                  </div>
                  <div v-if="filteredCreators.length === 0" class="empty-ph">暂无创作者档案</div>
                </div>
              </div>

              <div class="col-right" :class="{ 'mobile-visible': selectedCreator }">
                <div v-if="selectedCreator" class="creator-detail-panel">
                  <div class="panel-header">
                    <div class="ph-title">
                      创作者档案: {{ selectedCreator.name || selectedCreator.uid }}
                    </div>
                    <div class="panel-actions">
                      <button class="btn-ghost sm mobile-only" @click="selectedCreator = null">
                        返回列表
                      </button>
                      <button
                        class="trash-btn"
                        type="button"
                        title="删除账号"
                        aria-label="删除账号"
                        @click="deleteCreator"
                      >
                        🗑
                      </button>
                    </div>
                  </div>

                  <div class="creator-guild-summary">
                    <div class="creator-guild-card">
                      <span>访问许可</span>
                      <b>{{ selectedCreator.accessLabel || '0级公开档案许可' }}</b>
                      <em>{{ selectedCreator.accessShortLabel || '档案0' }}</em>
                    </div>
                    <div class="creator-guild-card">
                      <span>评级</span>
                      <b>{{
                        selectedCreator.ratingLabel || `${selectedCreator.rating || 'F'}级冒险者`
                      }}</b>
                      <em>等级 Lv{{ selectedCreator.level || 1 }}</em>
                    </div>
                    <div class="creator-guild-card">
                      <span>声望</span>
                      <b>{{ selectedCreator.reputation || 0 }}</b>
                      <em>金币 {{ selectedCreator.coins || 0 }}G</em>
                    </div>
                    <div class="creator-guild-card">
                      <span>作品</span>
                      <b>{{ selectedCreator.totalArtworks || 0 }}</b>
                      <em>
                        通过 {{ selectedCreator.approvedArtworks || 0 }} / 待审
                        {{ selectedCreator.pendingArtworks || 0 }}
                      </em>
                    </div>
                    <div class="creator-guild-card">
                      <span>窗口画作</span>
                      <b>{{ formatWhole(selectedCreatorProduction.artworksTotal) }} 张</b>
                      <em>
                        {{ creatorStatsWindowLabel }} ·
                        {{ creatorStatsAverageLabel }}
                        {{ formatAverage(selectedCreatorProduction.avgArtworksPerMonth) }} 张/月
                      </em>
                    </div>
                    <div class="creator-guild-card">
                      <span>窗口金币</span>
                      <b>{{ formatWhole(selectedCreatorProduction.coinsTotal) }}G</b>
                      <em>
                        {{ creatorStatsWindowLabel }} ·
                        {{ creatorStatsAverageLabel }}
                        {{ formatAverage(selectedCreatorProduction.avgCoinsPerMonth) }}G/月
                      </em>
                    </div>
                    <div class="creator-guild-card">
                      <span>来源</span>
                      <b>个人 {{ selectedCreator.personalArtworks || 0 }}</b>
                      <em>转载 {{ selectedCreator.networkArtworks || 0 }}</em>
                    </div>
                    <div class="creator-guild-card">
                      <span>内容</span>
                      <b>凉宫 {{ selectedCreator.haruhiArtworks || 0 }}</b>
                      <em>其他 {{ selectedCreator.otherArtworks || 0 }}</em>
                    </div>
                    <div class="creator-guild-card">
                      <span>评级计数</span>
                      <b>{{ selectedCreator.haruhiPersonalArtworks || 0 }}</b>
                      <em>已通过凉宫个人作品</em>
                    </div>
                    <div class="creator-guild-card">
                      <span>{{ creatorContactLabel(selectedCreator) }}</span>
                      <b>{{ creatorContact(selectedCreator) || '暂无' }}</b>
                      <em>QQ 为空时使用邮箱</em>
                    </div>
                  </div>

                  <div class="edit-form">
                    <div class="form-group">
                      <label>访问许可</label>
                      <div class="creator-access-row">
                        <select v-model="selectedCreator.accessTier" class="select">
                          <option v-for="a in accessOptions" :key="a.value" :value="a.value">
                            {{ a.label }}
                          </option>
                        </select>
                        <button class="btn-ghost sm" @click="saveProfileAccess(selectedCreator)">
                          保存许可
                        </button>
                      </div>
                    </div>

                    <div class="form-group">
                      <label>头像配置</label>
                      <div class="avatar-uploader">
                        <img
                          :src="
                            previewAvatar ||
                            selectedCreator.avatar_url ||
                            '/api/art/placeholder/80/80'
                          "
                          class="avatar-preview"
                        />
                        <div class="au-actions">
                          <input
                            type="file"
                            ref="fileInput"
                            accept="image/*"
                            @change="handleFileChange"
                            style="display: none"
                          />
                          <button class="btn-ghost sm" @click="$refs.fileInput.click()">
                            选择本地图片
                          </button>
                          <div class="tip-text">支持 jpg, png, webp</div>
                        </div>
                      </div>
                    </div>

                    <div class="form-group">
                      <label>关联 QQ 号</label>
                      <input
                        v-model="editCreatorForm.qq"
                        class="input"
                        placeholder="输入关联的 QQ 号码"
                      />
                      <div class="tip-text">
                        当前页面联系方式：{{ creatorContact(selectedCreator) || '暂无' }}。QQ
                        为空时使用邮箱。
                      </div>
                    </div>

                    <div class="form-actions">
                      <button class="btn" @click="updateCreator" :disabled="!isCreatorModified">
                        保存创作者资料
                      </button>
                      <span v-if="saveMsg" class="save-msg">{{ saveMsg }}</span>
                    </div>
                  </div>

                  <div class="divider"></div>

                  <div class="points-section">
                    <div class="label-lg">金币 / 积分管理</div>
                    <div class="points-action-row">
                      <div class="quick-points">
                        <button
                          v-for="v in [10, 20, 50, -10, -50]"
                          :key="v"
                          class="chip-btn"
                          :class="pointsForm.amount === v && 'active'"
                          @click="pointsForm.amount = v"
                        >
                          {{ v > 0 ? '+' + v : v }}
                        </button>
                      </div>
                      <div class="pa-form">
                        <div class="input-group">
                          <span class="input-prefix">分值</span>
                          <input
                            type="number"
                            v-model.number="pointsForm.amount"
                            class="input points-num sm"
                          />
                        </div>
                        <input
                          v-model="pointsForm.reason"
                          class="input sm"
                          placeholder="变更原因 (必填)"
                          style="flex: 1"
                        />
                        <button class="btn sm" @click="grantPoints" :disabled="!pointsForm.reason">
                          执行
                        </button>
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
                          <span class="ph-time">{{
                            new Date(log.granted_at).toLocaleDateString()
                          }}</span>
                          <span class="ph-val" :class="log.points > 0 ? 'pos' : 'neg'"
                            >{{ log.points > 0 ? '+' : '' }}{{ log.points }}</span
                          >
                          <span class="ph-reason">{{ log.note || log.artwork_title }}</span>
                        </div>
                        <div v-if="!creatorLogs.length" class="empty-ph">暂无积分记录</div>
                      </div>
                    </div>
                  </div>
                </div>
                <div v-else class="empty-select">
                  <div class="icon">🎨</div>
                  <div>请在左侧选择一个创作者档案</div>
                </div>
              </div>
            </div>
          </div>

          <div v-if="guildTab === 'quests'" class="guild-quest-page">
            <section v-if="guildQuestPage === 'list'" class="guild-list single guild-quest-list">
              <div class="panel-header compact guild-list-toolbar">
                <div>
                  <div class="ph-title">已有委托管理</div>
                  <div class="tip-text">查看、编辑、暂停或删除当前公会委托。</div>
                </div>
                <button class="btn sm" @click="openGuildQuestCreate">新增委托</button>
              </div>
              <article
                v-for="quest in guildQuests"
                :key="quest.id"
                class="guild-manage-row guild-quest-row has-trash"
                :class="{ 'is-editing': guildQuestEditingId === quest.id }"
              >
                <div class="guild-row-main">
                  <div class="m-title">{{ quest.title }}</div>
                  <p v-if="quest.description" class="guild-row-desc">{{ quest.description }}</p>
                  <div class="m-info">
                    <span class="tag">{{ quest.questType }}</span>
                    <span class="tag">{{ quest.difficulty }}</span>
                    <span class="tag">评级 {{ quest.requiredRating }}</span>
                    <span class="tag">声望 +{{ quest.rewardReputation }}</span>
                    <span class="tag">金币 +{{ quest.rewardCoins }}</span>
                    <span v-if="quest.deadlineDays && quest.repeatOnComplete" class="tag"
                      >完成后循环</span
                    >
                    <span v-else-if="quest.cycleDays" class="tag">{{
                      quest.cycleDays === 1 ? '每日循环' : `${quest.cycleDays}天循环`
                    }}</span>
                    <span v-if="quest.deadlineDays" class="tag"
                      >限时 {{ quest.deadlineDays }} 天</span
                    >
                    <span v-if="quest.fixedDeadlineAt" class="tag">固定截止</span>
                    <span v-if="quest.autoClaim" class="tag">自动接取</span>
                    <span class="tag">{{ quest.status }}</span>
                  </div>
                </div>
                <div class="guild-row-actions">
                  <button class="btn-ghost sm" @click="editGuildQuest(quest)">编辑</button>
                  <button
                    class="btn-ghost warn sm"
                    @click="
                      setGuildQuestStatus(quest, quest.status === 'active' ? 'paused' : 'active')
                    "
                  >
                    {{ quest.status === 'active' ? '暂停' : '启用' }}
                  </button>
                  <button
                    class="trash-btn"
                    type="button"
                    title="删除"
                    aria-label="删除委托"
                    @click="deleteGuildQuest(quest)"
                  >
                    🗑
                  </button>
                </div>
              </article>
              <div v-if="!guildQuests.length" class="empty-ph">暂无委托，点击上方按钮新增。</div>
            </section>

            <section v-else class="guild-editor guild-form-panel">
              <div class="panel-header compact guild-form-head">
                <div>
                  <div class="ph-title">{{ guildQuestEditingId ? '编辑委托' : '新增委托' }}</div>
                  <div class="tip-text">
                    {{
                      guildQuestEditingId
                        ? '修改后会覆盖当前委托配置。'
                        : '填写后会新增到已有委托列表。'
                    }}
                  </div>
                </div>
                <button class="btn-ghost sm" @click="openGuildQuestList">返回委托列表</button>
              </div>
              <div class="edit-form compact-form guild-form">
                <div class="guild-field guild-field--wide">
                  <div class="guild-field-label-row">
                    <label for="guild-quest-title">委托标题</label>
                    <button class="help-tip" type="button" aria-label="委托标题说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        用户在委托列表中首先看到的名称，建议短句说明目标。
                      </span>
                    </button>
                  </div>
                  <input
                    id="guild-quest-title"
                    v-model="guildQuestForm.title"
                    class="input"
                    placeholder="例如：浏览 5 个画廊作品"
                  />
                </div>
                <div class="guild-field guild-field--wide">
                  <div class="guild-field-label-row">
                    <label for="guild-quest-description">委托说明</label>
                    <button class="help-tip" type="button" aria-label="委托说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        展示给用户的任务说明，可写完成方式、注意事项或活动限制。
                      </span>
                    </button>
                  </div>
                  <textarea
                    id="guild-quest-description"
                    v-model="guildQuestForm.description"
                    class="textarea guild-textarea"
                    placeholder="写给用户看的任务说明"
                  ></textarea>
                </div>
                <div class="guild-form-grid two">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-type">委托类型</label>
                      <button class="help-tip" type="button" aria-label="委托类型说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          决定委托在用户侧的分类展示：日常适合常驻任务，限时适合活动任务，困难适合高门槛任务。
                        </span>
                      </button>
                    </div>
                    <select id="guild-quest-type" v-model="guildQuestForm.questType" class="select">
                      <option value="daily">日常委托</option>
                      <option value="limited">限时委托</option>
                      <option value="hard">困难委托</option>
                      <option value="unknown">未知委托</option>
                    </select>
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-difficulty">难度</label>
                      <button class="help-tip" type="button" aria-label="难度说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          用于前台标识任务挑战程度，方便用户判断投入成本；奖励仍按下方填写值发放。
                        </span>
                      </button>
                    </div>
                    <select
                      id="guild-quest-difficulty"
                      v-model="guildQuestForm.difficulty"
                      class="select"
                    >
                      <option value="easy">easy</option>
                      <option value="normal">normal</option>
                      <option value="hard">hard</option>
                      <option value="chaos">chaos</option>
                    </select>
                  </div>
                </div>
                <div class="guild-form-grid two">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-required-rating">最低评级</label>
                      <button class="help-tip" type="button" aria-label="最低评级说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          限制可接取委托的最低用户评级；F 门槛最低，X 门槛最高。
                        </span>
                      </button>
                    </div>
                    <select
                      id="guild-quest-required-rating"
                      v-model="guildQuestForm.requiredRating"
                      class="select"
                    >
                      <option v-for="r in ratingOptions" :key="r" :value="r">{{ r }}</option>
                    </select>
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-required-access">访问许可</label>
                      <button class="help-tip" type="button" aria-label="访问许可说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          限制可接取委托的访问层级；档案0最开放，后续层级门槛更高。
                        </span>
                      </button>
                    </div>
                    <select
                      id="guild-quest-required-access"
                      v-model="guildQuestForm.requiredAccess"
                      class="select"
                    >
                      <option v-for="a in accessOptions" :key="a.value" :value="a.value">
                        {{ a.label }}
                      </option>
                    </select>
                  </div>
                </div>
                <div class="guild-field guild-field--wide">
                  <div class="guild-field-label-row">
                    <label for="guild-quest-condition">完成条件</label>
                    <button class="help-tip" type="button" aria-label="完成条件说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        决定系统如何判断委托完成；手动验收适合无法自动统计的任务。
                      </span>
                    </button>
                  </div>
                  <select
                    id="guild-quest-condition"
                    v-model="guildQuestForm.conditionKind"
                    class="select"
                  >
                    <option v-for="c in conditionOptions" :key="c.value" :value="c.value">
                      {{ c.label }}
                    </option>
                  </select>
                </div>
                <div class="guild-form-grid three">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-target-count">目标次数</label>
                      <button class="help-tip" type="button" aria-label="目标次数说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          需要完成条件累计达到的次数，例如浏览 5 次就填 5。
                        </span>
                      </button>
                    </div>
                    <input
                      id="guild-quest-target-count"
                      type="number"
                      v-model.number="guildQuestForm.targetCount"
                      class="input"
                      placeholder="1"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-reputation">声望奖励</label>
                      <button class="help-tip" type="button" aria-label="声望奖励说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          完成委托后增加的公会声望，用于评级与成长，不等同于金币。
                        </span>
                      </button>
                    </div>
                    <input
                      id="guild-quest-reputation"
                      type="number"
                      v-model.number="guildQuestForm.rewardReputation"
                      class="input"
                      placeholder="0"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-coins">金币奖励</label>
                      <button class="help-tip" type="button" aria-label="金币奖励说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          完成委托后发放的金币奖励，可用于兑换商品。
                        </span>
                      </button>
                    </div>
                    <input
                      id="guild-quest-coins"
                      type="number"
                      v-model.number="guildQuestForm.rewardCoins"
                      class="input"
                      placeholder="0"
                    />
                  </div>
                </div>
                <div class="guild-form-grid three">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-time-limit-mode">时间限制</label>
                      <button class="help-tip" type="button" aria-label="时间限制说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          同一委托只能选择一种截止逻辑；不单独设置时，循环任务默认到下一次北京时间刷新点截止。
                        </span>
                      </button>
                    </div>
                    <div class="guild-time-limit">
                      <select
                        id="guild-quest-time-limit-mode"
                        v-model="guildQuestForm.timeLimitMode"
                        class="select"
                        @change="normalizeGuildQuestTimeLimit"
                      >
                        <option value="none">不单独设置</option>
                        <option value="days">按天数截止</option>
                        <option value="fixed">固定截止日期</option>
                      </select>
                      <input
                        v-if="guildQuestForm.timeLimitMode === 'days'"
                        id="guild-quest-deadline-days"
                        type="number"
                        min="1"
                        v-model.number="guildQuestForm.deadlineDays"
                        class="input"
                        placeholder="完成期限（天）"
                      />
                      <input
                        v-else-if="guildQuestForm.timeLimitMode === 'fixed'"
                        id="guild-quest-fixed-deadline"
                        type="datetime-local"
                        v-model="guildQuestForm.fixedDeadlineAt"
                        class="input"
                      />
                    </div>
                  </div>
                  <div v-if="guildQuestForm.timeLimitMode === 'days'" class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-repeat-on-complete">完成后循环</label>
                      <button class="help-tip" type="button" aria-label="完成后循环说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          开启后本轮完成并结算后，委托会立即恢复为可再次接取；关闭则只完成一次。
                        </span>
                      </button>
                    </div>
                    <label class="guild-toggle-card" for="guild-quest-repeat-on-complete">
                      <input
                        id="guild-quest-repeat-on-complete"
                        type="checkbox"
                        v-model="guildQuestForm.repeatOnComplete"
                      />
                      <span>
                        <b>{{ guildQuestForm.repeatOnComplete ? '循环开启' : '循环关闭' }}</b>
                        <small>按接取时间计算截止天数，完成后立即刷新。</small>
                      </span>
                    </label>
                  </div>
                  <template v-if="guildQuestForm.timeLimitMode === 'none'">
                    <div class="guild-field">
                      <div class="guild-field-label-row">
                        <label for="guild-quest-cycle-days">循环周期</label>
                        <button class="help-tip" type="button" aria-label="循环周期说明">
                          ?
                          <span class="help-tip__bubble" role="tooltip">
                            填 1 即每日循环；留空表示一次性任务，不会自动进入下一轮。
                          </span>
                        </button>
                      </div>
                      <input
                        id="guild-quest-cycle-days"
                        type="number"
                        min="1"
                        v-model.number="guildQuestForm.cycleDays"
                        class="input"
                        placeholder="日常填 1"
                      />
                    </div>
                    <div class="guild-field">
                      <div class="guild-field-label-row">
                        <label for="guild-quest-reset-hour">北京时间刷新点</label>
                        <button class="help-tip" type="button" aria-label="北京时间刷新点说明">
                          ?
                          <span class="help-tip__bubble" role="tooltip">
                            循环任务按北京时间几点刷新，默认 4 点，即每天凌晨 4 点进入下一轮。
                          </span>
                        </button>
                      </div>
                      <input
                        id="guild-quest-reset-hour"
                        type="number"
                        min="0"
                        max="23"
                        v-model.number="guildQuestForm.cycleResetHour"
                        class="input"
                        placeholder="4"
                      />
                    </div>
                  </template>
                </div>
                <div class="guild-form-grid three">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-sort-order">排序权重</label>
                      <button class="help-tip" type="button" aria-label="排序权重说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          数字越小展示越靠前，越大越靠后。默认 100；需要优先展示可填 10，放后面可填
                          999。
                        </span>
                      </button>
                    </div>
                    <input
                      id="guild-quest-sort-order"
                      type="number"
                      v-model.number="guildQuestForm.sortOrder"
                      class="input"
                      placeholder="100"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-quest-status">状态</label>
                      <button class="help-tip" type="button" aria-label="状态说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          active 为启用，paused 为暂停展示/接取，deleted 为删除状态。
                        </span>
                      </button>
                    </div>
                    <select id="guild-quest-status" v-model="guildQuestForm.status" class="select">
                      <option value="active">active</option>
                      <option value="paused">paused</option>
                      <option value="deleted">deleted</option>
                    </select>
                  </div>
                </div>
                <div class="guild-field guild-field--wide">
                  <div class="guild-field-label-row">
                    <label for="guild-quest-auto-claim">自动接取</label>
                    <button class="help-tip" type="button" aria-label="自动接取说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        开启后用户无需手动接取；达到评级和访问许可后，完成条件会自动计入并发放奖励。
                      </span>
                    </button>
                  </div>
                  <label class="guild-toggle-card" for="guild-quest-auto-claim">
                    <input
                      id="guild-quest-auto-claim"
                      type="checkbox"
                      v-model="guildQuestForm.autoClaim"
                    />
                    <span>
                      <b>{{ guildQuestForm.autoClaim ? '已开启自动接取' : '需要用户手动接取' }}</b>
                      <small>{{
                        guildQuestForm.autoClaim
                          ? '适合日常、浏览、点赞、评论等不需要确认意愿的任务。'
                          : '用户需要先在兑换页点击接取，之后才会记录任务进度。'
                      }}</small>
                    </span>
                  </label>
                </div>
                <div class="btns guild-form-actions">
                  <button class="btn" @click="saveGuildQuest" :disabled="guildSaving">
                    {{ guildQuestEditingId ? '更新委托' : '创建委托' }}
                  </button>
                  <button
                    class="btn-ghost"
                    @click="guildQuestEditingId ? openGuildQuestList() : resetGuildQuestForm()"
                  >
                    {{ guildQuestEditingId ? '取消编辑' : '清空' }}
                  </button>
                </div>
              </div>
            </section>
          </div>

          <div v-if="guildTab === 'rewards'" class="guild-quest-page">
            <section v-if="guildRewardPage === 'list'" class="guild-list single guild-reward-list">
              <div class="panel-header compact guild-list-toolbar">
                <div>
                  <div class="ph-title">已有商品管理</div>
                  <div class="tip-text">查看、编辑、上架或下架当前兑换商品。</div>
                </div>
                <div class="btns">
                  <button class="btn sm" @click="openGuildRewardCategoryCreate">
                    新增商品分类
                  </button>
                  <button class="btn sm" @click="openGuildRewardCreate">新增商品</button>
                </div>
              </div>
              <div v-if="showGuildRewardCategoryForm" class="guild-category-editor">
                <div class="guild-category-form">
                  <label>
                    <span>分类名称</span>
                    <input
                      v-model="guildRewardCategoryForm.name"
                      class="input sm"
                      placeholder="例如：游戏"
                    />
                  </label>
                  <label>
                    <span>排序权重</span>
                    <input
                      v-model.number="guildRewardCategoryForm.sortOrder"
                      class="input sm"
                      type="number"
                      placeholder="100"
                    />
                  </label>
                  <label>
                    <span>状态</span>
                    <select v-model="guildRewardCategoryForm.status" class="select sm">
                      <option value="active">active</option>
                      <option value="paused">paused</option>
                    </select>
                  </label>
                  <div class="guild-category-actions">
                    <button class="btn sm" :disabled="guildSaving" @click="saveGuildRewardCategory">
                      {{ guildRewardCategoryEditingId ? '更新分类' : '新增分类' }}
                    </button>
                    <button class="btn-ghost sm" @click="closeGuildRewardCategoryForm">取消</button>
                  </div>
                </div>
              </div>
              <div v-if="guildRewardCategories.length" class="guild-category-list">
                <article
                  v-for="category in guildRewardCategories"
                  :key="category.id"
                  class="guild-category-item"
                >
                  <div>
                    <b>{{ category.name }}</b>
                    <span>排序 {{ category.sortOrder }} · {{ category.status }}</span>
                  </div>
                  <div class="guild-category-item-actions">
                    <button class="btn-ghost sm" @click="editGuildRewardCategory(category)">
                      编辑
                    </button>
                    <button
                      class="btn-ghost warn sm"
                      @click="
                        setGuildRewardCategoryStatus(
                          category,
                          category.status === 'active' ? 'paused' : 'active'
                        )
                      "
                    >
                      {{ category.status === 'active' ? '停用' : '启用' }}
                    </button>
                  </div>
                </article>
              </div>
              <div class="guild-category-tabs" role="tablist" aria-label="商品分类筛选">
                <button
                  v-for="category in guildRewardCategoryTabs"
                  :key="category.id"
                  type="button"
                  class="guild-category-tab"
                  :class="{ on: guildRewardCategoryFilter === category.id }"
                  @click="guildRewardCategoryFilter = category.id"
                >
                  {{ category.name }}
                </button>
              </div>
              <article
                v-for="reward in filteredGuildRewards"
                :key="reward.id"
                class="guild-manage-row has-trash"
              >
                <div class="guild-row-main guild-reward-row-main">
                  <div v-if="reward.imageUrl" class="guild-reward-thumb">
                    <img :src="reward.imageUrl" :alt="reward.name" />
                  </div>
                  <div class="guild-row-copy">
                    <div class="m-title">{{ reward.name }}</div>
                    <p v-if="reward.description" class="guild-row-desc">{{ reward.description }}</p>
                    <div class="m-info">
                      <span v-if="reward.categoryName" class="tag">{{ reward.categoryName }}</span>
                      <span class="tag">{{ reward.rewardType }}</span>
                      <span class="tag">{{ reward.priceCoins }}G</span>
                      <span class="tag">库存 {{ reward.stock ?? '不限' }}</span>
                      <span class="tag">评级 {{ reward.requiredRating }}</span>
                      <span class="tag">{{ reward.status }}</span>
                    </div>
                  </div>
                </div>
                <div class="guild-row-actions">
                  <button class="btn-ghost sm" @click="editGuildReward(reward)">编辑</button>
                  <button
                    class="btn-ghost warn sm"
                    @click="
                      setGuildRewardStatus(reward, reward.status === 'active' ? 'paused' : 'active')
                    "
                  >
                    {{ reward.status === 'active' ? '下架' : '上架' }}
                  </button>
                  <button
                    class="trash-btn"
                    type="button"
                    title="删除"
                    aria-label="删除商品"
                    @click="deleteGuildReward(reward)"
                  >
                    🗑
                  </button>
                </div>
              </article>
              <div v-if="!filteredGuildRewards.length" class="empty-ph">
                {{ guildRewards.length ? '当前分类暂无商品。' : '暂无商品，点击上方按钮新增。' }}
              </div>
            </section>

            <section v-else class="guild-editor guild-form-panel">
              <div class="panel-header compact guild-form-head">
                <div>
                  <div class="ph-title">{{ guildRewardEditingId ? '编辑商品' : '新增商品' }}</div>
                  <div class="tip-text">
                    {{
                      guildRewardEditingId
                        ? '修改后会覆盖当前商品配置。'
                        : '填写后会新增到已有商品列表。'
                    }}
                  </div>
                </div>
                <button class="btn-ghost sm" @click="openGuildRewardList">返回商品列表</button>
              </div>
              <div class="edit-form compact-form guild-form">
                <div class="guild-field guild-field--wide">
                  <div class="guild-field-label-row">
                    <label for="guild-reward-name">商品名称</label>
                    <button class="help-tip" type="button" aria-label="商品名称说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        用户在兑换列表中看到的商品名，建议清楚说明兑换物。
                      </span>
                    </button>
                  </div>
                  <input
                    id="guild-reward-name"
                    v-model="guildRewardForm.name"
                    class="input"
                    placeholder="例如：头像框兑换券"
                  />
                </div>
                <div class="guild-field guild-field--wide">
                  <div class="guild-field-label-row">
                    <label for="guild-reward-description">商品说明</label>
                    <button class="help-tip" type="button" aria-label="商品说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        展示给用户的兑换说明，可写发放方式、限制条件或有效期。
                      </span>
                    </button>
                  </div>
                  <textarea
                    id="guild-reward-description"
                    v-model="guildRewardForm.description"
                    class="textarea guild-textarea"
                    placeholder="写给用户看的兑换说明"
                  ></textarea>
                </div>
                <div class="guild-field guild-field--wide">
                  <div class="guild-field-label-row">
                    <label for="guild-reward-category">商品分类</label>
                    <button class="help-tip" type="button" aria-label="商品分类说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        未设置分类的商品只会在“所有”分类中显示。
                      </span>
                    </button>
                  </div>
                  <select
                    id="guild-reward-category"
                    v-model="guildRewardForm.categoryId"
                    class="select"
                  >
                    <option :value="null">不设分类</option>
                    <option
                      v-for="category in guildRewardCategories"
                      :key="category.id"
                      :value="category.id"
                    >
                      {{ category.name }}{{ category.status !== 'active' ? '（停用）' : '' }}
                    </option>
                  </select>
                </div>
                <div class="guild-form-grid three">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-reward-type">商品类型</label>
                      <button class="help-tip" type="button" aria-label="商品类型说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          虚拟物品适合站内权益，实体物品适合需要线下或邮寄发放的奖励。
                        </span>
                      </button>
                    </div>
                    <select
                      id="guild-reward-type"
                      v-model="guildRewardForm.rewardType"
                      class="select"
                    >
                      <option value="virtual">虚拟物品</option>
                      <option value="physical">实体物品</option>
                    </select>
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-reward-price">金币价格</label>
                      <button class="help-tip" type="button" aria-label="金币价格说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          用户兑换时需要消耗的金币数量。
                        </span>
                      </button>
                    </div>
                    <input
                      id="guild-reward-price"
                      type="number"
                      v-model.number="guildRewardForm.priceCoins"
                      class="input"
                      placeholder="0"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-reward-stock">库存</label>
                      <button class="help-tip" type="button" aria-label="库存说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          限制可兑换数量；填 -1 表示不限库存。
                        </span>
                      </button>
                    </div>
                    <input
                      id="guild-reward-stock"
                      type="number"
                      v-model.number="guildRewardForm.stock"
                      class="input"
                      placeholder="-1"
                    />
                    <small>-1 表示不限库存</small>
                  </div>
                </div>
                <div class="guild-form-grid two">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-reward-required-rating">最低评级</label>
                      <button class="help-tip" type="button" aria-label="最低评级说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          限制可兑换商品的最低用户评级；F 门槛最低，X 门槛最高。
                        </span>
                      </button>
                    </div>
                    <select
                      id="guild-reward-required-rating"
                      v-model="guildRewardForm.requiredRating"
                      class="select"
                    >
                      <option v-for="r in ratingOptions" :key="r" :value="r">{{ r }}</option>
                    </select>
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-reward-required-access">访问许可</label>
                      <button class="help-tip" type="button" aria-label="访问许可说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          限制可兑换商品的访问层级；档案0最开放，后续层级门槛更高。
                        </span>
                      </button>
                    </div>
                    <select
                      id="guild-reward-required-access"
                      v-model="guildRewardForm.requiredAccess"
                      class="select"
                    >
                      <option v-for="a in accessOptions" :key="a.value" :value="a.value">
                        {{ a.label }}
                      </option>
                    </select>
                  </div>
                </div>
                <div class="guild-field guild-field--wide">
                  <div class="guild-field-label-row">
                    <label for="guild-reward-image">展示图</label>
                    <button class="help-tip" type="button" aria-label="展示图说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        商品展示图，可留空；上传后用于兑换列表中的图片展示，保存商品后正式生效。
                      </span>
                    </button>
                  </div>
                  <div class="reward-image-uploader">
                    <input
                      id="guild-reward-image"
                      ref="guildRewardImageInput"
                      class="reward-image-input"
                      type="file"
                      accept="image/*"
                      @change="handleGuildRewardImageChange"
                    />
                    <div
                      class="reward-image-preview"
                      :class="{ 'is-empty': !guildRewardForm.imageUrl }"
                    >
                      <img v-if="guildRewardForm.imageUrl" :src="guildRewardForm.imageUrl" alt="" />
                      <span v-else>未上传展示图</span>
                    </div>
                    <div class="reward-image-tools">
                      <button
                        class="btn-ghost sm"
                        type="button"
                        :disabled="guildRewardImageUploading"
                        @click="openGuildRewardImagePicker"
                      >
                        {{
                          guildRewardImageUploading
                            ? '上传中...'
                            : guildRewardForm.imageUrl
                              ? '更换图片'
                              : '上传图片'
                        }}
                      </button>
                      <button
                        v-if="guildRewardForm.imageUrl"
                        class="btn-ghost danger sm"
                        type="button"
                        :disabled="guildRewardImageUploading"
                        @click="clearGuildRewardImage"
                      >
                        清除图片
                      </button>
                      <small>支持 JPG、PNG、WebP、GIF、SVG、BMP、AVIF 等常见展示图片</small>
                    </div>
                  </div>
                </div>
                <div class="guild-form-grid two">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-reward-sort-order">排序权重</label>
                      <button class="help-tip" type="button" aria-label="排序权重说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          数字越小展示越靠前，越大越靠后。默认 100；需要优先展示可填 10，放后面可填
                          999。
                        </span>
                      </button>
                    </div>
                    <input
                      id="guild-reward-sort-order"
                      type="number"
                      v-model.number="guildRewardForm.sortOrder"
                      class="input"
                      placeholder="100"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="guild-reward-status">状态</label>
                      <button class="help-tip" type="button" aria-label="状态说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          active 为上架，paused 为下架暂停兑换，deleted 为删除状态。
                        </span>
                      </button>
                    </div>
                    <select
                      id="guild-reward-status"
                      v-model="guildRewardForm.status"
                      class="select"
                    >
                      <option value="active">active</option>
                      <option value="paused">paused</option>
                      <option value="deleted">deleted</option>
                    </select>
                  </div>
                </div>
                <div class="btns guild-form-actions">
                  <button
                    class="btn"
                    @click="saveGuildReward"
                    :disabled="guildSaving || guildRewardImageUploading"
                  >
                    {{ guildRewardEditingId ? '更新商品' : '保存商品' }}
                  </button>
                  <button
                    class="btn-ghost"
                    :disabled="guildRewardImageUploading"
                    @click="guildRewardEditingId ? openGuildRewardList() : resetGuildRewardForm()"
                  >
                    {{ guildRewardEditingId ? '取消编辑' : '清空' }}
                  </button>
                </div>
              </div>
            </section>
          </div>

          <div v-if="guildTab === 'questClaims'" class="guild-list single">
            <article v-for="item in guildQuestClaims" :key="item.id" class="guild-manage-row">
              <div class="guild-row-main">
                <div class="m-title">{{ item.questTitle }} · {{ item.name || item.uid }}</div>
                <div class="m-info">
                  <span class="tag">{{ guildClaimStatusLabel(item.status) }}</span>
                  <span class="tag">进度 {{ item.progress }}/{{ item.targetCount }}</span>
                  <span class="tag">声望 +{{ item.rewardReputation }}</span>
                  <span v-if="item.rewardCoins > 0" class="tag">金币 +{{ item.rewardCoins }}</span>
                  <span v-if="item.claimedAt" class="tag"
                    >接取 {{ formatDateTime(item.claimedAt) }}</span
                  >
                  <span v-if="item.cycleEndAt" class="tag"
                    >截止 {{ formatDateTime(item.cycleEndAt) }}</span
                  >
                  <span v-if="item.reviewedAt" class="tag"
                    >处理 {{ formatDateTime(item.reviewedAt) }}</span
                  >
                  <span v-if="item.adminNote" class="tag">备注：{{ item.adminNote }}</span>
                </div>
                <div v-if="item.submittedArtworks?.length" class="guild-claim-artworks">
                  <a
                    v-for="artwork in item.submittedArtworks"
                    :key="artwork.id"
                    class="guild-claim-artwork"
                    :href="`/gallery?artwork=${artwork.id}`"
                    target="_blank"
                    rel="noreferrer"
                  >
                    <span class="guild-claim-artwork__thumb">
                      <img
                        v-if="artwork.image_url"
                        :src="thumbUrl(artwork.image_url, 240)"
                        :alt="artwork.title || '提交作品'"
                      />
                      <i v-else>ART</i>
                    </span>
                    <span class="guild-claim-artwork__copy">
                      <b>{{ artwork.title || `作品 #${artwork.id}` }}</b>
                      <small>{{ formatDateTime(artwork.publishedAt || artwork.created_at) }}</small>
                      <em>{{ artwork.status }}</em>
                    </span>
                  </a>
                </div>
                <div v-else class="guild-claim-artworks-empty">尚未提交作品</div>
              </div>
              <div v-if="item.status === 'active'" class="guild-row-actions">
                <button class="btn-ghost sm" @click="approveQuestClaim(item)">批准</button>
                <button class="btn-ghost danger sm" @click="rejectQuestClaim(item)">拒绝</button>
              </div>
            </article>
            <div v-if="!guildQuestClaims.length" class="empty-ph">暂无待验收委托</div>
          </div>

          <div v-if="guildTab === 'redemptions'" class="guild-list single">
            <article
              v-for="item in guildRedemptions"
              :key="item.id"
              class="guild-manage-row is-clickable"
              @click="openRedemptionDetail(item)"
            >
              <div>
                <div class="m-title">{{ item.rewardName }} · {{ item.name || item.uid }}</div>
                <div class="m-info">
                  <span class="tag">{{ redemptionTypeLabel(item.rewardType) }}</span>
                  <span class="tag">{{ item.frozenCoins }}G</span>
                  <span class="tag">{{ redemptionStatusLabel(item.status) }}</span>
                  <span v-if="item.userNote" class="tag">备注：{{ item.userNote }}</span>
                </div>
              </div>
              <div class="guild-row-actions" @click.stop>
                <button class="btn-ghost sm" @click="openRedemptionDetail(item)">详情</button>
                <button
                  v-if="item.status === 'pending'"
                  class="btn-ghost sm"
                  @click="approveRedemption(item)"
                >
                  批准
                </button>
                <button
                  v-if="item.status === 'approved'"
                  class="btn-ghost sm"
                  @click="fulfillRedemption(item)"
                >
                  发放
                </button>
                <button
                  v-if="item.status === 'pending'"
                  class="btn-ghost danger sm"
                  @click="rejectRedemption(item)"
                >
                  拒绝
                </button>
              </div>
            </article>
            <div v-if="!guildRedemptions.length" class="empty-ph">暂无兑换申请</div>
          </div>

          <div v-if="guildTab === 'budget'" class="guild-budget-page">
            <div class="guild-budget-summary">
              <div class="guild-budget-card">
                <span>当前可用预算</span>
                <b>{{ formatWhole(guildBudget.summary.currentBudgetCoins) }}G</b>
                <em>同步到补给兑换柜台</em>
              </div>
              <div class="guild-budget-card">
                <span>累计补给</span>
                <b>{{ formatWhole(guildBudget.summary.totalSupplyCoins) }}G</b>
                <em>按 {{ guildBudget.summary.coinPerRmb || 15 }}G≈1元折算</em>
              </div>
              <div class="guild-budget-card">
                <span>累计消耗</span>
                <b>{{ formatWhole(guildBudget.summary.spentPhysicalCoins) }}G</b>
                <em>已批准/已发放实体兑换</em>
              </div>
            </div>

            <section class="guild-list single guild-budget-list">
              <div class="panel-header compact guild-list-toolbar">
                <div>
                  <div class="ph-title">预算列表</div>
                  <div class="tip-text">查看补给台账与实体兑换消耗。</div>
                </div>
                <div class="guild-budget-toolbar">
                  <div class="guild-budget-switch" role="group" aria-label="预算列表类型">
                    <button
                      :class="['guild-budget-switch-btn', guildBudgetTab === 'supplies' && 'on']"
                      type="button"
                      @click="guildBudgetTab = 'supplies'"
                    >
                      补给
                    </button>
                    <button
                      :class="['guild-budget-switch-btn', guildBudgetTab === 'spends' && 'on']"
                      type="button"
                      @click="guildBudgetTab = 'spends'"
                    >
                      消耗
                    </button>
                  </div>
                  <button
                    v-if="guildBudgetTab === 'supplies'"
                    class="btn sm"
                    type="button"
                    @click="showBudgetSupplyForm = !showBudgetSupplyForm"
                  >
                    新增预算补给
                  </button>
                </div>
              </div>

              <div
                v-if="showBudgetSupplyForm && guildBudgetTab === 'supplies'"
                class="guild-budget-form"
              >
                <label>
                  <span>预算类型</span>
                  <select v-model="guildBudgetSupplyForm.budgetType" class="input sm">
                    <option value="quarterly">季度预算</option>
                    <option value="activity">活动预算</option>
                    <option value="other">其他预算</option>
                  </select>
                </label>
                <label>
                  <span>预算额度</span>
                  <div class="guild-budget-amount-field">
                    <div class="guild-budget-unit-switch" role="group" aria-label="预算单位">
                      <button
                        :class="[
                          'guild-budget-switch-btn',
                          guildBudgetSupplyForm.amountUnit === 'rmb' && 'on',
                        ]"
                        type="button"
                        @click="guildBudgetSupplyForm.amountUnit = 'rmb'"
                      >
                        元
                      </button>
                      <button
                        :class="[
                          'guild-budget-switch-btn',
                          guildBudgetSupplyForm.amountUnit === 'coins' && 'on',
                        ]"
                        type="button"
                        @click="guildBudgetSupplyForm.amountUnit = 'coins'"
                      >
                        金币
                      </button>
                    </div>
                    <input
                      v-model.number="guildBudgetSupplyForm.amount"
                      class="input sm"
                      min="1"
                      step="1"
                      type="number"
                      placeholder="填写数量"
                    />
                  </div>
                </label>
                <div class="guild-budget-form-actions">
                  <span>将增加 {{ formatWhole(budgetSupplyPreviewCoins) }}G 库存预算</span>
                  <button
                    class="btn sm"
                    type="button"
                    :disabled="guildSaving"
                    @click="createBudgetSupply"
                  >
                    新增补给
                  </button>
                </div>
              </div>

              <template v-if="guildBudgetTab === 'supplies'">
                <article
                  v-for="item in guildBudget.supplies"
                  :key="item.id"
                  class="guild-manage-row"
                >
                  <div>
                    <div class="m-title">
                      {{ item.budgetTypeLabel }} · {{ formatWhole(item.amountCoins) }}G
                    </div>
                    <div class="m-info">
                      <span class="tag">{{ item.amountInput }}{{ item.amountUnitLabel }}</span>
                      <span class="tag">{{ formatDateTime(item.createdAt) }}</span>
                    </div>
                  </div>
                </article>
                <div v-if="!guildBudget.supplies.length" class="empty-ph">暂无预算补给记录</div>
              </template>

              <template v-else>
                <article v-for="item in guildBudget.spends" :key="item.id" class="guild-manage-row">
                  <div>
                    <div class="m-title">{{ item.rewardName }} · {{ item.name || item.uid }}</div>
                    <div class="m-info">
                      <span class="tag"
                        >{{ formatWhole(item.spentCoins || item.frozenCoins) }}G</span
                      >
                      <span class="tag">{{ item.status }}</span>
                      <span class="tag">{{
                        formatDateTime(item.fulfilledAt || item.reviewedAt || item.createdAt)
                      }}</span>
                    </div>
                  </div>
                </article>
                <div v-if="!guildBudget.spends.length" class="empty-ph">暂无实体补给消耗</div>
              </template>
            </section>
          </div>

          <div v-if="guildTab === 'rewardSettings'" class="guild-reward-settings-page">
            <section class="guild-editor guild-reward-settings-panel">
              <div class="panel-header compact">
                <div>
                  <div class="ph-title">投稿奖励规则</div>
                  <div class="tip-text">控制作品审核通过后发放的金币与公会声望。</div>
                </div>
                <button
                  class="btn-ghost sm"
                  type="button"
                  :disabled="rewardSettingsLoading"
                  @click="loadRewardSettings"
                >
                  刷新配置
                </button>
              </div>

              <div class="reward-settings-body">
                <div class="reward-settings-note">
                  <div class="guild-field-label-row">
                    <span>快照规则</span>
                    <button class="help-tip" type="button" aria-label="奖励快照说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        作品首次通过时会锁定当时奖励；之后改倍率不会追溯旧作品。
                      </span>
                    </button>
                  </div>
                  <p>
                    倍率只影响之后首次审核通过的作品。撤稿会扣回金币，但不会扣回已获得的公会声望。
                  </p>
                </div>

                <div class="guild-form-grid two reward-settings-grid">
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="reward-haruhi-points">凉宫个人作品基础金币</label>
                      <button class="help-tip" type="button" aria-label="基础金币说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          审核通过后发放的画廊积分，也就是兑换柜台使用的金币。
                        </span>
                      </button>
                    </div>
                    <input
                      id="reward-haruhi-points"
                      v-model.number="rewardSettingsForm.personalHaruhiPoints"
                      class="input"
                      min="0"
                      step="1"
                      type="number"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="reward-haruhi-reputation">凉宫个人作品基础声望</label>
                      <button class="help-tip" type="button" aria-label="基础声望说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          审核通过后增加的公会声望，用于等级与评级条件，不等同于金币。
                        </span>
                      </button>
                    </div>
                    <input
                      id="reward-haruhi-reputation"
                      v-model.number="rewardSettingsForm.personalHaruhiReputation"
                      class="input"
                      min="0"
                      step="1"
                      type="number"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="reward-other-points">其他个人作品基础金币</label>
                      <button class="help-tip" type="button" aria-label="其他作品金币说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          非凉宫个人作品通过审核时使用此基础金币；非个人作品默认不发投稿金币。
                        </span>
                      </button>
                    </div>
                    <input
                      id="reward-other-points"
                      v-model.number="rewardSettingsForm.personalOtherPoints"
                      class="input"
                      min="0"
                      step="1"
                      type="number"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="reward-other-reputation">其他个人作品基础声望</label>
                      <button class="help-tip" type="button" aria-label="其他作品声望说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          非凉宫个人作品通过审核时使用此基础声望；非个人作品默认不发投稿声望。
                        </span>
                      </button>
                    </div>
                    <input
                      id="reward-other-reputation"
                      v-model.number="rewardSettingsForm.personalOtherReputation"
                      class="input"
                      min="0"
                      step="1"
                      type="number"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="reward-points-multiplier">当前金币倍率</label>
                      <button class="help-tip" type="button" aria-label="金币倍率说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          节假日可手动改成 3-5 倍；结束后需要手动改回 1 倍。
                        </span>
                      </button>
                    </div>
                    <input
                      id="reward-points-multiplier"
                      v-model.number="rewardSettingsForm.pointsMultiplier"
                      class="input"
                      min="0"
                      max="20"
                      step="0.1"
                      type="number"
                    />
                  </div>
                  <div class="guild-field">
                    <div class="guild-field-label-row">
                      <label for="reward-reputation-multiplier">当前声望倍率</label>
                      <button class="help-tip" type="button" aria-label="声望倍率说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          只影响投稿审核通过带来的声望，不影响委托配置里的声望奖励。
                        </span>
                      </button>
                    </div>
                    <input
                      id="reward-reputation-multiplier"
                      v-model.number="rewardSettingsForm.reputationMultiplier"
                      class="input"
                      min="0"
                      max="20"
                      step="0.1"
                      type="number"
                    />
                  </div>
                </div>

                <div class="reward-preview-grid">
                  <div class="reward-preview-card">
                    <span>凉宫个人作品</span>
                    <b>{{ formatWhole(rewardSettingsPreview.haruhi.points) }}G</b>
                    <em>声望 +{{ formatWhole(rewardSettingsPreview.haruhi.reputation) }}</em>
                  </div>
                  <div class="reward-preview-card">
                    <span>其他个人作品</span>
                    <b>{{ formatWhole(rewardSettingsPreview.other.points) }}G</b>
                    <em>声望 +{{ formatWhole(rewardSettingsPreview.other.reputation) }}</em>
                  </div>
                  <div class="reward-preview-card reward-preview-card--muted">
                    <span>非个人作品</span>
                    <b>0G</b>
                    <em>默认不发投稿奖励</em>
                  </div>
                </div>

                <div class="guild-form-actions reward-settings-actions">
                  <span v-if="rewardSettingsForm.updatedAt" class="tip-text">
                    上次更新：{{ formatDateTime(rewardSettingsForm.updatedAt) }}
                  </span>
                  <button
                    class="btn"
                    type="button"
                    :disabled="rewardSettingsSaving"
                    @click="saveRewardSettings"
                  >
                    {{ rewardSettingsSaving ? '保存中...' : '保存奖励规则' }}
                  </button>
                </div>
              </div>
            </section>
          </div>

          <div v-if="guildTab === 'ratings'" class="guild-list single">
            <article v-for="item in guildRatings" :key="item.id" class="guild-manage-row">
              <div>
                <div class="m-title">
                  {{ item.name || item.uid }}：{{ item.fromRating }} → {{ item.targetRating }}
                </div>
                <div class="m-info">
                  <span class="tag">声望 {{ item.reputationSnapshot }}</span>
                  <span class="tag">凉宫作品 {{ item.haruhiCountSnapshot }}</span>
                  <span class="tag">{{ item.status }}</span>
                  <span v-if="item.userNote" class="tag">说明：{{ item.userNote }}</span>
                </div>
              </div>
              <div class="guild-row-actions">
                <button class="btn-ghost sm" @click="approveRating(item)">批准</button>
                <button class="btn-ghost danger sm" @click="rejectRating(item)">拒绝</button>
              </div>
            </article>
            <div v-if="!guildRatings.length" class="empty-ph">暂无评级申请</div>
          </div>

          <div v-if="guildTab === 'profiles'" class="guild-list single">
            <article v-for="item in guildProfiles" :key="item.uid" class="guild-manage-row">
              <div>
                <div class="m-title">{{ item.name || item.uid }}</div>
                <div class="m-info">
                  <span class="tag">评级 {{ item.rating }}</span>
                  <span class="tag">Lv{{ item.level }}</span>
                  <span class="tag">声望 {{ item.reputation }}</span>
                  <span class="tag">金币 {{ item.coins || 0 }}G</span>
                </div>
              </div>
              <div class="guild-row-actions access-editor">
                <div class="guild-inline-field">
                  <div class="guild-field-label-row">
                    <span>访问许可</span>
                    <button class="help-tip" type="button" aria-label="访问许可说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        给该用户分配公会档案访问层级；档案0最开放，后续层级可访问更高权限内容。
                      </span>
                    </button>
                  </div>
                  <select v-model="item.accessTier" class="select sm">
                    <option v-for="a in accessOptions" :key="a.value" :value="a.value">
                      {{ a.label }}
                    </option>
                  </select>
                </div>
                <button class="btn-ghost sm" @click="saveProfileAccess(item)">保存许可</button>
              </div>
            </article>
            <div v-if="!guildProfiles.length" class="empty-ph">暂无公会档案</div>
          </div>
        </div>

        <!-- ================= 公告管理 ================= -->
        <div v-if="mainTab === 'announcements'" class="tab-content guild-admin">
          <div v-if="annMsg" class="guild-msg">{{ annMsg }}</div>
          <div class="ann-admin-layout">
            <section v-if="annPage === 'form'" class="guild-editor ann-editor-panel">
              <div class="panel-header compact ann-editor-head">
                <div>
                  <div class="ph-title">{{ annEditingId ? '编辑公告' : '新增公告' }}</div>
                  <div class="tip-text">
                    {{
                      annEditingId ? '正在编辑 #' + annEditingId : '填写后会新增到已有公告列表。'
                    }}
                  </div>
                </div>
                <button class="btn-ghost sm" @click="openAnnouncementList">返回公告列表</button>
              </div>
              <div class="ann-editor-form">
                <div class="ann-field ann-field--title">
                  <div class="admin-field-label-row">
                    <label for="ann-title">标题</label>
                    <button class="help-tip" type="button" aria-label="公告标题说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        公告列表和详情页显示的主标题，建议简洁说明通知主题。
                      </span>
                    </button>
                  </div>
                  <input
                    id="ann-title"
                    v-model="annForm.title"
                    class="input"
                    placeholder="公告标题"
                  />
                </div>
                <div class="ann-meta-grid">
                  <div class="ann-field">
                    <div class="admin-field-label-row">
                      <label for="ann-category">分类</label>
                      <button class="help-tip" type="button" aria-label="公告分类说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          活动公告用于社团活动，维护公告用于功能维护、停机或系统调整。
                        </span>
                      </button>
                    </div>
                    <select id="ann-category" v-model="annForm.category" class="select">
                      <option value="activity">活动公告</option>
                      <option value="maintenance">维护公告</option>
                    </select>
                  </div>
                  <div class="ann-field">
                    <div class="admin-field-label-row">
                      <label for="ann-status">状态</label>
                      <button class="help-tip" type="button" aria-label="公告状态说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          已发布会对用户展示；草稿仅保存在后台，不进入公开公告栏。
                        </span>
                      </button>
                    </div>
                    <select id="ann-status" v-model="annForm.status" class="select">
                      <option value="published">已发布</option>
                      <option value="draft">草稿</option>
                    </select>
                  </div>
                  <div class="ann-field">
                    <div class="admin-field-label-row">
                      <label for="ann-published-at">发布时间</label>
                      <button class="help-tip" type="button" aria-label="发布时间说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          用于公告日期展示和排序；留空时由后端按默认时间处理。
                        </span>
                      </button>
                    </div>
                    <input
                      id="ann-published-at"
                      type="date"
                      v-model="annForm.publishedAt"
                      class="input"
                    />
                  </div>
                  <div class="ann-field">
                    <div class="admin-field-label-row">
                      <span>置顶</span>
                      <button class="help-tip" type="button" aria-label="置顶公告说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          置顶公告会优先显示在公告列表前面，适合重要通知。
                        </span>
                      </button>
                    </div>
                    <label class="ann-pin ann-pin--card"
                      ><input type="checkbox" v-model="annForm.pinned" /> 置顶公告</label
                    >
                  </div>
                </div>
                <div class="ann-field">
                  <div class="admin-field-label-row">
                    <label for="ann-summary">摘要</label>
                    <button class="help-tip" type="button" aria-label="公告摘要说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        一句话概述，会出现在公告列表和详情页标题下方。
                      </span>
                    </button>
                  </div>
                  <input
                    id="ann-summary"
                    v-model="annForm.summary"
                    class="input"
                    placeholder="摘要（一句话概述）"
                  />
                </div>
                <div class="ann-field ann-field--body">
                  <div class="admin-field-label-row">
                    <label for="ann-body">正文</label>
                    <button class="help-tip" type="button" aria-label="公告正文说明">
                      ?
                      <span class="help-tip__bubble" role="tooltip">
                        公告正文会保留换行；长公告在详情页内滚动显示。
                      </span>
                    </button>
                  </div>
                  <textarea
                    id="ann-body"
                    v-model="annForm.body"
                    class="textarea ann-body-editor"
                    placeholder="公告正文"
                  ></textarea>
                </div>
                <div class="ann-form-footer">
                  <div class="ann-field ann-field--tags">
                    <div class="admin-field-label-row">
                      <label for="ann-tags">标签</label>
                      <button class="help-tip" type="button" aria-label="公告标签说明">
                        ?
                        <span class="help-tip__bubble" role="tooltip">
                          用逗号分隔多个标签，可留空；用于公告详情页辅助归类。
                        </span>
                      </button>
                    </div>
                    <input
                      id="ann-tags"
                      v-model="annForm.tags"
                      class="input"
                      placeholder="标签，逗号分隔（可空）"
                    />
                  </div>
                  <div class="btns ann-editor-actions">
                    <button class="btn" @click="saveAnnouncement" :disabled="annSaving">
                      {{ annEditingId ? '更新公告' : '发布公告' }}
                    </button>
                    <button
                      class="btn-ghost"
                      @click="annEditingId ? openAnnouncementList() : resetAnnForm()"
                    >
                      {{ annEditingId ? '取消编辑' : '清空' }}
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <section v-else class="guild-list ann-list-panel">
              <div class="panel-header compact ann-list-head">
                <div>
                  <div class="ph-title">已有公告管理</div>
                  <div class="tip-text">
                    共 {{ announcements.length }} 条，点击编辑会进入公告表单
                  </div>
                </div>
                <div class="btns">
                  <button class="btn-ghost sm" @click="loadAnnouncementsAdmin">刷新</button>
                  <button class="btn sm" @click="openAnnouncementCreate">新增公告</button>
                </div>
              </div>
              <article
                v-for="a in announcements"
                :key="a.id"
                class="guild-manage-row ann-manage-row has-trash"
                :class="{ 'is-editing': annEditingId === a.id }"
              >
                <div class="ann-row-main">
                  <div class="m-title">{{ a.title }}</div>
                  <div class="m-info">
                    <span class="tag">{{
                      a.category === 'maintenance' ? '维护公告' : '活动公告'
                    }}</span>
                    <span class="tag">{{ a.status === 'draft' ? '草稿' : '已发布' }}</span>
                    <span v-if="a.pinned" class="tag">置顶</span>
                    <span class="tag">{{ (a.publishedAt || '').slice(0, 10) }}</span>
                  </div>
                  <p v-if="a.summary" class="ann-row-summary">{{ a.summary }}</p>
                  <p v-if="a.body" class="ann-row-body">{{ a.body }}</p>
                </div>
                <div class="guild-row-actions">
                  <button class="btn-ghost sm" @click="editAnnouncement(a)">编辑</button>
                  <button
                    class="trash-btn"
                    type="button"
                    title="删除"
                    aria-label="删除公告"
                    @click="deleteAnnouncement(a)"
                  >
                    🗑
                  </button>
                </div>
              </article>
              <div v-if="!announcements.length" class="empty-ph">暂无公告，点击上方按钮新增。</div>
            </section>
          </div>
        </div>
      </main>
    </div>

    <transition name="modal">
      <div
        v-if="selectedRedemption"
        class="admin-modal"
        @click.self="closeRedemptionDetail"
      >
        <article class="admin-dialog redemption-detail" role="dialog" aria-modal="true">
          <button
            class="dialog-close"
            type="button"
            aria-label="关闭"
            @click="closeRedemptionDetail"
          >
            ×
          </button>
          <span class="dialog-eyebrow">Redemption</span>
          <h2>{{ selectedRedemption.rewardName }}</h2>
          <div class="redemption-detail-grid">
            <span>用户</span>
            <b>{{ selectedRedemption.name || selectedRedemption.uid }}</b>
            <span>类型</span>
            <b>{{ redemptionTypeLabel(selectedRedemption.rewardType) }}</b>
            <span>金币</span>
            <b>{{ selectedRedemption.frozenCoins }}G</b>
            <span>状态</span>
            <b>{{ redemptionStatusLabel(selectedRedemption.status) }}</b>
            <span v-if="selectedRedemption.userNote">用户备注</span>
            <p v-if="selectedRedemption.userNote">{{ selectedRedemption.userNote }}</p>
          </div>
          <div class="redemption-history">
            <div
              v-for="entry in redemptionHistory(selectedRedemption)"
              :key="`${entry.kind}-${entry.at}`"
              class="redemption-history-item"
            >
              <i></i>
              <div>
                <b>{{ entry.label }}</b>
                <time>{{ formatDateTime(entry.at) }}</time>
                <p v-if="entry.note">{{ entry.note }}</p>
              </div>
            </div>
          </div>
          <div class="guild-row-actions">
            <button
              v-if="selectedRedemption.status === 'pending'"
              class="btn-ghost sm"
              @click="approveRedemption(selectedRedemption)"
            >
              批准
            </button>
            <button
              v-if="selectedRedemption.status === 'approved'"
              class="btn-ghost sm"
              @click="fulfillRedemption(selectedRedemption)"
            >
              发放
            </button>
            <button
              v-if="selectedRedemption.status === 'pending'"
              class="btn-ghost danger sm"
              @click="rejectRedemption(selectedRedemption)"
            >
              拒绝
            </button>
          </div>
        </article>
      </div>
    </transition>

    <!-- 作品预览弹窗 -->
    <ArtworkModal v-model="showPreview" :item="previewItem" />
  </section>
</template>

<script setup>
import { onMounted, ref, computed, watch } from 'vue'
import { useAdminStore } from '../stores/adminStore.js'
import { api, thumbUrl } from '../services/api.js'
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
const mainTab = ref('images') // images, comments, guild, announcements
const imageTab = ref('audit') // audit, list
const commentTab = ref('pending') // pending, all

// --- 图片管理数据 ---
const approvedList = ref([])
const artListPage = ref(1)
const artListFilter = ref({ content: 'all', source: 'all', q: '' })
const editingId = ref(null)
const editForm = ref({
  title: '',
  description: '',
  tags: '',
  source_type: 'personal',
  content_type: 'haruhi',
  origin_url: '',
  netLicenses: [],
  groupLicenses: [],
})

const NET_LICENSE_OPTIONS = [
  '可在b站、小红书等社交媒体转载',
  '允许用于视频等个人创作',
  '允许用于制作无料发放',
]

const GROUP_LICENSE_OPTIONS = [
  '允许应援团社交媒体官方账号转载',
  '允许用于应援团官方视频/游戏等创作企划',
  '允许制作无料周边发放',
  '允许制作贩售周边',
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
const creatorStatsWindowOptions = [
  { value: 'week', label: '近1周' },
  { value: '3m', label: '近3个月' },
  { value: '6m', label: '近半年' },
  { value: '1y', label: '近一年' },
]
const creatorStatsWindow = ref('3m')
const creatorProductionStats = ref({
  window: '3m',
  windowLabel: '近3个月',
  days: 90,
  months: 3,
  averageMonths: 3,
  overall: { artworksTotal: 0, avgArtworksPerMonth: 0, coinsTotal: 0, avgCoinsPerMonth: 0 },
  data: [],
})
const pointsForm = ref({ amount: 10, reason: '' })

// --- 公会系统管理数据 ---
const guildTab = ref('quests')
const guildQuestPage = ref('list')
const guildRewardPage = ref('list')
const guildMsg = ref('')
const guildSaving = ref(false)
const guildQuests = ref([])
const guildQuestClaims = ref([])
const guildRewards = ref([])
const guildRewardCategories = ref([])
const guildRewardCategoryFilter = ref('all')
const showGuildRewardCategoryForm = ref(false)
const guildRewardCategoryEditingId = ref(null)
const guildRewardCategoryForm = ref({ name: '', sortOrder: 100, status: 'active' })
const guildRedemptions = ref([])
const selectedRedemption = ref(null)
const guildBudget = ref({
  summary: { currentBudgetCoins: 0, totalSupplyCoins: 0, spentPhysicalCoins: 0, coinPerRmb: 15 },
  supplies: [],
  spends: [],
})
const guildBudgetTab = ref('supplies')
const showBudgetSupplyForm = ref(false)
const guildBudgetSupplyForm = ref({ budgetType: 'quarterly', amountUnit: 'rmb', amount: 600 })
const defaultRewardSettingsForm = () => ({
  personalHaruhiPoints: 120,
  personalOtherPoints: 30,
  personalHaruhiReputation: 120,
  personalOtherReputation: 30,
  pointsMultiplier: 1,
  reputationMultiplier: 1,
  pointsMultiplierBps: 10000,
  reputationMultiplierBps: 10000,
  updatedAt: '',
})
const rewardSettingsForm = ref(defaultRewardSettingsForm())
const rewardSettingsLoading = ref(false)
const rewardSettingsSaving = ref(false)
const guildRatings = ref([])
const guildProfiles = ref([])
const guildQuestEditingId = ref(null)
const guildRewardEditingId = ref(null)
const guildRewardImageInput = ref(null)
const guildRewardImageUploading = ref(false)

const ratingOptions = ['F', 'E', 'D', 'C', 'B', 'A', 'S', 'X']
const accessOptions = [
  { value: 'public_archive', label: '档案0 · 公开档案许可' },
  { value: 'observer_clearance', label: '观测1 · 观测员许可' },
  { value: 'anomaly_research', label: '异常2 · 异常观测许可' },
  { value: 'closed_space', label: '闭锁3 · 闭锁空间许可' },
]
const conditionOptions = [
  { value: 'browse_artworks', label: '浏览画廊作品' },
  { value: 'comment_artworks', label: '公开评论作品' },
  { value: 'like_artworks', label: '点赞作品' },
  { value: 'upload_personal_haruhi', label: '上传凉宫个人作品' },
  { value: 'upload_personal_any', label: '上传任意个人作品' },
  { value: 'upload_network', label: '提交转载作品' },
  { value: 'manual_admin_verify', label: '管理员手动验收' },
]

const defaultQuestForm = () => ({
  title: '',
  description: '',
  questType: 'daily',
  difficulty: 'normal',
  requiredRating: 'F',
  requiredAccess: 'observer_clearance',
  conditionKind: 'browse_artworks',
  targetCount: 1,
  rewardReputation: 0,
  rewardCoins: 0,
  deadlineHours: null,
  timeLimitMode: 'none',
  deadlineDays: null,
  fixedDeadlineAt: '',
  repeatOnComplete: false,
  cycleDays: 1,
  cycleResetHour: 4,
  autoClaim: false,
  status: 'active',
  sortOrder: 100,
})
const defaultRewardForm = () => ({
  name: '',
  description: '',
  rewardType: 'virtual',
  priceCoins: 0,
  stock: -1,
  categoryId: null,
  requiredRating: 'F',
  requiredAccess: 'observer_clearance',
  imageUrl: '',
  status: 'active',
  sortOrder: 100,
})
const guildQuestForm = ref(defaultQuestForm())
const guildRewardForm = ref(defaultRewardForm())

// 计算属性：过滤后的创作者
const filteredCreators = computed(() => {
  if (!creatorSearch.value) return creators.value
  const q = creatorSearch.value.toLowerCase()
  return creators.value.filter((c) =>
    [c.uid, c.name, c.qq, c.email, c.contactValue, c.rating, c.accessLabel, c.accessShortLabel]
      .filter(Boolean)
      .join(' ')
      .toLowerCase()
      .includes(q)
  )
})

const creatorProductionByUid = computed(() => {
  const map = new Map()
  for (const item of creatorProductionStats.value.data || []) {
    map.set(item.uid, item)
  }
  return map
})

const creatorStatsOverall = computed(
  () =>
    creatorProductionStats.value.overall || {
      artworksTotal: 0,
      avgArtworksPerMonth: 0,
      coinsTotal: 0,
      avgCoinsPerMonth: 0,
    }
)

const creatorStatsWindowLabel = computed(
  () =>
    creatorProductionStats.value.windowLabel || creatorStatsOptionLabel(creatorStatsWindow.value)
)

const creatorStatsAverageLabel = computed(() =>
  creatorStatsWindow.value === 'week' ? '折算月均' : '月均'
)

const selectedCreatorProduction = computed(() => {
  if (!selectedCreator.value) return emptyCreatorProduction()
  return creatorProductionByUid.value.get(selectedCreator.value.uid) || emptyCreatorProduction()
})

const budgetSupplyPreviewCoins = computed(() => {
  const amount = Number(guildBudgetSupplyForm.value.amount || 0)
  const coinPerRmb = Number(guildBudget.value.summary?.coinPerRmb || 15)
  return guildBudgetSupplyForm.value.amountUnit === 'rmb' ? amount * coinPerRmb : amount
})

const rewardSettingsPreview = computed(() => ({
  haruhi: {
    points: scaledReward(
      rewardSettingsForm.value.personalHaruhiPoints,
      rewardSettingsForm.value.pointsMultiplier
    ),
    reputation: scaledReward(
      rewardSettingsForm.value.personalHaruhiReputation,
      rewardSettingsForm.value.reputationMultiplier
    ),
  },
  other: {
    points: scaledReward(
      rewardSettingsForm.value.personalOtherPoints,
      rewardSettingsForm.value.pointsMultiplier
    ),
    reputation: scaledReward(
      rewardSettingsForm.value.personalOtherReputation,
      rewardSettingsForm.value.reputationMultiplier
    ),
  },
}))

const guildRewardCategoryTabs = computed(() => [
  { id: 'all', name: '所有' },
  ...guildRewardCategories.value.map((category) => ({
    id: Number(category.id),
    name: category.name,
  })),
])

const filteredGuildRewards = computed(() => {
  if (guildRewardCategoryFilter.value === 'all') return guildRewards.value
  return guildRewards.value.filter(
    (reward) => Number(reward.categoryId || 0) === Number(guildRewardCategoryFilter.value)
  )
})

// 计算是否有修改
const isCreatorModified = computed(() => {
  if (!selectedCreator.value) return false
  const qqChanged = editCreatorForm.value.qq !== (selectedCreator.value.qq || '')
  const fileChanged = !!editCreatorForm.value.file
  return qqChanged || fileChanged
})

// 计算属性：过滤后的评论
const filteredComments = computed(() => {
  if (commentTab.value === 'pending') return comments.value
  if (!commentSearch.value) return comments.value
  const q = commentSearch.value.toLowerCase()
  return comments.value.filter(
    (c) => c.body.toLowerCase().includes(q) || c.user_name.toLowerCase().includes(q)
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
      pageSize: 20,
    })
    approvedList.value = res.data || []
  } catch (e) {
    console.error(e)
  }
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
  if (!confirm(`⚠️ 警告：正在从数据库中永久删除作品 "${it.title}"。\n此操作不可恢复！是否继续？`))
    return
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
    source_type: it.source_type || 'personal',
    content_type: it.content_type || 'haruhi',
    origin_url: it.origin_url || '',
    netLicenses: [],
    groupLicenses: [],
  }

  if (Array.isArray(it.licenses)) {
    it.licenses.forEach((l) => {
      if (l.startsWith('NET:')) editForm.value.netLicenses.push(l.replace('NET:', ''))
      if (l.startsWith('GROUP:')) editForm.value.groupLicenses.push(l.replace('GROUP:', ''))
    })
  }
}
async function saveEdit(it) {
  const payload = {
    ...editForm.value,
    licenses: JSON.stringify([
      ...editForm.value.netLicenses.map((x) => `NET:${x}`),
      ...editForm.value.groupLicenses.map((x) => `GROUP:${x}`),
    ]),
  }
  await api.adminUpdateArtworkDetails(it.id, payload)

  // 更新本地数据
  it.title = editForm.value.title
  it.description = editForm.value.description
  it.source_type = editForm.value.source_type
  it.content_type = editForm.value.content_type
  it.origin_url = editForm.value.origin_url
  it.licenses = [
    ...editForm.value.netLicenses.map((x) => `NET:${x}`),
    ...editForm.value.groupLicenses.map((x) => `GROUP:${x}`),
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
  comments.value = comments.value.filter((x) => x.id !== c.id)
}

// ---------------- 创作者管理 (新逻辑) ----------------

async function loadCreators() {
  const res = await api.adminCreators()
  creators.value = res.data
  // 如果当前选中的创作者还在列表中，刷新它
  if (selectedCreator.value) {
    const fresh = creators.value.find((c) => c.uid === selectedCreator.value.uid)
    if (fresh) {
      selectedCreator.value = { ...selectedCreator.value, ...fresh }
    } else {
      selectedCreator.value = null
    }
  }
}

async function loadCreatorProductionStats() {
  const res = await api.adminGuildCreatorProductionStats({ window: creatorStatsWindow.value })
  creatorStatsWindow.value = res.window || creatorStatsWindow.value
  creatorProductionStats.value = {
    window: creatorStatsWindow.value,
    windowLabel: res.windowLabel || creatorStatsOptionLabel(creatorStatsWindow.value),
    days: Number(res.days || 90),
    months: Number(res.months || 3),
    averageMonths: Number(res.averageMonths || res.months || 3),
    overall: res.overall || {
      artworksTotal: 0,
      avgArtworksPerMonth: 0,
      coinsTotal: 0,
      avgCoinsPerMonth: 0,
    },
    data: res.data || [],
  }
}

async function setCreatorStatsWindow(value) {
  if (creatorStatsWindow.value === value) return
  creatorStatsWindow.value = value
  try {
    await loadCreatorProductionStats()
  } catch (e) {
    guildMsg.value = `生产统计加载失败：${e.message || '未知错误'}`
    clearGuildMsgSoon()
  }
}

function creatorStatsOptionLabel(value) {
  return creatorStatsWindowOptions.find((option) => option.value === value)?.label || '近3个月'
}

function emptyCreatorProduction() {
  return { artworksTotal: 0, avgArtworksPerMonth: 0, coinsTotal: 0, avgCoinsPerMonth: 0 }
}

function formatAverage(value) {
  return Number(value || 0).toLocaleString('zh-CN', { maximumFractionDigits: 1 })
}

function formatWhole(value) {
  return Number(value || 0).toLocaleString('zh-CN')
}

async function addCreator() {
  if (!newCreatorUid.value) return
  await api.adminAddCreator(newCreatorUid.value)
  newCreatorUid.value = ''
  await loadCreators()
  await loadCreatorProductionStats()
}

async function selectCreator(c) {
  selectedCreator.value = c
  // 绑定数据到表单，处理 QQ 可能为空的情况
  editCreatorForm.value = { uid: c.uid, qq: c.qq || '', file: null }
  previewAvatar.value = null
  saveMsg.value = ''
  pointsForm.value = { amount: 10, reason: '' }

  // 加载积分
  const res = await api.adminPointsLedger({ uid: c.uid })
  creatorLogs.value = res.data || []
}

function creatorContact(c) {
  return c?.contactValue || c?.qq || c?.email || ''
}

function creatorContactLabel(c) {
  if (c?.contactLabel) return c.contactLabel
  if (c?.contactType === 'qq' || c?.qq) return 'QQ'
  if (c?.contactType === 'email' || c?.email) return '邮箱'
  return '联系方式'
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
    // 统一身份后不允许管理员改 UID：仍带原 uid，后端检测 new_uid == old_uid 即跳过改名
    formData.append('new_uid', selectedCreator.value.uid)
    formData.append('qq', editCreatorForm.value.qq) // 添加 QQ
    if (editCreatorForm.value.file) {
      formData.append('avatar', editCreatorForm.value.file)
    }

    const res = await api.adminUpdateCreator(selectedCreator.value.uid, formData)

    saveMsg.value = '保存成功！'
    setTimeout(() => (saveMsg.value = ''), 2000)

    // 重新加载列表
    await loadCreators()

    // UID 不变，定位回当前创作者
    const newObj = creators.value.find((c) => c.uid === selectedCreator.value.uid)
    if (newObj) {
      selectCreator(newObj)
    }
  } catch (e) {
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
    await loadCreatorProductionStats()
  } catch (e) {
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
      note: pointsForm.value.reason,
    })
    // 刷新记录
    const res = await api.adminPointsLedger({ uid: selectedCreator.value.uid })
    creatorLogs.value = res.data || []
    pointsForm.value.reason = ''
    await loadCreators()
    await loadCreatorProductionStats()
  } catch (e) {
    alert('操作失败')
  }
}

// ---------------- 公会系统管理 ----------------

function clearGuildMsgSoon() {
  if (!guildMsg.value) return
  setTimeout(() => {
    guildMsg.value = ''
  }, 2600)
}

function cleanNullableNumber(value) {
  if (value === '' || value === undefined || value === null) return null
  const n = Number(value)
  return Number.isFinite(n) ? n : null
}

function cleanNonNegativeNumber(value, fallback = 0) {
  const n = Number(value)
  return Number.isFinite(n) && n >= 0 ? n : fallback
}

function scaledReward(base, multiplier) {
  return Math.round(cleanNonNegativeNumber(base) * cleanNonNegativeNumber(multiplier, 1))
}

function bpsToMultiplier(value) {
  return cleanNonNegativeNumber(value, 10000) / 10000
}

function multiplierToBps(value) {
  return Math.round(cleanNonNegativeNumber(value, 1) * 10000)
}

function normalizeRewardSettings(data = {}) {
  return {
    ...defaultRewardSettingsForm(),
    personalHaruhiPoints: Number(data.personalHaruhiPoints ?? 120),
    personalOtherPoints: Number(data.personalOtherPoints ?? 30),
    personalHaruhiReputation: Number(data.personalHaruhiReputation ?? 120),
    personalOtherReputation: Number(data.personalOtherReputation ?? 30),
    pointsMultiplierBps: Number(data.pointsMultiplierBps ?? 10000),
    reputationMultiplierBps: Number(data.reputationMultiplierBps ?? 10000),
    pointsMultiplier:
      data.pointsMultiplier !== undefined
        ? Number(data.pointsMultiplier)
        : bpsToMultiplier(data.pointsMultiplierBps),
    reputationMultiplier:
      data.reputationMultiplier !== undefined
        ? Number(data.reputationMultiplier)
        : bpsToMultiplier(data.reputationMultiplierBps),
    updatedAt: data.updatedAt || '',
  }
}

function validateRewardSettingsForm() {
  const fields = [
    ['凉宫个人作品基础金币', rewardSettingsForm.value.personalHaruhiPoints],
    ['其他个人作品基础金币', rewardSettingsForm.value.personalOtherPoints],
    ['凉宫个人作品基础声望', rewardSettingsForm.value.personalHaruhiReputation],
    ['其他个人作品基础声望', rewardSettingsForm.value.personalOtherReputation],
  ]
  for (const [label, value] of fields) {
    if (!Number.isFinite(Number(value)) || Number(value) < 0) {
      return `${label}必须是 0 或正数`
    }
  }
  for (const [label, value] of [
    ['金币倍率', rewardSettingsForm.value.pointsMultiplier],
    ['声望倍率', rewardSettingsForm.value.reputationMultiplier],
  ]) {
    const n = Number(value)
    if (!Number.isFinite(n) || n < 0 || n > 20) {
      return `${label}必须在 0 到 20 倍之间`
    }
  }
  return ''
}

async function loadRewardSettings() {
  rewardSettingsLoading.value = true
  try {
    const res = await api.adminRewardSettings()
    rewardSettingsForm.value = normalizeRewardSettings(res.data || res)
  } finally {
    rewardSettingsLoading.value = false
  }
}

async function saveRewardSettings() {
  const error = validateRewardSettingsForm()
  if (error) {
    guildMsg.value = error
    clearGuildMsgSoon()
    return
  }
  rewardSettingsSaving.value = true
  try {
    const payload = {
      personalHaruhiPoints: Math.round(
        cleanNonNegativeNumber(rewardSettingsForm.value.personalHaruhiPoints)
      ),
      personalOtherPoints: Math.round(
        cleanNonNegativeNumber(rewardSettingsForm.value.personalOtherPoints)
      ),
      personalHaruhiReputation: Math.round(
        cleanNonNegativeNumber(rewardSettingsForm.value.personalHaruhiReputation)
      ),
      personalOtherReputation: Math.round(
        cleanNonNegativeNumber(rewardSettingsForm.value.personalOtherReputation)
      ),
      pointsMultiplierBps: multiplierToBps(rewardSettingsForm.value.pointsMultiplier),
      reputationMultiplierBps: multiplierToBps(rewardSettingsForm.value.reputationMultiplier),
    }
    const res = await api.adminUpdateRewardSettings(payload)
    rewardSettingsForm.value = normalizeRewardSettings(res.data || res)
    guildMsg.value = '投稿奖励规则已保存'
    clearGuildMsgSoon()
  } finally {
    rewardSettingsSaving.value = false
  }
}

function toDateTimeLocal(value) {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  const pad = (n) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`
}

function getGuildQuestTimeLimitMode({ deadlineDays, fixedDeadlineAt }) {
  if (fixedDeadlineAt) return 'fixed'
  if (cleanNullableNumber(deadlineDays)) return 'days'
  return 'none'
}

function normalizeGuildQuestTimeLimit() {
  const mode = ['none', 'days', 'fixed'].includes(guildQuestForm.value.timeLimitMode)
    ? guildQuestForm.value.timeLimitMode
    : 'none'
  guildQuestForm.value.timeLimitMode = mode
  if (mode !== 'days') {
    guildQuestForm.value.deadlineDays = null
    guildQuestForm.value.repeatOnComplete = false
  }
  if (mode !== 'fixed') guildQuestForm.value.fixedDeadlineAt = ''
  if (mode !== 'none') {
    guildQuestForm.value.cycleDays = null
    guildQuestForm.value.cycleResetHour = 4
  }
}

async function loadGuildAdmin() {
  if (guildTab.value === 'quests') {
    const res = await api.adminGuildQuests()
    guildQuests.value = res.data || []
  } else if (guildTab.value === 'questClaims') {
    const res = await api.adminGuildQuestClaims()
    guildQuestClaims.value = res.data || []
  } else if (guildTab.value === 'rewards') {
    const res = await api.adminGuildRewards()
    guildRewards.value = res.data || []
    guildRewardCategories.value = res.categories || []
    if (
      guildRewardCategoryFilter.value !== 'all' &&
      !guildRewardCategories.value.some(
        (category) => Number(category.id) === Number(guildRewardCategoryFilter.value)
      )
    ) {
      guildRewardCategoryFilter.value = 'all'
    }
  } else if (guildTab.value === 'redemptions') {
    const res = await api.adminGuildRedemptions()
    guildRedemptions.value = res.data || []
  } else if (guildTab.value === 'budget') {
    const res = await api.adminGuildBudget()
    guildBudget.value = {
      summary: res.summary || {
        currentBudgetCoins: 0,
        totalSupplyCoins: 0,
        spentPhysicalCoins: 0,
        coinPerRmb: 15,
      },
      supplies: res.supplies || [],
      spends: res.spends || [],
    }
  } else if (guildTab.value === 'rewardSettings') {
    await loadRewardSettings()
  } else if (guildTab.value === 'ratings') {
    const res = await api.adminGuildRatingApplications()
    guildRatings.value = res.data || []
  } else if (guildTab.value === 'creators') {
    await Promise.all([loadCreators(), loadCreatorProductionStats()])
  } else if (guildTab.value === 'profiles') {
    const res = await api.adminGuildProfiles()
    guildProfiles.value = res.data || []
  }
}

async function switchGuildTab(tab) {
  guildTab.value = tab
  if (tab === 'quests') {
    guildQuestPage.value = 'list'
  } else if (tab === 'rewards') {
    guildRewardPage.value = 'list'
    closeGuildRewardCategoryForm()
  } else if (tab === 'budget') {
    showBudgetSupplyForm.value = false
  }
  await loadGuildAdmin()
}

function resetGuildQuestForm() {
  guildQuestEditingId.value = null
  guildQuestForm.value = defaultQuestForm()
}

function openGuildQuestCreate() {
  resetGuildQuestForm()
  guildQuestPage.value = 'form'
}

function openGuildQuestList() {
  resetGuildQuestForm()
  guildQuestPage.value = 'list'
}

function editGuildQuest(quest) {
  guildQuestEditingId.value = quest.id
  const deadlineDays = quest.deadlineDays ?? null
  const fixedDeadlineAt = toDateTimeLocal(quest.fixedDeadlineAt)
  const timeLimitMode = getGuildQuestTimeLimitMode({ deadlineDays, fixedDeadlineAt })
  guildQuestForm.value = {
    title: quest.title || '',
    description: quest.description || '',
    questType: quest.questType || 'daily',
    difficulty: quest.difficulty || 'normal',
    requiredRating: quest.requiredRating || 'F',
    requiredAccess: quest.requiredAccess || 'observer_clearance',
    conditionKind: quest.conditionKind || 'browse_artworks',
    targetCount: Number(quest.targetCount || 1),
    rewardReputation: Number(quest.rewardReputation || 0),
    rewardCoins: Number(quest.rewardCoins || 0),
    deadlineHours: quest.deadlineHours ?? null,
    timeLimitMode,
    deadlineDays: timeLimitMode === 'days' ? deadlineDays : null,
    fixedDeadlineAt: timeLimitMode === 'fixed' ? fixedDeadlineAt : '',
    repeatOnComplete: timeLimitMode === 'days' ? Boolean(quest.repeatOnComplete) : false,
    cycleDays:
      timeLimitMode === 'none'
        ? (quest.cycleDays ?? (quest.questType === 'daily' ? 1 : null))
        : null,
    cycleResetHour: Number(quest.cycleResetHour ?? 4),
    autoClaim: Boolean(quest.autoClaim),
    status: quest.status || 'active',
    sortOrder: Number(quest.sortOrder || 100),
  }
  guildQuestPage.value = 'form'
}

async function saveGuildQuest() {
  if (!guildQuestForm.value.title.trim()) return
  guildSaving.value = true
  try {
    normalizeGuildQuestTimeLimit()
    const { timeLimitMode, ...form } = guildQuestForm.value
    const payload = {
      ...form,
      deadlineHours: null,
      deadlineDays: timeLimitMode === 'days' ? cleanNullableNumber(form.deadlineDays) : null,
      repeatOnComplete: timeLimitMode === 'days' ? Boolean(form.repeatOnComplete) : false,
      cycleDays: timeLimitMode === 'none' ? cleanNullableNumber(form.cycleDays) : null,
      cycleResetHour:
        timeLimitMode === 'none' ? (cleanNullableNumber(form.cycleResetHour) ?? 4) : 4,
      fixedDeadlineAt: timeLimitMode === 'fixed' ? form.fixedDeadlineAt || '' : '',
    }
    if (guildQuestEditingId.value) {
      await api.adminUpdateGuildQuest(guildQuestEditingId.value, payload)
      guildMsg.value = '委托已更新'
    } else {
      await api.adminCreateGuildQuest(payload)
      guildMsg.value = '委托已新增'
    }
    resetGuildQuestForm()
    await loadGuildAdmin()
    guildQuestPage.value = 'list'
    clearGuildMsgSoon()
  } finally {
    guildSaving.value = false
  }
}

async function setGuildQuestStatus(quest, status) {
  await api.adminUpdateGuildQuestStatus(quest.id, status)
  guildMsg.value = `委托状态已切换为 ${status}`
  await loadGuildAdmin()
  clearGuildMsgSoon()
}

async function deleteGuildQuest(quest) {
  if (!confirm(`确认删除委托「${quest.title}」？`)) return
  await api.adminDeleteGuildQuest(quest.id)
  await loadGuildAdmin()
}

function resetGuildRewardCategoryForm() {
  guildRewardCategoryEditingId.value = null
  guildRewardCategoryForm.value = { name: '', sortOrder: 100, status: 'active' }
}

function openGuildRewardCategoryCreate() {
  resetGuildRewardCategoryForm()
  guildRewardPage.value = 'list'
  showGuildRewardCategoryForm.value = true
}

function closeGuildRewardCategoryForm() {
  showGuildRewardCategoryForm.value = false
  resetGuildRewardCategoryForm()
}

function editGuildRewardCategory(category) {
  guildRewardCategoryEditingId.value = category.id
  guildRewardCategoryForm.value = {
    name: category.name || '',
    sortOrder: Number(category.sortOrder || 100),
    status: category.status || 'active',
  }
  guildRewardPage.value = 'list'
  showGuildRewardCategoryForm.value = true
}

async function saveGuildRewardCategory() {
  const name = guildRewardCategoryForm.value.name.trim()
  if (!name) {
    guildMsg.value = '请填写商品分类名称'
    clearGuildMsgSoon()
    return
  }
  guildSaving.value = true
  try {
    const payload = {
      name,
      sortOrder: Number(guildRewardCategoryForm.value.sortOrder || 100),
      status: guildRewardCategoryForm.value.status || 'active',
    }
    if (guildRewardCategoryEditingId.value) {
      await api.adminUpdateGuildRewardCategory(guildRewardCategoryEditingId.value, payload)
      guildMsg.value = '商品分类已更新'
    } else {
      await api.adminCreateGuildRewardCategory(payload)
      guildMsg.value = '商品分类已新增'
    }
    closeGuildRewardCategoryForm()
    await loadGuildAdmin()
    clearGuildMsgSoon()
  } finally {
    guildSaving.value = false
  }
}

async function setGuildRewardCategoryStatus(category, status) {
  await api.adminUpdateGuildRewardCategoryStatus(category.id, status)
  guildMsg.value = `商品分类已${status === 'active' ? '启用' : '停用'}`
  await loadGuildAdmin()
  clearGuildMsgSoon()
}

function resetGuildRewardForm() {
  guildRewardEditingId.value = null
  guildRewardForm.value = defaultRewardForm()
  guildRewardImageUploading.value = false
  if (guildRewardImageInput.value) guildRewardImageInput.value.value = ''
}

function openGuildRewardCreate() {
  resetGuildRewardForm()
  closeGuildRewardCategoryForm()
  guildRewardPage.value = 'form'
}

function openGuildRewardList() {
  resetGuildRewardForm()
  guildRewardPage.value = 'list'
}

function editGuildReward(reward) {
  guildRewardEditingId.value = reward.id
  guildRewardForm.value = {
    name: reward.name || '',
    description: reward.description || '',
    rewardType: reward.rewardType || 'virtual',
    priceCoins: Number(reward.priceCoins || 0),
    stock: reward.stock ?? -1,
    categoryId: reward.categoryId ? Number(reward.categoryId) : null,
    requiredRating: reward.requiredRating || 'F',
    requiredAccess: reward.requiredAccess || 'observer_clearance',
    imageUrl: reward.imageUrl || '',
    status: reward.status || 'active',
    sortOrder: Number(reward.sortOrder || 100),
  }
  if (guildRewardImageInput.value) guildRewardImageInput.value.value = ''
  guildRewardPage.value = 'form'
}

function openGuildRewardImagePicker() {
  guildRewardImageInput.value?.click()
}

function clearGuildRewardImage() {
  guildRewardForm.value.imageUrl = ''
  if (guildRewardImageInput.value) guildRewardImageInput.value.value = ''
}

async function handleGuildRewardImageChange(e) {
  const file = e.target.files?.[0]
  if (!file) return

  guildRewardImageUploading.value = true
  try {
    const res = await api.adminUploadGuildRewardImage(file)
    guildRewardForm.value.imageUrl = res.url || ''
    guildMsg.value = '展示图已上传，保存商品后生效'
    clearGuildMsgSoon()
  } catch (err) {
    alert('上传失败: ' + (err.message || '未知错误'))
  } finally {
    guildRewardImageUploading.value = false
    if (e.target) e.target.value = ''
  }
}

async function saveGuildReward() {
  if (!guildRewardForm.value.name.trim()) return
  if (guildRewardImageUploading.value) return
  guildSaving.value = true
  try {
    const payload = {
      ...guildRewardForm.value,
      stock: cleanNullableNumber(guildRewardForm.value.stock),
      categoryId: cleanNullableNumber(guildRewardForm.value.categoryId),
    }
    if (guildRewardEditingId.value) {
      await api.adminUpdateGuildReward(guildRewardEditingId.value, payload)
      guildMsg.value = '商品已更新'
    } else {
      await api.adminCreateGuildReward(payload)
      guildMsg.value = '商品已新增'
    }
    resetGuildRewardForm()
    await loadGuildAdmin()
    guildRewardPage.value = 'list'
    clearGuildMsgSoon()
  } finally {
    guildSaving.value = false
  }
}

async function createBudgetSupply() {
  if (!guildBudgetSupplyForm.value.amount || guildBudgetSupplyForm.value.amount <= 0) {
    guildMsg.value = '请填写大于 0 的预算额度'
    clearGuildMsgSoon()
    return
  }
  guildSaving.value = true
  try {
    await api.adminCreateGuildBudgetSupply({
      budgetType: guildBudgetSupplyForm.value.budgetType,
      amountUnit: guildBudgetSupplyForm.value.amountUnit,
      amount: guildBudgetSupplyForm.value.amount,
    })
    guildMsg.value = '预算补给已新增'
    guildBudgetSupplyForm.value = { budgetType: 'quarterly', amountUnit: 'rmb', amount: 600 }
    showBudgetSupplyForm.value = false
    await loadGuildAdmin()
    clearGuildMsgSoon()
  } finally {
    guildSaving.value = false
  }
}

async function setGuildRewardStatus(reward, status) {
  await api.adminUpdateGuildRewardStatus(reward.id, status)
  guildMsg.value = `商品状态已切换为 ${status}`
  await loadGuildAdmin()
  clearGuildMsgSoon()
}

async function deleteGuildReward(reward) {
  if (!confirm(`确认删除商品「${reward.name}」？`)) return
  await api.adminDeleteGuildReward(reward.id)
  await loadGuildAdmin()
}

function adminNote(defaultText = '') {
  const note = window.prompt('管理员备注：', defaultText)
  return note === null ? null : note
}

function formatDateTime(value) {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
}

function guildClaimStatusLabel(status) {
  const map = {
    active: '待验收',
    completed: '已通过',
    rejected: '未通过',
    expired: '已截止',
  }
  return map[status] || status
}

function redemptionStatusLabel(status) {
  const map = {
    pending: '待审核',
    approved: '已批准',
    rejected: '已拒绝',
    cancelled: '已取消',
    fulfilled: '已发放',
  }
  return map[status] || status || '未知'
}

function redemptionTypeLabel(type) {
  const map = {
    physical: '实体补给',
    virtual: '虚拟补给',
    badge: '虚拟徽章',
  }
  return map[type] || type || '未知类型'
}

function openRedemptionDetail(item) {
  selectedRedemption.value = item
}

function closeRedemptionDetail() {
  selectedRedemption.value = null
}

function redemptionHistory(item) {
  if (!item) return []
  if (Array.isArray(item.history) && item.history.length) return item.history
  const history = []
  if (item.createdAt) {
    history.push({
      kind: 'applied',
      label: '提交申请',
      at: item.createdAt,
      note: item.userNote,
    })
  }
  if (item.reviewedAt) {
    history.push({
      kind: 'reviewed',
      label:
        item.status === 'rejected'
          ? '审核拒绝'
          : item.status === 'cancelled'
            ? '取消兑换'
            : '审核批准',
      at: item.reviewedAt,
      note: item.reviewNote || (item.status !== 'fulfilled' ? item.adminNote : ''),
    })
  }
  if (item.fulfilledAt) {
    history.push({
      kind: 'fulfilled',
      label: '发放完成',
      at: item.fulfilledAt,
      note: item.fulfilledNote || (item.status === 'fulfilled' ? item.adminNote : ''),
    })
  }
  return history
}

async function approveQuestClaim(item) {
  const note = adminNote('委托验收通过')
  if (note === null) return
  await api.adminApproveGuildQuestClaim(item.id, note)
  await loadGuildAdmin()
}

async function rejectQuestClaim(item) {
  const note = adminNote('委托验收未通过')
  if (note === null) return
  await api.adminRejectGuildQuestClaim(item.id, note)
  await loadGuildAdmin()
}

async function approveRedemption(item) {
  const note = adminNote('审核通过，等待发放')
  if (note === null) return
  await api.adminApproveGuildRedemption(item.id, note)
  closeRedemptionDetail()
  await loadGuildAdmin()
}

async function rejectRedemption(item) {
  const note = adminNote('不满足兑换条件')
  if (note === null) return
  await api.adminRejectGuildRedemption(item.id, note)
  closeRedemptionDetail()
  await loadGuildAdmin()
}

async function fulfillRedemption(item) {
  const note = adminNote('奖励已发放')
  if (note === null) return
  await api.adminFulfillGuildRedemption(item.id, note)
  closeRedemptionDetail()
  await loadGuildAdmin()
}

async function approveRating(item) {
  const note = adminNote('评级申请通过')
  if (note === null) return
  await api.adminApproveGuildRating(item.id, note)
  await loadGuildAdmin()
}

async function rejectRating(item) {
  const note = adminNote('暂未满足评级要求')
  if (note === null) return
  await api.adminRejectGuildRating(item.id, note)
  await loadGuildAdmin()
}

async function saveProfileAccess(item) {
  await api.adminUpdateGuildProfileAccess(item.uid, item.accessTier)
  guildMsg.value = `${item.uid} 的访问许可已更新`
  await loadGuildAdmin()
  clearGuildMsgSoon()
}

// 监听 Tab 切换加载数据
// ---- 公告管理 ----
const announcements = ref([])
const annPage = ref('list')
const annMsg = ref('')
const annSaving = ref(false)
const annEditingId = ref(null)
function defaultAnnForm() {
  return {
    category: 'activity',
    title: '',
    summary: '',
    body: '',
    tags: '',
    pinned: false,
    status: 'published',
    publishedAt: '',
  }
}
const annForm = ref(defaultAnnForm())
let annMsgTimer = null
function clearAnnMsgSoon() {
  if (annMsgTimer) clearTimeout(annMsgTimer)
  annMsgTimer = setTimeout(() => {
    annMsg.value = ''
  }, 2500)
}

async function loadAnnouncementsAdmin() {
  const res = await api.adminAnnouncements()
  announcements.value = Array.isArray(res?.data) ? res.data : []
}

function resetAnnForm() {
  annEditingId.value = null
  annForm.value = defaultAnnForm()
}

function openAnnouncementCreate() {
  resetAnnForm()
  annPage.value = 'form'
}

function openAnnouncementList() {
  resetAnnForm()
  annPage.value = 'list'
}

function editAnnouncement(a) {
  annEditingId.value = a.id
  annForm.value = {
    category: a.category || 'activity',
    title: a.title || '',
    summary: a.summary || '',
    body: a.body || '',
    tags: Array.isArray(a.tags) ? a.tags.join(', ') : '',
    pinned: Boolean(a.pinned),
    status: a.status || 'published',
    publishedAt: (a.publishedAt || '').slice(0, 10),
  }
  annPage.value = 'form'
}

async function saveAnnouncement() {
  if (!annForm.value.title.trim()) {
    annMsg.value = '标题不能为空'
    return
  }
  annSaving.value = true
  try {
    const payload = {
      category: annForm.value.category,
      title: annForm.value.title.trim(),
      summary: annForm.value.summary.trim(),
      body: annForm.value.body.trim(),
      tags: annForm.value.tags
        .split(/[,，]/)
        .map((s) => s.trim())
        .filter(Boolean),
      pinned: annForm.value.pinned,
      status: annForm.value.status,
      publishedAt: annForm.value.publishedAt || undefined,
    }
    if (annEditingId.value) {
      await api.adminUpdateAnnouncement(annEditingId.value, payload)
      annMsg.value = '公告已更新'
    } else {
      await api.adminCreateAnnouncement(payload)
      annMsg.value = '公告已发布'
    }
    resetAnnForm()
    await loadAnnouncementsAdmin()
    annPage.value = 'list'
    clearAnnMsgSoon()
  } finally {
    annSaving.value = false
  }
}

async function deleteAnnouncement(a) {
  if (!confirm(`确认删除公告「${a.title}」？`)) return
  await api.adminDeleteAnnouncement(a.id)
  await loadAnnouncementsAdmin()
}

watch(mainTab, (v) => {
  if (v === 'images' && imageTab.value === 'list') loadApprovedList()
  if (v === 'comments') switchCommentTab('pending')
  if (v === 'guild') {
    if (guildTab.value === 'quests') guildQuestPage.value = 'list'
    if (guildTab.value === 'rewards') guildRewardPage.value = 'list'
    loadGuildAdmin()
  }
  if (v === 'announcements') {
    annPage.value = 'list'
    loadAnnouncementsAdmin()
  }
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
  border-bottom: 1px solid var(--sos-bg-muted);
}
.head-left {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.h1 {
  font-size: 24px;
  font-weight: 900;
  color: var(--sos-text-primary);
}
.sub {
  font-size: 13px;
  color: var(--sos-text-tertiary);
  font-family: monospace;
}

.auth-box {
  padding: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}
.form2 {
  display: flex;
  gap: 10px;
}
.msg {
  color: var(--sos-danger);
  font-size: 13px;
  margin-top: 10px;
}

/* 布局 */
.panel-layout {
  display: flex;
  flex: 1;
  margin-top: 0px;
}
.main-nav {
  width: 140px;
  border-right: 1px solid var(--sos-border-default);
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
  color: var(--sos-text-secondary);
  border-left: 3px solid transparent;
  transition: all 0.2s;
}
.nav-item:hover {
  background: var(--sos-bg-muted);
  color: var(--sos-text-primary);
}
.nav-item.active {
  background: var(--sos-accent-soft);
  color: var(--sos-accent);
  border-left-color: var(--sos-accent);
}

.content-area {
  flex: 1;
  padding: 20px 30px;
  background: var(--sos-bg-subtle);
}

/* 子Tab */
.sub-tabs {
  display: flex;
  gap: 20px;
  border-bottom: 1px solid var(--sos-border-default);
  margin-bottom: 20px;
}
.sub-tab {
  padding: 10px 0;
  background: transparent;
  border: none;
  font-size: 15px;
  font-weight: 600;
  color: var(--sos-text-tertiary);
  cursor: pointer;
  border-bottom: 2px solid transparent;
}
.sub-tab:hover {
  color: var(--sos-text-secondary);
}
.sub-tab.on {
  color: var(--sos-text-primary);
  border-bottom-color: var(--sos-text-primary);
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 10px;
}
.toolbar.tight {
  margin-bottom: 10px;
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
}
.filter-bar {
  background: var(--sos-bg-surface);
  padding: 12px;
  border-radius: 8px;
  border: 1px solid var(--sos-border-default);
}
.filters {
  display: flex;
  gap: 10px;
  align-items: center;
  flex: 1;
}
.comment-search {
  width: 240px;
}
.tip {
  font-size: 13px;
  color: var(--sos-text-tertiary);
}
.add-row {
  display: flex;
  gap: 8px;
}

/* 两列式布局 */
.two-col-layout {
  display: flex;
  height: calc(100vh - 200px);
  min-height: 500px;
  gap: 20px;
}
.col-left {
  width: 280px;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--sos-border-default);
  padding-right: 16px;
}
.col-right {
  flex: 1;
  overflow-y: auto;
  background: var(--sos-bg-surface);
  border-radius: 8px;
  border: 1px solid var(--sos-border-default);
  padding: 0;
}

/* 创作者列表 */
.creator-list-v {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-right: 4px;
}
.creator-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}
.creator-item:hover {
  background: var(--sos-bg-surface);
  border-color: var(--sos-border-default);
}
.creator-item.active {
  background: var(--sos-accent-soft);
  border-color: var(--sos-accent-soft);
}
.creator-item--guild {
  align-items: flex-start;
}
.c-avatar.sm {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  object-fit: cover;
  background: var(--sos-border-default);
}
.c-info-mini {
  flex: 1;
  overflow: hidden;
}
.c-uid {
  font-weight: 600;
  font-size: 14px;
  color: var(--sos-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.c-sub {
  font-size: 11px;
  color: var(--sos-text-tertiary);
  display: flex;
  gap: 6px;
  align-items: center;
  flex-wrap: wrap;
}
.qq-badge {
  background: var(--sos-accent-soft);
  color: #1e40af;
  padding: 0 4px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: bold;
}
.c-arr {
  color: var(--sos-border-strong);
  font-size: 18px;
}

/* 创作者详情面板 */
.creator-detail-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.creator-guild-layout {
  min-height: 620px;
}
.creator-production-overview {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  padding: 0 0 14px;
}
.creator-production-card {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 14px 16px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--panel);
}
.creator-production-card--window {
  gap: 8px;
}
.creator-production-card span,
.creator-production-card em {
  font-size: 12px;
  color: var(--muted);
  font-style: normal;
}
.creator-production-card b {
  font-size: 18px;
  color: var(--ink);
}
.creator-window-switch {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 6px;
}
.creator-window-btn {
  min-width: 0;
  min-height: 30px;
  padding: 5px 8px;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: var(--panel-soft);
  color: var(--muted);
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
}
.creator-window-btn.on {
  border-color: var(--sos-accent);
  background: color-mix(in srgb, var(--sos-accent) 14%, var(--panel));
  color: var(--sos-accent);
}
.creator-guild-summary {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
  padding: 16px 24px 0;
}
.creator-guild-card {
  min-width: 0;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid var(--sos-border-default);
  background: linear-gradient(180deg, #fff, color-mix(in srgb, var(--sos-bg-subtle) 72%, #fff));
}
.creator-guild-card span,
.creator-guild-card em {
  display: block;
  color: var(--sos-text-tertiary);
  font-size: 11px;
  font-weight: 700;
}
.creator-guild-card b {
  display: block;
  margin: 4px 0 2px;
  color: var(--sos-text-primary);
  font-size: 14px;
  line-height: 1.25;
  overflow-wrap: anywhere;
}
.creator-guild-card em {
  font-style: normal;
}
.creator-access-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
}
.panel-header {
  padding: 16px 24px;
  border-bottom: 1px solid var(--sos-border-default);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--sos-bg-subtle);
}
.ph-title {
  font-size: 18px;
  font-weight: bold;
  color: var(--sos-text-primary);
}
.panel-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.empty-select {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--sos-text-tertiary);
  gap: 10px;
}
.empty-select .icon {
  font-size: 40px;
  opacity: 0.5;
}

.edit-form {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--sos-text-secondary);
}
.avatar-uploader {
  display: flex;
  align-items: center;
  gap: 20px;
}
.avatar-preview {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  object-fit: cover;
  border: 1px solid var(--sos-border-default);
}
.au-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.tip-text {
  font-size: 12px;
  color: var(--sos-text-tertiary);
}
.tip-text.warn {
  color: var(--sos-warning);
  margin-top: 4px;
}
.form-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 10px;
}
.save-msg {
  color: var(--sos-success);
  font-size: 13px;
  font-weight: 600;
}
.divider {
  height: 1px;
  background: var(--sos-border-default);
  margin: 0 24px;
}

/* 详情页内的积分部分 */
.points-section {
  padding: 24px;
}
.label-lg {
  font-size: 16px;
  font-weight: bold;
  color: var(--sos-text-primary);
  margin-bottom: 16px;
}
.points-action-row {
  margin-bottom: 16px;
}

.ph-list.compact {
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  overflow: hidden;
}
.ph-scroll-area {
  max-height: 200px;
  overflow-y: auto;
}
.ph-row {
  padding: 8px 12px;
  font-size: 12px;
}

/* 卡片通用 */
.card-grid {
  display: grid;
  gap: 16px;
}
.manage-card {
  display: flex;
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}
.m-thumb {
  width: 140px;
  height: 140px;
  position: relative;
  flex-shrink: 0;
}
.m-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* 状态标 */
.status-badge {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 4px;
  text-align: center;
  font-size: 11px;
  font-weight: bold;
  color: var(--sos-bg-surface);
  background: rgba(0, 0, 0, 0.6);
}
.status-badge.flagged {
  background: var(--sos-danger);
}
.status-badge.approved {
  background: var(--sos-success);
}
.status-badge.pending {
  background: var(--sos-warning);
}

.m-body {
  flex: 1;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.m-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.m-title {
  font-weight: bold;
  font-size: 16px;
  color: var(--sos-text-primary);
}

.m-info {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 12px;
  color: var(--sos-text-tertiary);
  flex-wrap: wrap;
}
.tag {
  background: var(--sos-bg-muted);
  padding: 2px 8px;
  border-radius: 4px;
}
.m-info .u-name {
  margin-left: auto;
}

.m-desc {
  font-size: 13px;
  color: var(--sos-text-secondary);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.ai-box {
  background: var(--sos-danger-soft);
  border: 1px solid var(--sos-danger-soft);
  padding: 6px 10px;
  border-radius: 4px;
  font-size: 12px;
  color: #b91c1c;
}

/* 编辑器 */
.inline-editor {
  padding: 12px;
  background: var(--sos-bg-subtle);
  border: 1px dashed var(--sos-border-strong);
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 8px 0;
}
.btns {
  display: flex;
  gap: 8px;
}

/* 操作栏 */
.m-actions {
  margin-top: auto;
  padding-top: 10px;
  display: flex;
  gap: 10px;
  align-items: center;
}
.m-actions.right {
  justify-content: flex-end;
}
.note-input {
  flex: 1;
  height: 34px;
  font-size: 13px;
}
.btn-group {
  display: flex;
  gap: 8px;
}

/* 评论表格 */
.comment-table {
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--sos-bg-surface);
  overflow: hidden;
}
.c-row {
  display: flex;
  padding: 12px 16px;
  border-bottom: 1px solid var(--sos-bg-muted);
  align-items: flex-start;
  gap: 12px;
}
.c-row.header {
  background: var(--sos-bg-subtle);
  font-weight: bold;
  font-size: 13px;
  color: var(--sos-text-tertiary);
  border-bottom: 1px solid var(--sos-border-default);
}
.col-user {
  width: 140px;
  font-size: 13px;
}
.col-content {
  flex: 1;
  font-size: 13px;
}
.col-status {
  width: 90px;
  text-align: center;
}
.col-action {
  width: 160px;
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}
.desktop-only {
  display: block;
}
.mobile-only {
  display: none !important;
}
.comment-list-mobile {
  display: none;
}

.u-name {
  font-weight: bold;
  color: var(--sos-text-primary);
}
.u-time {
  font-size: 11px;
  color: var(--sos-text-tertiary);
  margin-top: 2px;
}
.body-text {
  color: var(--sos-text-secondary);
  line-height: 1.4;
}
.ai-reason-mini {
  font-size: 11px;
  color: var(--sos-danger);
  margin-top: 4px;
  font-weight: 500;
}

.badge-mini {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 4px;
  background: var(--sos-bg-muted);
  color: var(--sos-text-tertiary);
}
.badge-mini.flagged {
  background: var(--sos-danger-soft);
  color: var(--sos-danger);
}
.badge-mini.public {
  background: var(--sos-success-soft);
  color: var(--sos-success);
}

.quick-points {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}
.chip-btn {
  padding: 4px 12px;
  border-radius: 20px;
  border: 1px solid var(--sos-border-default);
  background: var(--sos-bg-surface);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  color: var(--sos-text-secondary);
  transition: all 0.2s;
}
.chip-btn:hover {
  border-color: var(--sos-border-strong);
  background: var(--sos-bg-muted);
}
.chip-btn.active {
  background: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  border-color: var(--sos-text-primary);
}

.pa-form {
  display: flex;
  gap: 12px;
  align-items: stretch;
}
.input-group {
  position: relative;
  width: 100px;
  flex-shrink: 0;
}
.input-prefix {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 12px;
  color: var(--sos-text-tertiary);
  pointer-events: none;
}
.points-num {
  padding-left: 40px !important;
  text-align: center;
  font-weight: bold;
}

/* 按钮与输入框通用 */
.btn {
  padding: 0 16px;
  height: 38px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  background: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.btn:hover:not(:disabled) {
  background: var(--sos-text-secondary);
  transform: translateY(-1px);
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
}
.btn:active:not(:disabled) {
  transform: translateY(0);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: var(--sos-text-secondary);
}
.btn.sm {
  height: 32px;
  font-size: 12px;
  padding: 0 12px;
}
.btn.lg {
  height: 42px;
  font-size: 14px;
}
.btn.success {
  background: var(--sos-success);
}
.btn.danger {
  background: var(--sos-danger);
}

.btn-ghost {
  height: 38px;
  padding: 0 16px;
  background: transparent;
  border: 1px solid var(--sos-border-strong);
  color: var(--sos-text-secondary);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.btn-ghost:hover {
  background: var(--sos-bg-muted);
  border-color: var(--sos-text-tertiary);
  color: var(--sos-text-primary);
}
.btn-ghost.sm {
  height: 32px;
  font-size: 12px;
  padding: 0 12px;
}
.btn-ghost.warn {
  color: var(--sos-warning);
  border-color: #fbbf24;
}
.btn-ghost.danger {
  color: var(--sos-danger);
  border-color: #fca5a5;
}

.btn-text {
  background: transparent;
  border: none;
  color: var(--sos-accent);
  cursor: pointer;
  font-size: 13px;
  text-decoration: underline;
  padding: 0;
}
.btn-mini {
  padding: 4px 10px;
  font-size: 11px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  color: var(--sos-bg-surface);
  background: var(--sos-text-tertiary);
  font-weight: 600;
}
.btn-mini.success {
  background: var(--sos-success);
}
.btn-mini.warn {
  background: var(--sos-warning);
}
.btn-mini.danger {
  background: var(--sos-danger);
}

.trash-btn {
  width: 26px;
  height: 26px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  background: color-mix(in srgb, var(--sos-danger) 8%, #fff);
  border: 1px solid color-mix(in srgb, var(--sos-danger) 28%, #fca5a5);
  border-radius: 7px;
  color: var(--sos-danger);
  cursor: pointer;
  font-size: 13px;
  line-height: 1;
  transition: all 0.18s ease;
}

.trash-btn:hover,
.trash-btn:focus-visible {
  background: var(--sos-danger);
  border-color: var(--sos-danger);
  color: #fff;
  outline: none;
  transform: translateY(-1px);
  box-shadow: 0 6px 14px -10px var(--sos-danger);
}

.has-trash {
  position: relative;
}

.has-trash .trash-btn {
  position: absolute;
  right: 10px;
  bottom: 10px;
  z-index: 4;
}

.manage-card.has-trash .m-body,
.comment-card.has-trash {
  padding-bottom: 42px;
}

.c-row.has-trash {
  padding-right: 52px;
  padding-bottom: 34px;
}

.guild-manage-row.has-trash {
  padding-bottom: 42px;
}

.input,
.textarea,
.select {
  border: 1px solid color-mix(in srgb, var(--sos-border-strong) 82%, var(--sos-accent) 10%);
  border-radius: 8px;
  padding: 10px 12px;
  outline: none;
  font-size: 14px;
  width: 100%;
  transition: all 0.2s;
  background: linear-gradient(180deg, #fff, color-mix(in srgb, var(--sos-bg-subtle) 52%, #fff));
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.74);
}
.input.sm,
.select.sm {
  padding: 6px 10px;
  font-size: 13px;
}
.input:hover,
.textarea:hover,
.select:hover {
  border-color: color-mix(in srgb, var(--sos-accent) 38%, var(--sos-border-strong));
}
.input:focus,
.textarea:focus,
.select:focus {
  border-color: var(--sos-accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--sos-accent) 18%, transparent);
}
.textarea {
  resize: vertical;
  min-height: 80px;
}
.ann-body-editor {
  min-height: clamp(240px, 34vh, 460px);
  line-height: 1.7;
}

.ann-admin-layout {
  display: grid;
  gap: 18px;
}

.ann-editor-panel,
.ann-list-panel {
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--sos-accent) 7%, #fff),
    color-mix(in srgb, var(--sos-accent-2, #ec4899) 5%, #fff) 44%,
    #fff 88%
  );
  border: 1px solid color-mix(in srgb, var(--sos-border-default) 74%, var(--sos-accent) 18%);
  border-radius: 10px;
  overflow: visible;
  box-shadow:
    0 12px 28px -24px rgba(15, 23, 42, 0.36),
    0 1px 2px rgba(0, 0, 0, 0.04);
}

.ann-editor-head,
.ann-list-head {
  gap: 12px;
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--sos-accent) 10%, var(--sos-bg-subtle)),
    color-mix(in srgb, var(--sos-accent-2, #ec4899) 7%, var(--sos-bg-subtle))
  );
}

.ann-editor-form {
  display: grid;
  gap: 14px;
  padding: 18px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.76), rgba(255, 255, 255, 0.92));
}

.ann-field {
  display: flex;
  flex-direction: column;
  gap: 7px;
  min-width: 0;
}

.ann-field > span {
  color: var(--sos-text-secondary);
  font-size: 12px;
  font-weight: 700;
}

.ann-meta-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  align-items: end;
}

.ann-pin--card {
  min-height: 40px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-strong);
  border-radius: 8px;
  color: var(--sos-text-secondary);
  font-size: 13px;
  font-weight: 700;
}

.ann-pin--card input {
  width: 16px;
  height: 16px;
  margin: 0;
}

.ann-form-footer {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 14px;
  align-items: end;
}

.ann-editor-actions {
  justify-content: flex-end;
  flex-wrap: nowrap;
  padding-top: 2px;
}

.ann-list-panel {
  padding: 0;
}

.ann-list-panel .ann-list-head {
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--sos-accent) 10%, var(--sos-bg-subtle)),
    color-mix(in srgb, var(--sos-accent-2, #ec4899) 7%, var(--sos-bg-subtle))
  );
  border-bottom: 1px solid var(--sos-border-default);
}

.ann-list-panel > .ann-manage-row {
  margin: 10px;
}

.ann-row-main {
  display: grid;
  min-width: 0;
  gap: 6px;
}

.ann-row-summary,
.ann-row-body {
  margin: 0;
  color: var(--sos-text-secondary);
  font-size: 13px;
  line-height: 1.45;
}

.ann-row-body {
  max-height: 3.1em;
  overflow: hidden;
  color: var(--sos-text-tertiary);
  white-space: pre-wrap;
}

.ann-manage-row.is-editing {
  border-color: color-mix(in srgb, var(--sos-accent) 60%, #e5e7eb);
  background: color-mix(in srgb, var(--sos-accent) 7%, #fff);
  box-shadow: 0 8px 22px rgba(20, 184, 166, 0.12);
}

/* 积分记录 */
.ph-row {
  display: flex;
  padding: 10px 16px;
  border-bottom: 1px solid var(--sos-bg-muted);
  font-size: 13px;
}
.ph-row:last-child {
  border-bottom: none;
}
.ph-row.head {
  background: var(--sos-bg-subtle);
  font-weight: bold;
  color: var(--sos-text-tertiary);
  border-bottom: 1px solid var(--sos-border-default);
}
.ph-time {
  width: 100px;
  color: var(--sos-text-tertiary);
}
.ph-val {
  width: 70px;
  font-weight: bold;
}
.ph-val.pos {
  color: var(--sos-success);
}
.ph-val.neg {
  color: var(--sos-danger);
}
.ph-reason {
  flex: 1;
  color: var(--sos-text-secondary);
}
.empty-ph {
  padding: 24px;
  text-align: center;
  color: var(--sos-text-tertiary);
  font-size: 13px;
}

/* Expanded Editor Styles */
.inline-editor.expanded {
  gap: 12px;
}
.editor-row {
  display: flex;
  gap: 8px;
}
.editor-licenses {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: var(--sos-bg-surface);
  padding: 10px;
  border: 1px solid var(--sos-border-default);
  border-radius: 6px;
}
.lic-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.lic-title {
  font-size: 11px;
  font-weight: bold;
  color: var(--sos-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.chk-item {
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--sos-text-secondary);
  cursor: pointer;
}
.chk-item:hover {
  color: var(--sos-text-primary);
}

/* 公会系统后台 */
.guild-admin-layout {
  display: grid;
  grid-template-columns: 380px minmax(0, 1fr);
  gap: 18px;
}

.guild-admin-layout--wide {
  grid-template-columns: minmax(420px, 520px) minmax(0, 1fr);
}

.guild-quest-page {
  display: grid;
  gap: 18px;
}

.guild-editor,
.guild-list.single {
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--sos-accent) 7%, #fff),
    color-mix(in srgb, var(--sos-accent-2, #ec4899) 5%, #fff) 46%,
    #fff 88%
  );
  border: 1px solid color-mix(in srgb, var(--sos-border-default) 74%, var(--sos-accent) 18%);
  border-radius: 10px;
  overflow: visible;
  box-shadow:
    0 12px 28px -24px rgba(15, 23, 42, 0.36),
    0 1px 2px rgba(0, 0, 0, 0.04);
}

.guild-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.panel-header.compact {
  padding: 12px 16px;
}

.compact-form {
  padding: 16px;
  gap: 12px;
}

.guild-list-toolbar,
.guild-form-head {
  gap: 12px;
}

.guild-editor .panel-header.compact,
.guild-list.single .panel-header.compact {
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--sos-accent) 10%, var(--sos-bg-subtle)),
    color-mix(in srgb, var(--sos-accent-2, #ec4899) 7%, var(--sos-bg-subtle))
  );
}

.guild-form-panel {
  position: relative;
  overflow: visible;
}

.guild-form {
  display: grid;
  gap: 14px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.76), rgba(255, 255, 255, 0.92));
}

.guild-form-grid {
  display: grid;
  gap: 12px;
}

.guild-form-grid.two {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.guild-form-grid.three {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.guild-field,
.guild-inline-field {
  display: flex;
  flex-direction: column;
  gap: 7px;
  min-width: 0;
}

.guild-field > span,
.guild-inline-field > span,
.admin-field-label-row > label,
.admin-field-label-row > span,
.guild-field-label-row > label,
.guild-field-label-row > span {
  color: var(--sos-text-secondary);
  font-size: 12px;
  font-weight: 700;
}

.admin-field-label-row,
.guild-field-label-row {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.help-tip {
  position: relative;
  z-index: 40;
  width: 18px;
  height: 18px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--sos-bg-muted);
  border: 1px solid var(--sos-border-strong);
  border-radius: 50%;
  color: var(--sos-text-tertiary);
  cursor: help;
  font-size: 12px;
  font-weight: 800;
  line-height: 1;
}

.help-tip:hover,
.help-tip:focus-visible {
  border-color: var(--sos-accent);
  color: var(--sos-accent);
  outline: none;
}

.help-tip__bubble {
  position: absolute;
  left: 50%;
  bottom: calc(100% + 8px);
  z-index: 120;
  width: max-content;
  max-width: min(260px, 70vw);
  padding: 8px 10px;
  background: var(--sos-text-primary);
  border-radius: 8px;
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  line-height: 1.45;
  opacity: 0;
  pointer-events: none;
  box-shadow:
    0 14px 30px -16px rgba(15, 23, 42, 0.48),
    0 8px 18px -14px rgba(15, 23, 42, 0.34);
  text-align: left;
  transform: translate(-50%, 4px);
  transition:
    opacity 0.15s ease,
    transform 0.15s ease,
    visibility 0s linear 0.15s;
  visibility: hidden;
  white-space: normal;
}

.help-tip__bubble::after {
  position: absolute;
  left: 50%;
  bottom: -5px;
  width: 10px;
  height: 10px;
  background: var(--sos-text-primary);
  content: '';
  transform: translateX(-50%) rotate(45deg);
}

.help-tip:hover .help-tip__bubble,
.help-tip:focus-visible .help-tip__bubble {
  opacity: 1;
  transform: translate(-50%, 0);
  transition-delay: 0.3s, 0.3s, 0s;
  visibility: visible;
}

.guild-field small {
  color: var(--sos-text-tertiary);
  font-size: 11px;
  line-height: 1.35;
}

.guild-time-limit {
  display: grid;
  gap: 8px;
}

.guild-toggle-card {
  min-height: 58px;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  align-items: center;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--sos-border-strong) 74%, var(--sos-accent) 18%);
  border-radius: 10px;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--sos-accent) 6%, #fff),
    color-mix(in srgb, var(--sos-bg-subtle) 80%, #fff)
  );
  cursor: pointer;
}

.guild-toggle-card input {
  width: 18px;
  height: 18px;
  margin: 0;
  accent-color: var(--sos-accent);
}

.guild-toggle-card span {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.guild-toggle-card b {
  color: var(--sos-text-primary);
  font-size: 13px;
}

.guild-toggle-card small {
  color: var(--sos-text-tertiary);
}

.guild-field--wide {
  grid-column: 1 / -1;
}

.guild-textarea {
  min-height: 120px;
  line-height: 1.6;
}

.guild-form-actions {
  justify-content: flex-end;
  flex-wrap: wrap;
  padding-top: 4px;
  border-top: 1px dashed color-mix(in srgb, var(--sos-border-default) 78%, var(--sos-accent) 12%);
}

.guild-msg {
  margin-bottom: 12px;
  padding: 10px 12px;
  color: #065f46;
  font-size: 13px;
  font-weight: 700;
  background: #d1fae5;
  border: 1px solid #a7f3d0;
  border-radius: 8px;
}

.guild-budget-page {
  display: grid;
  gap: 16px;
}

.guild-budget-summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.guild-budget-card {
  display: grid;
  gap: 6px;
  min-width: 0;
  padding: 14px 16px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--panel);
}

.guild-budget-card span,
.guild-budget-card em {
  color: var(--sos-text-tertiary);
  font-size: 12px;
  font-style: normal;
  font-weight: 700;
}

.guild-budget-card b {
  color: var(--sos-text-primary);
  font-size: 24px;
  line-height: 1.2;
}

.guild-budget-toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  flex-wrap: wrap;
}

.guild-budget-switch,
.guild-budget-unit-switch {
  display: inline-grid;
  grid-auto-flow: column;
  gap: 4px;
  padding: 3px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--sos-bg-subtle);
}

.guild-budget-switch-btn {
  min-height: 30px;
  padding: 0 11px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--sos-text-secondary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 800;
}

.guild-budget-switch-btn.on {
  background: var(--panel);
  color: var(--sos-text-primary);
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.12);
}

.guild-budget-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr) auto;
  gap: 12px;
  align-items: end;
  padding: 14px 16px;
  border-bottom: 1px solid var(--sos-border-default);
  background: var(--sos-bg-subtle);
}

.guild-budget-form label {
  display: grid;
  gap: 6px;
  min-width: 0;
}

.guild-budget-form label > span,
.guild-budget-form-actions span {
  color: var(--sos-text-tertiary);
  font-size: 12px;
  font-weight: 700;
}

.guild-budget-amount-field {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 8px;
}

.guild-budget-form-actions {
  display: grid;
  gap: 6px;
  justify-items: end;
}

.guild-reward-settings-page {
  display: grid;
  gap: 16px;
}

.guild-reward-settings-panel {
  overflow: visible;
}

.reward-settings-body {
  display: grid;
  gap: 16px;
  padding: 16px;
}

.reward-settings-note {
  display: grid;
  gap: 6px;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--sos-border-default) 78%, var(--sos-accent) 14%);
  border-radius: 8px;
  background: color-mix(in srgb, var(--sos-bg-subtle) 86%, var(--sos-accent) 5%);
}

.reward-settings-note p {
  margin: 0;
  color: var(--sos-text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

.reward-settings-grid {
  align-items: start;
}

.reward-preview-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.reward-preview-card {
  display: grid;
  gap: 6px;
  min-width: 0;
  padding: 14px 16px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--panel);
}

.reward-preview-card span,
.reward-preview-card em {
  color: var(--sos-text-tertiary);
  font-size: 12px;
  font-style: normal;
  font-weight: 700;
}

.reward-preview-card b {
  color: var(--sos-text-primary);
  font-size: 24px;
  line-height: 1.2;
}

.reward-preview-card--muted b {
  color: var(--sos-text-tertiary);
}

.reward-settings-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.guild-manage-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 14px;
  align-items: center;
  padding: 14px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 10px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}

.guild-manage-row.is-clickable {
  cursor: pointer;
}

.guild-manage-row.is-clickable:hover {
  border-color: color-mix(in srgb, var(--sos-accent) 42%, #e5e7eb);
}

.admin-modal {
  position: fixed;
  inset: 0;
  z-index: 90;
  display: grid;
  place-items: center;
  padding: 22px;
  background: rgba(15, 23, 42, 0.48);
}

.admin-dialog {
  position: relative;
  width: min(560px, 100%);
  max-height: min(720px, calc(100vh - 44px));
  overflow: auto;
  padding: 22px;
  border: 1px solid var(--sos-border-default);
  border-radius: 10px;
  background: var(--sos-bg-panel);
  color: var(--sos-text-primary);
  box-shadow: 0 24px 70px rgba(15, 23, 42, 0.28);
}

.dialog-close {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 34px;
  height: 34px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--sos-bg-subtle);
  color: var(--sos-text-primary);
  cursor: pointer;
  font-size: 22px;
  line-height: 1;
}

.dialog-eyebrow {
  display: block;
  color: var(--sos-text-secondary);
  font-size: 12px;
  font-weight: 800;
  text-transform: uppercase;
}

.redemption-detail h2 {
  margin: 6px 42px 16px 0;
  font-size: 22px;
}

.redemption-detail-grid {
  display: grid;
  grid-template-columns: 88px minmax(0, 1fr);
  gap: 9px 12px;
  padding: 12px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--sos-bg-subtle);
}

.redemption-detail-grid span {
  color: var(--sos-text-secondary);
  font-size: 12px;
}

.redemption-detail-grid b,
.redemption-detail-grid p {
  min-width: 0;
  margin: 0;
}

.redemption-history {
  display: grid;
  gap: 10px;
  margin: 16px 0;
}

.redemption-history-item {
  display: grid;
  grid-template-columns: 12px minmax(0, 1fr);
  gap: 10px;
  align-items: start;
}

.redemption-history-item i {
  width: 10px;
  height: 10px;
  margin-top: 4px;
  border-radius: 50%;
  background: var(--sos-accent);
}

.redemption-history-item div {
  display: grid;
  gap: 3px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--sos-border-default);
}

.redemption-history-item time,
.redemption-history-item p {
  margin: 0;
  color: var(--sos-text-secondary);
  font-size: 12px;
}

.guild-row-main {
  display: grid;
  min-width: 0;
  gap: 6px;
}

.guild-category-editor,
.guild-category-list,
.guild-category-tabs {
  margin: 12px 16px 0;
}

.guild-category-editor {
  padding: 12px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--sos-bg-subtle);
}

.guild-category-form {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(120px, 0.6fr) minmax(120px, 0.6fr) auto;
  gap: 10px;
  align-items: end;
}

.guild-category-form label {
  display: grid;
  gap: 6px;
  min-width: 0;
}

.guild-category-form label > span {
  color: var(--sos-text-tertiary);
  font-size: 12px;
  font-weight: 700;
}

.guild-category-actions,
.guild-category-item-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}

.guild-category-list {
  display: grid;
  gap: 8px;
}

.guild-category-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--panel);
}

.guild-category-item > div:first-child {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.guild-category-item b {
  color: var(--sos-text-primary);
  font-size: 13px;
}

.guild-category-item span {
  color: var(--sos-text-tertiary);
  font-size: 12px;
}

.guild-category-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 3px;
  border: 1px solid var(--sos-border-default);
  border-radius: 8px;
  background: var(--sos-bg-subtle);
}

.guild-category-tab {
  min-height: 30px;
  padding: 0 11px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--sos-text-secondary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 800;
}

.guild-category-tab.on {
  background: var(--panel);
  color: var(--sos-text-primary);
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.12);
}

.guild-reward-row-main {
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 12px;
}

.guild-row-copy {
  display: grid;
  min-width: 0;
  gap: 6px;
}

.guild-reward-thumb {
  width: 72px;
  aspect-ratio: 16 / 10;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--sos-border-default) 78%, var(--sos-accent) 16%);
  border-radius: 8px;
  background: color-mix(in srgb, var(--sos-bg-subtle) 70%, #fff);
}

.guild-reward-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.guild-claim-artworks {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 8px;
  margin-top: 8px;
}

.guild-claim-artwork {
  display: grid;
  grid-template-columns: 58px minmax(0, 1fr);
  gap: 9px;
  align-items: center;
  min-height: 66px;
  padding: 7px;
  border: 1px solid color-mix(in srgb, var(--sos-border-default) 78%, var(--sos-accent) 16%);
  border-radius: 8px;
  background: color-mix(in srgb, var(--sos-bg-subtle) 68%, #fff);
  color: var(--sos-text-primary);
  text-decoration: none;
}

.guild-claim-artwork:hover {
  border-color: color-mix(in srgb, var(--sos-accent) 46%, var(--sos-border-default));
}

.guild-claim-artwork__thumb {
  width: 58px;
  aspect-ratio: 1;
  overflow: hidden;
  border-radius: 7px;
  background: color-mix(in srgb, var(--sos-accent) 10%, #eef2ff);
}

.guild-claim-artwork__thumb img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.guild-claim-artwork__thumb i {
  display: grid;
  place-items: center;
  width: 100%;
  height: 100%;
  color: var(--sos-text-secondary);
  font-size: 11px;
  font-style: normal;
  font-weight: 800;
}

.guild-claim-artwork__copy {
  display: grid;
  min-width: 0;
  gap: 3px;
}

.guild-claim-artwork__copy b {
  overflow: hidden;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.guild-claim-artwork__copy small,
.guild-claim-artwork__copy em,
.guild-claim-artworks-empty {
  color: var(--sos-text-secondary);
  font-size: 12px;
  font-style: normal;
}

.guild-row-desc {
  max-height: 3.1em;
  margin: 0;
  overflow: hidden;
  color: var(--sos-text-secondary);
  font-size: 13px;
  line-height: 1.45;
  white-space: pre-wrap;
}

.guild-quest-row.is-editing {
  border-color: color-mix(in srgb, var(--sos-accent) 60%, #e5e7eb);
  background: color-mix(in srgb, var(--sos-accent) 7%, #fff);
  box-shadow: 0 8px 22px rgba(20, 184, 166, 0.12);
}

.guild-row-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}

.guild-row-actions.access-editor {
  min-width: 260px;
}

.guild-row-actions.access-editor .guild-inline-field {
  min-width: 190px;
}

.reward-image-uploader {
  display: grid;
  grid-template-columns: minmax(220px, 320px) minmax(0, 1fr);
  gap: 14px;
  align-items: center;
  padding: 12px;
  border: 1px dashed color-mix(in srgb, var(--sos-border-strong) 72%, var(--sos-accent) 18%);
  border-radius: 10px;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--sos-accent) 6%, #fff),
    color-mix(in srgb, var(--sos-bg-subtle) 82%, #fff)
  );
}

.reward-image-input {
  display: none;
}

.reward-image-preview {
  display: grid;
  place-items: center;
  aspect-ratio: 16 / 9;
  min-height: 140px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--sos-border-default) 76%, var(--sos-accent) 16%);
  border-radius: 8px;
  background: #fff;
  color: var(--sos-text-tertiary);
  font-size: 13px;
  font-weight: 700;
}

.reward-image-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.reward-image-preview.is-empty {
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--sos-accent) 9%, #fff),
    color-mix(in srgb, var(--sos-accent-2, #ec4899) 6%, #fff)
  );
}

.reward-image-tools {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.reward-image-tools small {
  flex-basis: 100%;
  color: var(--sos-text-tertiary);
}

@media (max-width: 1024px) {
  .content-area {
    padding: 16px;
  }
  .filters {
    flex-wrap: wrap;
  }
  .two-col-layout {
    gap: 12px;
    min-height: 0;
    height: auto;
  }
  .col-left {
    width: 240px;
  }
  .guild-admin-layout {
    grid-template-columns: 1fr;
  }
  .guild-admin-layout--wide {
    grid-template-columns: 1fr;
  }
  .guild-form-grid.three {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .guild-budget-summary {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .guild-budget-form {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .guild-budget-form-actions {
    grid-column: 1 / -1;
    justify-items: start;
  }
  .creator-guild-summary {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    padding: 12px 16px 0;
  }
  .creator-production-overview {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .creator-window-switch {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .ann-meta-grid,
  .ann-form-footer {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .ann-editor-actions {
    justify-content: flex-start;
  }
}

@media (max-width: 768px) {
  .head-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
    margin-bottom: 14px;
  }
  .h1 {
    font-size: 20px;
  }

  .auth-box {
    padding: 16px 0 8px;
    align-items: stretch;
  }
  .form2 {
    flex-direction: column;
    width: 100%;
  }
  .form2 .btn {
    width: 100%;
  }

  .panel-layout {
    display: block;
  }
  .main-nav {
    width: auto;
    border-right: none;
    border-bottom: 1px solid var(--sos-border-default);
    padding: 0 0 10px;
    margin-bottom: 12px;
    flex-direction: row;
    gap: 8px;
    overflow-x: auto;
    position: sticky;
    top: 0;
    z-index: 8;
    background: var(--sos-bg-subtle);
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
    border-bottom-color: var(--sos-accent);
  }
  .content-area {
    padding: 12px;
  }

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
  .tip {
    font-size: 12px;
  }
  .comment-search {
    width: 100%;
  }
  .filter-bar {
    padding: 10px;
  }
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

  .card-grid {
    gap: 12px;
  }
  .manage-card {
    flex-direction: column;
  }
  .m-thumb {
    width: 100%;
    height: 190px;
  }
  .m-body {
    padding: 12px;
    gap: 10px;
  }
  .title-row {
    gap: 8px;
    align-items: flex-start;
  }
  .m-title {
    font-size: 15px;
    line-height: 1.35;
  }
  .m-actions {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .m-actions.right {
    justify-content: flex-start;
  }
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

  .desktop-only {
    display: none !important;
  }
  .mobile-only {
    display: inline-flex !important;
  }
  .comment-list-mobile {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .comment-card {
    background: var(--sos-bg-surface);
    border: 1px solid var(--sos-border-default);
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
  .col-left.mobile-hidden {
    display: none;
  }
  .creator-list-v {
    max-height: calc(100vh - 300px);
    padding-right: 0;
  }
  .col-right {
    display: none;
    min-height: 0;
  }
  .col-right.mobile-visible {
    display: block;
  }
  .panel-header {
    padding: 12px;
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .ph-title {
    font-size: 16px;
  }
  .panel-actions {
    width: 100%;
  }
  .panel-actions .btn-ghost {
    flex: 1;
  }
  .creator-guild-summary {
    grid-template-columns: 1fr;
    padding: 12px;
  }
  .creator-production-overview {
    grid-template-columns: 1fr;
    padding: 0 12px 12px;
  }
  .creator-access-row {
    grid-template-columns: 1fr;
  }
  .edit-form,
  .points-section {
    padding: 12px;
    gap: 14px;
  }
  .divider {
    margin: 0 12px;
  }
  .avatar-uploader {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  .au-actions {
    width: 100%;
    align-items: stretch;
  }
  .au-actions .btn-ghost {
    width: 100%;
  }
  .form-actions {
    flex-direction: column;
    align-items: stretch;
    margin-top: 4px;
  }
  .form-actions .btn {
    width: 100%;
  }
  .pa-form {
    flex-direction: column;
    gap: 8px;
  }
  .input-group {
    width: 100%;
  }
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
  .guild-manage-row {
    grid-template-columns: 1fr;
  }
  .guild-list-toolbar,
  .guild-form-head {
    align-items: stretch;
  }
  .guild-form-grid.two,
  .guild-form-grid.three {
    grid-template-columns: 1fr;
  }
  .guild-form-actions {
    display: grid;
    grid-template-columns: 1fr;
  }
  .ann-editor-form {
    padding: 12px;
  }
  .ann-meta-grid,
  .ann-form-footer {
    grid-template-columns: 1fr;
  }
  .ann-editor-actions {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .ann-list-panel > .ann-manage-row {
    margin: 8px;
  }
  .guild-row-actions,
  .guild-row-actions.access-editor {
    justify-content: flex-start;
    min-width: 0;
  }
  .guild-row-actions .btn-ghost,
  .guild-row-actions.access-editor .guild-inline-field,
  .guild-row-actions.access-editor .select {
    width: 100%;
  }
  .guild-reward-row-main,
  .reward-image-uploader {
    grid-template-columns: 1fr;
  }
  .guild-category-form {
    grid-template-columns: 1fr;
  }
  .guild-category-actions {
    grid-column: 1 / -1;
  }
  .guild-category-item {
    align-items: stretch;
    flex-direction: column;
  }
  .guild-reward-thumb {
    width: min(140px, 100%);
  }
  .reward-image-tools .btn-ghost {
    flex: 1 1 120px;
  }
  .guild-budget-summary,
  .guild-budget-form {
    grid-template-columns: 1fr;
  }
  .guild-budget-toolbar {
    justify-content: flex-start;
  }
  .guild-budget-amount-field {
    grid-template-columns: 1fr;
  }
}
</style>
