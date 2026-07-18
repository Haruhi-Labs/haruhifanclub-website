<template>
  <div class="adv-scope" :class="`rating-${profile.rating || 'F'}`">
    <div v-if="loading" class="adv-state">正在连接公会终端…</div>
    <div v-else-if="error" class="adv-state adv-state--error">{{ error }}</div>

    <template v-else>
      <!-- ===== 左：冒险者证件终端 ===== -->
      <aside class="adv-id">
        <header class="adv-id__band">
          <span class="adv-id__eyebrow">{{
            isTerminal ? 'Personal Terminal' : 'Adventurer License'
          }}</span>
          <span class="adv-id__org">SOS 绘画部公会</span>
        </header>

        <div class="adv-id__head">
          <div class="adv-id__avatar">
            <img v-if="profile.avatar_url" :src="profile.avatar_url" alt="" />
            <span v-else>{{ profileInitial }}</span>
            <b class="adv-id__rank">{{ profile.rating || 'F' }}</b>
          </div>
          <div class="adv-id__ident">
            <span class="adv-id__eyebrow adv-id__eyebrow--inline">{{
              isTerminal ? '个人终端' : '冒险者档案'
            }}</span>
            <h1>{{ displayName }}</h1>
            <span class="adv-id__rating">◈ {{ profile.rating || 'F' }} 级冒险者</span>
            <span v-if="isTerminal && userInfo.username" class="adv-id__handle">@{{ userInfo.username }}</span>
          </div>
        </div>

        <button
          v-if="!isTerminal && !social.isSelf"
          type="button"
          class="adv-profile-follow"
          :class="{ 'is-following': social.isFollowing }"
          :disabled="followLoading"
          @click="toggleProfileFollow"
        >
          {{ social.isFollowing ? '已关注' : '关注' }}
        </button>
        <span v-if="socialError" class="adv-profile-follow-error">{{ socialError }}</span>

        <dl class="adv-id__readout">
          <div class="adv-id__row">
            <dt>声望</dt>
            <span class="adv-id__lead" aria-hidden="true"></span>
            <dd>
              {{ profile.reputation || 0 }}<i>Lv{{ profile.level || 1 }}</i>
            </dd>
          </div>
          <div class="adv-id__row adv-id__row--coin">
            <dt>金币</dt>
            <span class="adv-id__lead" aria-hidden="true"></span>
            <dd>
              {{ profile.coins?.available || 0 }}G<i>冻结 {{ profile.coins?.frozen || 0 }}G</i>
            </dd>
          </div>
          <div class="adv-id__row">
            <dt>投稿</dt>
            <span class="adv-id__lead" aria-hidden="true"></span>
            <dd>
              {{ stats.total || 0
              }}<i>个人 {{ stats.personal || 0 }} / 转载 {{ stats.network || 0 }}</i>
            </dd>
          </div>
          <div class="adv-id__row">
            <dt>社交</dt>
            <span class="adv-id__lead" aria-hidden="true"></span>
            <dd class="adv-id__connections">
              <button type="button" @click="openConnections('following')">
                <b>{{ social.followingCount || 0 }}</b><i>关注</i>
              </button>
              <span aria-hidden="true">/</span>
              <button type="button" @click="openConnections('followers')">
                <b>{{ social.followerCount || 0 }}</b><i>粉丝</i>
              </button>
              <em class="adv-id__connections-arrow" aria-hidden="true">›</em>
            </dd>
          </div>
          <div class="adv-id__row">
            <dt>注册</dt>
            <span class="adv-id__lead" aria-hidden="true"></span>
            <dd class="is-date">
              {{ formatDate(userInfo.createdAt || profile.creatorCreatedAt) }}
            </dd>
          </div>
          <div v-if="contactDisplay" class="adv-id__row">
            <dt>{{ contactTypeLabel }}</dt>
            <span class="adv-id__lead" aria-hidden="true"></span>
            <dd class="adv-id__contact">{{ contactDisplay }}</dd>
          </div>
        </dl>

        <div class="adv-id__clearance">
          <span class="adv-id__clearance-k">访问许可</span>
          <strong class="adv-id__clearance-v">{{ profile.accessShortLabel || '档案0' }}</strong>
          <span class="adv-id__clearance-sub">{{ profile.accessLabel || '0级公开档案许可' }}</span>
        </div>

        <div class="adv-id__foot">
          <span class="adv-id__barcode" aria-hidden="true"></span>
          <span class="adv-id__issued">SOS BRIGADE · ART REGISTRY</span>
        </div>

        <RouterLink
          v-if="!isTerminal"
          class="sos-button sos-button--ghost sos-button--sm adv-id__back"
          :to="backTarget"
        >
          {{ backLabel }}
        </RouterLink>
      </aside>

      <!-- ===== 右：档案正文 ===== -->
      <main class="adv-dossier">
        <section class="adv-panel adv-metrics">
          <header class="adv-panel__head">
            <div>
              <span class="adv-panel__eyebrow">Guild Data</span>
              <h2>创作者公会数据</h2>
            </div>
          </header>
          <div class="adv-metrics__grid">
            <div class="adv-metric">
              <span>访问许可</span>
              <b>{{ profile.accessLabel || '0级公开档案许可' }}</b>
              <em>{{ profile.accessShortLabel || '档案0' }}</em>
            </div>
            <div class="adv-metric">
              <span>评级</span>
              <b>{{ profile.ratingLabel || `${profile.rating || 'F'}级冒险者` }}</b>
              <em>{{ profile.nextRating?.rating ? `下一评级 ${profile.nextRating.rating}` : '当前档案' }}</em>
            </div>
            <div class="adv-metric">
              <span>等级与声望</span>
              <b>Lv{{ profile.level || 1 }} · {{ profile.reputation || 0 }}</b>
              <em>公会声望</em>
            </div>
            <div class="adv-metric">
              <span>金币</span>
              <b>{{ profile.coins?.available || 0 }}G</b>
              <em>总 {{ profile.coins?.total || 0 }} / 冻结 {{ profile.coins?.frozen || 0 }}</em>
            </div>
            <div class="adv-metric">
              <span>作品统计</span>
              <b>{{ stats.total || 0 }}</b>
              <em>个人 {{ stats.personal || 0 }} / 转载 {{ stats.network || 0 }}</em>
            </div>
            <div class="adv-metric">
              <span>凉宫个人作品</span>
              <b>{{ profile.haruhiPersonalCount || 0 }}</b>
              <em>评级条件计数</em>
            </div>
            <div v-if="contactDisplay" class="adv-metric">
              <span>联系方式</span>
              <b>{{ contactDisplay }}</b>
              <em>{{ contactTypeLabel }}</em>
            </div>
            <div class="adv-metric">
              <span>最近投稿</span>
              <b>{{ formatDate(stats.latestUploadAt) }}</b>
              <em>首次 {{ formatDate(stats.firstUploadAt) }}</em>
            </div>
          </div>
        </section>

        <section class="adv-panel adv-archive">
          <header class="adv-panel__head">
            <div>
              <span class="adv-panel__eyebrow">Archive</span>
              <h2>{{ isTerminal ? '我的作品档案库' : '公开画作' }}</h2>
            </div>
            <span class="adv-panel__meta">{{ stats.haruhi || 0 }} 张凉宫画作</span>
          </header>

          <div v-if="artworks.length" class="adv-art-grid">
            <button
              v-for="art in artworks"
              :key="art.id"
              type="button"
              class="adv-art"
              @click="openArtwork(art)"
            >
              <span class="adv-art__media">
                <img
                  :src="thumbUrl(art.image_url, 320)"
                  :alt="art.title || 'artwork'"
                  loading="lazy"
                />
                <b class="adv-art__status" :class="`st-${art.status || 'approved'}`">{{
                  artStatusLabel(art.status)
                }}</b>
              </span>
              <span class="adv-art__title">{{ art.title || '未命名作品' }}</span>
            </button>
          </div>
          <div v-else class="adv-empty">这个冒险者还没有公开画作。</div>
        </section>

        <section class="adv-panel adv-archive">
          <header class="adv-panel__head">
            <div>
              <span class="adv-panel__eyebrow">Favorites</span>
              <h2>{{ isTerminal ? '我的收藏' : '收藏作品' }}</h2>
            </div>
            <span class="adv-panel__meta">{{ favorites.length }} 件</span>
          </header>

          <div v-if="favorites.length" class="adv-art-grid">
            <button
              v-for="art in favorites"
              :key="`favorite-${art.id}`"
              type="button"
              class="adv-art"
              @click="openArtwork(art)"
            >
              <span class="adv-art__media">
                <img :src="thumbUrl(art.image_url, 320)" :alt="art.title || 'artwork'" loading="lazy" />
              </span>
              <span class="adv-art__title">{{ art.title || '未命名作品' }}</span>
            </button>
          </div>
          <div v-else class="adv-empty adv-empty--sm">还没有收藏公开作品。</div>
        </section>

        <section class="adv-panel adv-social">
          <header class="adv-panel__head">
            <div>
              <span class="adv-panel__eyebrow">Connections</span>
              <h2>关注与粉丝</h2>
            </div>
            <span class="adv-panel__meta">{{ social.followingCount || 0 }} / {{ social.followerCount || 0 }}</span>
          </header>
          <div class="adv-social__columns">
            <div>
              <button type="button" class="adv-social__all" @click="openConnections('following')">
                <span>正在关注</span><b>{{ social.followingCount || 0 }}</b><em>查看全部</em>
              </button>
              <div v-if="social.following?.length" class="adv-social__list">
                <button v-for="item in social.following.slice(0, 4)" :key="`following-${item.uid}`" type="button" @click="openProfile(item.uid)">
                  <span class="adv-social__avatar">
                    <img v-if="item.avatar_url" :src="item.avatar_url" alt="" />
                    <span v-else>{{ socialInitial(item) }}</span>
                  </span>
                  <span><b>{{ item.displayName || item.uid }}</b><small>@{{ item.uid }}</small></span>
                </button>
              </div>
              <p v-else>暂未关注其他用户。</p>
            </div>
            <div>
              <button type="button" class="adv-social__all" @click="openConnections('followers')">
                <span>粉丝</span><b>{{ social.followerCount || 0 }}</b><em>查看全部</em>
              </button>
              <div v-if="social.followers?.length" class="adv-social__list">
                <button v-for="item in social.followers.slice(0, 4)" :key="`follower-${item.uid}`" type="button" @click="openProfile(item.uid)">
                  <span class="adv-social__avatar">
                    <img v-if="item.avatar_url" :src="item.avatar_url" alt="" />
                    <span v-else>{{ socialInitial(item) }}</span>
                  </span>
                  <span><b>{{ item.displayName || item.uid }}</b><small>@{{ item.uid }}</small></span>
                </button>
              </div>
              <p v-else>还没有粉丝。</p>
            </div>
          </div>
        </section>

        <section v-if="!isTerminal" class="adv-panel adv-messages">
          <header class="adv-panel__head">
            <div>
              <span class="adv-panel__eyebrow">Profile Messages</span>
              <h2>创作者留言板</h2>
            </div>
            <span class="adv-panel__meta">{{ messageTotal }} 条回音</span>
          </header>

          <form v-if="isLoggedIn" class="adv-message-compose" @submit.prevent="postMessage">
            <div class="adv-message-compose__head">
              <span>发送一段公开留言</span>
              <em>以 {{ accountName }} 署名</em>
            </div>
            <textarea
              v-model="messageBody"
              maxlength="600"
              rows="4"
              placeholder="想对这位创作者说些什么？"
              :disabled="postingMessage"
              @keydown.ctrl.enter.prevent="postMessage"
              @keydown.meta.enter.prevent="postMessage"
            ></textarea>
            <div class="adv-message-compose__foot">
              <span>{{ messageBody.length }}/600 · Ctrl / ⌘ + Enter 发送</span>
              <button
                type="submit"
                class="sos-button sos-button--primary sos-button--sm"
                :disabled="postingMessage || !messageBody.trim()"
              >
                {{ postingMessage ? '传送中…' : '发送留言' }}
              </button>
            </div>
          </form>
          <div v-else class="adv-message-login">
            <div>
              <b>登录后即可留言</b>
              <span>留言会使用你的账号昵称公开署名。</span>
            </div>
            <button
              type="button"
              class="sos-button sos-button--primary sos-button--sm"
              @click="goLogin"
            >
              登录 / 注册
            </button>
          </div>

          <p v-if="messageNotice" class="adv-message-notice" role="status">{{ messageNotice }}</p>
          <p v-if="messageError" class="adv-message-error" role="alert">{{ messageError }}</p>

          <div v-if="messagesLoading && !messages.length" class="adv-empty adv-empty--sm">
            正在接收留言回音…
          </div>
          <div v-else-if="messages.length" class="adv-message-list">
            <article v-for="message in messages" :key="message.id" class="adv-message">
              <div class="adv-message__avatar" aria-hidden="true">
                <img v-if="message.avatar_url" :src="message.avatar_url" alt="" />
                <span v-else>{{ messageInitial(message) }}</span>
              </div>
              <div class="adv-message__body">
                <header>
                  <b>{{ message.user_name || '画廊成员' }}</b>
                  <time :datetime="message.created_at || undefined">{{
                    formatMessageDate(message.created_at)
                  }}</time>
                </header>
                <p>{{ message.body }}</p>
              </div>
            </article>
          </div>
          <div v-else class="adv-empty adv-empty--sm">还没有留言，来留下第一段回音吧。</div>

          <button
            v-if="hasMoreMessages"
            type="button"
            class="adv-message-more"
            :disabled="messagesLoading"
            @click="loadMessages(messagePage + 1, true)"
          >
            {{ messagesLoading ? '接收中…' : '查看更多留言' }}
          </button>
        </section>

        <section v-if="isTerminal" class="adv-panel">
          <header class="adv-panel__head">
            <div>
              <span class="adv-panel__eyebrow">Quest Log</span>
              <h2>委托记录</h2>
            </div>
            <span class="adv-panel__meta">{{ claims.length }} 条</span>
          </header>
          <div v-if="claims.length" class="adv-quests">
            <div v-for="claim in claims.slice(0, 8)" :key="claim.id" class="adv-quest">
              <span class="adv-quest__title">{{ claim.title }}</span>
              <b class="adv-quest__prog">{{ claim.progress }}/{{ claim.targetCount }}</b>
              <em class="adv-quest__st" :class="`st-${claim.status}`">{{
                claimLabel(claim.status)
              }}</em>
            </div>
          </div>
          <div v-else class="adv-empty adv-empty--sm">还没有接取委托。</div>
        </section>

        <div v-if="isTerminal" class="adv-ledgers">
          <article class="adv-panel">
            <header class="adv-panel__head">
              <div>
                <span class="adv-panel__eyebrow">Coin Ledger</span>
                <h2>金币流水</h2>
              </div>
            </header>
            <div v-if="coinsHistory.length" class="adv-ledger">
              <div
                v-for="item in coinsHistory.slice(0, 8)"
                :key="`${item.createdAt}-${item.note}`"
                class="adv-ledger__row"
              >
                <span>{{ item.note || item.sourceType }}</span>
                <b :class="{ plus: item.coins > 0, minus: item.coins < 0 }">{{ item.coins > 0 ? '+' : '' }}{{ item.coins }}G</b>
              </div>
            </div>
            <div v-else class="adv-empty adv-empty--sm">暂无金币记录。</div>
          </article>

          <article class="adv-panel">
            <header class="adv-panel__head">
              <div>
                <span class="adv-panel__eyebrow">Redemption</span>
                <h2>兑换申请</h2>
              </div>
            </header>
            <div v-if="redemptions.length" class="adv-ledger">
              <div v-for="item in redemptions.slice(0, 8)" :key="item.id" class="adv-ledger__row">
                <span>{{ item.rewardName }}</span>
                <b class="adv-ledger__st" :class="`st-${item.status}`">{{
                  redemptionLabel(item.status)
                }}</b>
              </div>
            </div>
            <div v-else class="adv-empty adv-empty--sm">暂无兑换申请。</div>
          </article>
        </div>
      </main>
    </template>
  </div>

  <Teleport to="body">
    <div v-if="connectionsOpen" class="adv-connections-overlay" @click.self="closeConnections">
      <section class="adv-connections-dialog" role="dialog" aria-modal="true" :aria-label="connectionsTitle">
        <header>
          <div>
            <span>Connections</span>
            <h2>{{ connectionsTitle }}</h2>
          </div>
          <button type="button" aria-label="关闭名单" @click="closeConnections">×</button>
        </header>
        <nav aria-label="关注名单分类">
          <button
            v-for="tab in connectionTabs"
            :key="tab.kind"
            type="button"
            :class="{ 'is-active': connectionsKind === tab.kind }"
            @click="switchConnections(tab.kind)"
          >
            {{ tab.label }} <b>{{ tab.total }}</b>
          </button>
        </nav>
        <div class="adv-connections-body">
          <div v-if="connectionsLoading && !connectionItems.length" class="adv-connections-state">
            正在读取名单…
          </div>
          <div v-else-if="connectionsError" class="adv-connections-state is-error">
            {{ connectionsError }}
          </div>
          <div v-else-if="!connectionItems.length" class="adv-connections-state">
            {{ connectionsKind === 'following' ? '暂未关注其他用户。' : '还没有粉丝。' }}
          </div>
          <div v-else class="adv-connections-list">
            <button
              v-for="item in connectionItems"
              :key="`${connectionsKind}-${item.uid}`"
              type="button"
              @click="openConnectionProfile(item.uid)"
            >
              <span class="adv-social__avatar">
                <img v-if="item.avatar_url" :src="item.avatar_url" alt="" />
                <span v-else>{{ socialInitial(item) }}</span>
              </span>
              <span><b>{{ item.displayName || item.uid }}</b><small>@{{ item.uid }}</small></span>
              <i aria-hidden="true">›</i>
            </button>
          </div>
          <button
            v-if="connectionItems.length < connectionTotal"
            type="button"
            class="adv-connections-more"
            :disabled="connectionsLoading"
            @click="loadConnections(connectionPage + 1, true)"
          >
            {{ connectionsLoading ? '读取中…' : `继续加载（${connectionItems.length}/${connectionTotal}）` }}
          </button>
        </div>
      </section>
    </div>
  </Teleport>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { usePageMeta, canonicalUrl, absoluteUrl } from '@haruhi/seo'
import { useSession } from '@haruhi/auth-ui'
import { api, thumbUrl } from '../services/api.js'

const route = useRoute()
const router = useRouter()
const session = useSession('/api')

const loading = ref(true)
const error = ref('')
const profile = ref({})
const userInfo = ref({})
const stats = ref({})
const artworks = ref([])
const favorites = ref([])
const social = ref({
  followerCount: 0,
  followingCount: 0,
  isFollowing: false,
  isSelf: false,
  followers: [],
  following: [],
})
const followLoading = ref(false)
const socialError = ref('')
const connectionsOpen = ref(false)
const connectionsKind = ref('following')
const connectionItems = ref([])
const connectionTotal = ref(0)
const connectionPage = ref(1)
const connectionsLoading = ref(false)
const connectionsError = ref('')
let connectionLoadVersion = 0
const claims = ref([])
const coinsHistory = ref([])
const redemptions = ref([])
const messages = ref([])
const messageTotal = ref(0)
const messagePage = ref(1)
const messagesLoading = ref(false)
let profileLoadVersion = 0
let messageLoadVersion = 0
let messagesLoadingUid = ''
const postingMessage = ref(false)
const messageBody = ref('')
const messageError = ref('')
const messageNotice = ref('')

const isTerminal = computed(() => route.name === 'terminal')
const isLoggedIn = session.isLoggedIn
const accountName = computed(
  () => session.state.user?.nickname || session.state.user?.displayName || '账号昵称'
)
const hasMoreMessages = computed(() => messages.value.length < messageTotal.value)
const profileUid = computed(() => String(profile.value.uid || route.params.uid || '').trim())
const connectionsTitle = computed(() =>
  connectionsKind.value === 'following' ? `${displayName.value}正在关注` : `${displayName.value}的粉丝`
)
const connectionTabs = computed(() => [
  { kind: 'following', label: '关注', total: Number(social.value.followingCount || 0) },
  { kind: 'followers', label: '粉丝', total: Number(social.value.followerCount || 0) },
])
const backTarget = computed(() => {
  if (route.query.from === 'gallery') return { name: 'gallery' }
  if (route.query.from === 'ranking') return { name: 'exchange', query: { tab: 'ranking' } }
  return { name: 'exchange' }
})
const backLabel = computed(() => route.query.from === 'gallery' ? '返回画廊' : '返回公会指挥台')
const displayName = computed(
  () =>
    userInfo.value.displayName ||
    profile.value.displayName ||
    userInfo.value.username ||
    profile.value.uid ||
    'Observer'
)
const profileInitial = computed(() =>
  String(displayName.value || 'O')
    .slice(0, 1)
    .toUpperCase()
)
const contactDisplay = computed(
  () =>
    profile.value.contactValue ||
    profile.value.qq ||
    profile.value.email ||
    userInfo.value.email ||
    ''
)
const contactTypeLabel = computed(() => {
  if (profile.value.contactLabel) return profile.value.contactLabel
  if (profile.value.contactType === 'qq' || profile.value.qq) return 'QQ'
  if (profile.value.contactType === 'email' || profile.value.email || userInfo.value.email) {
    return '邮箱'
  }
  return '联系方式'
})

async function loadProfile() {
  const version = ++profileLoadVersion
  const targetIsTerminal = route.name === 'terminal'
  const targetUid = String(route.params.uid || '').trim()
  ++messageLoadVersion
  loading.value = true
  error.value = ''
  socialError.value = ''
  favorites.value = []
  messages.value = []
  messageTotal.value = 0
  messagePage.value = 1
  messagesLoading.value = false
  messagesLoadingUid = ''
  messageError.value = ''
  messageNotice.value = ''
  social.value = {
    followerCount: 0,
    followingCount: 0,
    isFollowing: false,
    isSelf: false,
    followers: [],
    following: [],
  }
  try {
    const res =
      targetIsTerminal
        ? await api.guildTerminal()
        : await api.guildProfile(targetUid)

    if (version !== profileLoadVersion) return

    profile.value = res.profile || {}
    userInfo.value = res.user || res.profile?.user || {}
    stats.value = res.stats || {}
    artworks.value = res.artworks || []
    favorites.value = res.favorites || []
    social.value = { ...social.value, ...(res.social || {}) }
    claims.value = res.claims || []
    coinsHistory.value = res.coinsHistory || []
    redemptions.value = res.redemptions || []
    if (!targetIsTerminal) await loadMessages(1, false, targetUid)
  } catch (err) {
    if (version === profileLoadVersion) error.value = err.message || '冒险者档案读取失败'
  } finally {
    if (version === profileLoadVersion) loading.value = false
  }
}

async function loadMessages(page = 1, append = false, requestedUid = '') {
  const uid = String(requestedUid || route.params.uid || '').trim()
  if (isTerminal.value || !uid) return
  if (messagesLoading.value && messagesLoadingUid === uid) return
  const version = ++messageLoadVersion
  messagesLoading.value = true
  messagesLoadingUid = uid
  messageError.value = ''
  try {
    const res = await api.guildProfileMessages(uid, { page, pageSize: 16 })
    if (version !== messageLoadVersion || String(route.params.uid || '').trim() !== uid) return
    const nextMessages = res.data || []
    messages.value = append ? [...messages.value, ...nextMessages] : nextMessages
    messageTotal.value = Number(res.total || 0)
    messagePage.value = page
  } catch (err) {
    if (version === messageLoadVersion) {
      if (!append) messages.value = []
      messageError.value = err.message || '留言读取失败'
    }
  } finally {
    if (version === messageLoadVersion) {
      messagesLoading.value = false
      messagesLoadingUid = ''
    }
  }
}

async function postMessage() {
  if (!isLoggedIn.value) {
    goLogin()
    return
  }
  const body = messageBody.value.trim()
  if (!body || postingMessage.value) return
  postingMessage.value = true
  messageError.value = ''
  messageNotice.value = ''
  try {
    const res = await api.postGuildProfileMessage(route.params.uid, body)
    messageBody.value = ''
    messageNotice.value = res.flagged ? res.message : '留言已送达。'
    await loadMessages(1)
  } catch (err) {
    messageError.value = err.message || '留言发送失败'
  } finally {
    postingMessage.value = false
  }
}

function goLogin() {
  router.push({ name: 'login', query: { redirect: route.fullPath } })
}

async function toggleProfileFollow() {
  if (!isLoggedIn.value) {
    goLogin()
    return
  }
  if (!route.params.uid || followLoading.value) return
  followLoading.value = true
  socialError.value = ''
  try {
    const res = await api.toggleGuildFollow(route.params.uid)
    social.value = {
      ...social.value,
      isFollowing: Boolean(res.following),
      followerCount: Number(res.followerCount || 0),
    }
  } catch (err) {
    socialError.value = err.message || '关注操作失败'
  } finally {
    followLoading.value = false
  }
}

async function loadConnections(page = 1, append = false) {
  if (!profileUid.value || connectionsLoading.value) return
  const version = ++connectionLoadVersion
  connectionsLoading.value = true
  connectionsError.value = ''
  try {
    const res = await api.guildProfileConnections(profileUid.value, {
      kind: connectionsKind.value,
      page,
      pageSize: 24,
    })
    if (version !== connectionLoadVersion) return
    const nextItems = res.data || []
    connectionItems.value = append ? [...connectionItems.value, ...nextItems] : nextItems
    connectionTotal.value = Number(res.total || 0)
    connectionPage.value = page
  } catch (err) {
    if (version !== connectionLoadVersion) return
    if (!append) connectionItems.value = []
    connectionsError.value = err.message || '名单读取失败'
  } finally {
    if (version === connectionLoadVersion) connectionsLoading.value = false
  }
}

function openConnections(kind) {
  connectionsKind.value = kind
  connectionsOpen.value = true
  connectionItems.value = []
  connectionTotal.value = 0
  connectionPage.value = 1
  connectionsError.value = ''
  void loadConnections(1)
}

function switchConnections(kind) {
  if (connectionsKind.value === kind) return
  connectionLoadVersion += 1
  connectionsLoading.value = false
  connectionsKind.value = kind
  connectionItems.value = []
  connectionTotal.value = 0
  connectionPage.value = 1
  connectionsError.value = ''
  void loadConnections(1)
}

function closeConnections() {
  connectionLoadVersion += 1
  connectionsOpen.value = false
  connectionsLoading.value = false
}

function openConnectionProfile(uid) {
  closeConnections()
  openProfile(uid)
}

function openProfile(uid) {
  if (!uid) return
  router.push({ name: 'adventurer-profile', params: { uid } })
}

function socialInitial(item) {
  return String(item?.displayName || item?.uid || '画').slice(0, 1).toUpperCase()
}

function messageInitial(message) {
  return String(message?.user_name || '画').slice(0, 1).toUpperCase()
}

function formatMessageDate(value) {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

// 页面 meta：仅公开档案页（/profile/:uid）在数据加载成功后设置；
// 创作者终端（/terminal，需登录、路由级 noindex）不参与
usePageMeta(() => {
  if (isTerminal.value || loading.value || error.value) return null
  const name = displayName.value
  return {
    title: `${name}的创作者档案 · 春日画廊`,
    description: `${name} 的创作者档案：已在春日画廊发布 ${stats.value.total || 0} 幅凉宫春日同人作品，冒险者评级 ${profile.value.rating || 'F'}。`,
    canonical: canonicalUrl(`/profile/${encodeURIComponent(route.params.uid || '')}`),
    ogType: 'profile',
    ogImage: profile.value.avatar_url ? absoluteUrl(profile.value.avatar_url) : undefined,
    jsonLd: {
      '@context': 'https://schema.org',
      '@type': 'ProfilePage',
      mainEntity: { '@type': 'Person', name },
    },
  }
})

function openArtwork(art) {
  router.push({ name: 'artwork-detail', params: { id: art.id } })
}

function formatDate(value) {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleDateString()
}

function artStatusLabel(status) {
  const map = { approved: '已通过', pending: '审核中', rejected: '已退回', hidden: '已隐藏' }
  return map[status] || status || '已通过'
}

function claimLabel(status) {
  const map = { active: '进行中', completed: '已完成', expired: '已截止', abandoned: '已放弃' }
  return map[status] || status
}

function redemptionLabel(status) {
  const map = {
    pending: '待审核',
    approved: '已批准',
    rejected: '已拒绝',
    cancelled: '已取消',
    fulfilled: '已发放',
  }
  return map[status] || status
}

function handleWindowKeydown(event) {
  if (event.key === 'Escape' && connectionsOpen.value) closeConnections()
}

watch(() => route.fullPath, () => {
  closeConnections()
  loadProfile()
})
onMounted(() => {
  window.addEventListener('keydown', handleWindowKeydown)
  session.ensureReady()
  loadProfile()
})
onBeforeUnmount(() => window.removeEventListener('keydown', handleWindowKeydown))
</script>

<style scoped>
.adv-scope {
  --accent: var(--sos-accent, hsl(172, 70%, 42%));
  --accent-strong: color-mix(in srgb, var(--accent) 78%, #06322d);
  --gold: hsl(42, 92%, 52%);
  --text: var(--sos-text-primary, #16242b);
  --muted: var(--sos-text-secondary, #5b6b72);
  --glass: color-mix(in srgb, #ffffff 66%, transparent);
  --line: color-mix(in srgb, var(--accent) 16%, #d3e3df);
  --mono: ui-monospace, 'SF Mono', 'JetBrains Mono', monospace;

  width: min(1180px, calc(100% - 32px));
  margin: 0 auto;
  padding: var(--sos-space-3, 12px) 0 var(--sos-space-8, 48px);
  display: grid;
  grid-template-columns: minmax(300px, 332px) minmax(0, 1fr);
  gap: var(--sos-space-4, 16px);
  align-items: start;
  color: var(--text);
}

/* 高阶评级 S/X/A → 金色证件 */
.adv-scope.rating-S,
.adv-scope.rating-X,
.adv-scope.rating-A {
  --accent: var(--gold);
  --accent-strong: color-mix(in srgb, var(--gold) 72%, #5a3a08);
}

.adv-state {
  grid-column: 1 / -1;
  padding: 60px 40px;
  text-align: center;
  font-weight: 800;
  color: var(--muted);
  border-radius: 20px;
  border: 1px solid var(--line);
  background: var(--glass);
  -webkit-backdrop-filter: blur(14px);
  backdrop-filter: blur(14px);
}
.adv-state--error {
  color: color-mix(in srgb, #d2453a 80%, #000);
}

/* ============ 左：冒险者证件 ============ */
.adv-id {
  position: sticky;
  top: var(--sos-space-3, 12px);
  display: flex;
  flex-direction: column;
  padding: 0 0 16px;
  border-radius: 22px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--accent) 22%, #ffffff);
  background:
    radial-gradient(
      120% 80% at 100% 0%,
      color-mix(in srgb, var(--accent) 16%, transparent),
      transparent 58%
    ),
    var(--glass);
  -webkit-backdrop-filter: blur(20px);
  backdrop-filter: blur(20px);
  box-shadow:
    0 30px 60px -34px rgba(16, 60, 56, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.72);
}

/* 分类条 */
.adv-id__band {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 11px 16px;
  background: linear-gradient(
    100deg,
    var(--accent-strong),
    color-mix(in srgb, var(--accent) 60%, #ec4faf) 130%
  );
  color: #fff;
}
.adv-id__eyebrow {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}
.adv-id__org {
  font-size: 11px;
  font-weight: 800;
  opacity: 0.86;
}

/* 紧凑身份头：头像 + 姓名 + 评级角标 */
.adv-id__head {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 17px 16px 6px;
}
.adv-id__avatar {
  position: relative;
  flex-shrink: 0;
  width: 72px;
  height: 72px;
  display: grid;
  place-items: center;
  border-radius: 18px;
  color: #fff;
  font-size: 30px;
  font-weight: 900;
  background: linear-gradient(
    150deg,
    var(--accent),
    color-mix(in srgb, var(--accent) 52%, #7bf0d8)
  );
  box-shadow:
    inset 0 0 0 3px rgba(255, 255, 255, 0.4),
    0 12px 26px -16px color-mix(in srgb, var(--accent) 65%, transparent);
}
.adv-id__avatar img {
  width: 100%;
  height: 100%;
  border-radius: 18px;
  object-fit: cover;
}
.adv-id__rank {
  position: absolute;
  right: -7px;
  bottom: -7px;
  min-width: 26px;
  height: 26px;
  padding: 0 5px;
  display: grid;
  place-items: center;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 900;
  font-style: normal;
  color: #fff;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.35);
  background: var(--accent);
  border: 2px solid color-mix(in srgb, #ffffff 80%, transparent);
  box-shadow: 0 5px 12px -5px rgba(0, 0, 0, 0.45);
}
.adv-id__ident {
  min-width: 0;
}
.adv-id__eyebrow--inline {
  display: block;
  color: var(--accent-strong);
  margin-bottom: 5px;
}
.adv-id__ident h1 {
  margin: 0;
  font-size: clamp(20px, 2.4vw, 25px);
  font-weight: 850;
  letter-spacing: -0.02em;
  line-height: 1.1;
  word-break: break-word;
}
.adv-id__rating {
  display: inline-block;
  margin-top: 9px;
  font-size: 12px;
  font-weight: 800;
  color: var(--accent-strong);
  padding: 3px 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 14%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 28%, transparent);
}
.adv-id__handle {
  display: block;
  margin-top: 7px;
  font-family: var(--mono);
  font-size: 12px;
  font-weight: 700;
  color: var(--muted);
}

/* 读数台账 */
.adv-id__readout {
  margin: 10px 18px 0;
  padding: 0;
}
.adv-id__row {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 10px 0;
}
.adv-id__row + .adv-id__row {
  border-top: 1px dashed var(--line);
}
.adv-id__row dt {
  font-size: 12px;
  font-weight: 700;
  color: var(--muted);
  letter-spacing: 0.04em;
  white-space: nowrap;
}
.adv-id__lead {
  flex: 1;
  align-self: center;
  height: 0;
  border-bottom: 1.5px dotted color-mix(in srgb, var(--accent) 32%, transparent);
}
.adv-id__row dd {
  margin: 0;
  font-size: 16px;
  font-weight: 850;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.adv-id__row dd.adv-id__contact {
  max-width: 190px;
  overflow-wrap: anywhere;
  white-space: normal;
  text-align: right;
  line-height: 1.25;
}
.adv-id__row dd.is-date {
  font-size: 14px;
}
.adv-id__row dd i {
  font-style: normal;
  font-size: 11px;
  font-weight: 700;
  color: var(--muted);
  margin-left: 7px;
}
.adv-id__connections {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.adv-id__connections button {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  padding: 2px 3px;
  color: inherit;
  font: inherit;
  cursor: pointer;
  background: transparent;
  border: 0;
  border-bottom: 1px solid transparent;
}
.adv-id__connections button:hover,
.adv-id__connections button:focus-visible {
  color: var(--accent-strong);
  border-bottom-color: var(--accent);
  outline: none;
}
.adv-id__connections button b { font: inherit; }
.adv-id__connections button i { margin-left: 0; }
.adv-id__connections > span { color: var(--muted); font-size: 11px; }
.adv-id__connections-arrow {
  margin-left: 1px;
  color: var(--accent-strong);
  font-size: 18px;
  font-style: normal;
  line-height: 1;
}
.adv-id__row--coin dd {
  color: var(--accent-strong);
}

/* 访问许可 */
.adv-id__clearance {
  margin: 14px 16px 0;
  padding: 13px 14px;
  border-radius: 14px;
  text-align: center;
  background: linear-gradient(
    155deg,
    color-mix(in srgb, var(--accent) 16%, #ffffff),
    color-mix(in srgb, var(--accent) 5%, #ffffff)
  );
  border: 1px solid color-mix(in srgb, var(--accent) 26%, transparent);
}
.adv-id__clearance-k {
  display: block;
  font-size: 10.5px;
  font-weight: 800;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted);
}
.adv-id__clearance-v {
  display: block;
  margin: 3px 0 2px;
  font-family: var(--mono);
  font-size: 24px;
  font-weight: 900;
  letter-spacing: 0.03em;
  color: var(--accent-strong);
}
.adv-id__clearance-sub {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--muted);
}

/* 条码 + 签发 */
.adv-id__foot {
  margin: 14px 16px 0;
}
.adv-id__barcode {
  display: block;
  height: 32px;
  border-radius: 5px;
  background-image: repeating-linear-gradient(
    90deg,
    var(--text) 0 1px,
    transparent 1px 3px,
    var(--text) 3px 5px,
    transparent 5px 6px,
    var(--text) 6px 9px,
    transparent 9px 11px
  );
  opacity: 0.62;
}
.adv-id__issued {
  display: block;
  margin-top: 7px;
  font-family: var(--mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--muted);
  text-align: center;
}

.adv-id__back {
  margin: 16px 16px 0;
  justify-content: center;
}

.adv-profile-follow {
  min-height: 38px;
  margin: 12px 16px 0;
  color: white;
  font: inherit;
  font-size: 13px;
  font-weight: 850;
  cursor: pointer;
  background: var(--accent-strong);
  border: 1px solid var(--accent-strong);
  border-radius: 999px;
}
.adv-profile-follow.is-following {
  color: var(--muted);
  background: color-mix(in srgb, #ffffff 52%, transparent);
  border-color: var(--line);
}
.adv-profile-follow:disabled { cursor: wait; opacity: 0.66; }
.adv-profile-follow-error {
  margin: 7px 18px 0;
  color: color-mix(in srgb, #d2453a 82%, var(--text));
  font-size: 11px;
  text-align: center;
}

/* ============ 右：档案正文 ============ */
.adv-dossier {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-4, 16px);
  min-width: 0;
}

.adv-panel {
  padding: clamp(18px, 2.2vw, 24px);
  border-radius: 20px;
  border: 1px solid var(--line);
  background: var(--glass);
  -webkit-backdrop-filter: blur(16px);
  backdrop-filter: blur(16px);
  box-shadow:
    0 22px 48px -32px rgba(16, 60, 56, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.66);
}
.adv-panel__head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: var(--sos-space-3);
  padding-bottom: var(--sos-space-2);
  border-bottom: 1px solid var(--line);
}
.adv-panel__eyebrow {
  display: block;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--accent-strong);
}
.adv-panel h2 {
  margin: 4px 0 0;
  font-size: clamp(17px, 2.2vw, 22px);
  font-weight: 850;
}
.adv-panel__meta {
  font-size: 12px;
  font-weight: 700;
  color: var(--muted);
  padding: 4px 11px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  white-space: nowrap;
}

.adv-metrics__grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}
.adv-metric {
  min-width: 0;
  padding: 12px;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--accent) 18%, var(--line));
  background: color-mix(in srgb, #ffffff 58%, transparent);
}
.adv-metric span,
.adv-metric em {
  display: block;
  font-size: 11px;
  font-weight: 750;
  color: var(--muted);
}
.adv-metric b {
  display: block;
  margin: 5px 0 3px;
  font-size: 15px;
  font-weight: 850;
  color: var(--text);
  line-height: 1.25;
  overflow-wrap: anywhere;
}
.adv-metric em {
  font-style: normal;
}

/* 作品 grid */
.adv-art-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: var(--sos-space-3);
}
.adv-art {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0;
  border: 0;
  background: transparent;
  cursor: pointer;
  text-align: left;
  font: inherit;
  color: inherit;
}
.adv-art__media {
  position: relative;
  display: block;
  aspect-ratio: 1;
  overflow: hidden;
  border-radius: 14px;
  border: 1px solid var(--line);
  background: color-mix(in srgb, var(--accent) 6%, #ffffff);
  box-shadow: 0 12px 26px -20px rgba(16, 60, 56, 0.5);
  transition:
    transform 0.28s cubic-bezier(0.22, 1, 0.36, 1),
    box-shadow 0.28s,
    border-color 0.28s;
}
.adv-art__media img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.4s cubic-bezier(0.22, 1, 0.36, 1);
}
.adv-art__status {
  position: absolute;
  left: 8px;
  bottom: 8px;
  font-size: 10px;
  font-weight: 800;
  padding: 3px 8px;
  border-radius: 999px;
  color: #fff;
  background: color-mix(in srgb, var(--accent-strong) 88%, transparent);
  -webkit-backdrop-filter: blur(6px);
  backdrop-filter: blur(6px);
}
.adv-art__status.st-pending {
  background: color-mix(in srgb, hsl(35, 90%, 45%) 90%, #000);
}
.adv-art__status.st-rejected {
  background: color-mix(in srgb, #d2453a 88%, #000);
}
.adv-art__status.st-hidden {
  background: color-mix(in srgb, #6b7280 88%, #000);
}
.adv-art:hover .adv-art__media {
  transform: translateY(-3px);
  box-shadow: 0 20px 38px -20px rgba(16, 60, 56, 0.55);
  border-color: color-mix(in srgb, var(--accent) 36%, transparent);
}
.adv-art:hover .adv-art__media img {
  transform: scale(1.08);
}
.adv-art__title {
  font-size: 13.5px;
  font-weight: 750;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.adv-social__columns {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 18px;
}
.adv-social__all {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 7px;
  margin: 0 0 9px;
  padding: 0;
  color: var(--muted);
  font: inherit;
  font-size: 12px;
  font-weight: 850;
  text-align: left;
  cursor: pointer;
  background: transparent;
  border: 0;
}
.adv-social__all b {
  min-width: 22px;
  padding: 2px 7px;
  color: var(--accent-strong);
  font-size: 10px;
  text-align: center;
  background: color-mix(in srgb, var(--accent) 11%, transparent);
  border-radius: 999px;
}
.adv-social__all em {
  margin-left: auto;
  color: var(--accent-strong);
  font-size: 10px;
  font-style: normal;
}
.adv-social__all:hover em { text-decoration: underline; }
.adv-social__columns p {
  margin: 0;
  padding: 18px 0;
  color: var(--muted);
  font-size: 12px;
}
.adv-social__list { display: grid; gap: 7px; }
.adv-social__list button {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 9px;
  padding: 8px;
  color: var(--text);
  font: inherit;
  text-align: left;
  cursor: pointer;
  background: color-mix(in srgb, #ffffff 42%, transparent);
  border: 1px solid var(--line);
  border-radius: 10px;
}
.adv-social__avatar {
  display: grid;
  width: 34px;
  height: 34px;
  overflow: hidden;
  place-items: center;
  flex: 0 0 auto;
  color: white;
  font-size: 12px;
  font-weight: 900;
  background: linear-gradient(145deg, var(--accent), color-mix(in srgb, var(--accent) 45%, #d94b9e));
  border-radius: 50%;
}
.adv-social__avatar img { width: 100%; height: 100%; object-fit: cover; }
.adv-social__list button > span:last-child { display: grid; min-width: 0; gap: 2px; }
.adv-social__list b { overflow: hidden; font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
.adv-social__list small { color: var(--muted); font-size: 10px; }

.adv-connections-overlay {
  position: fixed;
  inset: 0;
  z-index: 2200;
  display: grid;
  place-items: center;
  padding: 16px;
  background: rgba(11, 22, 31, 0.48);
  -webkit-backdrop-filter: blur(8px);
  backdrop-filter: blur(8px);
}
.adv-connections-dialog {
  --dialog-accent: var(--sos-accent, #20aa98);
  --dialog-text: var(--sos-text-primary, #16242b);
  --dialog-muted: var(--sos-text-secondary, #65737c);
  --dialog-line: color-mix(in srgb, var(--dialog-accent) 18%, #d7e2e3);
  display: flex;
  width: min(520px, calc(100vw - 32px));
  max-height: min(720px, calc(100dvh - 32px));
  overflow: hidden;
  flex-direction: column;
  color: var(--dialog-text);
  background: color-mix(in srgb, #fff 94%, #f8ebf3);
  border: 1px solid color-mix(in srgb, var(--dialog-accent) 22%, white);
  border-radius: 20px;
  box-shadow: 0 34px 90px -35px rgba(5, 30, 35, 0.72);
}
.adv-connections-dialog > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 22px 14px;
}
.adv-connections-dialog > header span {
  color: var(--dialog-accent);
  font: 800 10px/1.2 ui-monospace, SFMono-Regular, Menlo, monospace;
  letter-spacing: 0.15em;
  text-transform: uppercase;
}
.adv-connections-dialog > header h2 { margin: 4px 0 0; font-size: 21px; }
.adv-connections-dialog > header button {
  display: grid;
  width: 38px;
  height: 38px;
  place-items: center;
  flex: 0 0 auto;
  padding: 0;
  color: var(--dialog-muted);
  font: 300 28px/1 sans-serif;
  cursor: pointer;
  background: color-mix(in srgb, var(--dialog-accent) 6%, white);
  border: 1px solid var(--dialog-line);
  border-radius: 50%;
}
.adv-connections-dialog > nav {
  display: grid;
  grid-template-columns: 1fr 1fr;
  padding: 0 22px;
  border-bottom: 1px solid var(--dialog-line);
}
.adv-connections-dialog > nav button {
  position: relative;
  padding: 11px 8px 13px;
  color: var(--dialog-muted);
  font: inherit;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
  background: transparent;
  border: 0;
}
.adv-connections-dialog > nav button::after {
  position: absolute;
  right: 16px;
  bottom: -1px;
  left: 16px;
  height: 3px;
  content: '';
  background: transparent;
  border-radius: 3px 3px 0 0;
}
.adv-connections-dialog > nav button.is-active { color: var(--dialog-text); }
.adv-connections-dialog > nav button.is-active::after { background: var(--dialog-accent); }
.adv-connections-dialog > nav b { margin-left: 3px; font-size: 11px; }
.adv-connections-body {
  min-height: 230px;
  padding: 12px 14px 16px;
  overflow-y: auto;
  overscroll-behavior: contain;
}
.adv-connections-list { display: grid; gap: 5px; }
.adv-connections-list > button {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 11px;
  padding: 10px;
  color: var(--dialog-text);
  font: inherit;
  text-align: left;
  cursor: pointer;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 12px;
}
.adv-connections-list > button:hover {
  background: color-mix(in srgb, var(--dialog-accent) 6%, white);
  border-color: var(--dialog-line);
}
.adv-connections-dialog .adv-social__avatar {
  width: 42px;
  height: 42px;
  background: linear-gradient(145deg, var(--dialog-accent), #d94b9e);
}
.adv-connections-list > button > span:nth-child(2) { display: grid; min-width: 0; gap: 3px; }
.adv-connections-list b { overflow: hidden; font-size: 13px; text-overflow: ellipsis; white-space: nowrap; }
.adv-connections-list small { color: var(--dialog-muted); font-size: 10px; }
.adv-connections-list i { color: var(--dialog-muted); font-size: 24px; font-style: normal; }
.adv-connections-state {
  display: grid;
  min-height: 220px;
  place-items: center;
  color: var(--dialog-muted);
  font-size: 13px;
  text-align: center;
}
.adv-connections-state.is-error { color: #c44343; }
.adv-connections-more {
  display: block;
  width: calc(100% - 20px);
  min-height: 40px;
  margin: 10px auto 0;
  color: var(--dialog-accent);
  font: inherit;
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
  background: color-mix(in srgb, var(--dialog-accent) 7%, white);
  border: 1px solid var(--dialog-line);
  border-radius: 999px;
}
.adv-connections-more:disabled { cursor: wait; opacity: 0.62; }

/* 创作者公开留言板 */
.adv-messages {
  overflow: hidden;
}
.adv-message-compose {
  margin-bottom: 16px;
  padding: 14px;
  border: 1px solid color-mix(in srgb, var(--accent) 22%, var(--line));
  border-radius: 15px;
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent) 8%, transparent), transparent 58%),
    color-mix(in srgb, #ffffff 54%, transparent);
}
.adv-message-compose__head,
.adv-message-compose__foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.adv-message-compose__head {
  margin-bottom: 10px;
}
.adv-message-compose__head span {
  font-size: 13px;
  font-weight: 820;
  color: var(--text);
}
.adv-message-compose__head em {
  font-size: 11px;
  font-style: normal;
  font-weight: 700;
  color: var(--accent-strong);
}
.adv-message-compose textarea {
  display: block;
  width: 100%;
  min-height: 92px;
  padding: 12px 13px;
  resize: vertical;
  border: 1px solid var(--line);
  border-radius: 12px;
  outline: none;
  color: var(--text);
  background: color-mix(in srgb, #ffffff 76%, transparent);
  font: inherit;
  font-size: 13.5px;
  line-height: 1.65;
  transition: border-color 0.2s, box-shadow 0.2s;
}
.adv-message-compose textarea:focus {
  border-color: color-mix(in srgb, var(--accent) 58%, transparent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 12%, transparent);
}
.adv-message-compose__foot {
  margin-top: 10px;
}
.adv-message-compose__foot > span {
  color: var(--muted);
  font-family: var(--mono);
  font-size: 10px;
}
.adv-message-login {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
  padding: 15px 16px;
  border: 1px dashed color-mix(in srgb, var(--accent) 34%, var(--line));
  border-radius: 14px;
  background: color-mix(in srgb, var(--accent) 6%, transparent);
}
.adv-message-login div {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.adv-message-login b {
  font-size: 13.5px;
}
.adv-message-login span {
  color: var(--muted);
  font-size: 12px;
}
.adv-message-notice,
.adv-message-error {
  margin: 0 0 12px;
  padding: 9px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
}
.adv-message-notice {
  color: var(--accent-strong);
  background: color-mix(in srgb, var(--accent) 11%, transparent);
}
.adv-message-error {
  color: color-mix(in srgb, #d2453a 82%, #000);
  background: color-mix(in srgb, #ef5350 10%, transparent);
}
.adv-message-list {
  display: flex;
  flex-direction: column;
}
.adv-message {
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr);
  gap: 12px;
  padding: 15px 2px;
}
.adv-message + .adv-message {
  border-top: 1px dashed var(--line);
}
.adv-message__avatar {
  width: 42px;
  height: 42px;
  display: grid;
  place-items: center;
  overflow: hidden;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--accent) 24%, #fff);
  color: #fff;
  background: linear-gradient(145deg, var(--accent), color-mix(in srgb, var(--accent) 48%, #78d9dc));
  font-size: 15px;
  font-weight: 900;
  box-shadow: 0 10px 20px -14px color-mix(in srgb, var(--accent) 70%, transparent);
}
.adv-message__avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.adv-message__body {
  min-width: 0;
}
.adv-message__body header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}
.adv-message__body header b {
  min-width: 0;
  overflow: hidden;
  color: var(--text);
  font-size: 13px;
  font-weight: 820;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.adv-message__body time {
  flex-shrink: 0;
  color: var(--muted);
  font-family: var(--mono);
  font-size: 10px;
}
.adv-message__body p {
  margin: 6px 0 0;
  color: var(--text);
  font-size: 13.5px;
  line-height: 1.68;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}
.adv-message-more {
  display: block;
  margin: 10px auto 0;
  padding: 8px 16px;
  border: 1px solid var(--line);
  border-radius: 999px;
  color: var(--accent-strong);
  background: color-mix(in srgb, #ffffff 52%, transparent);
  font: inherit;
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
}
.adv-message-more:disabled {
  cursor: wait;
  opacity: 0.6;
}

/* 委托记录 */
.adv-quests {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.adv-quest {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 10px;
  padding: 11px 13px;
  border-radius: 12px;
  background: color-mix(in srgb, #ffffff 50%, transparent);
  border: 1px solid var(--line);
}
.adv-quest__title {
  font-size: 13.5px;
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.adv-quest__prog {
  font-size: 12px;
  font-weight: 800;
  font-family: var(--mono);
  color: var(--accent-strong);
}
.adv-quest__st {
  font-size: 10.5px;
  font-weight: 800;
  font-style: normal;
  padding: 2px 9px;
  border-radius: 999px;
  background: color-mix(in srgb, #000 6%, transparent);
  color: var(--muted);
}
.adv-quest__st.st-active {
  color: var(--accent-strong);
  background: color-mix(in srgb, var(--accent) 14%, transparent);
}
.adv-quest__st.st-completed {
  color: #fff;
  background: var(--accent);
}

/* 流水台账 */
.adv-ledgers {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sos-space-4);
}
.adv-ledger {
  display: flex;
  flex-direction: column;
}
.adv-ledger__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 2px;
  font-size: 13px;
}
.adv-ledger__row + .adv-ledger__row {
  border-top: 1px dashed var(--line);
}
.adv-ledger__row span {
  color: var(--text);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.adv-ledger__row b {
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}
.adv-ledger__row b.plus {
  color: var(--accent-strong);
}
.adv-ledger__row b.minus {
  color: color-mix(in srgb, #d2453a 80%, #000);
}
.adv-ledger__st {
  font-size: 11.5px;
  padding: 3px 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  color: var(--accent-strong);
}
.adv-ledger__st.st-rejected,
.adv-ledger__st.st-cancelled {
  color: color-mix(in srgb, #d2453a 80%, #000);
  background: color-mix(in srgb, #ef5350 12%, transparent);
}

.adv-empty {
  padding: 40px;
  text-align: center;
  color: var(--muted);
  font-weight: 600;
}
.adv-empty--sm {
  padding: 18px;
  font-size: 13px;
}

/* ============ 响应式 ============ */
@media (max-width: 900px) {
  .adv-scope {
    grid-template-columns: 1fr;
  }
  .adv-id {
    position: static;
  }
  .adv-metrics__grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .adv-ledgers {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 560px) {
  .adv-social__columns { grid-template-columns: 1fr; }
  .adv-connections-overlay { align-items: end; padding: 0; }
  .adv-connections-dialog {
    width: 100%;
    max-height: min(82dvh, 720px);
    border-radius: 20px 20px 0 0;
  }
  .adv-message-login,
  .adv-message-compose__head,
  .adv-message-compose__foot {
    align-items: stretch;
    flex-direction: column;
  }
  .adv-message-login .sos-button,
  .adv-message-compose__foot .sos-button {
    justify-content: center;
    width: 100%;
  }
  .adv-message__body header {
    align-items: flex-start;
    flex-direction: column;
    gap: 2px;
  }
}

/* ============ 关灯（暗色）适配 ============ */
/* 整条选择器必须放进 :global(...)，否则 Vue scoped 会丢弃括号外的后代选择器，
   只剩 html.art-lights-out，把变量错设到 <html> 上而被 .adv-scope 的本地定义遮蔽。 */
:global(html.art-lights-out .adv-scope) {
  --text: #f3f8ff;
  --muted: rgba(214, 230, 255, 0.7);
  --glass: rgba(14, 24, 46, 0.6);
  --line: rgba(120, 165, 220, 0.2);
  --accent-strong: color-mix(in srgb, var(--accent) 66%, #d6fff4);
}
:global(html.art-lights-out .adv-id),
:global(html.art-lights-out .adv-state) {
  background: rgba(14, 24, 46, 0.62);
}
:global(html.art-lights-out .adv-panel),
:global(html.art-lights-out .adv-quest),
:global(html.art-lights-out .adv-metric),
:global(html.art-lights-out .adv-art__media),
:global(html.art-lights-out .adv-message-compose),
:global(html.art-lights-out .adv-message-compose textarea),
:global(html.art-lights-out .adv-message-more) {
  background: rgba(12, 22, 44, 0.5);
}
:global(html.art-lights-out .adv-id__clearance) {
  background: linear-gradient(155deg, rgba(20, 58, 52, 0.5), rgba(12, 22, 44, 0.55));
}
:global(html.art-lights-out .adv-id__row--coin dd) {
  color: color-mix(in srgb, var(--accent) 78%, #d6fff4);
}
</style>
