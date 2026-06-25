<template>
  <section class="guild-page">
    <header class="guild-hero panel">
      <div>
        <p class="eyebrow">SOS Brigade Guild</p>
        <h1>冒险者公会委托板</h1>
        <p>
          画廊积分已等量迁移为金币。游客可以查看规则、排行榜和补给目录；
          登录后可接取委托、申请评级并提交兑换申请。
        </p>
      </div>

      <div class="guild-card" :class="`rating-${profile.rating}`">
        <span>冒险者评级</span>
        <strong>{{ profile.rating }}</strong>
        <em>{{ profile.ratingLabel || '游客观测中' }}</em>
      </div>
    </header>

    <section class="guild-status">
      <article class="status-tile panel">
        <span>冒险者编号</span>
        <strong>{{ profile.uid || 'VISITOR' }}</strong>
      </article>
      <article class="status-tile panel">
        <span>金币</span>
        <strong>{{ coinText }}</strong>
      </article>
      <article class="status-tile panel">
        <span>声望 / 等级</span>
        <strong>{{ profile.reputation || 0 }} · Lv{{ profile.level || 1 }}</strong>
      </article>
      <article class="status-tile panel">
        <span>访问许可</span>
        <strong>{{ profile.accessShortLabel || '档案0' }}</strong>
      </article>
    </section>

    <div class="rule-corner">
      <button type="button" class="rule-toggle" @click="rulesOpen = !rulesOpen">
        规则书
      </button>
      <div v-if="rulesOpen" class="rule-popover">
        <button
          v-for="book in ruleBooks"
          :key="book.id"
          type="button"
          class="rule-btn"
          @click="activeRule = book; rulesOpen = false"
        >
          {{ book.title }}
        </button>
      </div>
    </div>

    <nav class="guild-tabs panel" aria-label="公会功能分页">
      <button
        v-for="tab in guildTabs"
        :key="tab.id"
        type="button"
        class="guild-tab"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        <span>{{ tab.eyebrow }}</span>
        <strong>{{ tab.label }}</strong>
        <em>{{ tab.summary }}</em>
      </button>
    </nav>

    <main v-if="activeTab === 'quests'" class="guild-tab-panel">
      <section class="quest-column panel">
        <header class="section-head">
          <div>
            <p class="eyebrow">Bounty Board</p>
            <h2>委托列表</h2>
          </div>
          <button type="button" class="ghost-btn" @click="loadGuild">刷新</button>
        </header>

        <div
          v-for="group in questGroups"
          :key="group.id"
          class="quest-group"
        >
          <div class="quest-group-title">
            <span>{{ group.label }}</span>
            <b>{{ group.items.length }}</b>
          </div>

          <article
            v-for="quest in group.items"
            :key="quest.id"
            class="quest-card"
            :class="[`difficulty-${quest.difficulty}`, { locked: !quest.unlocked, completed: questStatus(quest) === 'completed' }]"
          >
            <div class="quest-main">
              <div class="quest-title-row">
                <h3>{{ quest.title }}</h3>
                <span>{{ questStatusLabel(quest) }}</span>
              </div>
              <p>{{ quest.description || conditionLabel(quest.conditionKind) }}</p>

              <div class="quest-meta">
                <span>评级 {{ quest.requiredRating }}</span>
                <span>{{ quest.requiredAccessLabel }}</span>
                <span v-if="quest.deadlineHours">{{ quest.deadlineHours }}h</span>
                <span>声望 +{{ quest.rewardReputation }}</span>
                <span v-if="quest.rewardCoins > 0">金币 +{{ quest.rewardCoins }}</span>
              </div>

              <div class="progress-line" :aria-label="`${questProgress(quest)} / ${questTarget(quest)}`">
                <span :style="{ width: `${questProgressPercent(quest)}%` }"></span>
              </div>
            </div>

            <button
              type="button"
              class="action-btn"
              :disabled="questButtonDisabled(quest) || busyQuestId === quest.id"
              @click="claimQuest(quest)"
            >
              {{ questButtonLabel(quest) }}
            </button>
          </article>
        </div>
      </section>
    </main>

    <section v-else-if="activeTab === 'rewards'" class="reward-section panel guild-tab-panel">
      <header class="section-head">
        <div>
          <p class="eyebrow">Reward Counter</p>
          <h2>SOS团补给兑换柜台</h2>
        </div>
        <RouterLink v-if="session.state.user" class="ghost-btn link-btn" to="/terminal">查看个人终端</RouterLink>
      </header>

      <div class="reward-grid">
        <article
          v-for="reward in rewards"
          :key="reward.id"
          class="reward-card"
          :class="{ locked: !reward.unlocked }"
        >
          <div class="reward-visual">
            <img v-if="reward.imageUrl" :src="reward.imageUrl" :alt="reward.name">
            <span v-else>{{ reward.rewardType === 'physical' ? '实体' : '虚拟' }}</span>
          </div>
          <div class="reward-copy">
            <h3>{{ reward.name }}</h3>
            <p>{{ reward.description || '管理员发放的公会补给。' }}</p>
            <div class="quest-meta">
              <span>{{ reward.priceCoins }}G</span>
              <span>评级 {{ reward.requiredRating }}</span>
              <span>{{ reward.requiredAccessLabel }}</span>
              <span v-if="reward.stock !== null && reward.stock !== undefined">库存 {{ reward.stock }}</span>
            </div>
          </div>
          <button
            type="button"
            class="action-btn"
            :disabled="!reward.unlocked || busyRewardId === reward.id"
            @click="redeemReward(reward)"
          >
            {{ reward.unlocked ? '提交兑换' : '条件不足' }}
          </button>
        </article>
      </div>
      <div v-if="!rewards.length" class="empty">暂无可查看的补给项目</div>

      <div v-if="redemptions.length" class="redemption-strip">
        <span>我的兑换申请</span>
        <b v-for="item in redemptions.slice(0, 4)" :key="item.id">
          {{ item.rewardName }} · {{ redemptionLabel(item.status) }}
        </b>
      </div>
    </section>

    <section v-else-if="activeTab === 'ranking'" class="panel rank-panel rank-page guild-tab-panel">
      <header class="section-head">
        <div>
          <p class="eyebrow">Ranking</p>
          <h2>冒险者排行榜</h2>
        </div>
      </header>

      <RouterLink
        v-for="(item, index) in leaderboard"
        :key="item.uid"
        class="leader-row"
        :to="{ name: 'adventurer-profile', params: { uid: item.uid } }"
      >
        <span class="leader-no">#{{ index + 1 }}</span>
        <span class="leader-rating" :class="`rating-${item.rating}`">{{ item.rating }}</span>
        <span class="leader-name">{{ item.uid }}</span>
        <b>{{ item.coins }}G</b>
      </RouterLink>
      <div v-if="!leaderboard.length" class="empty">暂无公会成员记录</div>
    </section>

    <section v-else-if="activeTab === 'rating'" class="panel rating-panel rating-page guild-tab-panel">
      <header class="section-head">
        <div>
          <p class="eyebrow">Rank Up</p>
          <h2>评级申请</h2>
        </div>
      </header>

      <p class="side-copy">{{ nextRatingText }}</p>
      <textarea v-model="ratingNote" placeholder="给管理员的申请说明，可留空"></textarea>
      <button
        type="button"
        class="action-btn full"
        :disabled="!canApplyRating || applyingRating"
        @click="applyRating"
      >
        {{ applyButtonText }}
      </button>
    </section>

    <footer class="guild-feedback panel" aria-live="polite">
      {{ feedback }}
    </footer>

    <div v-if="activeRule" class="rule-modal" @click.self="activeRule = null">
      <article class="rule-dialog panel">
        <button class="close-btn" type="button" @click="activeRule = null">×</button>
        <p class="eyebrow">Guild Rule Book</p>
        <h2>{{ activeRule.title }}</h2>
        <ul>
          <li v-for="line in activeRule.lines" :key="line">{{ line }}</li>
        </ul>
      </article>
    </div>
  </section>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useSession } from '@haruhi/auth-ui'
import { api } from '../services/api.js'

const router = useRouter()
const session = useSession('/api')

const quests = ref([])
const rewards = ref([])
const leaderboard = ref([])
const redemptions = ref([])
const profile = ref({
  uid: '',
  rating: 'F',
  ratingLabel: '',
  level: 1,
  reputation: 0,
  accessShortLabel: '',
  coins: { total: 0, available: 0, frozen: 0 }
})
const feedback = ref('公会柜台待机中。委托进度会由系统根据你的真实画廊行为自动查验。')
const ratingNote = ref('')
const activeRule = ref(null)
const activeTab = ref('quests')
const rulesOpen = ref(false)
const busyQuestId = ref(null)
const busyRewardId = ref(null)
const applyingRating = ref(false)

const ruleBooks = [
  {
    id: 'coins',
    title: '金币与声望',
    lines: [
      '原积分按 1:1 等量迁移为金币，金币可用于兑换虚拟或实体奖励。',
      '上传作品可以获得金币与声望；浏览、评论、点赞等委托只增加声望。',
      '兑换会先冻结对应金币，管理员审核通过后正式扣除，拒绝则解除冻结。'
    ]
  },
  {
    id: 'rating',
    title: '冒险者评级 FEDCBASX',
    lines: [
      '评级从 F 到 X 逐级提升，代表你在画廊中的长期贡献与观测资历。',
      '声望决定冒险者等级，评级提升还需要满足凉宫个人作品数量等条件。',
      '评级提升后会解锁更困难的委托，困难委托通常给予大量声望。'
    ]
  },
  {
    id: 'access',
    title: '访问许可',
    lines: [
      '档案0：公开档案许可，可浏览公开画廊与公会规则。',
      '观测1：观测员许可，可参与日常委托与基础兑换。',
      '异常2 / 闭锁3：更高阶观测许可，由管理员根据社团需要授予。'
    ]
  },
  {
    id: 'redemption',
    title: '兑换与发放',
    lines: [
      '虚拟奖励由管理员审核后在站内或指定渠道发放。',
      '实体奖励需要管理员线下核对信息后发放，页面仅提交兑换申请。',
      '商品、库存、委托内容和奖励数值都可由管理员后台维护。'
    ]
  }
]

const coinText = computed(() => {
  const coins = profile.value.coins || {}
  return `${Number(coins.available || 0)}G`
})

const guildTabs = computed(() => [
  {
    id: 'quests',
    eyebrow: 'Bounty',
    label: '委托',
    summary: `${quests.value.length}项`
  },
  {
    id: 'rewards',
    eyebrow: 'Counter',
    label: '兑换',
    summary: `${rewards.value.length}项`
  },
  {
    id: 'ranking',
    eyebrow: 'Ranking',
    label: '排行',
    summary: `${leaderboard.value.length}人`
  },
  {
    id: 'rating',
    eyebrow: 'Rank Up',
    label: '评级',
    summary: profile.value.nextRating?.target ? `申请${profile.value.nextRating.target}` : '当前'
  }
])

const questGroups = computed(() => {
  const groups = [
    { id: 'daily', label: '日常委托', items: [] },
    { id: 'limited', label: '限时委托', items: [] },
    { id: 'hard', label: '困难委托', items: [] },
    { id: 'unknown', label: '未知委托', items: [] }
  ]
  for (const quest of quests.value) {
    const type = quest.questType || 'daily'
    const target = groups.find((item) => item.id === type) || groups[0]
    target.items.push(quest)
  }
  return groups.filter((group) => group.items.length)
})

const canApplyRating = computed(() => {
  const next = profile.value.nextRating
  return !!session.state.user && !!next?.target && next?.eligible
})

const nextRatingText = computed(() => {
  if (!session.state.user) return '登录后可查看下一评级申请条件。'
  const next = profile.value.nextRating
  if (!next?.target) return '你已经站在当前规则书的最高评级。'
  const status = next.eligible ? '已满足申请条件' : '尚未满足申请条件'
  return `下一评级 ${next.target}：需要 ${next.requiredReputation} 声望 / ${next.requiredHaruhiCount} 张凉宫个人作品，当前${status}。`
})

const applyButtonText = computed(() => {
  if (!session.state.user) return '登录后申请'
  const next = profile.value.nextRating
  if (!next?.target) return '暂无更高评级'
  return `申请 ${next.target} 评级`
})

function requireLogin() {
  if (session.state.user) return true
  router.push({ name: 'login', query: { redirect: '/exchange' } })
  return false
}

async function loadGuild() {
  try {
    const [questRes, rewardRes, rankRes] = await Promise.all([
      api.guildQuests(),
      api.guildRewards(),
      api.guildLeaderboard()
    ])
    quests.value = questRes.data || []
    rewards.value = rewardRes.data || []
    leaderboard.value = rankRes.data || []
    profile.value = questRes.profile || rewardRes.profile || profile.value

    if (session.state.user) {
      const redRes = await api.guildMyRedemptions()
      redemptions.value = redRes.data || []
    } else {
      redemptions.value = []
    }
  } catch (error) {
    feedback.value = `公会数据加载失败：${error.message || '未知错误'}`
  }
}

function questStatus(quest) {
  return quest.claim?.status || (quest.unlocked ? 'ready' : 'locked')
}

function questStatusLabel(quest) {
  const status = questStatus(quest)
  if (status === 'completed') return '已完成'
  if (status === 'active') return '进行中'
  if (status === 'locked') return '未解锁'
  return '可接取'
}

function questProgress(quest) {
  return Number(quest.claim?.progress || 0)
}

function questTarget(quest) {
  return Number(quest.claim?.targetCount || quest.targetCount || 1)
}

function questProgressPercent(quest) {
  const target = questTarget(quest)
  if (target <= 0) return 0
  return Math.min(100, Math.round((questProgress(quest) / target) * 100))
}

function questButtonDisabled(quest) {
  return !quest.unlocked || questStatus(quest) === 'completed'
}

function questButtonLabel(quest) {
  const status = questStatus(quest)
  if (!session.state.user) return '登录接取'
  if (!quest.unlocked) return '权限不足'
  if (status === 'completed') return '已结算'
  if (status === 'active') return `${questProgress(quest)}/${questTarget(quest)}`
  return '接取委托'
}

function conditionLabel(kind) {
  const map = {
    browse_artworks: '浏览画廊作品',
    comment_artworks: '留下公开评论',
    like_artworks: '给作品点赞',
    upload_personal_haruhi: '上传凉宫个人作品',
    upload_personal_any: '上传个人作品',
    upload_network: '提交转载资料',
    manual_admin_verify: '等待管理员验收'
  }
  return map[kind] || '完成公会指定动作'
}

async function claimQuest(quest) {
  if (!requireLogin()) return
  if (!quest.unlocked || questStatus(quest) === 'completed') return

  busyQuestId.value = quest.id
  try {
    await api.guildClaimQuest(quest.id)
    feedback.value = `已接取「${quest.title}」。系统会自动检查完成状态。`
    await loadGuild()
  } catch (error) {
    feedback.value = error.message || '接取委托失败'
  } finally {
    busyQuestId.value = null
  }
}

async function applyRating() {
  if (!requireLogin() || !canApplyRating.value) return
  applyingRating.value = true
  try {
    await api.guildApplyRating({
      targetRating: profile.value.nextRating.target,
      note: ratingNote.value
    })
    ratingNote.value = ''
    feedback.value = `已提交 ${profile.value.nextRating.target} 评级申请，等待管理员审核。`
  } catch (error) {
    feedback.value = error.message || '评级申请失败'
  } finally {
    applyingRating.value = false
  }
}

async function redeemReward(reward) {
  if (!requireLogin() || !reward.unlocked) return
  const note = window.prompt(`提交「${reward.name}」兑换申请，给管理员的备注：`, '')
  if (note === null) return

  busyRewardId.value = reward.id
  try {
    const res = await api.guildRedeemReward(reward.id, { note })
    feedback.value = res.message || `已提交「${reward.name}」兑换申请。`
    await loadGuild()
  } catch (error) {
    feedback.value = error.message || '兑换申请失败'
  } finally {
    busyRewardId.value = null
  }
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

onMounted(async () => {
  if (!session.state.ready) {
    try {
      await session.refresh()
    } catch {}
  }
  await loadGuild()
})
</script>

<style scoped>
.guild-page {
  position: relative;
  display: grid;
  width: min(1220px, calc(100% - 32px));
  max-width: 1220px;
  margin: 0 auto;
  padding: 18px 0 36px;
  gap: 18px;
}

.panel {
  border: 0;
  border-top: 1px solid rgba(148, 163, 184, 0.22);
  border-bottom: 1px solid rgba(148, 163, 184, 0.18);
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.42), rgba(255, 247, 251, 0.2)),
    radial-gradient(circle at 8% 0%, rgba(245, 51, 93, 0.12), transparent 34%);
  box-shadow: none;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border-radius: 0;
}

.guild-hero {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 180px;
  gap: 24px;
  align-items: center;
  padding: 26px;
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
h3,
p {
  margin: 0;
}

.guild-hero h1 {
  color: var(--text);
  font-size: clamp(32px, 5vw, 54px);
  font-weight: 950;
  letter-spacing: 0;
}

.guild-hero p,
.side-copy,
.quest-card p,
.reward-card p {
  color: var(--muted);
  font-weight: 750;
  line-height: 1.7;
}

.guild-card {
  display: grid;
  min-height: 160px;
  place-items: center;
  padding: 18px;
  color: #fff;
  text-align: center;
  background:
    radial-gradient(circle at 34% 22%, rgba(255, 255, 255, 0.45), transparent 30%),
    linear-gradient(135deg, #f43f5e, #6366f1);
  border: 1px solid rgba(255, 255, 255, 0.48);
  border-radius: 28px;
  box-shadow: 0 18px 34px rgba(244, 63, 94, 0.2);
}

.guild-card span,
.guild-card em {
  font-size: 12px;
  font-weight: 950;
}

.guild-card strong {
  font-size: 64px;
  font-weight: 950;
  line-height: 1;
}

.guild-status {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.status-tile {
  padding: 16px 18px;
}

.status-tile span,
.redemption-strip span {
  color: var(--muted);
  font-size: 12px;
  font-weight: 950;
}

.status-tile strong {
  display: block;
  margin-top: 8px;
  overflow: hidden;
  color: var(--text);
  font-size: 22px;
  font-weight: 950;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rule-corner {
  position: absolute;
  top: 20px;
  right: 0;
  z-index: 6;
  display: grid;
  justify-items: end;
  gap: 8px;
}

.rule-toggle {
  min-height: 34px;
  padding: 8px 13px;
  color: #0f4f63;
  font-size: 13px;
  font-weight: 950;
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid rgba(103, 232, 249, 0.34);
  border-radius: 999px;
  box-shadow: 0 10px 24px rgba(15, 23, 42, 0.08);
  cursor: pointer;
}

.rule-popover {
  display: grid;
  width: min(260px, calc(100vw - 32px));
  gap: 8px;
  padding: 10px;
  background: rgba(255, 255, 255, 0.78);
  border: 1px solid rgba(148, 163, 184, 0.22);
  border-radius: 16px;
  box-shadow: 0 18px 34px rgba(15, 23, 42, 0.13);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
}

.guild-tabs {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0;
  padding: 0;
}

.guild-tab {
  display: grid;
  gap: 4px;
  min-width: 0;
  padding: 14px;
  color: var(--muted);
  text-align: left;
  background: transparent;
  border: 0;
  border-right: 1px solid rgba(148, 163, 184, 0.16);
  border-radius: 0;
  cursor: pointer;
  transition: color 0.18s ease, background 0.18s ease;
}

.guild-tab:last-child {
  border-right: 0;
}

.guild-tab:hover,
.guild-tab.active {
  color: var(--text);
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.5), rgba(255, 247, 251, 0.26)),
    radial-gradient(circle at 0% 0%, rgba(244, 63, 94, 0.12), transparent 40%);
  box-shadow: inset 0 -2px 0 rgba(244, 63, 94, 0.62);
  transform: none;
}

.guild-tab span {
  overflow: hidden;
  color: var(--accent);
  font-size: 10px;
  font-weight: 950;
  letter-spacing: 0.08em;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.guild-tab strong {
  overflow: hidden;
  font-size: 18px;
  font-weight: 950;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.guild-tab em {
  overflow: hidden;
  font-size: 12px;
  font-style: normal;
  font-weight: 850;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.guild-tab-panel {
  min-width: 0;
}

.guild-rules {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 18px;
}

.rule-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: flex-end;
}

.rule-btn,
.ghost-btn,
.action-btn {
  min-height: 38px;
  padding: 9px 14px;
  font-weight: 950;
  border-radius: 999px;
  cursor: pointer;
}

.rule-btn,
.ghost-btn {
  color: #0f4f63;
  background: rgba(255, 255, 255, 0.54);
  border: 1px solid rgba(103, 232, 249, 0.26);
}

.link-btn {
  display: inline-flex;
  align-items: center;
  text-decoration: none;
}

.guild-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  gap: 18px;
}

.quest-column,
.guild-side,
.reward-section {
  min-width: 0;
}

.quest-column,
.rank-panel,
.rating-panel,
.reward-section,
.guild-feedback {
  padding: 18px;
}

.rank-page,
.rating-page {
  max-width: 880px;
}

.rule-dialog.panel {
  border: 1px solid rgba(255, 255, 255, 0.55);
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.86), rgba(255, 247, 251, 0.68)),
    radial-gradient(circle at 8% 0%, rgba(245, 51, 93, 0.12), transparent 34%);
  box-shadow: 0 18px 40px rgba(31, 41, 55, 0.16);
  border-radius: 22px;
}

.guild-side {
  display: grid;
  gap: 18px;
}

.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

.section-head h2 {
  color: var(--text);
  font-size: 22px;
  font-weight: 950;
}

.section-head.compact {
  margin-bottom: 12px;
}

.quest-group + .quest-group {
  margin-top: 18px;
}

.quest-group-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
  color: var(--text);
  font-weight: 950;
}

.quest-group-title b {
  display: grid;
  width: 30px;
  height: 30px;
  place-items: center;
  color: #f43f5e;
  background: rgba(244, 63, 94, 0.1);
  border-radius: 999px;
}

.quest-card {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 110px;
  gap: 14px;
  align-items: center;
  padding: 14px;
  margin-bottom: 10px;
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.74);
  border-radius: 18px;
  transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
}

.quest-card:hover,
.reward-card:hover {
  border-color: rgba(103, 232, 249, 0.36);
  box-shadow: 0 14px 28px rgba(15, 23, 42, 0.12);
  transform: translateY(-2px);
}

.quest-card.locked,
.reward-card.locked {
  opacity: 0.72;
  filter: saturate(0.75);
}

.quest-card.completed {
  border-color: rgba(34, 197, 94, 0.34);
}

.quest-title-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.quest-title-row h3,
.reward-card h3 {
  color: var(--text);
  font-size: 16px;
  font-weight: 950;
}

.quest-title-row span {
  flex: 0 0 auto;
  padding: 4px 8px;
  color: #f43f5e;
  font-size: 11px;
  font-weight: 950;
  background: rgba(244, 63, 94, 0.1);
  border-radius: 999px;
}

.quest-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
}

.quest-meta span {
  padding: 4px 8px;
  color: rgba(15, 23, 42, 0.7);
  font-size: 12px;
  font-weight: 850;
  background: rgba(255, 255, 255, 0.66);
  border: 1px solid rgba(148, 163, 184, 0.18);
  border-radius: 999px;
}

.progress-line {
  height: 8px;
  margin-top: 12px;
  overflow: hidden;
  background: rgba(148, 163, 184, 0.18);
  border-radius: 999px;
}

.progress-line span {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, #f43f5e, #38bdf8, #a78bfa);
  border-radius: inherit;
}

.action-btn {
  color: #fff;
  background: linear-gradient(135deg, #f43f5e, #7c3aed);
  border: 0;
  box-shadow: 0 12px 24px rgba(244, 63, 94, 0.18);
}

.action-btn.full {
  width: 100%;
  margin-top: 10px;
}

.action-btn:disabled {
  color: rgba(15, 23, 42, 0.52);
  background: rgba(148, 163, 184, 0.18);
  box-shadow: none;
  cursor: not-allowed;
}

.leader-row {
  display: grid;
  grid-template-columns: 42px 36px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
  padding: 10px 0;
  color: var(--text);
  text-decoration: none;
  border-bottom: 1px solid rgba(148, 163, 184, 0.15);
}

.leader-name {
  overflow: hidden;
  font-weight: 850;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.leader-no {
  color: var(--muted);
  font-weight: 950;
}

.leader-rating {
  display: grid;
  width: 30px;
  height: 30px;
  place-items: center;
  color: #fff;
  font-weight: 950;
  background: linear-gradient(135deg, #f43f5e, #6366f1);
  border-radius: 999px;
}

.rating-panel textarea {
  width: 100%;
  min-height: 74px;
  margin-top: 12px;
  padding: 12px;
  color: var(--text);
  background: rgba(255, 255, 255, 0.64);
  border: 1px solid rgba(148, 163, 184, 0.28);
  border-radius: 14px;
  resize: vertical;
}

.reward-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.reward-card {
  display: grid;
  grid-template-columns: 88px minmax(0, 1fr) 112px;
  gap: 14px;
  align-items: center;
  padding: 14px;
  background: rgba(255, 255, 255, 0.58);
  border: 1px solid rgba(255, 255, 255, 0.72);
  border-radius: 18px;
  transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
}

.reward-visual {
  display: grid;
  aspect-ratio: 1;
  place-items: center;
  overflow: hidden;
  color: #fff;
  font-weight: 950;
  background: linear-gradient(135deg, #0ea5e9, #f43f5e);
  border-radius: 16px;
}

.reward-visual img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.redemption-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-top: 14px;
}

.redemption-strip b {
  padding: 6px 10px;
  color: #0f4f63;
  background: rgba(255, 255, 255, 0.6);
  border-radius: 999px;
}

.guild-feedback {
  color: var(--text);
  font-weight: 850;
  line-height: 1.6;
}

.rule-modal {
  position: fixed;
  inset: 0;
  z-index: 1200;
  display: grid;
  place-items: center;
  padding: 22px;
  background: rgba(15, 23, 42, 0.34);
  backdrop-filter: blur(10px);
}

.rule-dialog {
  position: relative;
  width: min(560px, 100%);
  padding: 24px;
}

.rule-dialog ul {
  padding-left: 20px;
  color: var(--text);
  font-weight: 750;
  line-height: 1.8;
}

.close-btn {
  position: absolute;
  top: 12px;
  right: 14px;
  width: 34px;
  height: 34px;
  color: var(--text);
  background: rgba(255, 255, 255, 0.64);
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 999px;
  cursor: pointer;
}

.empty {
  color: var(--muted);
  font-weight: 800;
}

:global(html.art-lights-out) .panel {
  border-top-color: rgba(125, 211, 252, 0.18);
  border-bottom-color: rgba(125, 211, 252, 0.14);
  background:
    linear-gradient(135deg, rgba(8, 14, 33, 0.44), rgba(28, 22, 58, 0.26)),
    radial-gradient(circle at 10% 0%, rgba(125, 211, 252, 0.12), transparent 34%);
  box-shadow: none;
}

:global(html.art-lights-out) .rule-toggle,
:global(html.art-lights-out) .rule-popover,
:global(html.art-lights-out) .rule-dialog.panel {
  color: #bae6fd;
  background: rgba(12, 20, 44, 0.78);
  border-color: rgba(125, 211, 252, 0.18);
  box-shadow: 0 18px 34px rgba(0, 0, 0, 0.32);
}

:global(html.art-lights-out) .guild-hero h1,
:global(html.art-lights-out) .section-head h2,
:global(html.art-lights-out) .quest-title-row h3,
:global(html.art-lights-out) .reward-card h3,
:global(html.art-lights-out) .status-tile strong,
:global(html.art-lights-out) .leader-row,
:global(html.art-lights-out) .guild-feedback {
  color: #f7fbff;
}

:global(html.art-lights-out) .quest-card,
:global(html.art-lights-out) .reward-card,
:global(html.art-lights-out) .guild-tab,
:global(html.art-lights-out) .rating-panel textarea {
  color: #eaf6ff;
  background: rgba(12, 20, 44, 0.58);
  border-color: rgba(125, 211, 252, 0.12);
}

:global(html.art-lights-out) .guild-tab:hover,
:global(html.art-lights-out) .guild-tab.active {
  color: #f7fbff;
  background:
    linear-gradient(135deg, rgba(14, 23, 54, 0.78), rgba(35, 28, 68, 0.64)),
    radial-gradient(circle at 0% 0%, rgba(125, 211, 252, 0.14), transparent 40%);
  box-shadow: inset 0 -2px 0 rgba(125, 211, 252, 0.64);
}

:global(html.art-lights-out) .quest-meta span,
:global(html.art-lights-out) .rule-btn,
:global(html.art-lights-out) .ghost-btn,
:global(html.art-lights-out) .redemption-strip b {
  color: #bae6fd;
  background: rgba(255, 255, 255, 0.07);
  border-color: rgba(125, 211, 252, 0.14);
}

@media (max-width: 980px) {
  .guild-hero,
  .guild-layout,
  .reward-grid {
    grid-template-columns: 1fr;
  }

  .guild-status {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .guild-tabs {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 680px) {
  .guild-page {
    gap: 14px;
  }

  .guild-hero,
  .guild-rules,
  .quest-card,
  .reward-card {
    grid-template-columns: 1fr;
  }

  .guild-rules {
    align-items: flex-start;
    flex-direction: column;
  }

  .rule-actions {
    justify-content: flex-start;
  }

  .guild-status {
    grid-template-columns: 1fr;
  }

  .guild-tabs {
    display: flex;
    overflow-x: auto;
    padding-bottom: 12px;
  }

  .guild-tab {
    min-width: 132px;
  }
}
</style>
