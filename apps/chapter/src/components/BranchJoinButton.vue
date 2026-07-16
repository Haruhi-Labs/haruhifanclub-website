<script setup>
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosButton, SosInput, SosModal, SosNotice } from '@haruhi/ui'
import { api, session } from '@/api'

const props = defineProps({
  branch: { type: Object, required: true },
  membership: { type: Object, default: null },
  compact: { type: Boolean, default: false },
})
const emit = defineEmits(['joined'])
const route = useRoute()
const router = useRouter()
const open = ref(false)
const password = ref('')
const busy = ref(false)
const error = ref('')

const joinedHere = computed(() => props.membership?.branchId === props.branch.id)
const branchUnavailable = computed(() => props.branch.status !== 'active')
const joinedElsewhere = computed(
  () => props.membership && props.membership.branchId !== props.branch.id
)
const label = computed(() => {
  if (joinedHere.value) return '已加入'
  if (joinedElsewhere.value) return `已加入${props.membership.branchName}`
  if (branchUnavailable.value) return '暂停加入'
  return '加入该支部'
})

async function begin() {
  await session.ensureReady()
  if (!session.state.user) {
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (joinedHere.value || joinedElsewhere.value || branchUnavailable.value) return
  error.value = ''
  password.value = ''
  open.value = true
}

async function submit() {
  if (!password.value || busy.value) return
  busy.value = true
  error.value = ''
  try {
    const result = await api.post(`/branches/${props.branch.slug}/join`, {
      password: password.value,
    })
    open.value = false
    password.value = ''
    emit('joined', { ...result, branchName: props.branch.name })
  } catch (reason) {
    error.value = reason?.message || '暂时无法加入该支部'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <SosButton
    :size="compact ? 'sm' : undefined"
    :variant="joinedHere || joinedElsewhere ? 'secondary' : 'primary'"
    :disabled="joinedHere || joinedElsewhere || branchUnavailable"
    @click.stop.prevent="begin"
  >
    {{ label }}
  </SosButton>
  <SosModal v-model:open="open" title="确认加入地方支部" :close-on-backdrop="!busy">
    <div class="join-confirm">
      <SosNotice tone="warning" title="加入前请确认">
        每个账号同时只能加入一个地方支部。加入后不能自行退出；如需退出，必须提交申请并由所属支部管理员批准。
      </SosNotice>
      <p>
        你将加入<strong>{{ branch.name }}</strong>。请输入当前账号密码完成二次确认。
      </p>
      <label>
        当前账号密码
        <SosInput
          v-model="password"
          type="password"
          autocomplete="current-password"
          placeholder="请输入当前账号密码"
          @keyup.enter="submit"
        />
      </label>
      <SosNotice v-if="error" tone="danger" title="无法加入">{{ error }}</SosNotice>
    </div>
    <template #footer>
      <SosButton variant="ghost" :disabled="busy" @click="open = false">取消</SosButton>
      <SosButton :loading="busy" :disabled="!password" @click="submit">确认加入</SosButton>
    </template>
  </SosModal>
</template>
