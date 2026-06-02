<template>
  <div class="panel">
    <div class="toolbar">
      <h3 style="font-weight: bold; color: #374151; margin: 0;">收款与联系配置</h3>
      <button class="admin-btn btn-blue" :disabled="saving || loading" @click="saveConfig">
        {{ saving ? '保存中...' : '保存配置' }}
      </button>
    </div>

    <div class="panel-body">
      <div v-if="loading" class="text-sub">正在加载配置...</div>
      <template v-else>
        <div class="settings-grid">
          <div class="setting-block">
            <label class="form-label">微信收款二维码</label>
            <div class="qr-preview">
              <img v-if="form.payment.wechatQr" :src="form.payment.wechatQr" alt="微信收款码">
              <span v-else class="text-sub">未配置</span>
            </div>
            <input v-model.trim="form.payment.wechatQr" class="form-input" placeholder="粘贴图片URL或上传图片">
            <input type="file" accept="image/*" @change="uploadQr($event, 'wechatQr')">
          </div>

          <div class="setting-block">
            <label class="form-label">支付宝收款二维码</label>
            <div class="qr-preview">
              <img v-if="form.payment.alipayQr" :src="form.payment.alipayQr" alt="支付宝收款码">
              <span v-else class="text-sub">未配置</span>
            </div>
            <input v-model.trim="form.payment.alipayQr" class="form-input" placeholder="粘贴图片URL或上传图片">
            <input type="file" accept="image/*" @change="uploadQr($event, 'alipayQr')">
          </div>

          <div class="setting-block">
            <label class="form-label">好友二维码</label>
            <div class="qr-preview">
              <img v-if="form.payment.friendQr" :src="form.payment.friendQr" alt="好友二维码">
              <span v-else class="text-sub">未配置</span>
            </div>
            <input v-model.trim="form.payment.friendQr" class="form-input" placeholder="粘贴图片URL或上传图片">
            <input type="file" accept="image/*" @change="uploadQr($event, 'friendQr')">
          </div>
        </div>

      </template>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref, onMounted } from 'vue'
import { useShopStore } from '@/stores/shopStore'

const store = useShopStore()
const loading = ref(false)
const saving = ref(false)

const form = reactive({
  payment: {
    wechatQr: '',
    alipayQr: '',
    friendQr: ''
  }
})

const applyConfigToForm = (config) => {
  const payment = config?.payment || {}
  form.payment.wechatQr = payment.wechatQr || ''
  form.payment.alipayQr = payment.alipayQr || ''
  form.payment.friendQr = payment.friendQr || ''
}

const uploadQr = async (event, key) => {
  const file = event.target.files?.[0]
  if (!file) return
  const imageUrl = await store.uploadImage(file, {
    purpose: 'qr',
    convertToWebp: false
  })
  if (imageUrl) {
    form.payment[key] = imageUrl
  }
  event.target.value = ''
}

const saveConfig = async () => {
  saving.value = true
  const success = await store.updateSiteConfig({
    payment: {
      wechatQr: form.payment.wechatQr,
      alipayQr: form.payment.alipayQr,
      friendQr: form.payment.friendQr
    }
  })
  if (success) {
    applyConfigToForm(store.state.siteConfig)
  }
  saving.value = false
}

onMounted(async () => {
  loading.value = true
  const config = await store.fetchSiteConfig(true)
  if (config) applyConfigToForm(config)
  loading.value = false
})
</script>

<style scoped>
.settings-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1rem;
}

@media (min-width: 900px) {
  .settings-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

.setting-block {
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 1rem;
  background: #fafafa;
}

.qr-preview {
  width: 180px;
  height: 180px;
  border: 1px dashed #d1d5db;
  border-radius: 8px;
  background: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  margin-bottom: 0.75rem;
}

.qr-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
