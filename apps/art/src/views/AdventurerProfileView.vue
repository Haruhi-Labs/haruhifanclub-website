<template>
  <section class="terminal-page">
    <div v-if="loading" class="terminal-loading panel">正在连接公会终端...</div>
    <div v-else-if="error" class="terminal-loading panel">{{ error }}</div>

    <template v-else>
      <header class="terminal-hero panel" :class="`rating-${profile.rating}`">
        <div class="avatar-shell">
          <img v-if="profile.avatar_url" :src="profile.avatar_url" alt="">
          <span v-else>{{ profileInitial }}</span>
        </div>
        <div class="hero-copy">
          <p class="eyebrow">{{ isTerminal ? 'Personal Terminal' : 'Adventurer Profile' }}</p>
          <h1>{{ displayName }}</h1>
          <div class="identity-line">
            <span>{{ profile.uid }}</span>
            <b>{{ profile.rating }} 级冒险者</b>
            <b>Lv{{ profile.level }}</b>
            <b>{{ profile.accessLabel }}</b>
          </div>
        </div>
        <RouterLink v-if="!isTerminal" class="ghost-btn" to="/exchange">返回公会</RouterLink>
      </header>

      <section class="terminal-stats">
        <article class="stat-card panel">
          <span>声望</span>
          <strong>{{ profile.reputation || 0 }}</strong>
          <em>等级 Lv{{ profile.level || 1 }}</em>
        </article>
        <article class="stat-card panel">
          <span>金币</span>
          <strong>{{ profile.coins?.available || 0 }}G</strong>
          <em>冻结 {{ profile.coins?.frozen || 0 }}G</em>
        </article>
        <article class="stat-card panel">
          <span>注册时间</span>
          <strong>{{ formatDate(userInfo.createdAt || profile.creatorCreatedAt) }}</strong>
          <em>画廊档案</em>
        </article>
        <article class="stat-card panel">
          <span>投稿统计</span>
          <strong>{{ stats.total || 0 }}</strong>
          <em>个人 {{ stats.personal || 0 }} / 转载 {{ stats.network || 0 }}</em>
        </article>
      </section>

      <main class="terminal-layout">
        <section class="panel artwork-panel">
          <div class="section-head">
            <div>
              <p class="eyebrow">Archive</p>
              <h2>{{ isTerminal ? '我的画廊档案' : '公开画作' }}</h2>
            </div>
            <span>{{ stats.haruhi || 0 }} 张凉宫画作</span>
          </div>

          <div v-if="artworks.length" class="profile-art-grid">
            <button
              v-for="art in artworks"
              :key="art.id"
              type="button"
              class="profile-art-card"
              @click="openArtwork(art)"
            >
              <img :src="thumbUrl(art.image_url, 360)" :alt="art.title || 'artwork'" loading="lazy">
              <span>{{ art.title || '未命名作品' }}</span>
              <b>{{ art.status || 'approved' }}</b>
            </button>
          </div>
          <div v-else class="empty">这个冒险者还没有公开画作。</div>
        </section>

        <aside class="terminal-side">
          <section class="panel license-card">
            <p class="eyebrow">Clearance</p>
            <h2>访问许可卡</h2>
            <div class="license-code">{{ profile.accessShortLabel || '档案0' }}</div>
            <p>{{ profile.accessLabel || '0级公开档案许可' }}</p>
            <p class="muted-line">冒险者评级 {{ profile.rating }} · {{ profile.ratingLabel }}</p>
          </section>

          <section v-if="isTerminal" class="panel activity-card">
            <p class="eyebrow">Quest Log</p>
            <h2>委托记录</h2>
            <div v-if="claims.length" class="activity-list">
              <div v-for="claim in claims.slice(0, 6)" :key="claim.id" class="activity-row">
                <span>{{ claim.title }}</span>
                <b>{{ claim.progress }}/{{ claim.targetCount }}</b>
                <em>{{ claimLabel(claim.status) }}</em>
              </div>
            </div>
            <div v-else class="empty small">还没有接取委托。</div>
          </section>
        </aside>
      </main>

      <section v-if="isTerminal" class="history-layout">
        <article class="panel history-panel">
          <p class="eyebrow">Coin Ledger</p>
          <h2>金币流水</h2>
          <div v-for="item in coinsHistory.slice(0, 8)" :key="`${item.createdAt}-${item.note}`" class="ledger-row">
            <span>{{ item.note || item.sourceType }}</span>
            <b :class="{ plus: item.coins > 0, minus: item.coins < 0 }">{{ item.coins > 0 ? '+' : '' }}{{ item.coins }}G</b>
          </div>
          <div v-if="!coinsHistory.length" class="empty small">暂无金币记录。</div>
        </article>

        <article class="panel history-panel">
          <p class="eyebrow">Redemption</p>
          <h2>兑换申请</h2>
          <div v-for="item in redemptions.slice(0, 8)" :key="item.id" class="ledger-row">
            <span>{{ item.rewardName }}</span>
            <b>{{ redemptionLabel(item.status) }}</b>
          </div>
          <div v-if="!redemptions.length" class="empty small">暂无兑换申请。</div>
        </article>
      </section>
    </template>
  </section>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, thumbUrl } from '../services/api.js'

const route = useRoute()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const profile = ref({})
const userInfo = ref({})
const stats = ref({})
const artworks = ref([])
const claims = ref([])
const coinsHistory = ref([])
const redemptions = ref([])

const isTerminal = computed(() => route.name === 'terminal')
const displayName = computed(() => (
  userInfo.value.displayName ||
  userInfo.value.username ||
  profile.value.uid ||
  'Observer'
))
const profileInitial = computed(() => String(displayName.value || 'O').slice(0, 1).toUpperCase())

async function loadProfile() {
  loading.value = true
  error.value = ''
  try {
    const res = isTerminal.value
      ? await api.guildTerminal()
      : await api.guildProfile(route.params.uid)

    profile.value = res.profile || {}
    userInfo.value = res.user || res.profile?.user || {}
    stats.value = res.stats || {}
    artworks.value = res.artworks || []
    claims.value = res.claims || []
    coinsHistory.value = res.coinsHistory || []
    redemptions.value = res.redemptions || []
  } catch (err) {
    error.value = err.message || '冒险者档案读取失败'
  } finally {
    loading.value = false
  }
}

function openArtwork(art) {
  router.push({ name: 'gallery', query: { artwork: art.id } })
}

function formatDate(value) {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleDateString()
}

function claimLabel(status) {
  const map = { active: '进行中', completed: '已完成', abandoned: '已放弃' }
  return map[status] || status
}

function redemptionLabel(status) {
  const map = {
    pending: '待审核',
    approved: '已批准',
    rejected: '已拒绝',
    cancelled: '已取消',
    fulfilled: '已发放'
  }
  return map[status] || status
}

watch(() => route.fullPath, loadProfile)
onMounted(loadProfile)
</script>

<style scoped>
.terminal-page {
  display: grid;
  width: min(1180px, calc(100% - 32px));
  max-width: 1180px;
  margin: 0 auto;
  padding: 18px 0 36px;
  gap: 18px;
}

.panel {
  border: 0;
  border-top: 1px solid rgba(148, 163, 184, 0.22);
  border-bottom: 1px solid rgba(148, 163, 184, 0.18);
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.42), rgba(245, 250, 255, 0.22)),
    radial-gradient(circle at 10% 0%, rgba(103, 232, 249, 0.16), transparent 38%);
  box-shadow: none;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border-radius: 0;
}

.terminal-loading {
  padding: 28px;
  color: var(--text);
  font-weight: 950;
}

.terminal-hero {
  display: grid;
  grid-template-columns: 96px minmax(0, 1fr) auto;
  gap: 18px;
  align-items: center;
  padding: 22px;
}

.avatar-shell {
  display: grid;
  width: 88px;
  height: 88px;
  place-items: center;
  overflow: hidden;
  color: #fff;
  font-size: 36px;
  font-weight: 950;
  background: linear-gradient(135deg, #f43f5e, #6366f1, #38bdf8);
  border: 2px solid rgba(255, 255, 255, 0.7);
  border-radius: 26px;
  box-shadow: 0 14px 30px rgba(244, 63, 94, 0.2);
}

.avatar-shell img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.eyebrow {
  margin: 0 0 6px;
  color: var(--accent);
  font-size: 12px;
  font-weight: 950;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h1,
h2,
p {
  margin: 0;
}

.terminal-hero h1 {
  color: var(--text);
  font-size: clamp(32px, 5vw, 52px);
  font-weight: 950;
}

.identity-line {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.identity-line span,
.identity-line b,
.section-head span,
.muted-line {
  padding: 5px 10px;
  color: rgba(15, 23, 42, 0.72);
  font-size: 12px;
  font-weight: 900;
  background: rgba(255, 255, 255, 0.62);
  border: 1px solid rgba(148, 163, 184, 0.18);
  border-radius: 999px;
}

.ghost-btn {
  min-height: 38px;
  padding: 9px 14px;
  color: #0f4f63;
  font-weight: 950;
  text-decoration: none;
  background: rgba(255, 255, 255, 0.64);
  border: 1px solid rgba(103, 232, 249, 0.26);
  border-radius: 999px;
}

.terminal-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.stat-card {
  padding: 16px 18px;
}

.stat-card span {
  color: var(--muted);
  font-size: 12px;
  font-weight: 950;
}

.stat-card strong {
  display: block;
  margin-top: 8px;
  overflow: hidden;
  color: var(--text);
  font-size: 24px;
  font-weight: 950;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stat-card em {
  color: var(--muted);
  font-size: 12px;
  font-weight: 800;
}

.terminal-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 320px;
  gap: 18px;
}

.artwork-panel,
.license-card,
.activity-card,
.history-panel {
  padding: 18px;
}

.terminal-side {
  display: grid;
  gap: 18px;
}

.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 14px;
  margin-bottom: 14px;
}

.section-head h2,
.license-card h2,
.activity-card h2,
.history-panel h2 {
  color: var(--text);
  font-size: 22px;
  font-weight: 950;
}

.profile-art-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.profile-art-card {
  position: relative;
  display: grid;
  min-height: 170px;
  overflow: hidden;
  padding: 0;
  color: #fff;
  text-align: left;
  background: #101827;
  border: 0;
  border-radius: 18px;
  cursor: pointer;
}

.profile-art-card img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.24s ease;
}

.profile-art-card:hover img {
  transform: scale(1.05);
}

.profile-art-card::after {
  position: absolute;
  inset: 0;
  content: "";
  background: linear-gradient(180deg, transparent 42%, rgba(15, 23, 42, 0.74));
}

.profile-art-card span,
.profile-art-card b {
  position: relative;
  z-index: 1;
  align-self: end;
  padding: 0 12px 12px;
  font-weight: 950;
}

.profile-art-card b {
  position: absolute;
  top: 10px;
  right: 10px;
  align-self: auto;
  padding: 4px 8px;
  color: #0f172a;
  font-size: 11px;
  background: rgba(255, 255, 255, 0.78);
  border-radius: 999px;
}

.license-code {
  margin: 18px 0 10px;
  color: #fff;
  font-size: 36px;
  font-weight: 950;
  text-align: center;
  background: linear-gradient(135deg, #0ea5e9, #f43f5e);
  border-radius: 20px;
  padding: 22px;
}

.license-card p {
  color: var(--muted);
  font-weight: 800;
  line-height: 1.6;
}

.activity-list {
  display: grid;
  gap: 8px;
}

.activity-row,
.ledger-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 10px;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid rgba(148, 163, 184, 0.16);
}

.activity-row span,
.ledger-row span {
  overflow: hidden;
  color: var(--text);
  font-weight: 850;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.activity-row b,
.activity-row em,
.ledger-row b {
  color: var(--muted);
  font-size: 12px;
  font-weight: 950;
}

.history-layout {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 18px;
}

.ledger-row {
  grid-template-columns: minmax(0, 1fr) auto;
}

.plus {
  color: #059669 !important;
}

.minus {
  color: #dc2626 !important;
}

.empty {
  color: var(--muted);
  font-weight: 800;
}

.empty.small {
  font-size: 13px;
}

:global(html.art-lights-out) .panel {
  border-top-color: rgba(125, 211, 252, 0.18);
  border-bottom-color: rgba(125, 211, 252, 0.14);
  background:
    linear-gradient(135deg, rgba(8, 14, 33, 0.44), rgba(28, 22, 58, 0.26)),
    radial-gradient(circle at 10% 0%, rgba(125, 211, 252, 0.12), transparent 34%);
  box-shadow: none;
}

:global(html.art-lights-out) .terminal-hero h1,
:global(html.art-lights-out) .stat-card strong,
:global(html.art-lights-out) .section-head h2,
:global(html.art-lights-out) .license-card h2,
:global(html.art-lights-out) .activity-card h2,
:global(html.art-lights-out) .history-panel h2,
:global(html.art-lights-out) .activity-row span,
:global(html.art-lights-out) .ledger-row span {
  color: #f7fbff;
}

:global(html.art-lights-out) .identity-line span,
:global(html.art-lights-out) .identity-line b,
:global(html.art-lights-out) .section-head span,
:global(html.art-lights-out) .ghost-btn,
:global(html.art-lights-out) .muted-line {
  color: #bae6fd;
  background: rgba(255, 255, 255, 0.07);
  border-color: rgba(125, 211, 252, 0.14);
}

@media (max-width: 980px) {
  .terminal-hero,
  .terminal-layout,
  .history-layout {
    grid-template-columns: 1fr;
  }

  .terminal-stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .profile-art-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 620px) {
  .terminal-stats,
  .profile-art-grid {
    grid-template-columns: 1fr;
  }
}

.terminal-page {
  position: relative;
  width: min(1200px, calc(100% - 32px));
  padding: 30px 0 56px;
  gap: 26px;
  isolation: isolate;
}

.terminal-page::before,
.terminal-page::after {
  position: absolute;
  inset: 0;
  z-index: -1;
  pointer-events: none;
  content: "";
}

.terminal-page::before {
  background:
    linear-gradient(125deg, transparent 0 18%, rgba(56, 189, 248, 0.09) 18% 19%, transparent 19% 100%),
    linear-gradient(22deg, transparent 0 62%, rgba(244, 63, 94, 0.08) 62% 63%, transparent 63% 100%),
    radial-gradient(circle at 12% 14%, rgba(99, 102, 241, 0.14), transparent 30%),
    radial-gradient(circle at 92% 12%, rgba(56, 189, 248, 0.12), transparent 26%);
}

.terminal-page::after {
  top: 116px;
  bottom: auto;
  height: 1px;
  background: linear-gradient(90deg, #38bdf8, transparent 45%, #f43f5e);
  opacity: 0.54;
}

.panel {
  position: relative;
  overflow: visible;
  background: transparent;
  border: 0;
  border-radius: 0;
  box-shadow: none;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

.terminal-hero {
  min-height: 220px;
  grid-template-columns: 122px minmax(0, 1fr) auto;
  padding: 38px 0 34px 34px;
  border-left: 10px solid #38bdf8;
  border-bottom: 1px solid rgba(15, 23, 42, 0.14);
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.5), transparent 76%),
    repeating-linear-gradient(90deg, rgba(15, 23, 42, 0.035) 0 1px, transparent 1px 36px);
}

.terminal-hero::before {
  position: absolute;
  right: 8%;
  bottom: -24px;
  width: 260px;
  height: 110px;
  border: 1px solid rgba(56, 189, 248, 0.2);
  transform: skewX(-18deg);
  content: "";
}

.avatar-shell {
  width: 104px;
  height: 104px;
  border: 0;
  border-radius: 0;
  box-shadow: none;
  clip-path: polygon(0 0, 86% 0, 100% 18%, 100% 100%, 14% 100%, 0 82%);
}

.terminal-hero h1 {
  font-size: clamp(44px, 7vw, 82px);
  line-height: 0.95;
}

.identity-line span,
.identity-line b,
.section-head span,
.muted-line {
  background: transparent;
  border: 0;
  border-bottom: 1px solid rgba(15, 23, 42, 0.16);
  border-radius: 0;
}

.terminal-stats {
  gap: 0;
  border-top: 1px solid rgba(15, 23, 42, 0.12);
  border-bottom: 1px solid rgba(15, 23, 42, 0.12);
}

.stat-card {
  padding: 20px 18px;
  border-right: 1px solid rgba(15, 23, 42, 0.1);
}

.stat-card:last-child {
  border-right: 0;
}

.stat-card strong {
  font-size: 30px;
}

.terminal-layout {
  grid-template-columns: minmax(0, 1.12fr) minmax(280px, 0.88fr);
  gap: 44px;
  align-items: start;
}

.artwork-panel,
.license-card,
.activity-card,
.history-panel {
  padding: 30px 0;
  border-top: 1px solid rgba(15, 23, 42, 0.12);
  border-bottom: 1px solid rgba(15, 23, 42, 0.1);
}

.section-head {
  padding-left: 22px;
  border-left: 6px solid #38bdf8;
}

.section-head h2,
.license-card h2,
.activity-card h2,
.history-panel h2 {
  font-size: 30px;
}

.profile-art-grid {
  gap: 2px;
  background: rgba(15, 23, 42, 0.1);
}

.profile-art-card {
  min-height: 196px;
  border-radius: 0;
  box-shadow: none;
}

.profile-art-card b {
  top: 0;
  right: 0;
  color: #fff;
  background: rgba(15, 23, 42, 0.76);
  border-radius: 0;
}

.terminal-side {
  gap: 26px;
}

.license-code {
  margin: 22px 0 14px;
  padding: 24px 0;
  color: transparent;
  font-size: clamp(42px, 7vw, 76px);
  line-height: 0.95;
  background: linear-gradient(135deg, #38bdf8, #6366f1 50%, #f43f5e);
  background-clip: text;
  -webkit-background-clip: text;
  border-top: 1px solid rgba(56, 189, 248, 0.28);
  border-bottom: 1px solid rgba(244, 63, 94, 0.22);
  border-radius: 0;
}

.activity-row,
.ledger-row {
  padding: 14px 0;
  border-bottom-color: rgba(15, 23, 42, 0.14);
}

.history-layout {
  gap: 44px;
}

.ghost-btn {
  border-radius: 0;
  box-shadow: none;
}

:global(html.art-lights-out) .terminal-page::before {
  background:
    linear-gradient(125deg, transparent 0 18%, rgba(125, 211, 252, 0.1) 18% 19%, transparent 19% 100%),
    linear-gradient(22deg, transparent 0 62%, rgba(244, 63, 94, 0.1) 62% 63%, transparent 63% 100%),
    radial-gradient(circle at 12% 14%, rgba(99, 102, 241, 0.14), transparent 30%),
    radial-gradient(circle at 92% 12%, rgba(125, 211, 252, 0.12), transparent 26%);
}

:global(html.art-lights-out) .panel,
:global(html.art-lights-out) .terminal-stats,
:global(html.art-lights-out) .artwork-panel,
:global(html.art-lights-out) .license-card,
:global(html.art-lights-out) .activity-card,
:global(html.art-lights-out) .history-panel {
  background: transparent;
  border-top-color: rgba(125, 211, 252, 0.18);
  border-bottom-color: rgba(125, 211, 252, 0.14);
  box-shadow: none;
}

:global(html.art-lights-out) .terminal-hero {
  background:
    linear-gradient(90deg, rgba(12, 20, 44, 0.46), transparent 76%),
    repeating-linear-gradient(90deg, rgba(125, 211, 252, 0.04) 0 1px, transparent 1px 36px);
}

:global(html.art-lights-out) .identity-line span,
:global(html.art-lights-out) .identity-line b,
:global(html.art-lights-out) .section-head span,
:global(html.art-lights-out) .ghost-btn,
:global(html.art-lights-out) .muted-line {
  background: transparent;
  border-color: rgba(125, 211, 252, 0.16);
}

@media (max-width: 980px) {
  .terminal-hero,
  .terminal-layout,
  .history-layout {
    grid-template-columns: 1fr;
  }

  .terminal-hero {
    padding-right: 18px;
  }
}

@media (max-width: 620px) {
  .terminal-page {
    width: min(100% - 20px, 1200px);
  }

  .terminal-stats {
    grid-template-columns: 1fr;
  }

  .stat-card {
    border-right: 0;
    border-bottom: 1px solid rgba(15, 23, 42, 0.1);
  }
}
</style>
