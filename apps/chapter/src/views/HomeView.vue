<script setup>
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { resolveUploadUrl } from '@haruhi/api-client'
import { SosButton, SosEmptyState, SosInput, SosPage, SosPageHeader } from '@haruhi/ui'
import { api, session } from '@/api'
import ActivityTimeline from '@/components/ActivityTimeline.vue'
import BranchJoinButton from '@/components/BranchJoinButton.vue'

const branches = ref([])
const timeline = ref([])
const events = ref([])
const membership = ref(null)
const directoryQuery = ref('')
const loading = ref(true)
const error = ref('')
const visibleBranches = computed(() => {
  const needle = directoryQuery.value.trim().toLocaleLowerCase('zh-CN')
  return branches.value
    .filter((entry) => {
      if (!needle) return true
      return [
        entry.branch.name,
        entry.branch.shortName,
        entry.branch.localityName,
        entry.branch.summary,
      ].some((value) =>
        String(value || '')
          .toLocaleLowerCase('zh-CN')
          .includes(needle)
      )
    })
    .sort((left, right) => {
      const leftMine = Number(left.branch.id) === Number(membership.value?.branchId)
      const rightMine = Number(right.branch.id) === Number(membership.value?.branchId)
      if (leftMine !== rightMine) return leftMine ? -1 : 1
      if ((left.branch.status === 'paused') !== (right.branch.status === 'paused')) {
        return left.branch.status === 'paused' ? 1 : -1
      }
      return left.branch.name.localeCompare(right.branch.name, 'zh-CN')
    })
})
const upcomingEvents = computed(() => events.value.slice(0, 3))

function nextEventFor(entry) {
  return events.value.find((item) => item.branchSlug === entry.branch.slug)
}

async function loadHome() {
  loading.value = true
  error.value = ''
  try {
    await session.ensureReady()
    const [branchResult, timelineResult, eventResult, membershipResult] = await Promise.all([
      api.get('/branches'),
      api.get('/timeline'),
      api.get('/events'),
      session.state.user ? api.get('/membership') : Promise.resolve(null),
    ])
    branches.value = branchResult.items || []
    timeline.value = (timelineResult.items || []).slice(0, 4)
    events.value = eventResult.items || []
    membership.value = membershipResult?.membership || null
  } catch (reason) {
    error.value = reason?.message || '暂时无法载入支部资料'
  } finally {
    loading.value = false
  }
}

onMounted(loadHome)
</script>

<template>
  <SosPage contained="wide">
    <section class="chapter-hero">
      <div>
        <p class="sos-eyebrow">CHAPTER NETWORK</p>
        <h1>在你所在的城市，找到同好与行动伙伴</h1>
        <p>地方支部独立运营、自由组织，同时共享应援团的账号、设计规范和公共基础设施。</p>
        <div class="chapter-actions">
          <a href="#directory" class="sos-button sos-button--primary">浏览支部</a>
          <RouterLink to="/events" class="sos-button">查看活动</RouterLink>
        </div>
      </div>
      <div class="chapter-hero__signal" aria-hidden="true">
        <span>SOS</span><small>LOCAL SIGNAL</small>
      </div>
    </section>

    <div v-if="error" class="home-error">
      <p class="sos-notice sos-notice--danger">{{ error }}</p>
      <SosButton variant="secondary" @click="loadHome">重新载入</SosButton>
    </div>

    <section id="directory" class="chapter-section">
      <SosPageHeader
        title-as="h2"
        eyebrow="DIRECTORY"
        title="地方支部"
        copy="每个支部拥有自己的组织方式、Logo、QQ群与负责人。"
      />
      <div class="directory-toolbar">
        <label class="filter-field">
          <span>查找支部</span>
          <SosInput v-model="directoryQuery" placeholder="输入城市、地区或支部名称" />
        </label>
        <span v-if="!loading">{{ visibleBranches.length }} / {{ branches.length }} 个支部</span>
      </div>
      <div v-if="loading" class="chapter-grid">
        <div v-for="n in 3" :key="n" class="chapter-card chapter-card--loading"></div>
      </div>
      <SosEmptyState
        v-else-if="!visibleBranches.length"
        :title="branches.length ? '没有匹配的支部' : '支部资料正在准备'"
        :copy="branches.length ? '请尝试输入其他城市或支部名称。' : '首个支部公开后会显示在这里。'"
      />
      <div v-else class="chapter-grid">
        <article
          v-for="entry in visibleBranches"
          :key="entry.branch.id"
          class="chapter-card branch-card"
        >
          <RouterLink :to="`/branches/${entry.branch.slug}`" class="branch-card__link">
            <div
              class="branch-card__cover"
              :style="
                entry.brand?.coverPath
                  ? { backgroundImage: `url(${resolveUploadUrl(entry.brand.coverPath)})` }
                  : null
              "
            >
              <img
                v-if="entry.brand?.logoPath"
                :src="resolveUploadUrl(entry.brand.logoPath)"
                :alt="entry.brand.logoAlt || entry.branch.name"
              />
              <span v-else>{{ (entry.branch.shortName || entry.branch.name).slice(0, 2) }}</span>
            </div>
            <div class="chapter-card__body">
              <p class="sos-eyebrow">{{ entry.branch.localityName || '地方支部' }}</p>
              <h2>{{ entry.branch.name }}</h2>
              <p>{{ entry.branch.summary || entry.brand?.tagline || '欢迎了解我们。' }}</p>
              <strong class="branch-member-count">{{ entry.memberCount || 0 }} 名成员</strong>
              <span
                v-if="Number(entry.branch.id) === Number(membership?.branchId)"
                class="sos-badge"
              >我的支部</span>
              <span v-if="entry.branch.status === 'paused'" class="sos-badge">暂停运营</span>
              <p v-if="nextEventFor(entry)" class="branch-next-event">
                <small>下一场活动</small>
                <strong>{{ nextEventFor(entry).title }}</strong>
                <span>{{ nextEventFor(entry).startsAt?.replace('T', ' ').slice(0, 16) }}</span>
              </p>
            </div>
          </RouterLink>
          <footer class="branch-card__actions">
            <BranchJoinButton
              :branch="entry.branch"
              :membership="membership"
              compact
              @joined="membership = $event"
            />
          </footer>
        </article>
      </div>
    </section>

    <section class="chapter-section chapter-columns">
      <div>
        <SosPageHeader title-as="h2" eyebrow="ACTIVITY TIMELINE" title="活动时间线" />
        <SosEmptyState
          v-if="!timeline.length"
          title="暂无公开活动日程"
          copy="支部发布活动后会按时间显示在这里。"
        />
        <ActivityTimeline v-else :items="timeline" compact />
        <RouterLink v-if="timeline.length" to="/timeline" class="chapter-more">
          查看活动时间线 →
        </RouterLink>
      </div>
      <div>
        <SosPageHeader title-as="h2" eyebrow="EVENTS" title="近期地方活动" />
        <SosEmptyState v-if="!events.length" title="暂无公开活动" />
        <RouterLink
          v-for="item in upcomingEvents"
          :key="item.id"
          :to="`/branches/${item.branchSlug}/events/${item.slug}`"
          class="chapter-list-item"
        >
          <small>{{ item.startsAt?.slice(0, 16).replace('T', ' ') }} · {{ item.branchName }}</small><strong>{{ item.title }}</strong><span>{{ item.venueName || '线上活动' }}</span>
        </RouterLink>
        <RouterLink v-if="upcomingEvents.length" to="/events" class="chapter-more">
          查看全部活动 →
        </RouterLink>
      </div>
    </section>
  </SosPage>
</template>
