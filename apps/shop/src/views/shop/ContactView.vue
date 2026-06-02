<script setup>
import { reactive, ref } from 'vue'
import { useShopStore } from '@/stores/shopStore'

const store = useShopStore()
const submitting = ref(false)
const form = reactive({
  name: '',
  contact: '',
  orderId: '',
  content: ''
})

const resetForm = () => {
  form.name = ''
  form.contact = ''
  form.orderId = ''
  form.content = ''
}

const submitMessage = async () => {
  if (submitting.value) return
  const name = form.name.trim()
  const contact = form.contact.trim()
  const orderId = form.orderId.trim()
  const content = form.content.trim()

  if (!name) {
    store.showNotification('请填写您的称呼')
    return
  }
  if (!contact) {
    store.showNotification('请填写联系方式')
    return
  }
  if (!content) {
    store.showNotification('请填写留言内容')
    return
  }

  submitting.value = true
  const success = await store.submitContactMessage({ name, contact, orderId, content })
  submitting.value = false
  if (success) resetForm()
}
</script>

<template>
  <div class="contact-card">
    <div class="contact-info">
      <div>
        <h3 style="font-size: 1.25rem; font-weight: bold; margin-bottom: 1rem;">联系方式</h3>
        <p style="margin-bottom: 1.5rem; opacity: 0.8; font-size: 0.9rem;">如果您对周边或订单有任何疑问，请留言。</p>
        <ul class="contact-list" style="list-style: none; padding: 0; font-size: 0.875rem; line-height: 2;">
          <li class="contact-item contact-item-email">
            <i class="fa fa-envelope mr-2"></i>
            <span class="contact-email-text">haruhifanclub@outlook.com</span>
          </li>
          <li class="contact-item"><i class="fa-brands fa-weixin mr-2" style="color: #07c160;"></i> haruhiism15532</li>
        </ul>
      </div>
      <div style="margin-top: 2rem; padding-top: 2rem; border-top: 1px solid rgba(255,255,255,0.2); opacity: 0.6; font-size: 0.75rem;">
        &copy; 凉宫春日应援团
      </div>
    </div>
    <div class="contact-form">
      <h3 style="font-size: 1.25rem; font-weight: bold; color: #1f2937; margin-bottom: 1.5rem;">发送留言</h3>
      <form style="display: flex; flex-direction: column; gap: 1rem;" @submit.prevent="submitMessage">
        <div class="form-grid">
          <input v-model.trim="form.name" type="text" maxlength="60" placeholder="您的称呼" class="input-field">
          <input v-model.trim="form.contact" type="text" maxlength="80" placeholder="联系方式（QQ）" class="input-field">
        </div>
        <input v-model.trim="form.orderId" type="text" maxlength="60" placeholder="订单号 (选填)" class="input-field">
        <textarea
          v-model="form.content"
          placeholder="请描述您的问题..."
          class="input-field"
          style="height: 8rem;"
          maxlength="2000"
        ></textarea>
        <div style="display: flex; align-items: center; justify-content: space-between; gap: 0.75rem;">
          <span class="message-limit">{{ form.content.length }}/2000</span>
          <button type="submit" class="market-btn primary-action" style="padding: 0.5rem 2rem;" :disabled="submitting">
            {{ submitting ? '发送中...' : '发送' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.message-limit {
  font-size: 0.75rem;
  color: #9ca3af;
}

.contact-list {
  margin: 0;
}

.contact-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  min-width: 0;
}

.contact-item-email {
  white-space: nowrap;
  overflow: hidden;
}

.contact-email-text {
  display: inline-block;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
