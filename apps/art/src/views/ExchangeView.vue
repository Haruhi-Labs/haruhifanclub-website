<template>
  <div class="guild-shell">
    <!-- ============ 左栏：冒险者状态台账 ============ -->
    <aside class="g-deck">
      <div class="g-deck__inner">
        <p class="g-deck__brand">SOS Brigade Guild<span>冒险者公会指挥台</span></p>

        <!-- 评级徽章 -->
        <div class="g-crest" :class="`rating-${profile.rating}`">
          <div class="g-crest__badge">
            <span class="g-crest__ring" aria-hidden="true"></span>
            <strong>{{ profile.rating }}</strong>
          </div>
          <div class="g-crest__meta">
            <span class="g-crest__eyebrow">冒险者评级</span>
            <b>{{ profile.ratingLabel || '游客观测中' }}</b>
          </div>
        </div>

        <!-- 下一评级声望进度 -->
        <div v-if="profile.nextRating?.rating" class="g-nextrank">
          <div class="g-nextrank__head">
            <span>晋升 {{ profile.nextRating.rating }}</span>
            <b>{{ profile.reputation || 0 }} / {{ profile.nextRating.requiredReputation }}</b>
          </div>
          <div class="g-nextrank__bar">
            <i
              :style="{
                width: reqPercent(profile.reputation, profile.nextRating.requiredReputation) + '%',
              }"
            ></i>
          </div>
        </div>

        <!-- 状态台账 -->
        <dl class="g-ledger">
          <div class="g-ledger__row">
            <dt>冒险者编号</dt>
            <dd class="mono">{{ profile.uid || 'VISITOR' }}</dd>
          </div>
          <div class="g-ledger__row g-ledger__row--coin">
            <dt>金币结余</dt>
            <dd class="mono">{{ coinText }}</dd>
          </div>
          <div v-if="profile.coins?.frozen" class="g-ledger__sub">
            <dt>· 冻结中</dt>
            <dd class="mono">{{ profile.coins.frozen }}G</dd>
          </div>
          <div class="g-ledger__row">
            <dt>声望 / 等级</dt>
            <dd class="mono">{{ profile.reputation || 0 }} · Lv{{ profile.level || 1 }}</dd>
          </div>
          <div class="g-ledger__row">
            <dt>访问许可</dt>
            <dd>{{ profile.accessShortLabel || '档案0' }}</dd>
          </div>
        </dl>

        <!-- 垂直导航 -->
        <nav class="g-nav" role="tablist" aria-label="公会功能">
          <button
            v-for="tab in guildTabs"
            :key="tab.id"
            type="button"
            class="g-nav__item"
            :class="{ 'is-active': activeTab === tab.id }"
            role="tab"
            :aria-selected="activeTab === tab.id"
            @click="selectGuildTab(tab.id)"
          >
            <span class="g-nav__bar" aria-hidden="true"></span>
            <span class="g-nav__label">{{ tab.label }}</span>
            <span class="g-nav__count">{{ tab.summary }}</span>
          </button>
        </nav>
      </div>
    </aside>

    <!-- ============ 右栏：委托终端 ============ -->
    <main class="g-stage">
      <!-- 委托 -->
      <section v-if="activeTab === 'quests'" class="g-view">
        <header class="g-view__head">
          <div>
            <span class="g-view__eyebrow">Bounty Board</span>
            <h1>委托终端</h1>
            <p>系统会根据你的真实画廊行为自动查验进度并结算奖励。</p>
          </div>
          <button
            type="button"
            class="sos-button sos-button--ghost sos-button--sm"
            @click="loadGuild"
          >
            刷新
          </button>
        </header>

        <div v-for="group in questGroups" :key="group.id" class="g-questgroup">
          <div class="g-questgroup__title">
            <span>{{ group.label }}</span
            ><b>{{ group.items.length }}</b>
          </div>
          <article
            v-for="quest in group.items"
            :key="quest.id"
            class="g-quest"
            :class="{ 'is-locked': !quest.unlocked, 'is-done': questStatus(quest) === 'completed' }"
          >
            <div class="g-quest__body">
              <div class="g-quest__top">
                <h3>{{ quest.title }}</h3>
                <span class="g-quest__status" :class="questStatus(quest)">{{
                  questStatusLabel(quest)
                }}</span>
              </div>
              <p>{{ quest.description || conditionLabel(quest.conditionKind) }}</p>
              <div class="g-chips">
                <span class="g-chip">评级 {{ quest.requiredRating }}</span>
                <span class="g-chip">{{ quest.requiredAccessLabel }}</span>
                <span v-if="questDeadlineText(quest)" class="g-chip g-chip--time">{{
                  questDeadlineText(quest)
                }}</span>
                <span v-else-if="quest.deadlineDays" class="g-chip g-chip--time">
                  接取后 {{ quest.deadlineDays }} 天
                </span>
                <span v-if="quest.deadlineDays && quest.repeatOnComplete" class="g-chip"
                  >完成后刷新</span
                >
                <span v-else-if="quest.cycleDays" class="g-chip">{{
                  quest.cycleDays === 1 ? '每日刷新' : `${quest.cycleDays} 天循环`
                }}</span>
                <span v-if="quest.autoClaim" class="g-chip">自动接取</span>
                <span class="g-chip g-chip--rep">声望 +{{ quest.rewardReputation }}</span>
                <span v-if="quest.rewardCoins > 0" class="g-chip g-chip--coin"
                  >金币 +{{ quest.rewardCoins }}</span
                >
              </div>
              <div
                class="g-progress"
                :aria-label="`${questProgress(quest)} / ${questTarget(quest)}`"
              >
                <div class="g-progress__track">
                  <span
                    class="g-progress__fill"
                    :style="{ width: `${questProgressPercent(quest)}%` }"
                  ></span>
                </div>
                <i class="g-progress__num">{{ questProgress(quest) }}/{{ questTarget(quest) }}</i>
              </div>
            </div>
            <button
              type="button"
              class="sos-button sos-button--primary"
              :disabled="questButtonDisabled(quest) || busyQuestId === quest.id"
              @click="claimQuest(quest)"
            >
              {{ questButtonLabel(quest) }}
            </button>
          </article>
        </div>
        <div v-if="!quests.length" class="g-empty">暂无可接取的委托</div>
      </section>

      <!-- 兑换 -->
      <section v-else-if="activeTab === 'rewards'" class="g-view">
        <header class="g-view__head">
          <div>
            <span class="g-view__eyebrow">Reward Counter</span>
            <h1>补给兑换柜台</h1>
            <p>用金币兑换 SOS团补给，提交后冻结金币并等待管理员审核。</p>
          </div>
          <RouterLink
            v-if="session.state.user"
            class="sos-button sos-button--ghost sos-button--sm"
            to="/terminal"
            >个人终端 ›</RouterLink
          >
        </header>

        <div v-if="rewardBudget" class="g-budget">
          <div class="g-budget__main">
            <div class="g-budget__copy">
              <span>柜台预算库存</span>
              <p>
                每个季度会固定补给预算，此外根据活动可能会另行补给额外预算。按
                {{ rewardBudget.coinPerRmb || 15 }}G≈1元折算。
              </p>
            </div>
            <div class="g-budget__amount">
              <b class="mono">{{ formatCoins(rewardBudget.currentBudgetCoins) }}</b>
              <button type="button" @click="showBudgetRecords = !showBudgetRecords">
                补给记录
              </button>
            </div>
          </div>
          <div v-if="showBudgetRecords" class="g-budget-records">
            <div v-for="item in recentBudgetSupplies" :key="item.id" class="g-budget-record">
              <span>{{ item.budgetTypeLabel || '补给预算' }}</span>
              <b>{{ formatCoins(item.amountCoins) }}</b>
              <time>{{ formatShortDateTime(item.createdAt) }}</time>
            </div>
            <div v-if="!recentBudgetSupplies.length" class="g-budget-record g-budget-record--empty">
              暂无补给记录
            </div>
          </div>
        </div>

        <div class="g-rewards">
          <article
            v-for="reward in rewards"
            :key="reward.id"
            class="g-reward"
            :class="{ 'is-locked': !reward.unlocked }"
          >
            <div class="g-reward__visual">
              <img v-if="reward.imageUrl" :src="reward.imageUrl" :alt="reward.name" />
              <span v-else>{{ reward.rewardType === 'physical' ? '实体补给' : '虚拟补给' }}</span>
            </div>
            <div class="g-reward__copy">
              <h3>{{ reward.name }}</h3>
              <p>{{ reward.description || '管理员发放的公会补给。' }}</p>
              <div class="g-chips">
                <span class="g-chip g-chip--coin">{{ reward.priceCoins }}G</span>
                <span class="g-chip">评级 {{ reward.requiredRating }}</span>
                <span class="g-chip">{{ reward.requiredAccessLabel }}</span>
                <span v-if="reward.stock !== null && reward.stock !== undefined" class="g-chip"
                  >库存 {{ reward.stock }}</span
                >
              </div>
            </div>
            <button
              type="button"
              class="sos-button sos-button--primary sos-button--block"
              :disabled="!reward.unlocked || busyRewardId === reward.id"
              @click="openRedeem(reward)"
            >
              {{ reward.unlocked ? '提交兑换' : '条件不足' }}
            </button>
          </article>
        </div>
        <div v-if="!rewards.length" class="g-empty">暂无可查看的补给项目</div>

        <div v-if="redemptions.length" class="g-redemptions">
          <span class="g-redemptions__title">我的兑换申请</span>
          <div class="g-redemptions__list">
            <b v-for="item in redemptions.slice(0, 5)" :key="item.id" :class="`st-${item.status}`">
              {{ item.rewardName }} · {{ redemptionLabel(item.status) }}
            </b>
          </div>
        </div>
      </section>

      <!-- 排行 -->
      <section v-else-if="activeTab === 'ranking'" class="g-view">
        <header class="g-view__head">
          <div>
            <span class="g-view__eyebrow">Ranking</span>
            <h1>冒险者排行榜</h1>
            <p>按历史累计获得积分排序，点击任意冒险者查看其公开档案。</p>
          </div>
        </header>

        <div class="g-board">
          <div v-if="currentLeader" class="g-board-current">
            <span class="g-board-current__label">当前排名</span>
            <RouterLink
              class="g-leader g-leader--current"
              :to="{
                name: 'adventurer-profile',
                params: { uid: currentLeader.uid },
                query: { from: 'ranking' },
              }"
            >
              <span class="g-leader__no" :data-rank="currentLeader.rank"
                >#{{ currentLeader.rank }}</span
              >
              <span class="g-leader__rating" :class="`rating-${currentLeader.rating}`">{{
                currentLeader.rating
              }}</span>
              <span class="g-leader__name">
                {{ currentLeader.name || currentLeader.uid }}
                <i v-if="currentLeader.level">Lv{{ currentLeader.level }}</i>
              </span>
              <b class="mono">{{ currentLeader.earned }}G</b>
            </RouterLink>
          </div>
          <div v-if="currentLeader" class="g-board-separator"><span>排行榜</span></div>
          <RouterLink
            v-for="(item, index) in leaderboard"
            :key="item.uid"
            class="g-leader"
            :class="{ 'is-top': index < 3 }"
            :to="{
              name: 'adventurer-profile',
              params: { uid: item.uid },
              query: { from: 'ranking' },
            }"
          >
            <span class="g-leader__no" :data-rank="item.rank || index + 1">{{
              item.rank || index + 1
            }}</span>
            <span class="g-leader__rating" :class="`rating-${item.rating}`">{{ item.rating }}</span>
            <span class="g-leader__name">
              {{ item.name || item.uid }}
              <i v-if="item.level">Lv{{ item.level }}</i>
            </span>
            <b class="mono">{{ item.earned }}G</b>
          </RouterLink>
        </div>
        <div v-if="!leaderboard.length" class="g-empty">暂无公会成员记录</div>
      </section>

      <!-- 评级 -->
      <section v-else-if="activeTab === 'rating'" class="g-view">
        <header class="g-view__head">
          <div>
            <span class="g-view__eyebrow">Rank Up</span>
            <h1>评级申请</h1>
            <p>{{ nextRatingText }}</p>
          </div>
        </header>

        <div v-if="profile.nextRating?.rating" class="g-rating-reqs">
          <div class="g-req">
            <div class="g-req__head">
              <span>声望</span
              ><b class="mono"
                >{{ profile.reputation || 0 }} / {{ profile.nextRating.requiredReputation }}</b
              >
            </div>
            <div class="g-req__bar">
              <i
                :style="{
                  width:
                    reqPercent(profile.reputation, profile.nextRating.requiredReputation) + '%',
                }"
              ></i>
            </div>
          </div>
          <div class="g-req">
            <div class="g-req__head">
              <span>凉宫个人作品</span
              ><b class="mono"
                >{{ profile.haruhiPersonalCount || 0 }} /
                {{ profile.nextRating.requiredHaruhiCount }}</b
              >
            </div>
            <div class="g-req__bar">
              <i
                :style="{
                  width:
                    reqPercent(
                      profile.haruhiPersonalCount,
                      profile.nextRating.requiredHaruhiCount
                    ) + '%',
                }"
              ></i>
            </div>
          </div>
        </div>

        <div class="g-feedback">
          <span class="g-feedback__dot" aria-hidden="true"></span>
          <span>{{ ratingAuditText }}</span>
        </div>
      </section>

      <!-- 规则书 -->
      <section v-else-if="activeTab === 'rules'" class="g-view">
        <header class="g-view__head">
          <div>
            <span class="g-view__eyebrow">Guild Rule Book</span>
            <h1>公会规则书</h1>
            <p>冒险者公会的运作规约，共 {{ ruleBooks.length }} 章，章节内容由管理员后台维护。</p>
          </div>
        </header>
        <div class="g-rules-grid">
          <article v-for="(book, index) in ruleBooks" :key="book.id" class="g-rulecard">
            <div class="g-rulecard__head">
              <span class="g-rulecard__no">{{ String(index + 1).padStart(2, '0') }}</span>
              <h3>{{ book.title }}</h3>
            </div>
            <ul>
              <li v-for="line in book.lines" :key="line">{{ line }}</li>
            </ul>
            <div v-if="book.tables?.length" class="g-rule-actions">
              <button
                v-for="table in book.tables"
                :key="table.title"
                type="button"
                class="g-rule-table-button"
                @click="openRuleTable(table)"
              >
                {{ table.title }}
              </button>
            </div>
          </article>
        </div>
      </section>

      <!-- feedback -->
      <footer class="g-feedback" aria-live="polite">
        <span class="g-feedback__dot" aria-hidden="true"></span>
        {{ feedback }}
      </footer>
    </main>

    <!-- ============ 兑换备注弹层（替代 window.prompt）============ -->
    <transition name="g-fade">
      <div v-if="redeemTarget" class="g-modal" @click.self="closeRedeem">
        <article class="g-dialog">
          <button class="g-dialog__close" type="button" @click="closeRedeem" aria-label="关闭">
            ×
          </button>
          <span class="g-dialog__eyebrow">Redeem</span>
          <h2>兑换「{{ redeemTarget.name }}」</h2>
          <p class="g-dialog__lede">
            将冻结 <b>{{ redeemTarget.priceCoins }}G</b>，提交后等待管理员审核通过才正式扣除。
            <template v-if="redeemTarget.rewardType === 'physical'"
              >实体补给请在备注中填写联系方式 / 领取方式。</template
            >
          </p>
          <textarea
            class="sos-textarea"
            v-model="redeemNote"
            placeholder="给管理员的备注（可留空）"
          ></textarea>
          <div class="g-dialog__actions">
            <button type="button" class="sos-button sos-button--ghost" @click="closeRedeem">
              取消
            </button>
            <button
              type="button"
              class="sos-button sos-button--primary"
              :disabled="busyRewardId === redeemTarget.id"
              @click="confirmRedeem"
            >
              确认提交
            </button>
          </div>
        </article>
      </div>
    </transition>

    <transition name="g-fade">
      <div v-if="activeRuleTable" class="g-modal" @click.self="closeRuleTable">
        <article class="g-dialog g-dialog--table" role="dialog" aria-modal="true">
          <button class="g-dialog__close" type="button" @click="closeRuleTable" aria-label="关闭">
            ×
          </button>
          <span class="g-dialog__eyebrow">Rule Table</span>
          <h2>{{ activeRuleTable.title }}</h2>
          <div class="g-rule-modal-table">
            <table>
              <thead>
                <tr>
                  <th v-for="head in activeRuleTable.headers" :key="head">{{ head }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(row, rowIndex) in activeRuleTable.rows" :key="row.join('-')">
                  <td v-for="(cell, cellIndex) in row" :key="`${rowIndex}-${cellIndex}`">
                    {{ cell }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div class="g-dialog__actions">
            <button type="button" class="sos-button sos-button--ghost" @click="closeRuleTable">
              关闭
            </button>
          </div>
        </article>
      </div>
    </transition>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSession } from '@haruhi/auth-ui'
import { api } from '../services/api.js'

const router = useRouter()
const route = useRoute()
const session = useSession('/api')

const quests = ref([])
const rewards = ref([])
const rewardBudget = ref(null)
const showBudgetRecords = ref(false)
const leaderboard = ref([])
const currentLeader = ref(null)
const redemptions = ref([])
const profile = ref({
  uid: '',
  rating: 'F',
  ratingLabel: '',
  level: 1,
  reputation: 0,
  accessShortLabel: '',
  coins: { total: 0, available: 0, frozen: 0 },
})
const feedback = ref('公会柜台待机中。委托进度会由系统根据你的真实画廊行为自动查验。')
const GUILD_TAB_IDS = ['quests', 'rewards', 'ranking', 'rating', 'rules']
const activeTab = ref(normalizeGuildTab(route.query.tab))
const busyQuestId = ref(null)
const busyRewardId = ref(null)
const nowTick = ref(Date.now())
let countdownTimer = null

// 兑换备注弹层（替代原生 window.prompt）
const redeemTarget = ref(null)
const redeemNote = ref('')
const activeRuleTable = ref(null)

const ratingApplicationTable = {
  title: '评级申请条件',
  headers: ['评级', '最低声望', '通过审核的凉宫个人作品'],
  rows: [
    ['F', '0', '0（初始评级）'],
    ['E', '100', '1'],
    ['D', '300', '2'],
    ['C', '800', '4'],
    ['B', '1500', '7'],
    ['A', '3000', '12'],
    ['S', '6000', '20'],
    ['X', '12000', '35'],
  ],
}

const levelReputationTable = {
  title: '等级与声望门槛',
  headers: ['等级', '最低声望'],
  rows: [
    ['1', '0'],
    ['2', '100'],
    ['3', '200'],
    ['4', '300'],
    ['5', '400'],
    ['9', '800'],
    ['10', '900'],
    ['16', '1500'],
    ['31', '3000'],
    ['61', '6000'],
    ['121', '12000'],
  ],
}

const ruleBooks = [
  {
    id: 'coins',
    title: '金币与声望',
    lines: [
      '原积分按 1:1 等量迁移为金币，金币可用于兑换虚拟或实体奖励。',
      '上传作品可以获得金币与声望；浏览、评论、点赞等委托只增加声望。',
      '兑换会先冻结对应金币，管理员审核通过后正式扣除，拒绝则解除冻结。',
      '实体补给每个季度会固定补给预算，此外根据活动可能会另行补给额外预算。',
      '金币对人民币（元）的比例约为 15:1；实体补给预算没用完会持续留存，用于兑换柜台的实体补给发放。',
      '金币与人民币比例仅用于活动激励和库存预算估算，不代表现金兑付承诺。',
    ],
    tables: [levelReputationTable],
  },
  {
    id: 'rating',
    title: '冒险者评级 FEDCBASX',
    lines: [
      '评级从 F 到 X 逐级提升，代表你在画廊中的长期贡献与观测资历。',
      '声望决定冒险者等级，评级提升还需要满足凉宫个人作品数量等条件。',
      '评级申请在满足下一评级条件后自动提交，由管理员验收通过后生效。',
      '凉宫个人作品数量仅统计已审核通过的个人来源凉宫作品。',
      '冒险者等级按公式计算：等级 = floor(声望 / 100) + 1。',
      '评级提升后会解锁更困难的委托，困难委托通常给予大量声望。',
    ],
    tables: [ratingApplicationTable],
  },
  {
    id: 'access',
    title: '访问许可',
    lines: [
      '档案0：公开档案许可，可浏览公开画廊与公会规则。',
      '观测1：观测员许可，可参与日常委托与基础兑换。',
      '异常2 / 闭锁3：更高阶观测许可，由管理员根据社团需要授予。',
    ],
  },
  {
    id: 'redemption',
    title: '兑换与发放',
    lines: [
      '虚拟奖励由管理员审核后在站内或指定渠道发放。',
      '实体奖励需要管理员线下核对信息后发放，页面仅提交兑换申请。',
      '商品、库存、委托内容和奖励数值都可由管理员后台维护。',
    ],
  },
]

const coinText = computed(() => {
  const coins = profile.value.coins || {}
  return `${Number(coins.available || 0)}G`
})

const recentBudgetSupplies = computed(() => rewardBudget.value?.recentSupplies || [])

const guildTabs = computed(() => [
  { id: 'quests', eyebrow: 'Bounty', label: '委托', summary: `${quests.value.length} 项` },
  { id: 'rewards', eyebrow: 'Counter', label: '兑换', summary: `${rewards.value.length} 项` },
  { id: 'ranking', eyebrow: 'Ranking', label: '排行', summary: `${leaderboard.value.length} 人` },
  {
    id: 'rating',
    eyebrow: 'Rank Up',
    label: '评级',
    summary: profile.value.pendingRatingApplication?.targetRating
      ? `待验收 ${profile.value.pendingRatingApplication.targetRating}`
      : profile.value.nextRating?.rating
        ? `目标 ${profile.value.nextRating.rating}`
        : '当前',
  },
  { id: 'rules', eyebrow: 'Rule Book', label: '规则书', summary: `${ruleBooks.length} 章` },
])

const questGroups = computed(() => {
  const groups = [
    { id: 'daily', label: '日常委托', items: [] },
    { id: 'limited', label: '限时委托', items: [] },
    { id: 'hard', label: '困难委托', items: [] },
    { id: 'unknown', label: '未知委托', items: [] },
  ]
  for (const quest of quests.value) {
    const type = quest.questType || 'daily'
    const target = groups.find((item) => item.id === type) || groups[0]
    target.items.push(quest)
  }
  return groups.filter((group) => group.items.length)
})

const nextRatingText = computed(() => {
  if (!session.state.user) return '登录后可查看下一评级验收条件。'
  const next = profile.value.nextRating
  if (!next?.rating) return '你已经站在当前规则书的最高评级。'
  const status = next.available ? '已满足验收条件' : '尚未满足验收条件'
  return `下一评级 ${next.rating}：需要 ${next.requiredReputation} 声望 / ${next.requiredHaruhiCount} 张凉宫个人作品，当前${status}。`
})

const ratingAuditText = computed(() => {
  if (!session.state.user) return '登录后系统会根据真实档案检测评级资格。'
  const pending = profile.value.pendingRatingApplication
  if (pending?.targetRating) return `${pending.targetRating} 评级申请已自动提交，等待管理员验收。`
  const next = profile.value.nextRating
  if (!next?.rating) return '当前评级已经达到最高阶。'
  if (next.available) return `${next.rating} 评级条件已满足，系统会自动提交验收申请。`
  return '继续完成委托与投稿，满足条件后会自动进入管理员验收。'
})

function reqPercent(current, required) {
  const req = Number(required || 0)
  if (req <= 0) return 100
  return Math.min(100, Math.round((Number(current || 0) / req) * 100))
}

function requireLogin() {
  if (session.state.user) return true
  router.push({ name: 'login', query: { redirect: '/exchange' } })
  return false
}

function normalizeGuildTab(tab) {
  return GUILD_TAB_IDS.includes(String(tab || '')) ? String(tab) : 'quests'
}

function selectGuildTab(tab) {
  const next = normalizeGuildTab(tab)
  activeTab.value = next
  router.replace({
    query: {
      ...route.query,
      tab: next === 'quests' ? undefined : next,
    },
  })
}

async function loadGuild() {
  try {
    const [questRes, rewardRes, rankRes] = await Promise.all([
      api.guildQuests(),
      api.guildRewards(),
      api.guildLeaderboard(),
    ])
    quests.value = questRes.data || []
    rewards.value = rewardRes.data || []
    rewardBudget.value = rewardRes.budget || null
    leaderboard.value = rankRes.data || []
    currentLeader.value = rankRes.me || null
    profile.value = questRes.profile || rewardRes.profile || profile.value

    if (session.state.user) {
      try {
        const redRes = await api.guildMyRedemptions()
        redemptions.value = redRes.data || []
      } catch {
        redemptions.value = []
      }
    } else {
      redemptions.value = []
    }
  } catch (error) {
    // 本地开发无后端时回落到内置 mock，便于预览界面（生产连真实后端不会走到这里）
    if (import.meta.env.DEV) {
      applyMock()
      feedback.value = '本地预览：当前展示的是内置 mock 公会数据（无后端连接）。'
    } else {
      feedback.value = `公会数据加载失败：${error.message || '未知错误'}`
    }
  }
}

watch(
  () => route.query.tab,
  (tab) => {
    activeTab.value = normalizeGuildTab(tab)
  }
)

function questStatus(quest) {
  if (questExpired(quest) && questStatusRaw(quest) !== 'completed') return 'expired'
  return questStatusRaw(quest)
}
function questStatusRaw(quest) {
  return quest.claim?.status || (quest.unlocked ? 'ready' : 'locked')
}
function questStatusLabel(quest) {
  const status = questStatus(quest)
  if (status === 'completed') return '已完成'
  if (status === 'rejected') return '未通过'
  if (status === 'active' && quest.conditionKind === 'manual_admin_verify') return '待验收'
  if (status === 'active') return '进行中'
  if (status === 'locked') return '未解锁'
  if (status === 'expired') return '已截止'
  if (quest.autoClaim) return '自动接取'
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
  return (
    !quest.unlocked ||
    ['completed', 'expired', 'rejected'].includes(questStatus(quest)) ||
    (quest.autoClaim && !!session.state.user)
  )
}
function questButtonLabel(quest) {
  const status = questStatus(quest)
  if (!session.state.user) return quest.autoClaim ? '登录参与' : '登录接取'
  if (!quest.unlocked) return '权限不足'
  if (status === 'completed') return '已结算'
  if (status === 'rejected') return '未通过'
  if (status === 'expired') return '已截止'
  if (status === 'active' && quest.conditionKind === 'manual_admin_verify') return '等待验收'
  if (quest.autoClaim) return status === 'active' ? '自动进行' : '自动接取'
  if (status === 'active') return `${questProgress(quest)}/${questTarget(quest)}`
  return '接取委托'
}

function questDeadlineAt(quest) {
  const raw = quest.deadlineAt || quest.cycleEndAt
  if (!raw) return null
  const date = new Date(raw)
  return Number.isNaN(date.getTime()) ? null : date
}

function questExpired(quest) {
  const deadline = questDeadlineAt(quest)
  return !!deadline && deadline.getTime() <= nowTick.value
}

function questDeadlineText(quest) {
  const deadline = questDeadlineAt(quest)
  if (!deadline) return ''
  const delta = deadline.getTime() - nowTick.value
  if (delta <= 0) return '已截止'
  return `距截止 ${formatRemaining(delta)}`
}

function formatRemaining(ms) {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000))
  const days = Math.floor(totalSeconds / 86400)
  const hours = Math.floor((totalSeconds % 86400) / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  if (days > 0) return `${days}天${hours}小时`
  if (hours > 0) return `${hours}小时${minutes}分`
  return `${minutes}分`
}

function formatCoins(value) {
  return `${Number(value || 0).toLocaleString('zh-CN')}G`
}

function formatShortDateTime(value) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  return new Intl.DateTimeFormat('zh-CN', {
    timeZone: 'Asia/Shanghai',
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).format(date)
}

function conditionLabel(kind) {
  const map = {
    browse_artworks: '浏览画廊作品',
    comment_artworks: '留下公开评论',
    like_artworks: '给作品点赞',
    upload_personal_haruhi: '上传凉宫个人作品',
    upload_personal_any: '上传个人作品',
    upload_network: '提交转载资料',
    manual_admin_verify: '等待管理员验收',
  }
  return map[kind] || '完成公会指定动作'
}

async function claimQuest(quest) {
  if (!requireLogin()) return
  if (quest.autoClaim) return
  if (!quest.unlocked || ['completed', 'expired'].includes(questStatus(quest))) return

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

function openRedeem(reward) {
  if (!requireLogin() || !reward.unlocked) return
  redeemTarget.value = reward
  redeemNote.value = ''
}
function closeRedeem() {
  redeemTarget.value = null
  redeemNote.value = ''
}
function openRuleTable(table) {
  activeRuleTable.value = table
}
function closeRuleTable() {
  activeRuleTable.value = null
}
async function confirmRedeem() {
  const reward = redeemTarget.value
  if (!reward) return
  busyRewardId.value = reward.id
  try {
    const res = await api.guildRedeemReward(reward.id, { note: redeemNote.value })
    feedback.value = res.message || `已提交「${reward.name}」兑换申请。`
    closeRedeem()
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
    fulfilled: '已发放',
  }
  return map[status] || status
}

// ---- 本地开发预览用 mock（仅 DEV，结构对齐后端字段）----
function applyMock() {
  profile.value = {
    uid: 'u12',
    rating: 'D',
    ratingLabel: 'D级冒险者',
    level: 8,
    reputation: 760,
    accessShortLabel: '观测1',
    accessTier: 'observer_clearance',
    haruhiPersonalCount: 3,
    coins: { total: 1880, available: 1720, frozen: 160 },
    nextRating: { rating: 'C', requiredReputation: 800, requiredHaruhiCount: 4, available: false },
    pendingRatingApplication: null,
  }
  quests.value = [
    {
      id: 1,
      title: '每日观测：浏览 3 个画廊作品',
      description: '接取后浏览任意 3 个公开画廊作品。',
      questType: 'daily',
      difficulty: 'normal',
      requiredRating: 'F',
      requiredAccessLabel: '1级观测员许可',
      conditionKind: 'browse_artworks',
      targetCount: 3,
      rewardReputation: 15,
      rewardCoins: 0,
      deadlineDays: null,
      cycleDays: 1,
      cycleResetHour: 4,
      deadlineAt: mockDeadline(10),
      autoClaim: true,
      unlocked: true,
      claim: { status: 'active', progress: 2, targetCount: 3 },
    },
    {
      id: 2,
      title: '每日回声：评论 1 个作品',
      description: '接取后在任意作品详情留下 1 条评论。',
      questType: 'daily',
      difficulty: 'normal',
      requiredRating: 'F',
      requiredAccessLabel: '1级观测员许可',
      conditionKind: 'comment_artworks',
      targetCount: 1,
      rewardReputation: 20,
      rewardCoins: 0,
      deadlineDays: null,
      cycleDays: 1,
      cycleResetHour: 4,
      deadlineAt: mockDeadline(10),
      autoClaim: true,
      unlocked: true,
      claim: null,
    },
    {
      id: 3,
      title: 'SOS团特别委托：上传凉宫作品',
      description: '接取后投稿并通过审核 1 张凉宫个人作品。',
      questType: 'limited',
      difficulty: 'hard',
      requiredRating: 'E',
      requiredAccessLabel: '1级观测员许可',
      conditionKind: 'upload_personal_haruhi',
      targetCount: 1,
      rewardReputation: 180,
      rewardCoins: 30,
      deadlineDays: 3,
      fixedDeadlineAt: '',
      repeatOnComplete: true,
      deadlineAt: mockDeadline(72),
      autoClaim: false,
      unlocked: true,
      claim: null,
    },
  ]
  rewards.value = [
    {
      id: 1,
      name: 'SOS团电子徽章',
      description: '个人主页展示用的虚拟徽章。',
      rewardType: 'virtual',
      priceCoins: 80,
      stock: null,
      requiredRating: 'F',
      requiredAccessLabel: '1级观测员许可',
      imageUrl: '',
      unlocked: true,
    },
    {
      id: 2,
      name: '画廊应援明信片',
      description: '实体奖励，需在备注中填写联系方式或领取方式。',
      rewardType: 'physical',
      priceCoins: 300,
      stock: 20,
      requiredRating: 'E',
      requiredAccessLabel: '1级观测员许可',
      imageUrl: '',
      unlocked: true,
    },
  ]
  rewardBudget.value = {
    coinPerRmb: 15,
    currentBudgetCoins: 12600,
    totalSupplyCoins: 15000,
    spentPhysicalCoins: 2400,
    recentSupplies: [
      {
        id: 3,
        budgetTypeLabel: '活动预算',
        amountCoins: 3000,
        createdAt: mockDeadline(-24),
      },
      {
        id: 2,
        budgetTypeLabel: '季度预算',
        amountCoins: 9000,
        createdAt: mockDeadline(-24 * 20),
      },
      {
        id: 1,
        budgetTypeLabel: '其他预算',
        amountCoins: 3000,
        createdAt: mockDeadline(-24 * 35),
      },
    ],
  }
  // 排序依据为「历史累计获得积分(earned)」而非评级：低评级但获得多者可排在前。
  leaderboard.value = [
    {
      uid: '橙海',
      name: '橙海',
      rank: 1,
      rating: 'A',
      earned: 1880,
      level: 19,
      reputation: 1880,
      avatar_url: '',
    },
    {
      uid: 'u12',
      name: 'Kyon',
      rank: 2,
      rating: 'D',
      earned: 1720,
      level: 8,
      reputation: 760,
      avatar_url: '',
    },
    {
      uid: 'Mizuhasi',
      name: 'Mizuhasi',
      rank: 3,
      rating: 'B',
      earned: 1570,
      level: 16,
      reputation: 1570,
      avatar_url: '',
    },
  ]
  currentLeader.value = {
    uid: 'u12',
    name: 'Kyon',
    rank: 2,
    rating: 'D',
    earned: 1720,
    level: 8,
    reputation: 760,
    avatar_url: '',
  }
  redemptions.value = [{ id: 1, rewardName: 'SOS团电子徽章', status: 'pending' }]
}

function mockDeadline(hours) {
  return new Date(Date.now() + hours * 3600 * 1000).toISOString()
}

onMounted(async () => {
  countdownTimer = window.setInterval(() => {
    nowTick.value = Date.now()
  }, 30000)
  if (!session.state.ready) {
    try {
      await session.refresh()
    } catch {}
  }
  await loadGuild()
})

onUnmounted(() => {
  if (countdownTimer) window.clearInterval(countdownTimer)
})
</script>

<style scoped>
.guild-shell {
  /* 右侧终端 — 浅色青绿玻璃 */
  --g-accent: var(--sos-accent, hsl(172, 70%, 42%));
  --g-accent-strong: color-mix(in srgb, var(--g-accent) 78%, #06322d);
  --g-gold: hsl(42, 92%, 52%);
  --g-text: var(--sos-text-primary, #16242b);
  --g-muted: var(--sos-text-secondary, #5b6b72);
  --g-line: color-mix(in srgb, var(--g-accent) 16%, #d3e3df);
  /* 左侧台账 — 深青墨指挥台 */
  --deck-1: hsl(177, 46%, 13%);
  --deck-2: hsl(172, 44%, 19%);
  --deck-line: color-mix(in srgb, #7af0d6 18%, transparent);
  --deck-text: #ecf8f3;
  --deck-muted: color-mix(in srgb, #cfeee4 64%, transparent);
  --deck-accent: hsl(166, 82%, 60%);
  --mono: ui-monospace, 'SF Mono', 'Roboto Mono', 'JetBrains Mono', monospace;

  max-width: 1200px;
  margin: 0 auto;
  padding: var(--sos-space-3, 12px) var(--sos-space-4, 16px) var(--sos-space-8, 48px);
  display: grid;
  grid-template-columns: 296px minmax(0, 1fr);
  align-items: start;
  gap: var(--sos-space-4, 16px);
  color: var(--g-text);
}
.mono {
  font-family: var(--mono);
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.01em;
}

/* ============ 左栏 指挥台 ============ */
.g-deck {
  position: sticky;
  top: var(--sos-space-3, 12px);
}
.g-deck__inner {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 22px 20px;
  border-radius: 22px;
  color: var(--deck-text);
  background:
    radial-gradient(
      120% 80% at 100% 0%,
      color-mix(in srgb, var(--deck-accent) 22%, transparent),
      transparent 60%
    ),
    linear-gradient(165deg, var(--deck-2), var(--deck-1));
  border: 1px solid var(--deck-line);
  box-shadow:
    0 30px 60px -34px rgba(4, 32, 28, 0.7),
    inset 0 1px 0 rgba(255, 255, 255, 0.08);
  overflow: hidden;
}
.g-deck__inner::before {
  /* 细网格纹理，加强终端质感 */
  content: '';
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(color-mix(in srgb, var(--deck-accent) 8%, transparent) 1px, transparent 1px),
    linear-gradient(
      90deg,
      color-mix(in srgb, var(--deck-accent) 8%, transparent) 1px,
      transparent 1px
    );
  background-size: 26px 26px;
  -webkit-mask-image: radial-gradient(120% 90% at 50% 0%, #000, transparent 75%);
  mask-image: radial-gradient(120% 90% at 50% 0%, #000, transparent 75%);
  opacity: 0.4;
  pointer-events: none;
}
.g-deck__inner > * {
  position: relative;
}

.g-deck__brand {
  margin: 0;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--deck-accent);
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.g-deck__brand span {
  font-size: 13px;
  letter-spacing: 0.04em;
  text-transform: none;
  font-weight: 700;
  color: var(--deck-text);
}

/* 评级徽章 */
.g-crest {
  display: flex;
  align-items: center;
  gap: 14px;
}
.g-crest__badge {
  position: relative;
  display: grid;
  place-items: center;
  width: 70px;
  height: 70px;
  flex-shrink: 0;
  border-radius: 50%;
  background: radial-gradient(
    circle at 50% 35%,
    color-mix(in srgb, var(--deck-accent) 34%, transparent),
    color-mix(in srgb, var(--deck-accent) 8%, transparent)
  );
  border: 1px solid color-mix(in srgb, var(--deck-accent) 50%, transparent);
  box-shadow:
    inset 0 1px 6px color-mix(in srgb, var(--deck-accent) 30%, transparent),
    0 0 24px -6px color-mix(in srgb, var(--deck-accent) 50%, transparent);
}
.g-crest__badge strong {
  font-size: 32px;
  font-weight: 900;
  line-height: 1;
  color: var(--deck-text);
  text-shadow: 0 0 14px color-mix(in srgb, var(--deck-accent) 60%, transparent);
}
.g-crest__ring {
  position: absolute;
  inset: -5px;
  border-radius: 50%;
  border: 1.5px dashed color-mix(in srgb, var(--deck-accent) 55%, transparent);
  animation: g-spin 26s linear infinite;
}
@keyframes g-spin {
  to {
    transform: rotate(360deg);
  }
}
.g-crest__meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.g-crest__eyebrow {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.1em;
  color: var(--deck-muted);
}
.g-crest__meta b {
  font-size: 17px;
  font-weight: 800;
}
.g-crest.rating-S .g-crest__badge strong,
.g-crest.rating-X .g-crest__badge strong,
.g-crest.rating-A .g-crest__badge strong {
  color: hsl(44, 96%, 66%);
  text-shadow: 0 0 14px hsl(42, 92%, 52%);
}

/* 下一评级进度 */
.g-nextrank {
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.g-nextrank__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 700;
  color: var(--deck-muted);
}
.g-nextrank__head b {
  color: var(--deck-accent);
  font-family: var(--mono);
  font-size: 12px;
}
.g-nextrank__bar {
  height: 6px;
  border-radius: 999px;
  overflow: hidden;
  background: color-mix(in srgb, #000 28%, transparent);
}
.g-nextrank__bar i {
  display: block;
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(
    90deg,
    var(--deck-accent),
    color-mix(in srgb, var(--deck-accent) 50%, #bef7e7)
  );
  box-shadow: 0 0 12px -2px var(--deck-accent);
  transition: width 0.5s ease;
}

/* 状态台账 */
.g-ledger {
  margin: 0;
  display: flex;
  flex-direction: column;
  padding: 4px 0;
  border-top: 1px solid var(--deck-line);
  border-bottom: 1px solid var(--deck-line);
}
.g-ledger__row,
.g-ledger__sub {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  padding: 7px 0;
}
.g-ledger__row + .g-ledger__row {
  border-top: 1px dashed color-mix(in srgb, var(--deck-line) 60%, transparent);
}
.g-ledger dt {
  font-size: 12.5px;
  color: var(--deck-muted);
  font-weight: 600;
}
.g-ledger dd {
  margin: 0;
  font-size: 15px;
  font-weight: 800;
  color: var(--deck-text);
}
.g-ledger__row--coin dd {
  color: var(--deck-accent);
  font-size: 17px;
}
.g-ledger__sub {
  padding: 2px 0 7px;
}
.g-ledger__sub dt,
.g-ledger__sub dd {
  font-size: 11.5px;
  color: var(--deck-muted);
  font-weight: 600;
}

/* 垂直导航 */
.g-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.g-nav__item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 12px 11px 14px;
  border: 1px solid transparent;
  border-radius: 12px;
  background: transparent;
  color: var(--deck-muted);
  cursor: pointer;
  text-align: left;
  transition:
    background 0.18s,
    color 0.18s,
    border-color 0.18s;
}
.g-nav__bar {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 0;
  border-radius: 2px;
  background: var(--deck-accent);
  transition: height 0.2s;
}
.g-nav__label {
  flex: 1;
  font-size: 15px;
  font-weight: 700;
  color: inherit;
}
.g-nav__count {
  font-size: 11.5px;
  font-weight: 700;
  font-family: var(--mono);
  color: var(--deck-muted);
}
.g-nav__item:hover {
  background: color-mix(in srgb, #fff 7%, transparent);
  color: var(--deck-text);
}
.g-nav__item.is-active {
  background: color-mix(in srgb, var(--deck-accent) 16%, transparent);
  border-color: color-mix(in srgb, var(--deck-accent) 38%, transparent);
  color: var(--deck-text);
}
.g-nav__item.is-active .g-nav__bar {
  height: 60%;
}
.g-nav__item.is-active .g-nav__count {
  color: var(--deck-accent);
}

/* 规则书章节卡（右栏） */
.g-rules-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--sos-space-3);
}
@media (min-width: 760px) {
  .g-rules-grid {
    grid-template-columns: 1fr 1fr;
  }
}
.g-rulecard {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 18px;
  border-radius: 15px;
  border: 1px solid var(--g-line);
  background: color-mix(in srgb, #ffffff 54%, transparent);
}
.g-rulecard__head {
  display: flex;
  align-items: baseline;
  gap: 11px;
  padding-bottom: 11px;
  border-bottom: 1px solid var(--g-line);
}
.g-rulecard__no {
  font-family: var(--mono);
  font-size: 22px;
  font-weight: 800;
  line-height: 1;
  color: color-mix(in srgb, var(--g-accent) 44%, transparent);
}
.g-rulecard__head h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 800;
}
.g-rulecard ul {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.g-rulecard li {
  font-size: 13px;
  line-height: 1.6;
  color: var(--g-muted);
}
.g-rule-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: auto;
  padding-top: 2px;
}
.g-rule-table-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 34px;
  padding: 7px 13px;
  border-radius: 9px;
  border: 1px solid color-mix(in srgb, var(--g-accent) 24%, transparent);
  background: color-mix(in srgb, var(--g-accent) 8%, #ffffff);
  color: var(--g-accent-strong);
  font-size: 12.5px;
  font-weight: 800;
  cursor: pointer;
  transition:
    background 0.18s,
    border-color 0.18s,
    transform 0.18s;
}
.g-rule-table-button:hover {
  transform: translateY(-1px);
  border-color: color-mix(in srgb, var(--g-accent) 42%, transparent);
  background: color-mix(in srgb, var(--g-accent) 14%, #ffffff);
}
.g-rule-table-button:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--g-accent) 72%, transparent);
  outline-offset: 2px;
}

/* ============ 右栏 终端 ============ */
.g-stage {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3, 12px);
  min-height: 70vh;
}
.g-view {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3, 12px);
  padding: clamp(18px, 2.2vw, 26px);
  border-radius: 22px;
  border: 1px solid var(--g-line);
  background: color-mix(in srgb, #ffffff 70%, transparent);
  -webkit-backdrop-filter: blur(16px);
  backdrop-filter: blur(16px);
  box-shadow:
    0 22px 48px -32px rgba(16, 60, 56, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.7);
}
.g-view__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--sos-space-3);
  padding-bottom: var(--sos-space-2);
  border-bottom: 1px solid var(--g-line);
}
.g-view__eyebrow {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--g-accent-strong);
}
.g-view__head h1 {
  margin: 5px 0 4px;
  font-size: clamp(20px, 2.6vw, 27px);
  font-weight: 850;
  letter-spacing: -0.01em;
}
.g-view__head p {
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--g-muted);
  max-width: 52ch;
}
.g-empty {
  padding: 36px;
  text-align: center;
  color: var(--g-muted);
  font-weight: 600;
}

/* 委托 */
.g-questgroup {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.g-questgroup__title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
  font-weight: 800;
  color: var(--g-muted);
  letter-spacing: 0.05em;
}
.g-questgroup__title::before {
  content: '';
  width: 4px;
  height: 14px;
  border-radius: 2px;
  background: var(--g-accent);
}
.g-questgroup__title b {
  color: var(--g-accent-strong);
}

.g-quest {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--sos-space-4);
  padding: 15px 18px;
  border-radius: 15px;
  border: 1px solid var(--g-line);
  background: color-mix(in srgb, #ffffff 54%, transparent);
  transition:
    border-color 0.2s,
    box-shadow 0.2s,
    transform 0.2s;
}
.g-quest:hover {
  transform: translateY(-1px);
  box-shadow: 0 16px 30px -22px rgba(16, 60, 56, 0.5);
  border-color: color-mix(in srgb, var(--g-accent) 32%, transparent);
}
.g-quest.is-locked {
  opacity: 0.62;
}
.g-quest.is-done {
  border-color: color-mix(in srgb, var(--g-accent) 42%, transparent);
  background: color-mix(in srgb, var(--g-accent) 8%, #ffffff);
}
.g-quest__body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.g-quest__top {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.g-quest__top h3 {
  margin: 0;
  font-size: 15.5px;
  font-weight: 800;
}
.g-quest__status {
  font-size: 10.5px;
  font-weight: 800;
  padding: 2px 9px;
  border-radius: 999px;
  color: var(--g-muted);
  background: color-mix(in srgb, #000 6%, transparent);
}
.g-quest__status.active {
  color: var(--g-accent-strong);
  background: color-mix(in srgb, var(--g-accent) 14%, transparent);
}
.g-quest__status.completed {
  color: #fff;
  background: var(--g-accent);
}
.g-quest__status.locked {
  color: color-mix(in srgb, #b4690e 80%, #000);
  background: color-mix(in srgb, hsl(35, 90%, 55%) 16%, transparent);
}
.g-quest__status.expired {
  color: color-mix(in srgb, #d2453a 82%, #000);
  background: color-mix(in srgb, #ef5350 14%, transparent);
}
.g-quest__status.rejected {
  color: color-mix(in srgb, #a1322c 86%, #000);
  background: color-mix(in srgb, #d2453a 12%, transparent);
}
.g-quest__body p {
  margin: 0;
  font-size: 13px;
  line-height: 1.55;
  color: var(--g-muted);
}

.g-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.g-chip {
  font-size: 11px;
  font-weight: 700;
  padding: 3px 9px;
  border-radius: 8px;
  color: var(--g-muted);
  background: color-mix(in srgb, #000 5%, transparent);
}
.g-chip--rep {
  color: var(--g-accent-strong);
  background: color-mix(in srgb, var(--g-accent) 13%, transparent);
}
.g-chip--coin {
  color: color-mix(in srgb, var(--g-gold) 72%, #5a3a08);
  background: color-mix(in srgb, var(--g-gold) 16%, transparent);
}
.g-chip--time {
  color: color-mix(in srgb, #d2453a 80%, #000);
  background: color-mix(in srgb, #ef5350 14%, transparent);
}

.g-progress {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 2px;
}
.g-progress__track {
  position: relative;
  flex: 1;
  height: 8px;
  border-radius: 999px;
  overflow: hidden;
  background: color-mix(in srgb, #000 8%, transparent);
}
.g-progress__fill {
  position: absolute;
  inset: 0 auto 0 0;
  border-radius: 999px;
  background: linear-gradient(
    90deg,
    var(--g-accent),
    color-mix(in srgb, var(--g-accent) 60%, #7bf0d8)
  );
  transition: width 0.4s ease;
}
.g-progress__num {
  flex-shrink: 0;
  min-width: 34px;
  text-align: right;
  font-size: 11px;
  font-weight: 800;
  color: var(--g-muted);
  font-family: var(--mono);
}

/* 兑换 */
.g-budget {
  --g-budget-gap: 8px;

  display: flex;
  flex-direction: column;
  gap: var(--g-budget-gap);
  padding: 16px 18px;
  border-radius: 15px;
  border: 1px solid color-mix(in srgb, var(--g-accent) 26%, transparent);
  background: color-mix(in srgb, var(--g-accent) 8%, #ffffff);
  box-shadow: 0 18px 34px -28px rgba(16, 60, 56, 0.46);
}
.g-budget__main {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  column-gap: 12px;
  row-gap: var(--g-budget-gap);
  align-items: start;
}
.g-budget__copy {
  display: contents;
  min-width: 0;
}
.g-budget__copy span {
  grid-column: 1;
  grid-row: 1;
  align-self: center;
  font-size: 18px;
  font-weight: 800;
  color: var(--g-text);
}
.g-budget__copy p {
  grid-column: 1;
  grid-row: 2;
}
.g-budget__amount {
  display: contents;
}
.g-budget__amount b {
  grid-column: 2;
  grid-row: 1;
  justify-self: end;
  font-size: 22px;
  color: var(--g-accent-strong);
}
.g-budget__amount button {
  grid-column: 2;
  grid-row: 2;
  justify-self: end;
  border: 1px solid color-mix(in srgb, var(--g-accent) 28%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--g-accent) 8%, var(--g-card));
  color: var(--g-accent-strong);
  font-size: 11px;
  font-weight: 800;
  cursor: pointer;
}
.g-budget p {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--g-muted);
}
.g-budget-records {
  display: grid;
  gap: var(--g-budget-gap);
  padding-top: 2px;
}
.g-budget-record {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 8px;
  align-items: center;
  font-size: 12px;
  color: var(--g-muted);
}
.g-budget-record b {
  color: var(--g-text);
}
.g-budget-record time {
  color: var(--g-muted);
}
.g-budget-record--empty {
  display: block;
}
.g-rewards {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--sos-space-3);
}
@media (min-width: 780px) {
  .g-rewards {
    grid-template-columns: 1fr 1fr;
  }
}
.g-reward {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 15px;
  border-radius: 15px;
  border: 1px solid var(--g-line);
  background: color-mix(in srgb, #ffffff 54%, transparent);
  transition:
    border-color 0.2s,
    box-shadow 0.2s,
    transform 0.2s;
}
.g-reward:hover {
  transform: translateY(-2px);
  box-shadow: 0 18px 34px -24px rgba(16, 60, 56, 0.5);
  border-color: color-mix(in srgb, var(--g-accent) 30%, transparent);
}
.g-reward.is-locked {
  opacity: 0.6;
}
.g-reward__visual {
  display: grid;
  place-items: center;
  aspect-ratio: 16/9;
  border-radius: 11px;
  overflow: hidden;
  color: var(--g-accent-strong);
  font-weight: 800;
  font-size: 13.5px;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--g-accent) 14%, #ffffff),
    color-mix(in srgb, var(--g-accent) 4%, #ffffff)
  );
  border: 1px solid color-mix(in srgb, var(--g-accent) 18%, transparent);
}
.g-reward__visual img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.g-reward__copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}
.g-reward__copy h3 {
  margin: 0;
  font-size: 15.5px;
  font-weight: 800;
}
.g-reward__copy p {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--g-muted);
}

.g-redemptions {
  margin-top: 2px;
  padding: 13px 16px;
  border-radius: 13px;
  background: color-mix(in srgb, var(--g-accent) 7%, transparent);
  border: 1px solid color-mix(in srgb, var(--g-accent) 16%, transparent);
}
.g-redemptions__title {
  font-size: 12px;
  font-weight: 800;
  color: var(--g-accent-strong);
}
.g-redemptions__list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}
.g-redemptions__list b {
  font-size: 12px;
  font-weight: 700;
  padding: 4px 11px;
  border-radius: 999px;
  background: color-mix(in srgb, #ffffff 70%, transparent);
  color: var(--g-text);
}
.g-redemptions__list b.st-approved,
.g-redemptions__list b.st-fulfilled {
  color: var(--g-accent-strong);
}
.g-redemptions__list b.st-rejected,
.g-redemptions__list b.st-cancelled {
  color: color-mix(in srgb, #d2453a 80%, #000);
}

/* 排行 */
.g-board {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.g-board-current {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.g-board-current__label {
  color: var(--g-accent-strong);
  font-size: 12px;
  font-weight: 850;
}
.g-board-separator {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 12px;
  align-items: center;
  margin: 6px 0;
  color: var(--g-muted);
  font-size: 11px;
  font-weight: 850;
}
.g-board-separator::before,
.g-board-separator::after {
  height: 1px;
  background: color-mix(in srgb, var(--g-line) 76%, var(--g-accent) 18%);
  content: '';
}
.g-leader {
  display: grid;
  grid-template-columns: 38px 38px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--sos-space-3);
  padding: 11px 16px;
  border-radius: 13px;
  border: 1px solid transparent;
  background: color-mix(in srgb, #ffffff 48%, transparent);
  color: inherit;
  text-decoration: none;
  transition:
    border-color 0.2s,
    background 0.2s,
    transform 0.2s;
}
.g-leader--current {
  border-color: color-mix(in srgb, var(--g-accent) 34%, transparent);
  background: color-mix(in srgb, var(--g-accent) 11%, #ffffff);
}
.g-leader:hover {
  background: color-mix(in srgb, #ffffff 80%, transparent);
  transform: translateX(2px);
  border-color: color-mix(in srgb, var(--g-accent) 26%, transparent);
}
.g-leader__no {
  font-size: 16px;
  font-weight: 850;
  color: var(--g-muted);
  text-align: center;
  font-family: var(--mono);
}
.g-leader--current .g-leader__no {
  color: var(--g-accent-strong);
}
.g-leader.is-top .g-leader__no {
  color: var(--g-gold);
}
.g-leader__rating {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border-radius: 9px;
  font-size: 14px;
  font-weight: 900;
  color: #fff;
  background: var(--g-accent);
}
.g-leader__rating.rating-S,
.g-leader__rating.rating-X,
.g-leader__rating.rating-A {
  background: var(--g-gold);
  color: #3a2606;
}
.g-leader__name {
  font-weight: 750;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.g-leader__name i {
  font-style: normal;
  font-size: 11px;
  font-weight: 700;
  color: var(--g-muted);
  margin-left: 8px;
}
.g-leader b {
  font-weight: 850;
  color: var(--g-accent-strong);
}

/* 评级 */
.g-rating-reqs {
  display: grid;
  gap: 14px;
  padding: 16px;
  border-radius: 13px;
  background: color-mix(in srgb, var(--g-accent) 6%, transparent);
}
.g-req__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  font-size: 12.5px;
  font-weight: 700;
  margin-bottom: 6px;
}
.g-req__head b {
  color: var(--g-accent-strong);
}
.g-req__bar {
  height: 8px;
  border-radius: 999px;
  overflow: hidden;
  background: color-mix(in srgb, #000 8%, transparent);
}
.g-req__bar i {
  display: block;
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(
    90deg,
    var(--g-accent),
    color-mix(in srgb, var(--g-accent) 60%, #7bf0d8)
  );
  transition: width 0.5s ease;
}

/* feedback */
.g-feedback {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 18px;
  border-radius: 14px;
  font-size: 13px;
  font-weight: 600;
  color: var(--g-muted);
  border: 1px solid var(--g-line);
  background: color-mix(in srgb, #ffffff 50%, transparent);
  -webkit-backdrop-filter: blur(10px);
  backdrop-filter: blur(10px);
}
.g-feedback__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--g-accent);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--g-accent) 20%, transparent);
  flex-shrink: 0;
}

/* 弹窗 */
.g-modal {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: grid;
  place-items: center;
  padding: 20px;
  background: color-mix(in srgb, #06201d 56%, transparent);
  -webkit-backdrop-filter: blur(4px);
  backdrop-filter: blur(4px);
}
.g-dialog {
  position: relative;
  width: min(480px, 100%);
  max-height: 86vh;
  overflow: auto;
  padding: clamp(22px, 3vw, 30px);
  border-radius: 20px;
  border: 1px solid var(--g-line);
  background: #fff;
  box-shadow: 0 40px 80px -30px rgba(8, 40, 36, 0.5);
}
.g-dialog__close {
  position: absolute;
  top: 14px;
  right: 14px;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 50%;
  background: color-mix(in srgb, #000 6%, transparent);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  color: var(--g-muted);
}
.g-dialog__close:hover {
  background: color-mix(in srgb, #000 12%, transparent);
}
.g-dialog--table {
  width: min(680px, 100%);
}
.g-dialog__eyebrow {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--g-accent-strong);
}
.g-dialog h2 {
  margin: 6px 0 14px;
  font-size: 21px;
  font-weight: 850;
}
.g-dialog__lede {
  margin: 0 0 14px;
  font-size: 13.5px;
  line-height: 1.7;
  color: var(--g-muted);
}
.g-dialog__lede b {
  color: var(--g-accent-strong);
}
.g-rule-modal-table {
  overflow-x: auto;
  border: 1px solid var(--g-line);
  border-radius: 12px;
}
.g-rule-modal-table table {
  width: 100%;
  min-width: 420px;
  border-collapse: collapse;
  font-size: 13px;
}
.g-rule-modal-table th,
.g-rule-modal-table td {
  padding: 10px 12px;
  border-bottom: 1px solid var(--g-line);
  text-align: left;
  color: var(--g-muted);
}
.g-rule-modal-table th {
  background: color-mix(in srgb, var(--g-accent) 8%, transparent);
  color: var(--g-text);
  font-weight: 850;
}
.g-rule-modal-table tr:last-child td {
  border-bottom: 0;
}
.g-rule-modal-table th:first-child,
.g-rule-modal-table td:first-child {
  font-family: var(--mono);
  color: var(--g-text);
  white-space: nowrap;
}
.g-dialog__actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 14px;
}

.g-fade-enter-active,
.g-fade-leave-active {
  transition: opacity 0.2s ease;
}
.g-fade-enter-from,
.g-fade-leave-to {
  opacity: 0;
}

/* ============ 响应式 ============ */
@media (max-width: 880px) {
  .guild-shell {
    grid-template-columns: 1fr;
  }
  .g-deck {
    position: static;
  }
  .g-nav {
    flex-direction: row;
    flex-wrap: wrap;
  }
  .g-nav__item {
    flex: 1 1 calc(50% - 4px);
  }
  .g-quest {
    grid-template-columns: 1fr;
  }
}

/* ============ 关灯（暗色）适配 ============ */
/* 整条选择器必须放进 :global(...)，否则 Vue scoped 会丢弃括号外的后代选择器。 */
:global(html.art-lights-out .guild-shell) {
  --g-text: #f3f8ff;
  --g-muted: rgba(214, 230, 255, 0.7);
  --g-line: rgba(120, 165, 220, 0.2);
  --g-accent-strong: color-mix(in srgb, var(--g-accent) 66%, #d6fff4);
}
:global(html.art-lights-out .g-view) {
  background: rgba(14, 24, 46, 0.6);
}
:global(html.art-lights-out .g-quest),
:global(html.art-lights-out .g-budget),
:global(html.art-lights-out .g-reward),
:global(html.art-lights-out .g-leader),
:global(html.art-lights-out .g-rulecard) {
  background: rgba(12, 22, 44, 0.5);
}
:global(html.art-lights-out .g-quest.is-done) {
  border-color: color-mix(in srgb, var(--g-accent) 34%, rgba(120, 165, 220, 0.2));
  background: rgba(16, 42, 48, 0.58);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.07);
}
:global(html.art-lights-out .g-quest.is-done:hover) {
  background: rgba(19, 52, 58, 0.66);
}
:global(html.art-lights-out .g-quest__status.completed) {
  background: rgba(42, 129, 132, 0.68);
  color: rgba(235, 255, 250, 0.96);
}
:global(html.art-lights-out .g-rule-table-button) {
  border-color: color-mix(in srgb, var(--g-accent) 22%, rgba(160, 185, 230, 0.16));
  background: rgba(22, 36, 62, 0.7);
  color: color-mix(in srgb, var(--g-accent) 62%, #e3f3ff);
}
:global(html.art-lights-out .g-rule-table-button:hover) {
  border-color: color-mix(in srgb, var(--g-accent) 38%, rgba(180, 205, 245, 0.22));
  background: rgba(28, 48, 78, 0.82);
}
:global(html.art-lights-out .g-feedback) {
  background: rgba(12, 22, 44, 0.45);
}
:global(html.art-lights-out .g-dialog) {
  background: #0f1a30;
}
</style>
