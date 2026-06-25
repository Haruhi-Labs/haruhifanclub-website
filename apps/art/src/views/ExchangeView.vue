<template>
  <section class="container-card exchange-page">
    <header class="exchange-header">
      <p class="eyebrow">Guild Board v1</p>
      <h1>冒险者公会悬赏系统</h1>
      <p>本页使用本地 mock 委托数据，先验证声望、接取、完成与反馈流程。</p>
    </header>

    <section class="guild-status panel" aria-label="公会状态栏">
      <div class="status-item">
        <span>冒险者编号</span>
        <strong>{{ adventurerNumber }}</strong>
      </div>
      <div class="status-item">
        <span>声望</span>
        <strong>{{ reputation }}</strong>
      </div>
      <div class="status-item">
        <span>等级</span>
        <strong>{{ guildState.rank }}</strong>
      </div>
    </section>

    <main class="quest-board" aria-label="委托列表">
      <section
        v-for="section in questSections"
        :key="section.id"
        class="quest-section panel"
        :class="`quest-section-${section.id}`"
      >
        <header class="quest-section-header">
          <div>
            <p>{{ section.kicker }}</p>
            <h2>{{ section.title }}</h2>
          </div>
          <span>{{ section.quests.length }}</span>
        </header>

        <article
          v-for="quest in section.quests"
          :key="quest.id"
          class="quest-card"
          :class="{
            accepted: isAccepted(quest),
            completed: isCompleted(quest),
            locked: isLocked(quest)
          }"
        >
          <div class="quest-main">
            <div class="quest-title-row">
              <h3>{{ quest.title }}</h3>
              <span class="quest-state">{{ questStatusLabel(quest) }}</span>
            </div>

            <div class="quest-meta">
              <span>奖励 {{ rewardLabel(quest) }}</span>
              <span v-if="quest.deadline">剩余 {{ quest.deadline }}</span>
              <span v-if="quest.progress">进度 {{ quest.progress }}</span>
            </div>

            <div v-if="quest.progress" class="quest-progress" aria-label="委托进度">
              <span :style="{ width: `${questProgressPercent(quest)}%` }"></span>
            </div>
          </div>

          <button
            type="button"
            class="quest-action"
            :disabled="isLocked(quest) || isCompleted(quest)"
            @click="handleQuestAction(quest)"
          >
            {{ questButtonLabel(quest) }}
          </button>
        </article>
      </section>
    </main>

    <footer class="guild-feedback panel" aria-live="polite">
      <span>公会记录</span>
      <p>{{ feedbackMessage }}</p>
    </footer>
  </section>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'

const guildState = {
  reputation: 1200,
  rank: 'Adept Observer',
  quests: {
    daily: [
      { id: 1, title: '每日观测：上传作品', reward: 10, progress: '0/1', status: 'active' },
      { id: 2, title: '浏览3个画廊作品', reward: 5, progress: '1/3', status: 'active' }
    ],
    limited: [
      { id: 10, title: 'SOS团特别委托：夏日主题投稿', reward: 200, deadline: '72小时', status: 'active' }
    ],
    unknown: [
      { id: 99, title: '??? 未知委托（权限不足）', reward: '???', status: 'locked' }
    ]
  }
}

const reputation = ref(guildState.reputation)
const adventurerNumber = ref('OBS-10981')
const feedbackMessage = ref('等待冒险者接取委托。完成委托后，声望会在本地状态中即时增加。')
const acceptedQuestIds = ref(new Set())
const completedQuestIds = ref(new Set())

const questSections = computed(() => [
  {
    id: 'daily',
    kicker: 'Daily',
    title: '日常委托',
    quests: guildState.quests.daily
  },
  {
    id: 'limited',
    kicker: 'Limited',
    title: '限时委托',
    quests: guildState.quests.limited
  },
  {
    id: 'unknown',
    kicker: 'Unknown',
    title: '未知委托',
    quests: guildState.quests.unknown
  }
])

onMounted(() => {
  if (typeof window === 'undefined') return

  const storageKey = 'haruhi-art-guild-adventurer-number'
  const savedNumber = window.localStorage.getItem(storageKey)

  if (savedNumber) {
    adventurerNumber.value = savedNumber
    return
  }

  const generatedNumber = `OBS-${10000 + Math.floor(Math.random() * 9000)}`
  adventurerNumber.value = generatedNumber
  window.localStorage.setItem(storageKey, generatedNumber)
})

function isLocked(quest) {
  return quest.status === 'locked'
}

function isAccepted(quest) {
  return acceptedQuestIds.value.has(quest.id)
}

function isCompleted(quest) {
  return completedQuestIds.value.has(quest.id)
}

function rewardLabel(quest) {
  return typeof quest.reward === 'number' ? `+${quest.reward} reputation` : quest.reward
}

function questStatusLabel(quest) {
  if (isLocked(quest)) return 'Locked'
  if (isCompleted(quest)) return 'Claimed'
  if (isAccepted(quest)) return 'Active'
  return 'Ready'
}

function questButtonLabel(quest) {
  if (isLocked(quest)) return '权限不足'
  if (isCompleted(quest)) return '已领取'
  if (isAccepted(quest)) return '完成/领取'
  return '接取委托'
}

function questProgressPercent(quest) {
  if (!quest.progress) return 0

  const [current, total] = quest.progress.split('/').map(Number)
  if (!Number.isFinite(current) || !Number.isFinite(total) || total <= 0) return 0
  return Math.min(100, Math.max(0, (current / total) * 100))
}

function handleQuestAction(quest) {
  if (isLocked(quest) || isCompleted(quest)) return

  if (!isAccepted(quest)) {
    acceptedQuestIds.value = new Set([...acceptedQuestIds.value, quest.id])
    feedbackMessage.value = `已接取「${quest.title}」。委托状态更新为 Active。`
    return
  }

  completedQuestIds.value = new Set([...completedQuestIds.value, quest.id])

  if (typeof quest.reward === 'number') {
    reputation.value += quest.reward
    feedbackMessage.value = `完成「${quest.title}」，声望 +${quest.reward}。`
    return
  }

  feedbackMessage.value = `完成「${quest.title}」，但该委托奖励仍处于未知状态。`
}
</script>

<style scoped>
.exchange-page {
  display: grid;
  max-width: 1120px;
  gap: 22px;
}

.exchange-header {
  max-width: 760px;
  margin: 0 auto;
  text-align: center;
}

.eyebrow {
  margin: 0 0 8px;
  color: var(--accent);
  font-size: 12px;
  font-weight: 950;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.exchange-header h1 {
  margin: 0;
  color: var(--text);
  font-size: clamp(32px, 5vw, 48px);
  font-weight: 950;
}

.exchange-header p {
  margin: 12px 0 0;
  color: var(--muted);
  font-weight: 700;
}

.guild-status {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1px;
  overflow: hidden;
  border-color: rgba(245, 51, 93, 0.16);
}

.status-item {
  min-width: 0;
  padding: 18px 20px;
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.68), rgba(255, 247, 250, 0.42)),
    radial-gradient(circle at 12% 16%, rgba(245, 51, 93, 0.11), transparent 32%);
}

.status-item span,
.guild-feedback span,
.quest-section-header p {
  display: block;
  color: var(--muted);
  font-size: 0.75rem;
  font-weight: 950;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.status-item strong {
  display: block;
  margin-top: 8px;
  overflow: hidden;
  color: var(--text);
  font-size: clamp(1.2rem, 2vw, 1.55rem);
  font-weight: 950;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.quest-board {
  display: grid;
  grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr) minmax(260px, 0.82fr);
  gap: 18px;
}

.quest-section {
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 14px;
  padding: 18px;
  overflow: hidden;
}

.quest-section::before {
  position: absolute;
  inset: 0;
  pointer-events: none;
  content: '';
  background:
    linear-gradient(90deg, rgba(245, 51, 93, 0.08), transparent 22%),
    linear-gradient(180deg, rgba(89, 168, 255, 0.08), transparent 45%);
  opacity: 0.9;
}

.quest-section-limited {
  border-color: rgba(245, 51, 93, 0.28);
  box-shadow:
    0 18px 34px rgba(245, 51, 93, 0.11),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
}

.quest-section-unknown {
  border-style: dashed;
}

.quest-section-header,
.quest-card {
  position: relative;
  z-index: 1;
}

.quest-section-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 14px;
}

.quest-section-header h2 {
  margin: 5px 0 0;
  color: var(--text);
  font-size: 1.32rem;
  font-weight: 950;
}

.quest-section-header > span {
  display: grid;
  width: 36px;
  height: 36px;
  place-items: center;
  color: var(--accent);
  font-weight: 950;
  background: var(--accent-soft);
  border: 1px solid rgba(245, 51, 93, 0.12);
  border-radius: 12px;
}

.quest-card {
  display: grid;
  gap: 16px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.66);
  border: 1px solid rgba(255, 255, 255, 0.72);
  border-radius: 18px;
  box-shadow: 0 12px 26px rgba(16, 24, 40, 0.08);
  transition:
    transform 0.18s ease,
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

.quest-card:hover {
  border-color: rgba(89, 168, 255, 0.3);
  box-shadow: 0 16px 32px rgba(16, 24, 40, 0.12);
  transform: translateY(-2px);
}

.quest-card.accepted {
  border-color: rgba(89, 168, 255, 0.34);
}

.quest-card.completed {
  border-color: rgba(34, 197, 94, 0.32);
}

.quest-card.locked {
  color: var(--muted);
  background: rgba(245, 245, 248, 0.58);
  border-style: dashed;
  box-shadow: none;
  filter: grayscale(0.35);
}

.quest-title-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.quest-title-row h3 {
  margin: 0;
  color: var(--text);
  font-size: 1rem;
  font-weight: 950;
  line-height: 1.42;
}

.quest-state {
  flex: 0 0 auto;
  padding: 4px 8px;
  color: var(--accent);
  font-size: 0.68rem;
  font-weight: 950;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  background: var(--accent-soft);
  border: 1px solid rgba(245, 51, 93, 0.12);
  border-radius: 999px;
}

.quest-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.quest-meta span {
  padding: 5px 9px;
  color: var(--muted);
  font-size: 0.76rem;
  font-weight: 850;
  background: rgba(255, 255, 255, 0.58);
  border: 1px solid rgba(148, 163, 184, 0.18);
  border-radius: 999px;
}

.quest-progress {
  height: 8px;
  margin-top: 14px;
  overflow: hidden;
  background: rgba(148, 163, 184, 0.18);
  border-radius: 999px;
}

.quest-progress span {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, #f5335d, #59a8ff);
  border-radius: inherit;
}

.quest-action {
  width: 100%;
  min-height: 42px;
  padding: 10px 14px;
  color: #fff;
  font-weight: 950;
  background: linear-gradient(135deg, #f5335d, #7c6dff);
  border: 0;
  border-radius: 14px;
  box-shadow: 0 10px 22px rgba(245, 51, 93, 0.18);
  cursor: pointer;
  transition:
    transform 0.18s ease,
    box-shadow 0.18s ease,
    opacity 0.18s ease;
}

.quest-action:hover:not(:disabled) {
  box-shadow: 0 14px 28px rgba(245, 51, 93, 0.24);
  transform: translateY(-1px);
}

.quest-action:disabled {
  color: var(--muted);
  background: rgba(148, 163, 184, 0.16);
  box-shadow: none;
  cursor: not-allowed;
}

.guild-feedback {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px;
}

.guild-feedback p {
  flex: 1;
  margin: 0;
  color: var(--text);
  font-weight: 800;
  line-height: 1.6;
  text-align: right;
}

:global(html.art-lights-out) .status-item,
:global(html.art-lights-out) .quest-card {
  background: rgba(12, 20, 44, 0.58);
  border-color: rgba(111, 206, 255, 0.12);
}

:global(html.art-lights-out) .quest-section-limited {
  border-color: rgba(255, 128, 166, 0.34);
}

:global(html.art-lights-out) .quest-card.locked {
  background: rgba(24, 28, 42, 0.42);
  border-color: rgba(174, 184, 204, 0.18);
}

:global(html.art-lights-out) .quest-title-row h3,
:global(html.art-lights-out) .status-item strong,
:global(html.art-lights-out) .guild-feedback p,
:global(html.art-lights-out) .quest-section-header h2 {
  color: #f7fbff;
}

:global(html.art-lights-out) .quest-meta span {
  color: rgba(220, 232, 255, 0.72);
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.08);
}

@media (max-width: 980px) {
  .quest-board {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .guild-status {
    grid-template-columns: 1fr;
  }

  .guild-feedback {
    align-items: flex-start;
    flex-direction: column;
  }

  .guild-feedback p {
    text-align: left;
  }
}
</style>
