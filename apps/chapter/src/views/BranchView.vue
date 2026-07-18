<script setup>
import { computed, nextTick, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { resolveUploadUrl } from '@haruhi/api-client'
import { canonicalUrl, usePageMeta } from '@haruhi/seo'
import {
  SosAvatar,
  SosButton,
  SosEmptyState,
  SosInput,
  SosModal,
  SosNotice,
  SosPage,
  SosTabs,
  SosTextarea,
} from '@haruhi/ui'
import { api, safeContactUrl, safeExternalUrl, session } from '@/api'
import ActivityTimeline from '@/components/ActivityTimeline.vue'
import BranchJoinButton from '@/components/BranchJoinButton.vue'
import EventCard from '@/components/EventCard.vue'
import OrganizationTree from '@/components/OrganizationTree.vue'

const route = useRoute()
const router = useRouter()
const data = ref(null)
const extra = ref(null)
const membership = ref(null)
const membershipSummary = ref({ memberCount: 0 })
const loading = ref(true)
const error = ref('')
const sectionLoading = ref(false)
const sectionError = ref('')
const memberSearch = ref('')
const memberTab = ref('ordinary')
const memberPage = ref(1)
const leaveOpen = ref(false)
const leaveReason = ref('')
const leaveBusy = ref(false)
const leaveError = ref('')
let loadRequestId = 0

const section = computed(() => route.meta.section || 'home')
const slug = computed(() => route.params.branchSlug)
const branch = computed(() => data.value?.branch)
const brand = computed(() => data.value?.brand)
const labels = {
  home: '首页',
  organization: '组织',
  members: '成员',
  events: '活动动态',
  merchandise: '特色周边',
  join: '加入我们',
}
const navigation = computed(() => {
  const source = data.value?.sections || []
  const hasJoinSection = source.some((item) => item.sectionKey === 'join')
  const seen = new Set()
  const items = []

  for (const item of source) {
    if (['about', 'posts', 'timeline'].includes(item.sectionKey)) continue
    if (item.sectionKey === 'contact' && hasJoinSection) continue
    const key = item.sectionKey === 'contact' ? 'join' : item.sectionKey
    if (!['organization', 'members', 'events', 'join'].includes(key) || seen.has(key)) continue
    seen.add(key)
    items.push({
      key,
      label: ['events', 'join'].includes(key) ? labels[key] : item.label || labels[key],
    })
  }

  if (data.value?.hasMerchandise) {
    const joinIndex = items.findIndex((item) => item.key === 'join')
    items.splice(joinIndex < 0 ? items.length : joinIndex, 0, {
      key: 'merchandise',
      label: labels.merchandise,
    })
  }

  return [{ key: 'home', label: labels.home }, ...items]
})
const joinedHere = computed(() => membership.value?.branchId === branch.value?.id)
const upcomingEvents = computed(() =>
  (extra.value?.events || extra.value?.items || []).filter(
    (item) => new Date(item.endsAt || item.startsAt).getTime() >= Date.now()
  )
)
const pastEvents = computed(() =>
  (extra.value?.events || extra.value?.items || []).filter(
    (item) => new Date(item.endsAt || item.startsAt).getTime() < Date.now()
  )
)
const officialMembers = computed(() => extra.value?.official || [])
const visibleOfficialMembers = computed(() =>
  officialMembers.value.filter((item) =>
    memberTab.value === 'active' ? item.status === 'active' : item.status === 'alumni'
  )
)
const joinContacts = computed(() => {
  const people = extra.value?.contacts?.people || []
  const primary = people.filter((item) => item.isPrimary)
  return primary.length ? primary : people
})
const memberCounts = computed(() => ({
  total: Number(membershipSummary.value.memberCount || 0),
  ordinary: Number(membershipSummary.value.ordinaryMemberCount || 0),
  active: Number(membershipSummary.value.activeMemberCount || 0),
  alumni: Number(membershipSummary.value.alumniMemberCount || 0),
}))
const memberTabs = computed(() => [
  {
    value: 'ordinary',
    label: `普通成员 ${memberCounts.value.ordinary} 人`,
    controls: 'members-panel-ordinary',
  },
  {
    value: 'active',
    label: `在任成员 ${memberCounts.value.active} 人`,
    controls: 'members-panel-active',
  },
  {
    value: 'alumni',
    label: `往届成员 ${memberCounts.value.alumni} 人`,
    controls: 'members-panel-alumni',
  },
])
const memberPageCount = computed(() =>
  Math.max(1, Math.ceil(Number(extra.value?.total || 0) / Number(extra.value?.pageSize || 48)))
)

usePageMeta(() =>
  branch.value
    ? {
        title: `${labels[section.value] || ''} · ${branch.value.name}`,
        description: branch.value.summary,
        canonical: canonicalUrl(route.path),
        ogImage: brand.value?.coverPath
          ? new URL(resolveUploadUrl(brand.value.coverPath), window.location.origin).href
          : undefined,
      }
    : null
)

async function loadMembership() {
  await session.ensureReady()
  if (!session.state.user) {
    membership.value = null
    return
  }
  try {
    const result = await api.get('/membership')
    membership.value = result.membership || null
  } catch {
    membership.value = null
  }
}

async function fetchMembers(branchSlug = slug.value) {
  const params = new URLSearchParams({ page: String(memberPage.value) })
  if (memberSearch.value.trim()) params.set('q', memberSearch.value.trim())
  const [ordinary, official] = await Promise.all([
    api.get(`/branches/${branchSlug}/members?${params}`),
    api.get(`/branches/${branchSlug}/official-members`),
  ])
  return { ...ordinary, official: official.items || [] }
}

async function loadMembers() {
  extra.value = await fetchMembers()
}

async function searchMembers() {
  memberPage.value = 1
  await loadMembers()
}

async function changeMemberPage(page) {
  memberPage.value = Math.min(Math.max(1, page), memberPageCount.value)
  await loadMembers()
}

async function changeMemberTab(value) {
  memberTab.value = ['ordinary', 'active', 'alumni'].includes(value) ? value : 'ordinary'
  const query = { ...route.query }
  if (memberTab.value === 'ordinary') delete query.member
  else query.member = memberTab.value
  await router.replace({ path: route.path, query })
}

async function fetchSection(branchSlug, sectionName) {
  if (sectionName === 'members') {
    return fetchMembers(branchSlug)
  }
  if (['organization', 'events', 'merchandise'].includes(sectionName)) {
    return api.get(`/branches/${branchSlug}/${sectionName}`)
  }
  if (sectionName === 'home') {
    const [timeline, events, qq, contacts] = await Promise.all([
      api.get(`/branches/${branchSlug}/timeline`),
      api.get(`/branches/${branchSlug}/events`),
      api.get(`/branches/${branchSlug}/qq-groups`),
      api.get(`/branches/${branchSlug}/contacts`),
    ])
    return {
      timeline: timeline.items || [],
      events: events.items || [],
      qqGroups: qq.items || [],
      contacts,
    }
  }
  if (sectionName === 'join') {
    const [qq, contacts] = await Promise.all([
      api.get(`/branches/${branchSlug}/qq-groups`),
      api.get(`/branches/${branchSlug}/contacts`),
    ])
    return { qqGroups: qq.items || [], contacts }
  }
  return null
}

async function load() {
  const requestId = ++loadRequestId
  const branchSlug = slug.value
  const sectionName = section.value
  loading.value = true
  error.value = ''
  sectionLoading.value = false
  sectionError.value = ''
  try {
    const [branchData, summary, sectionData] = await Promise.all([
      api.get(`/branches/${branchSlug}`),
      api.get(`/branches/${branchSlug}/membership-summary`),
      fetchSection(branchSlug, sectionName),
      loadMembership(),
    ])
    if (requestId !== loadRequestId) return
    data.value = branchData
    membershipSummary.value = summary
    extra.value = sectionData
    memberTab.value = ['active', 'alumni'].includes(String(route.query.member))
      ? String(route.query.member)
      : 'ordinary'
  } catch (reason) {
    if (requestId !== loadRequestId) return
    error.value = reason?.message || '无法载入支部资料'
  } finally {
    if (requestId === loadRequestId) loading.value = false
  }
}

async function scrollActiveNavigation() {
  await nextTick()
  const reduceMotion = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
  document.querySelector('.branch-nav [aria-current="page"]')?.scrollIntoView({
    behavior: reduceMotion ? 'auto' : 'smooth',
    block: 'nearest',
    inline: 'center',
  })
}

async function loadActiveSection() {
  const requestId = ++loadRequestId
  const branchSlug = slug.value
  const sectionName = section.value
  sectionLoading.value = true
  sectionError.value = ''
  try {
    const sectionData = await fetchSection(branchSlug, sectionName)
    if (requestId !== loadRequestId) return
    extra.value = sectionData
    memberTab.value = ['active', 'alumni'].includes(String(route.query.member))
      ? String(route.query.member)
      : 'ordinary'
  } catch (reason) {
    if (requestId !== loadRequestId) return
    sectionError.value = reason?.message || '无法载入该栏目'
  } finally {
    if (requestId === loadRequestId) sectionLoading.value = false
  }
  if (requestId === loadRequestId && !sectionError.value) await scrollActiveNavigation()
}

function methodsFor(personId) {
  return (extra.value?.methods || extra.value?.contacts?.methods || []).filter(
    (item) => item.personId === personId
  )
}

function onJoined(result) {
  membership.value = result
  membershipSummary.value.memberCount = Number(membershipSummary.value.memberCount || 0) + 1
  membershipSummary.value.ordinaryMemberCount =
    Number(membershipSummary.value.ordinaryMemberCount || 0) + 1
}

async function requestLeave() {
  if (!leaveReason.value.trim()) return
  leaveBusy.value = true
  leaveError.value = ''
  try {
    await api.post('/membership/leave-request', { reason: leaveReason.value.trim() })
    membership.value = { ...membership.value, state: 'leave_requested' }
    leaveOpen.value = false
    leaveReason.value = ''
  } catch (reason) {
    leaveError.value = reason?.message || '无法提交退出申请'
  } finally {
    leaveBusy.value = false
  }
}

watch(
  [slug, section],
  ([nextSlug], previous) => {
    const previousSlug = previous?.[0]
    if (!previousSlug || nextSlug !== previousSlug || !data.value) void load()
    else void loadActiveSection()
  },
  { immediate: true }
)
</script>

<template>
  <div v-if="loading" class="chapter-loading" role="status">正在载入支部资料……</div>
  <SosPage v-else-if="error" contained="content">
    <SosEmptyState title="暂时无法显示" :copy="error">
      <template #actions>
        <SosButton variant="secondary" @click="load">重新载入</SosButton>
        <RouterLink to="/" class="sos-button">返回支部目录</RouterLink>
      </template>
    </SosEmptyState>
  </SosPage>
  <div v-else-if="branch" class="branch-page" :data-accent="brand?.accentKey || 'blue'">
    <header
      class="branch-hero"
      :style="
        brand?.coverPath
          ? {
              '--branch-cover': `url(${resolveUploadUrl(brand.coverPath)})`,
              '--branch-cover-position': `${(brand.coverFocalX || 0.5) * 100}% ${(brand.coverFocalY || 0.5) * 100}%`,
            }
          : null
      "
      :class="{ 'branch-hero--compact': section !== 'home' }"
    >
      <div class="branch-hero__inner">
        <img
          v-if="brand?.logoPath"
          :src="resolveUploadUrl(brand.logoPath)"
          :alt="brand.logoAlt || branch.name"
        />
        <div class="branch-hero__copy">
          <p>{{ branch.localityName || '地方支部' }}</p>
          <component :is="section === 'home' ? 'h1' : 'p'" class="branch-hero__title">
            {{ branch.name }}
          </component>
          <span>{{ brand?.tagline || branch.summary }}</span>
          <div class="chapter-actions">
            <BranchJoinButton :branch="branch" :membership="membership" @joined="onJoined" />
            <SosButton
              v-if="joinedHere && membership?.state === 'active'"
              variant="secondary"
              @click="leaveOpen = true"
            >
              申请退出
            </SosButton>
            <span v-if="membership?.state === 'leave_requested'" class="sos-badge"
              >退出申请处理中</span
            >
          </div>
        </div>
      </div>
    </header>
    <nav class="branch-nav">
      <div>
        <RouterLink
          v-for="item in navigation"
          :key="item.key"
          :to="item.key === 'home' ? `/branches/${slug}` : `/branches/${slug}/${item.key}`"
          :aria-current="section === item.key ? 'page' : undefined"
        >
          {{ item.label }}
        </RouterLink>
      </div>
    </nav>

    <SosPage contained="wide">
      <div
        v-if="sectionLoading"
        class="chapter-loading chapter-loading--section"
        role="status"
        aria-live="polite"
      >
        正在载入{{ labels[section] || '栏目' }}……
      </div>
      <section v-else-if="sectionError" class="chapter-section">
        <SosEmptyState title="暂时无法显示此栏目" :copy="sectionError">
          <template #actions>
            <SosButton variant="secondary" @click="loadActiveSection">重新载入</SosButton>
          </template>
        </SosEmptyState>
      </section>
      <section v-else-if="section === 'home'" class="chapter-section">
        <div class="branch-intro">
          <div>
            <p class="sos-eyebrow">ABOUT THIS CHAPTER</p>
            <h2>{{ branch.shortName || branch.name }}</h2>
            <p>{{ branch.aboutText || branch.summary || '欢迎了解本支部。' }}</p>
          </div>
          <aside>
            <strong>所在地区</strong><span>{{ branch.localityName || '未填写' }}</span>
            <strong>支部成员</strong><span>{{ membershipSummary.memberCount }} 人</span>
            <strong>成立时间</strong><span>{{ branch.foundedOn || '未填写' }}</span>
          </aside>
        </div>
        <div class="chapter-columns chapter-section">
          <div>
            <h2>活动时间线</h2>
            <SosEmptyState
              v-if="!extra.timeline.length"
              title="暂无公开活动日程"
              copy="支部发布活动后会按时间显示在这里。"
            />
            <ActivityTimeline v-else :items="extra.timeline.slice(0, 4)" compact />
          </div>
          <div>
            <h2>即将举行</h2>
            <SosEmptyState v-if="!upcomingEvents.length" title="暂无活动" />
            <RouterLink
              v-for="item in upcomingEvents.slice(0, 4)"
              :key="item.id"
              :to="`/branches/${item.branchSlug}/events/${item.slug}`"
              class="chapter-list-item"
            >
              <small>{{ item.startsAt?.replace('T', ' ').slice(0, 16) }}</small
              ><strong>{{ item.title }}</strong
              ><span>{{ item.venueName || '线上活动' }}</span>
            </RouterLink>
          </div>
        </div>
        <div v-if="extra.qqGroups.length" class="contact-callout">
          <div>
            <p class="sos-eyebrow">JOIN US</p>
            <h2>加入 {{ branch.shortName || branch.name }}</h2>
            <p>主群：{{ extra.qqGroups[0].name }} · QQ群 {{ extra.qqGroups[0].groupNumber }}</p>
          </div>
          <RouterLink :to="`/branches/${slug}/join`" class="sos-button sos-button--primary">
            查看加入方式与联系方式
          </RouterLink>
        </div>
      </section>

      <section v-else-if="section === 'join'" class="chapter-section">
        <h1>加入我们</h1>
        <p class="prose-text">
          {{ branch.joinText || '点击加入支部，并通过下方 QQ 群或负责人参与日常活动。' }}
        </p>
        <div class="join-page-action">
          <BranchJoinButton :branch="branch" :membership="membership" @joined="onJoined" />
        </div>
        <div class="chapter-grid">
          <article
            v-for="group in extra.qqGroups"
            :key="group.id"
            class="chapter-card chapter-card__body"
          >
            <span v-if="group.isPrimary" class="sos-badge">主群</span>
            <h2>{{ group.name }}</h2>
            <strong>QQ群 {{ group.groupNumber }}</strong>
            <p>{{ group.description || group.audienceLabel }}</p>
            <img
              v-if="group.qrImagePath"
              class="qq-qr"
              :src="resolveUploadUrl(group.qrImagePath)"
              :alt="`${group.name}加群二维码`"
            />
            <a
              v-if="safeExternalUrl(group.joinUrl)"
              :href="safeExternalUrl(group.joinUrl)"
              class="sos-button sos-button--primary"
              >申请加群</a
            >
          </article>
        </div>
        <div v-if="extra.contacts?.people?.length" class="join-contact-section">
          <h2>联系方式</h2>
          <div class="chapter-grid">
            <article
              v-for="person in joinContacts"
              :key="person.id"
              class="chapter-card chapter-card__body"
            >
              <h3>{{ person.displayName }}</h3>
              <strong>{{ person.roleTitle }}</strong>
              <p>{{ person.responsibility }}</p>
              <ul class="contact-methods">
                <li v-for="method in methodsFor(person.id)" :key="method.id">
                  <span>{{ method.label || method.methodType }}</span>
                  <a v-if="safeContactUrl(method.url)" :href="safeContactUrl(method.url)">{{
                    method.value
                  }}</a>
                  <strong v-else>{{ method.value }}</strong>
                </li>
              </ul>
            </article>
          </div>
        </div>
        <SosEmptyState
          v-if="!extra.qqGroups.length && !extra.contacts?.people?.length"
          title="暂未公开加入渠道"
          copy="请稍后再来，或从支部首页了解最新情况。"
        />
      </section>

      <section v-else-if="section === 'members'" class="chapter-section">
        <div class="section-heading-with-count">
          <div>
            <p class="sos-eyebrow">MEMBERS</p>
            <h1>支部成员</h1>
          </div>
          <div class="member-total-count">
            <span>总成员人数</span><strong>{{ memberCounts.total }} 人</strong>
          </div>
        </div>
        <SosTabs
          :model-value="memberTab"
          :items="memberTabs"
          class="member-tabs"
          @update:model-value="changeMemberTab"
        />
        <div :id="`members-panel-${memberTab}`" role="tabpanel">
          <form
            v-if="memberTab === 'ordinary'"
            class="chapter-filter"
            @submit.prevent="searchMembers"
          >
            <SosInput v-model="memberSearch" placeholder="搜索成员昵称" /><button
              class="sos-button sos-button--primary"
            >
              查询
            </button>
          </form>
          <template v-if="memberTab === 'ordinary'">
            <SosEmptyState v-if="!extra.items?.length" title="暂时没有普通成员" />
            <template v-else>
              <ul class="ordinary-member-list">
                <li v-for="member in extra.items" :key="member.membershipId">
                  <SosAvatar
                    :src="member.avatar || undefined"
                    :name="member.displayName"
                    size="sm"
                  />
                  <strong>{{ member.displayName }}</strong>
                  <small v-if="member.state === 'leave_requested'">退出申请处理中</small>
                </li>
              </ul>
              <nav v-if="memberPageCount > 1" class="member-pagination" aria-label="普通成员分页">
                <SosButton
                  variant="secondary"
                  size="sm"
                  :disabled="memberPage <= 1"
                  @click="changeMemberPage(memberPage - 1)"
                >
                  上一页
                </SosButton>
                <span>第 {{ memberPage }} / {{ memberPageCount }} 页</span>
                <SosButton
                  variant="secondary"
                  size="sm"
                  :disabled="memberPage >= memberPageCount"
                  @click="changeMemberPage(memberPage + 1)"
                >
                  下一页
                </SosButton>
              </nav>
            </template>
          </template>
          <SosEmptyState
            v-else-if="!visibleOfficialMembers.length"
            :title="memberTab === 'active' ? '暂未设置在任成员' : '暂无往届成员'"
          />
          <div v-else class="member-grid">
            <article v-for="member in visibleOfficialMembers" :key="member.id" class="member-card">
              <SosAvatar :src="member.avatar || undefined" :name="member.displayName" size="lg" />
              <h2>{{ member.displayName }}</h2>
              <p>{{ member.bio }}</p>
              <small>{{ member.status === 'active' ? '在任' : '往届' }}</small>
            </article>
          </div>
        </div>
      </section>

      <section v-else-if="section === 'organization'" class="chapter-section">
        <h1>组织架构</h1>
        <p>{{ extra.version?.summary }}</p>
        <SosEmptyState v-if="!extra.version" title="暂未公开组织架构" />
        <OrganizationTree
          v-else
          :units="extra.units"
          :assignments="extra.assignments"
          :display-mode="extra.version.displayMode || 'grouped'"
        />
      </section>

      <section v-else-if="section === 'events'" class="chapter-section">
        <h1>活动动态</h1>
        <h2>即将举行</h2>
        <SosEmptyState v-if="!upcomingEvents.length" title="暂无即将举行的活动" />
        <div class="chapter-feed">
          <EventCard v-for="item in upcomingEvents" :key="item.id" :item="item" />
        </div>
        <h2 class="section-break">往期活动</h2>
        <SosEmptyState v-if="!pastEvents.length" title="暂无往期活动" />
        <div class="chapter-feed">
          <EventCard v-for="item in pastEvents" :key="item.id" :item="item" />
        </div>
      </section>

      <section v-else-if="section === 'merchandise'" class="chapter-section">
        <p class="sos-eyebrow">CHAPTER MERCHANDISE</p>
        <h1>特色周边</h1>
        <p class="prose-text">由 {{ branch.shortName || branch.name }} 设计与制作的特色纪念品。</p>
        <SosEmptyState
          v-if="!extra.items?.length"
          title="周边筹备中"
          copy="本支部暂时没有公开的特色周边。"
        />
        <div v-else class="merchandise-grid">
          <article v-for="item in extra.items" :key="item.id" class="merchandise-card sos-card">
            <div class="merchandise-card__media">
              <img :src="resolveUploadUrl(item.imagePath)" :alt="item.name" />
            </div>
            <div class="merchandise-card__body">
              <div v-if="item.tags?.length" class="merchandise-card__tags">
                <span v-for="tag in item.tags" :key="tag">{{ tag }}</span>
              </div>
              <h2>{{ item.name }}</h2>
              <p v-if="item.description">{{ item.description }}</p>
            </div>
          </article>
        </div>
      </section>
    </SosPage>

    <SosModal v-model:open="leaveOpen" title="申请退出支部">
      <div class="join-confirm">
        <SosNotice tone="warning" title="退出需要管理员审批">
          提交后你仍是该支部成员并计入人数；管理员批准后才能加入其他支部。 </SosNotice
        ><label
          >退出原因<SosTextarea
            v-model="leaveReason"
            :rows="5"
            placeholder="请说明退出原因" /></label
        ><SosNotice v-if="leaveError" tone="danger" title="提交失败">{{ leaveError }}</SosNotice>
      </div>
      <template #footer>
        <SosButton variant="ghost" :disabled="leaveBusy" @click="leaveOpen = false">取消</SosButton
        ><SosButton :loading="leaveBusy" :disabled="!leaveReason.trim()" @click="requestLeave">
          提交申请
        </SosButton>
      </template>
    </SosModal>
  </div>
</template>
