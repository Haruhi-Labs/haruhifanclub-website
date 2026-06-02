<template>
  <div class="result-page">
    <div class="icon-circle"><i class="fa fa-clock"></i></div>
    <h2 class="success-title">支付已提交审核</h2>
    <p style="color: #4b5563; margin-bottom: 0.5rem;">感谢您对应援团的支持！</p>
    <p style="font-size: 0.875rem; color: #6b7280; margin-bottom: 2rem;">欢迎加入春日商城客服QQ群：<a href="https://qm.qq.com/q/o2J6WGZGka" target="_blank">965630594</a></p>
    <div class="success-actions">
        <button class="market-btn primary-action" @click="$router.push({ path: '/query', query: { id: orderId } })">查看订单详情</button>
        <button class="market-btn btn-ghost" @click="$router.push('/')">返回首页</button>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useShopStore } from '@/stores/shopStore'

const store = useShopStore()
const orderId = computed(() => store.state.currentOrder?.id || '')
</script>

<style scoped>
.success-title {
    font-size: 1.875rem;
    font-weight: bold;
    color: #1f2937;
    margin-bottom: 1rem;
}

.success-actions {
    display: flex;
    flex-direction: row;
    justify-content: center;
    gap: 1rem;
}

@media (max-width: 639px) {
    .success-title {
        font-size: 1.35rem;
    }
    .success-actions {
        flex-direction: column;
        align-items: stretch;
    }
    .success-actions .market-btn {
        width: 100%;
        justify-content: center;
    }
}
</style>