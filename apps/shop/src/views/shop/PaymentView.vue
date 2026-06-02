<template>
  <div class="payment-container" v-if="order">
    <div class="text-center" style="margin-bottom: 2rem;">
        <h2 style="font-size: 1.5rem; font-weight: bold; color: #1f2937; margin: 0 0 0.5rem 0;">订单已提交</h2>
        <p style="color: #666; margin: 0;">请尽快完成支付，订单将保留 24 小时</p>
        <p style="color: #666; margin: 0;">务必点击“我已完成支付”，避免订单失效。</p>
    </div>
    <div class="payment-grid">
        <div class="order-info-col">
            <label style="font-size: 0.75rem; color: #666; text-transform: uppercase; margin-bottom: 0.25rem;">订单编号</label>
            <div class="flex-row items-center gap-2 order-sn-row" style="margin-bottom: 1.5rem;">
                <span class="order-sn">{{ order.id }}</span>
                <button @click="copy(order.id)" style="font-size: 0.75rem; background: #dbeafe; color: #2563eb; border: none; padding: 2px 6px; border-radius: 4px; cursor: pointer;">复制</button>
            </div>
            <label style="font-size: 0.75rem; color: #666; text-transform: uppercase; margin-bottom: 0.25rem;">
                {{ isMergedOrder ? '本次追加应付' : '应付金额' }}
            </label>
            <div class="payment-amount">¥{{ displayPayableAmount }}</div>
            <div v-if="isMergedOrder" style="font-size: 0.75rem; color: #6b7280; margin-bottom: 0.4rem;">
                旧订单金额 ¥{{ mergeSourceAmount }}，合并总金额 ¥{{ mergeMergedAmount }}
            </div>
            <div v-if="isMergedOrder && mergeShippingAdjustment < 0" style="font-size: 0.75rem; color: #16a34a; margin-bottom: 0.4rem;">
                已抵扣运费 -¥{{ mergeShippingRefund }}
            </div>
            <div v-else-if="isMergedOrder && mergeShippingAdjustment > 0" style="font-size: 0.75rem; color: #dc2626; margin-bottom: 0.4rem;">
                追加运费 +¥{{ mergeShippingExtra }}
            </div>
            <div v-if="Number(order.discountAmount) > 0" style="font-size: 0.75rem; color: #6b7280; margin-bottom: 0.5rem;">
                原价 ¥{{ order.originalTotal }}，优惠 -¥{{ order.discountAmount }}
            </div>
            <div style="font-size: 0.75rem; color: #9ca3af; margin-top: auto;">* 请务必支付准确金额</div>
        </div>
        <div class="qr-col">
            <!-- 支付方式切换 -->
            <div class="pay-method-switch">
                <button @click="payMethod = 'wechat'"
                    class="pay-method-btn"
                    :style="payMethod === 'wechat' ? 'background: #07c160; color: #fff; border: none; padding: 0.5rem 1.25rem; cursor: pointer; flex: 1; font-size: 0.875rem; font-weight: 500; display: flex; align-items: center; justify-content: center; gap: 0.35rem; white-space: nowrap;' : 'background: #fff; color: #374151; border: none; padding: 0.5rem 1.25rem; cursor: pointer; flex: 1; font-size: 0.875rem; display: flex; align-items: center; justify-content: center; gap: 0.35rem; white-space: nowrap;'">
                    <i class="fab fa-weixin"></i> 微信
                </button>
                <button @click="payMethod = 'alipay'"
                    class="pay-method-btn"
                    :style="payMethod === 'alipay' ? 'background: #1677ff; color: #fff; border: none; padding: 0.5rem 1.25rem; cursor: pointer; flex: 1; font-size: 0.875rem; font-weight: 500; display: flex; align-items: center; justify-content: center; gap: 0.35rem; white-space: nowrap;' : 'background: #fff; color: #374151; border: none; padding: 0.5rem 1.25rem; cursor: pointer; flex: 1; font-size: 0.875rem; display: flex; align-items: center; justify-content: center; gap: 0.35rem; white-space: nowrap;'">
                    <i class="fab fa-alipay"></i> 支付宝
                </button>
            </div>
            <div class="qr-placeholder">
                <img
                    v-if="currentQrUrl"
                    :src="currentQrUrl"
                    :alt="payMethod === 'wechat' ? '微信收款码' : '支付宝收款码'"
                    class="payment-qr-image clickable-qr"
                    @click="openQrPreview(currentQrUrl, payMethod === 'wechat' ? '微信收款码' : '支付宝收款码')"
                >
                <div v-else style="position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; flex-direction: column; gap: 0.25rem;">
                    <i class="fa fa-qrcode" style="font-size: 2.5rem; color: #333; opacity: 0.7;"></i>
                    <span style="font-size: 0.75rem; color: #6b7280;">后台未配置二维码</span>
                </div>
            </div>
            <p style="font-size: 0.875rem; font-weight: bold; margin: 0;" :style="payMethod === 'wechat' ? 'color: #07c160;' : 'color: #1677ff;'">
                {{ payMethod === 'wechat' ? '微信扫码支付' : '支付宝扫码支付' }}
            </p>
            <div style="margin-top: 0.75rem; font-size: 0.75rem; color: #4b5563;">
                请备注: <span class="remark-code">{{ order.id.split('-').pop() }}</span>
            </div>
            <div style="margin-top: 0.5rem; font-size: 0.75rem; color: #6b7280;">
                {{ PAYMENT_NOTE_TEXT }}
            </div>
        </div>
    </div>

    <p style="font-size: 0.75rem; color: #9ca3af; margin: 0 0 1.5rem 0; text-align: center;">管理员核对账单将有一定延迟，如果订单长期未确认，请联系我们。</p>

    <div class="flex-row gap-4 payment-actions" style="justify-content: center;">
        <button class="market-btn btn-ghost payment-action-btn" style="padding: 0.5rem 1.5rem;" @click="helpModal = true">
            <i class="fa fa-question-circle"></i> 无法付款
        </button>
        <button class="market-btn btn-green payment-action-btn" style="padding: 0.5rem 2rem;" @click="confirmPay">
            <i class="fa fa-check mr-2"></i> 我已完成支付
        </button>
    </div>

    <!-- 无法付款弹窗 -->
    <div v-if="helpModal" style="position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 100;" @click.self="helpModal = false">
        <div class="help-modal-card">
            <h3 style="font-size: 1.125rem; font-weight: bold; color: #1f2937; margin: 0 0 0.75rem 0;">添加好友转账</h3>
            <div class="help-modal-qr-box">
                <img
                    v-if="paymentConfig.friendQr"
                    :src="paymentConfig.friendQr"
                    alt="好友二维码"
                    class="payment-qr-image clickable-qr"
                    @click="openQrPreview(paymentConfig.friendQr, '好友二维码')"
                >
                <i v-else class="fab fa-weixin" style="font-size: 4rem; color: #07c160;"></i>
            </div>
            <p style="font-size: 0.875rem; color: #374151; line-height: 1.6; margin: 0 0 0.5rem 0;">
                {{ FRIEND_TRANSFER_HELP_TEXT }}
            </p>
            <p style="font-size: 0.875rem; color: #374151; line-height: 1.6; margin: 0 0 1rem 0;">
                添加好友时请备注订单号，添加好友后即可点击“我已完成支付”，无需等待转账完成。
            </p>
            <div style="display: inline-block; background: #f0fdf4; border: 1px solid #bbf7d0; padding: 0.35rem 0.75rem; border-radius: 6px; margin-bottom: 1.25rem;">
                <span style="font-family: monospace; font-weight: bold; color: #15803d; font-size: 0.9rem;">{{ order.id }}</span>
                <button @click="copy(order.id)" style="font-size: 0.7rem; background: #dcfce7; color: #15803d; border: none; padding: 2px 6px; border-radius: 4px; cursor: pointer; margin-left: 0.5rem;">复制</button>
            </div>
            <div>
                <button @click="helpModal = false" style="background: #f3f4f6; color: #374151; border: none; padding: 0.5rem 2rem; border-radius: 8px; cursor: pointer; font-size: 0.875rem;">我知道了</button>
            </div>
        </div>
    </div>

    <div v-if="qrPreview.show" class="qr-preview-overlay" @click.self="closeQrPreview">
        <div class="qr-preview-card">
            <button class="qr-preview-close" @click="closeQrPreview">关闭</button>
            <div class="qr-preview-title">{{ qrPreview.title }}</div>
            <img :src="qrPreview.url" :alt="qrPreview.title" class="qr-preview-image">
            <div class="qr-preview-tip">长按图片可保存到相册</div>
        </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useShopStore } from '@/stores/shopStore'
import { trackEvent } from '@/utils/analytics'

const store = useShopStore()
const router = useRouter()
const order = computed(() => store.state.currentOrder)
const roundMoney = (value) => Number((Number(value) || 0).toFixed(2))
const mergeMeta = computed(() => order.value?.mergeMeta || null)
const isMergedOrder = computed(() => Boolean(mergeMeta.value))
const mergeSourceAmount = computed(() => roundMoney(mergeMeta.value?.sourceAmount))
const mergeAppendedAmount = computed(() => roundMoney(mergeMeta.value?.appendedAmount))
const mergeMergedAmount = computed(() => roundMoney(mergeMeta.value?.mergedAmount || order.value?.total))
const mergeShippingAdjustment = computed(() => {
    const raw = Number(mergeMeta.value?.shippingAdjustment)
    if (Number.isFinite(raw)) return roundMoney(raw)

    const sourceShippingFee = Number(mergeMeta.value?.sourceShippingFee) || 0
    const appendedShippingFee = Number(mergeMeta.value?.appendedShippingFee) || 0
    const mergedShippingFee = Number(mergeMeta.value?.mergedShippingFee) || 0
    return roundMoney(mergedShippingFee - sourceShippingFee - appendedShippingFee)
})
const mergeShippingRefund = computed(() => roundMoney(Math.max(0, -mergeShippingAdjustment.value)))
const mergeShippingExtra = computed(() => roundMoney(Math.max(0, mergeShippingAdjustment.value)))
const displayPayableAmount = computed(() => {
    if (!isMergedOrder.value) return roundMoney(order.value?.total)
    const fromMeta = Number(mergeMeta.value?.incrementalPayable)
    if (Number.isFinite(fromMeta)) return roundMoney(fromMeta)
    return roundMoney(Math.max(0, mergeAppendedAmount.value + mergeShippingAdjustment.value))
})
const PAYMENT_NOTE_TEXT = '支付时请备注订单号后四位。'
const FRIEND_TRANSFER_HELP_TEXT = '保存二维码添加好友并转账。'

const payMethod = ref('wechat')
const helpModal = ref(false)
const qrPreview = reactive({
    show: false,
    url: '',
    title: ''
})
const paymentConfig = computed(() => store.state.siteConfig?.payment || {})
const currentQrUrl = computed(() => {
    if (payMethod.value === 'wechat') return paymentConfig.value.wechatQr
    return paymentConfig.value.alipayQr
})

onMounted(() => {
    store.fetchSiteConfig(false)
})

const copy = (text) => {
    navigator.clipboard.writeText(text)
    store.showNotification('已复制')
}

const openQrPreview = (url, title) => {
    if (!url) return
    qrPreview.url = url
    qrPreview.title = title || '二维码'
    qrPreview.show = true
}

const closeQrPreview = () => {
    qrPreview.show = false
}

const confirmPay = async () => {
    if (!order.value?.id) {
        store.showNotification('订单信息缺失，请重新下单')
        router.push('/')
        return
    }
    const success = await store.submitOrderPayment(order.value.id)
    if (!success) return
    trackEvent('payment_submitted', { orderId: order.value.id, payMethod: payMethod.value })
    store.clearCart()
    router.push('/success')
}
</script>

<style scoped>
.payment-qr-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 6px;
}

.order-sn-row {
    flex-wrap: wrap;
}

.payment-amount {
    font-size: 1.875rem;
    font-weight: bold;
    color: #1f2937;
    margin-bottom: 0.5rem;
}

.pay-method-switch {
    display: flex;
    gap: 0;
    margin-bottom: 1rem;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid #e5e7eb;
}

.payment-actions {
    justify-content: center;
}

.help-modal-card {
    background: #fff;
    border-radius: 12px;
    padding: 2rem;
    max-width: 320px;
    width: 90%;
    text-align: center;
}

.help-modal-qr-box {
    width: 180px;
    height: 180px;
    margin: 0 auto 1rem;
    background: #f3f4f6;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.clickable-qr {
    cursor: zoom-in;
}

.qr-preview-overlay {
    position: fixed;
    inset: 0;
    z-index: 140;
    background: rgba(0, 0, 0, 0.68);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
}

.qr-preview-card {
    width: min(92vw, 420px);
    background: #fff;
    border-radius: 12px;
    padding: 1rem;
    text-align: center;
    position: relative;
}

.qr-preview-close {
    position: absolute;
    right: 0.75rem;
    top: 0.75rem;
    border: none;
    border-radius: 6px;
    background: #f3f4f6;
    color: #374151;
    padding: 0.25rem 0.6rem;
    cursor: pointer;
    font-size: 0.75rem;
}

.qr-preview-title {
    font-size: 0.95rem;
    color: #111827;
    font-weight: 700;
    margin: 0.25rem 0 0.75rem;
}

.qr-preview-image {
    width: 100%;
    max-width: 360px;
    aspect-ratio: 1 / 1;
    object-fit: contain;
    border-radius: 10px;
    background: #fff;
}

.qr-preview-tip {
    margin-top: 0.6rem;
    font-size: 0.8rem;
    color: #6b7280;
}

@media (max-width: 639px) {
    .pay-method-btn {
        padding: 0.5rem 0.35rem !important;
        font-size: 0.8rem !important;
    }
    .payment-actions {
        flex-direction: column;
        align-items: stretch;
        gap: 0.7rem;
    }
    .payment-action-btn {
        width: 100%;
        justify-content: center;
    }
    .help-modal-card {
        padding: 1.25rem;
        max-width: 94vw;
    }
    .help-modal-qr-box {
        width: min(64vw, 180px);
        height: min(64vw, 180px);
    }
    .order-sn-row {
        flex-direction: column;
        align-items: flex-start;
        gap: 0.35rem;
    }
    .pay-method-switch {
        width: 100%;
    }
    .payment-amount {
        font-size: 1.5rem;
    }
}
</style>
