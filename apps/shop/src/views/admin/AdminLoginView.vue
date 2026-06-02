<template>
  <div class="login-page">
    <div class="login-card">
        <h2>后台登录</h2>
        <p class="hint">请输入管理员账号和密码</p>

        <label class="label">账号</label>
        <input v-model="form.username" class="input" type="text" placeholder="管理员账号" @keyup.enter="submit">

        <label class="label">密码</label>
        <input v-model="form.password" class="input" type="password" placeholder="管理员密码" @keyup.enter="submit">

        <button class="login-btn" :disabled="submitting" @click="submit">
            {{ submitting ? '登录中...' : '登录' }}
        </button>

        <button class="back-btn" @click="$router.push('/')">返回商城</button>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useShopStore } from '@/stores/shopStore'
import '@/assets/admin.css'

const route = useRoute()
const router = useRouter()
const store = useShopStore()

const form = reactive({ username: '', password: '' })
const submitting = ref(false)

const submit = async () => {
    const username = form.username.trim()
    const password = form.password
    if (!username || !password) {
        store.showNotification('请输入账号和密码')
        return
    }

    submitting.value = true
    const success = await store.adminLogin(username, password)
    submitting.value = false

    if (!success) return
    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/admin/dashboard'
    router.replace(redirect)
}
</script>

<style scoped>
.login-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(160deg, #1f2937 0%, #2563eb 100%);
    padding: 1rem;
}

.login-card {
    width: 100%;
    max-width: 360px;
    background: #fff;
    border-radius: 12px;
    padding: 1.5rem;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.25);
}

h2 {
    margin: 0 0 0.25rem 0;
    color: #1f2937;
}

.hint {
    margin: 0 0 1rem 0;
    color: #6b7280;
    font-size: 0.875rem;
}

.label {
    display: block;
    font-size: 0.875rem;
    color: #374151;
    margin-bottom: 0.25rem;
}

.input {
    width: 100%;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    padding: 0.625rem 0.75rem;
    margin-bottom: 0.875rem;
    outline: none;
}

.input:focus {
    border-color: #2563eb;
}

.login-btn,
.back-btn {
    width: 100%;
    border: none;
    border-radius: 8px;
    padding: 0.625rem 0.75rem;
    cursor: pointer;
}

.login-btn {
    background: #2563eb;
    color: #fff;
    margin-top: 0.25rem;
}

.login-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.back-btn {
    margin-top: 0.625rem;
    background: #f3f4f6;
    color: #374151;
}
</style>
