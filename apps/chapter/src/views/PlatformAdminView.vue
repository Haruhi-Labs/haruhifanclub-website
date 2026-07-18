<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { hasCapability } from '@haruhi/api-client'
import {
  SosButton,
  SosEmptyState,
  SosInput,
  SosPage,
  SosPageHeader,
  SosTabs,
  SosTextarea,
} from '@haruhi/ui'
import { api, session } from '@/api'

const branches = ref([])
const route = useRoute()
const router = useRouter()
const error = ref('')
const notice = ref('')
const form = reactive({
  slug: '',
  name: '',
  shortName: '',
  summary: '',
  localityName: '',
  status: 'draft',
})
const grantTarget = ref('')
const grantCapabilities = ref(['branch.lifecycle.manage', 'branch.platform.intervene'])
const superPassword = ref('')
const capabilityOptions = [
  ['branch.lifecycle.manage', '创建支部并管理支部生命周期'],
  ['branch.platform.intervene', '处理异常内容并暂停或归档支部'],
  ['branch.profile.manage', '管理所有支部的资料与页面导航'],
  ['branch.brand.manage', '管理所有支部的品牌素材'],
  ['branch.contacts.manage', '管理所有支部的联系方式'],
  ['branch.members.manage', '管理所有支部的成员'],
  ['branch.organization.manage', '管理所有支部的组织架构'],
  ['branch.timeline.write', '编辑所有支部的活动相册'],
  ['branch.timeline.publish', '发布所有支部的活动相册照片'],
  ['branch.events.write', '编辑所有支部的活动'],
  ['branch.events.publish', '发布所有支部的活动'],
  ['branch.events.attendees.manage', '管理所有支部的活动报名与签到'],
  ['branch.permissions.manage', '管理所有支部的管理员权限'],
  ['branch.audit.read', '查看所有支部的操作记录'],
]
const user = computed(() => session.state.user)
const canCreate = computed(() =>
  hasCapability(user.value, 'branch.lifecycle.manage', 'platform', 'chapter')
)
const tabs = computed(() => [
  { value: 'branches', label: '支部管理' },
  ...(canCreate.value ? [{ value: 'create', label: '创建支部' }] : []),
  ...(user.value?.isSuperAdmin ? [{ value: 'grants', label: '平台授权与安全' }] : []),
])
const activeTab = computed(() => {
  const requested = String(route.query.tab || 'branches')
  return tabs.value.some((tab) => tab.value === requested) ? requested : 'branches'
})

function changeTab(value) {
  router.replace({ path: route.path, query: value === 'branches' ? {} : { tab: value } })
}

async function load() {
  try {
    branches.value = (await api.get('/admin/branches')).items || []
  } catch (reason) {
    error.value = reason?.message || '无法载入管理列表'
  }
}

async function create() {
  error.value = ''
  try {
    await api.post('/admin/branches', form)
    Object.assign(form, {
      slug: '',
      name: '',
      shortName: '',
      summary: '',
      localityName: '',
      status: 'draft',
    })
    notice.value = '支部已创建'
    await load()
  } catch (reason) {
    error.value = reason?.message || '创建失败'
  }
}

async function savePlatformGrant() {
  error.value = ''
  try {
    if (!grantTarget.value.trim()) throw new Error('请填写需要授权的用户名')
    if (grantCapabilities.value.length && !superPassword.value) {
      throw new Error('授予平台权限必须输入当前超管密码')
    }
    await api.put('/admin/platform/grants', {
      username: grantTarget.value.trim(),
      capabilities: grantCapabilities.value,
      superPassword: superPassword.value || undefined,
    })
    notice.value = '平台能力已更新'
  } catch (reason) {
    error.value = reason?.message || '授权失败'
  } finally {
    superPassword.value = ''
  }
}

onMounted(async () => {
  await session.ensureReady()
  await load()
})
</script>

<template>
  <SosPage contained="wide">
    <SosPageHeader
      eyebrow="CHAPTER CONSOLE"
      title="地方支部管理"
      copy="你只会看到自己有权管理的支部。"
    />
    <p v-if="error" class="sos-notice sos-notice--danger">{{ error }}</p>
    <p v-if="notice" class="sos-notice sos-notice--success">{{ notice }}</p>
    <SosTabs
      :model-value="activeTab"
      :items="tabs"
      class="platform-admin-tabs"
      label="平台管理栏目"
      @update:model-value="changeTab"
    />
    <SosEmptyState
      v-if="activeTab === 'branches' && !branches.length"
      title="没有可管理的支部"
      copy="平台管理员可以创建首个支部或为账号分配实例能力。"
    />
    <div v-else-if="activeTab === 'branches'" class="admin-branch-list">
      <article v-for="branch in branches" :key="branch.id" class="chapter-card chapter-card__body">
        <p class="sos-eyebrow">{{ branch.status }}</p>
        <h2>{{ branch.name }}</h2>
        <p>{{ branch.localityName }} · {{ branch.summary }}</p>
        <RouterLink :to="`/branches/${branch.slug}/manage`" class="sos-button sos-button--primary">
          进入管理
        </RouterLink><RouterLink
          v-if="branch.status === 'active'"
          :to="`/branches/${branch.slug}`"
          class="sos-button"
        >
          查看公开页
        </RouterLink>
      </article>
    </div>
    <section v-if="activeTab === 'create' && canCreate" class="admin-panel">
      <h2>创建支部</h2>
      <div class="form-grid">
        <label>名称<SosInput v-model="form.name" /></label><label>Slug<SosInput v-model="form.slug" placeholder="beijing" /></label><label>简称<SosInput v-model="form.shortName" /></label><label>城市或地区<SosInput v-model="form.localityName" /></label><label>状态<select v-model="form.status" class="sos-select">
          <option value="draft">草稿</option>
          <option value="active">公开</option>
        </select></label><label class="form-wide">简介<SosTextarea v-model="form.summary" :rows="3" /> </label>
      </div>
      <SosButton @click="create">创建支部</SosButton>
    </section>
    <section v-if="activeTab === 'grants' && user?.isSuperAdmin" class="admin-panel">
      <h2>平台能力授权</h2>
      <p class="chapter-muted">仅超级管理员可操作。平台能力会作用于全部地方支部，请谨慎授予。</p>
      <div class="grant-editor">
        <label>用户名<SosInput v-model="grantTarget" placeholder="输入已注册用户名" /></label>
        <fieldset>
          <legend>允许执行的操作</legend>
          <label
            v-for="capability in capabilityOptions"
            :key="capability[0]"
            class="capability-option"
          >
            <input v-model="grantCapabilities" type="checkbox" :value="capability[0]" />
            <span><strong>{{ capability[1] }}</strong><small>{{ capability[0] }}</small></span>
          </label>
        </fieldset>
        <div class="super-confirm">
          <strong>总负责人二次确认</strong>
          <p>授予平台权限会让账号跨越所有支部边界，必须输入当前登录超管本人的密码。</p>
          <label>
            当前超管密码
            <input
              v-model="superPassword"
              class="sos-input"
              type="password"
              autocomplete="current-password"
            />
          </label>
        </div>
        <SosButton variant="secondary" @click="savePlatformGrant">保存平台授权</SosButton>
      </div>
    </section>
  </SosPage>
</template>
