<script setup>
import { computed, reactive, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { resolveUploadUrl } from '@haruhi/api-client'
import { canonicalUrl, usePageMeta } from '@haruhi/seo'
import {
  SosAvatar,
  SosBreadcrumb,
  SosButton,
  SosEmptyState,
  SosInput,
  SosModal,
  SosNotice,
  SosPage,
  SosProgress,
  useToast,
} from '@haruhi/ui'
import { api, safeExternalUrl, session } from '@/api'
import EventAlbum from '@/components/EventAlbum.vue'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const item = ref(null)
const operations = ref({ topics: [], people: [], partners: [], cohosts: [], questions: [] })
const registration = ref(null)
const attendees = ref([])
const mine = ref(null)
const relatedEvents = ref([])
const photos = ref([])
const error = ref('')
const registerOpen = ref(false)
const registerBusy = ref(false)
const registerError = ref('')
const cancelOpen = ref(false)
const cancelBusy = ref(false)
const actionBusy = ref(false)
const actionError = ref('')
const anonymous = ref(false)
const answers = reactive({})

const internalRegistration = computed(() =>
  ['internal', 'both'].includes(item.value?.registrationMode)
)
const externalRegistration = computed(
  () => ['external', 'both'].includes(item.value?.registrationMode) && item.value?.registrationUrl
)
const registrationLabel = computed(() => {
  if (!mine.value) return registration.value?.state === 'waitlist' ? '加入候补' : '报名活动'
  return {
    pending: '等待审核',
    confirmed: '已确认报名',
    waitlisted: '候补中',
    rejected: '报名未通过',
    cancelled: '重新报名',
  }[mine.value.state]
})
const breadcrumbs = computed(() => [
  { label: '地方活动', href: '/events' },
  { label: item.value?.branchName || '支部', href: `/branches/${item.value?.branchSlug}` },
  { label: item.value?.title || '活动' },
])

usePageMeta(() =>
  item.value
    ? {
        title: `${item.value.title} · ${item.value.branchName}`,
        description: item.value.summary,
        canonical: canonicalUrl(route.path),
        ogType: 'article',
        ogImage: item.value.coverPath
          ? new URL(resolveUploadUrl(item.value.coverPath), window.location.origin).href
          : undefined,
      }
    : null
)

async function load() {
  error.value = ''
  await session.ensureReady()
  try {
    const result = await api.get(
      `/branches/${route.params.branchSlug}/events/${route.params.contentSlug}`
    )
    item.value = result.item
    operations.value = result.operations || operations.value
    registration.value = result.registration
    attendees.value = result.attendees || []
    mine.value = result.myRegistration || null
    anonymous.value = mine.value?.publicMode === 'anonymous'
    Object.assign(answers, mine.value?.answers || {})
    const [branchEvents, albumResult] = await Promise.all([
      api.get(`/branches/${item.value.branchSlug}/events`).catch(() => ({ items: [] })),
      api
        .get(`/branches/${item.value.branchSlug}/events/${item.value.slug}/photos`)
        .catch(() => ({ items: [] })),
    ])
    relatedEvents.value = (branchEvents.items || [])
      .filter((event) => event.id !== item.value.id)
      .slice(0, 3)
    photos.value = albumResult.items || []
  } catch (reason) {
    error.value = reason?.message || '活动不存在'
  }
}

async function openRegistration() {
  await session.ensureReady()
  if (!session.state.user) {
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (mine.value && !['cancelled', 'rejected'].includes(mine.value.state)) return
  registerError.value = ''
  registerOpen.value = true
}

function toggleMultiple(questionId, value, checked) {
  const key = String(questionId)
  const current = Array.isArray(answers[key]) ? [...answers[key]] : []
  if (checked && !current.includes(value)) current.push(value)
  if (!checked) current.splice(current.indexOf(value), 1)
  answers[key] = current
}

async function submitRegistration() {
  registerBusy.value = true
  registerError.value = ''
  try {
    await api.post(`/branches/${item.value.branchSlug}/events/${item.value.slug}/registration`, {
      answers: { ...answers },
      publicMode: anonymous.value ? 'anonymous' : 'named',
    })
    registerOpen.value = false
    await load()
    toast.success('报名已提交')
  } catch (reason) {
    registerError.value = reason?.message || '无法提交报名'
  } finally {
    registerBusy.value = false
  }
}

async function cancelRegistration() {
  cancelBusy.value = true
  actionError.value = ''
  try {
    await api.patch(`/branches/${item.value.branchSlug}/events/${item.value.slug}/registration`, {
      action: 'cancel',
    })
    cancelOpen.value = false
    await load()
    toast.success('已取消报名')
  } catch (reason) {
    actionError.value = reason?.message || '暂时无法取消报名'
  } finally {
    cancelBusy.value = false
  }
}

async function changePublicMode() {
  actionBusy.value = true
  actionError.value = ''
  const publicMode = mine.value.publicMode === 'anonymous' ? 'named' : 'anonymous'
  try {
    await api.patch(`/branches/${item.value.branchSlug}/events/${item.value.slug}/registration`, {
      publicMode,
    })
    await load()
    toast.success(publicMode === 'anonymous' ? '已改为匿名展示' : '已改为实名展示')
  } catch (reason) {
    actionError.value = reason?.message || '暂时无法修改公开方式'
  } finally {
    actionBusy.value = false
  }
}

watch(() => route.fullPath, load, { immediate: true })
</script>

<template>
  <SosPage contained="reading">
    <SosEmptyState v-if="error" title="无法显示活动" :copy="error">
      <template #actions>
        <SosButton variant="secondary" @click="load">重新载入</SosButton>
        <RouterLink to="/events" class="sos-button">返回地方活动</RouterLink>
      </template>
    </SosEmptyState>
    <article v-else-if="item" class="content-detail">
      <SosBreadcrumb :items="breadcrumbs" />
      <h1>{{ item.title }}</h1>
      <div class="event-topic-list">
        <span v-for="topic in operations.topics" :key="topic" class="sos-badge">{{ topic }}</span>
      </div>
      <div class="event-facts event-facts--stacked">
        <div>
          <strong>时间</strong
          ><span
            >{{ item.startsAt?.replace('T', ' ').slice(0, 16)
            }}<template v-if="item.endsAt">
              至 {{ item.endsAt.replace('T', ' ').slice(0, 16) }}</template
            ></span
          >
        </div>
        <div>
          <strong>地点</strong><span>{{ item.venueName || '线上' }}</span
          ><small>{{ item.address }}</small>
        </div>
        <div>
          <strong>活动形式</strong
          ><span>{{
            item.format === 'online' ? '线上' : item.format === 'hybrid' ? '线上线下混合' : '线下'
          }}</span>
        </div>
      </div>

      <section
        v-if="internalRegistration || externalRegistration"
        class="event-section registration-card"
      >
        <div>
          <p class="sos-eyebrow">REGISTRATION</p>
          <h2>活动报名</h2>
          <p v-if="registration.capacity">
            已确认 {{ registration.confirmed }} / {{ registration.capacity }} 人，候补
            {{ registration.waitlisted }} 人
          </p>
          <p v-else>已确认 {{ registration.confirmed }} 人</p>
        </div>
        <SosProgress
          v-if="registration.capacity"
          :value="registration.confirmed"
          :max="registration.capacity"
          label="已确认"
          :value-label="`${registration.confirmed} / ${registration.capacity} 人`"
        />
        <div class="chapter-actions">
          <SosButton
            v-if="internalRegistration"
            :disabled="
              (mine && !['cancelled', 'rejected'].includes(mine.state)) ||
              ['disabled', 'not_open', 'closed'].includes(registration.state)
            "
            @click="openRegistration"
          >
            {{ registrationLabel }}
          </SosButton>
          <a
            v-if="externalRegistration && safeExternalUrl(item.registrationUrl)"
            :href="safeExternalUrl(item.registrationUrl)"
            class="sos-button"
            >前往外部报名</a
          >
          <SosButton
            v-if="mine && !['cancelled', 'rejected'].includes(mine.state)"
            variant="secondary"
            :loading="actionBusy"
            @click="changePublicMode"
          >
            {{ mine.publicMode === 'anonymous' ? '改为实名展示' : '改为匿名展示' }}
          </SosButton>
          <SosButton
            v-if="mine && !['cancelled', 'rejected'].includes(mine.state)"
            variant="ghost"
            :disabled="actionBusy"
            @click="cancelOpen = true"
          >
            取消报名
          </SosButton>
        </div>
        <SosNotice v-if="mine" tone="info" title="我的报名状态">
          {{ registrationLabel
          }}<template v-if="mine.publicMode === 'anonymous'">
            · 公开名单显示为匿名参与者 {{ String(mine.anonymousNumber).padStart(3, '0') }}
          </template>
        </SosNotice>
        <SosNotice v-if="actionError" tone="danger" title="操作失败">{{ actionError }}</SosNotice>
      </section>

      <img v-if="item.coverPath" :src="resolveUploadUrl(item.coverPath)" :alt="item.title" />
      <p class="content-detail__summary">{{ item.summary }}</p>
      <div class="prose-text">{{ item.content }}</div>

      <section v-if="photos.length" class="event-section">
        <p class="sos-eyebrow">ACTIVITY ALBUM</p>
        <h2>活动相册（{{ photos.length }}）</h2>
        <EventAlbum :items="photos" />
      </section>

      <section v-if="operations.cohosts.length" class="event-section">
        <h2>联合主办</h2>
        <div class="event-entity-grid">
          <RouterLink
            v-for="cohost in operations.cohosts"
            :key="cohost.id"
            :to="`/branches/${cohost.slug}`"
            class="chapter-card chapter-card__body"
          >
            <h3>{{ cohost.name }}</h3>
          </RouterLink>
        </div>
      </section>
      <section v-if="operations.people.length" class="event-section">
        <h2>嘉宾与活动人员</h2>
        <div class="member-grid">
          <article v-for="person in operations.people" :key="person.id" class="member-card">
            <SosAvatar
              :src="resolveUploadUrl(person.avatarPath) || undefined"
              :name="person.name"
              size="lg"
            />
            <h3>{{ person.name }}</h3>
            <strong>{{ person.title }}</strong>
            <p>{{ person.organization }}</p>
            <small>{{ person.role }}</small>
          </article>
        </div>
      </section>
      <section v-if="operations.partners.length" class="event-section">
        <h2>合作方</h2>
        <div class="event-entity-grid">
          <a
            v-for="partner in operations.partners"
            :key="partner.id"
            :href="safeExternalUrl(partner.url) || undefined"
            class="chapter-card chapter-card__body"
            ><img
              v-if="partner.logoPath"
              class="partner-logo"
              :src="resolveUploadUrl(partner.logoPath)"
              :alt="partner.name"
            />
            <h3>{{ partner.name }}</h3>
            <small>{{ partner.partnerType }}</small></a
          >
        </div>
      </section>

      <section v-if="attendees.length" class="event-section">
        <h2>参与者（{{ attendees.length }}）</h2>
        <div class="attendee-list">
          <div v-for="attendee in attendees" :key="attendee.id">
            <SosAvatar
              :src="attendee.avatar || undefined"
              :name="attendee.anonymous ? '?' : attendee.displayName"
            /><span>{{ attendee.displayName }}</span>
          </div>
        </div>
      </section>
      <section v-if="relatedEvents.length" class="event-section">
        <h2>同支部近期活动</h2>
        <div class="related-event-list">
          <RouterLink
            v-for="event in relatedEvents"
            :key="event.id"
            :to="`/branches/${event.branchSlug}/events/${event.slug}`"
          >
            <time :datetime="event.startsAt">{{
              event.startsAt?.replace('T', ' ').slice(0, 16)
            }}</time>
            <strong>{{ event.title }}</strong>
          </RouterLink>
        </div>
      </section>
    </article>

    <SosModal v-model:open="registerOpen" title="报名活动" wide>
      <div class="registration-form">
        <SosNotice tone="info" title="参与者名单">
          默认公开你的账号昵称与头像；选择匿名后，公开页只显示默认头像和活动内编号，管理员仍可查看真实账号及报名资料。
        </SosNotice>
        <label class="check-label"
          ><input v-model="anonymous" type="checkbox" /> 在公开参与者名单中匿名</label
        >
        <label v-for="question in operations.questions" :key="question.id">
          {{ question.label }}<span v-if="question.required">（必填）</span>
          <SosInput
            v-if="question.questionType === 'short_text'"
            v-model="answers[String(question.id)]"
          />
          <select
            v-else-if="question.questionType === 'single'"
            v-model="answers[String(question.id)]"
            class="sos-select"
          >
            <option value="">请选择</option>
            <option v-for="option in question.options" :key="option" :value="option">
              {{ option }}
            </option>
          </select>
          <span v-else class="question-options"
            ><label v-for="option in question.options" :key="option"
              ><input
                type="checkbox"
                :checked="(answers[String(question.id)] || []).includes(option)"
                @change="toggleMultiple(question.id, option, $event.target.checked)"
              />{{ option }}</label
            ></span
          >
        </label>
        <SosNotice v-if="registerError" tone="danger" title="报名失败">
          {{ registerError }}
        </SosNotice>
      </div>
      <template #footer>
        <SosButton variant="ghost" :disabled="registerBusy" @click="registerOpen = false">
          取消 </SosButton
        ><SosButton :loading="registerBusy" @click="submitRegistration"> 提交报名 </SosButton>
      </template>
    </SosModal>
    <SosModal v-model:open="cancelOpen" title="取消报名">
      <p>确定取消本次报名吗？如活动已满，名额会自动递补给候补成员。</p>
      <SosNotice v-if="actionError" tone="danger" title="取消失败">{{ actionError }}</SosNotice>
      <template #footer>
        <SosButton variant="ghost" :disabled="cancelBusy" @click="cancelOpen = false">
          返回
        </SosButton>
        <SosButton variant="danger" :loading="cancelBusy" @click="cancelRegistration">
          确认取消报名
        </SosButton>
      </template>
    </SosModal>
  </SosPage>
</template>
