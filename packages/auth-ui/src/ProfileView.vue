<script setup>
import { ref, reactive, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import {
  SosCard,
  SosField,
  SosInput,
  SosTextarea,
  SosButton,
  SosNotice,
  SosAvatar,
  SosEyebrow,
  SosTitle,
} from '@haruhi/ui'
import { useSession } from './useSession.js'
import AvatarCropper from './AvatarCropper.vue'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  settingsPath: { type: String, default: '/account/settings' },
  site: { type: String, default: undefined },
  // 嵌入个人控制台时为 true：不自绘 .hauth-root/.hauth-account 外壳，融入 layout。
  embedded: { type: Boolean, default: false },
})

const session = useSession(props.apiBase)
const router = useRouter()

const form = reactive({ nickname: '', bio: '' })
const saving = ref(false)
const error = ref('')
const okMsg = ref('')

const user = computed(() => session.state.user)
// 头像直接读登录态（上传/移除后即时刷新各处展示）；存的是站内绝对路径，可直接当 src
const avatar = computed(() => (session.state.user && session.state.user.avatar) || '')

function load() {
  const u = session.state.user
  if (u) {
    form.nickname = u.nickname || ''
    form.bio = u.bio || ''
  }
}

onMounted(async () => {
  if (!session.state.ready) await session.refresh()
  if (!session.state.user) {
    router.push(props.loginPath + '?redirect=' + encodeURIComponent('/account'))
    return
  }
  load()
})

async function save() {
  error.value = ''
  okMsg.value = ''
  if (!form.nickname.trim()) {
    error.value = '昵称不能为空'
    return
  }
  saving.value = true
  try {
    await session.updateProfile({
      nickname: form.nickname.trim(),
      bio: form.bio.trim(),
    })
    okMsg.value = '资料已保存'
  } catch (e) {
    error.value = e?.message || '保存失败'
  } finally {
    saving.value = false
  }
}

// ---------- 头像上传 / 裁切 / 移除 ----------
const fileInput = ref(null)
const cropperOpen = ref(false)
const cropSrc = ref('')
const avatarBusy = ref(false)
const MAX_PICK_BYTES = 12 * 1024 * 1024

function pickFile() {
  error.value = ''
  okMsg.value = ''
  fileInput.value?.click()
}

function onFileChange(e) {
  const file = e.target.files && e.target.files[0]
  // 重置，便于再次选同一文件也能触发 change
  e.target.value = ''
  if (!file) return
  if (!file.type.startsWith('image/')) {
    error.value = '请选择图片文件'
    return
  }
  if (file.size > MAX_PICK_BYTES) {
    error.value = '图片过大（上限 12 MB）'
    return
  }
  revokeCropSrc()
  cropSrc.value = URL.createObjectURL(file)
  cropperOpen.value = true
}

function revokeCropSrc() {
  if (cropSrc.value && cropSrc.value.startsWith('blob:')) URL.revokeObjectURL(cropSrc.value)
  cropSrc.value = ''
}

function closeCropper() {
  cropperOpen.value = false
  revokeCropSrc()
}

async function onCropConfirm(blob) {
  avatarBusy.value = true
  error.value = ''
  okMsg.value = ''
  try {
    await session.uploadAvatar(blob)
    okMsg.value = '头像已更新'
    closeCropper()
  } catch (e) {
    error.value = e?.message || '头像上传失败'
  } finally {
    avatarBusy.value = false
  }
}

async function removeAvatar() {
  if (avatarBusy.value) return
  avatarBusy.value = true
  error.value = ''
  okMsg.value = ''
  try {
    await session.removeAvatar()
    okMsg.value = '头像已移除'
  } catch (e) {
    error.value = e?.message || '移除失败'
  } finally {
    avatarBusy.value = false
  }
}

onBeforeUnmount(revokeCropSrc)
</script>

<template>
  <div :class="embedded ? '' : 'hauth-root sos-scope'" :data-sos-site="embedded ? undefined : site">
    <div v-if="user" :class="embedded ? 'sos-stack huc-page' : 'hauth-account'">
      <header class="sos-stack sos-stack--tight">
        <SosEyebrow>账号</SosEyebrow>
        <SosTitle as="h1" size="xl">个人资料</SosTitle>
        <p class="sos-copy">这些信息会作为你发布内容的署名展示。</p>
      </header>

      <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
      <SosNotice v-if="okMsg" tone="success">{{ okMsg }}</SosNotice>

      <!-- 身份卡 -->
      <SosCard as="section">
        <div class="sos-cluster">
          <div class="hauth-identity">
            <SosAvatar :src="avatar || undefined" :name="form.nickname || 'U'" size="lg" />
            <div class="hauth-identity__main">
              <p class="hauth-identity__name">{{ form.nickname || '未命名' }}</p>
              <p class="hauth-identity__mail">{{ user.email || user.username }}</p>
            </div>
          </div>
          <SosButton variant="secondary" size="sm" as="a" :href="settingsPath">
            账号设置 →
          </SosButton>
        </div>
      </SosCard>

      <!-- 头像 -->
      <SosCard as="section">
        <SosTitle as="h2" style="font-size: var(--sos-text-lg)">头像</SosTitle>
        <div class="hauth-avatar-edit" style="margin-top: var(--sos-space-5)">
          <SosAvatar :src="avatar || undefined" :name="form.nickname || 'U'" size="lg" />
          <div class="hauth-avatar-edit__actions">
            <div class="hauth-avatar-edit__row">
              <SosButton size="sm" :loading="avatarBusy" @click="pickFile">
                {{ avatar ? '更换头像' : '上传头像' }}
              </SosButton>
              <SosButton
                v-if="avatar"
                variant="ghost"
                size="sm"
                :disabled="avatarBusy"
                @click="removeAvatar"
              >
                移除
              </SosButton>
            </div>
            <p class="hauth-avatar-edit__hint">支持 JPG / PNG / WebP 等图片，上传后可裁切。</p>
          </div>
        </div>
        <input ref="fileInput" type="file" accept="image/*" hidden @change="onFileChange" />
      </SosCard>

      <!-- 编辑表单 -->
      <SosCard as="section">
        <SosTitle as="h2" style="font-size: var(--sos-text-lg)">编辑资料</SosTitle>
        <form class="sos-stack" style="margin-top: var(--sos-space-5)" @submit.prevent="save">
          <SosField label="昵称" required>
            <SosInput v-model="form.nickname" maxlength="32" required />
          </SosField>
          <SosField label="个人简介" help="可选，最多 280 字">
            <SosTextarea
              v-model="form.bio"
              :rows="4"
              maxlength="280"
              placeholder="介绍一下你自己…"
            />
          </SosField>
          <div>
            <SosButton type="submit" :loading="saving">保存资料</SosButton>
          </div>
        </form>
      </SosCard>
    </div>

    <AvatarCropper
      :open="cropperOpen"
      :src="cropSrc"
      @confirm="onCropConfirm"
      @cancel="closeCropper"
    />
  </div>
</template>
