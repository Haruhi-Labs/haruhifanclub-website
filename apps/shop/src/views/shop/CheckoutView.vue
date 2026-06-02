<template>
  <div class="checkout-layout">
    <!-- 左侧表单 -->
    <div>
        <div class="form-card">
            <h3 class="section-title">收货信息</h3>
            <div class="form-grid">
                <div class="col-span-2 input-group">
                    <label>收货人姓名 <span class="required">*</span></label>
                    <input v-model="form.name" type="text" class="input-field" placeholder="请填写收货人姓名">
                </div>
                <div class="input-group">
                    <label>手机号码 <span class="required">*</span></label>
                    <input v-model="form.phone" type="tel" class="input-field" placeholder="11位手机号" maxlength="11">
                </div>
                <div class="input-group">
                    <label>电子邮箱 <span class="required">*</span></label>
                    <input v-model="form.email" type="email" class="input-field" placeholder="用于接收订单通知">
                </div>
                
                <!-- 省市区选择器 (动态加载) -->
                <div class="col-span-2 input-group">
                    <label>所在地区 <span class="required">*</span></label>
                    <div class="flex-row gap-2">
                        <select v-model="form.province" class="select-field" @change="onProvinceChange">
                            <option value="">{{ loadingAddress ? '加载中...' : '选择省' }}</option>
                            <option v-for="p in addressData" :key="p.code" :value="p.name">{{ p.name }}</option>
                        </select>
                        <select v-model="form.city" class="select-field" @change="onCityChange" :disabled="!form.province">
                            <option value="">选择市</option>
                            <option v-for="c in availableCities" :key="c.code" :value="c.name">{{ c.name }}</option>
                        </select>
                        <select v-model="form.district" class="select-field" :disabled="!form.city">
                            <option value="">选择区/县</option>
                            <option v-for="d in availableDistricts" :key="d.code" :value="d.name">{{ d.name }}</option>
                        </select>
                    </div>
                </div>

                <div class="col-span-2 input-group">
                    <label>详细地址 <span class="required">*</span></label>
                    <textarea v-model="form.address" class="input-field" placeholder="街道、门牌号等"></textarea>
                </div>
            </div>
        </div>
        
        <div class="form-card">
            <h3 class="section-title">支付说明</h3>
            <div style="padding: 1rem; background-color: #fefce8; color: #854d0e; border-radius: var(--radius-md); font-size: 0.875rem;">
                <i class="fa fa-exclamation-triangle mr-1"></i> 
                订单提交后系统将为您锁定库存，请在 24 小时内完成支付。
            </div>
        </div>

        <div class="form-card">
            <div class="merge-title-row">
                <h3 class="section-title" style="margin-bottom: 0;">订单合并</h3>
                <label class="merge-toggle">
                    <input v-model="mergeForm.enabled" type="checkbox">
                    <span>合并到旧订单</span>
                </label>
            </div>
            <p style="margin: 0.5rem 0 0; font-size: 0.8rem; color: #6b7280;">
                支持合并所有未发货订单，需要校验完整订单号与手机号后四位。合并后邮费可一起计算，若合并后达到免邮标准，还可额外返还已付邮费。
            </p>
            <p style="margin: 0.4rem 0 0; font-size: 0.78rem; color: #9ca3af;">
                若旧订单与本次填写的收货信息不一致，合并后将以本次填写的收货信息为准。
            </p>

            <div v-if="mergeForm.enabled" class="merge-panel">
                <div class="merge-grid">
                    <input
                        v-model.trim="mergeForm.orderId"
                        type="text"
                        class="input-field"
                        placeholder="旧订单完整订单号"
                    >
                    <input
                        v-model.trim="mergeForm.phoneLast4"
                        type="text"
                        class="input-field"
                        placeholder="旧订单手机号后四位"
                        maxlength="4"
                        inputmode="numeric"
                    >
                </div>
                <div class="merge-actions">
                    <button
                        class="market-btn btn-ghost"
                        :disabled="mergeValidating"
                        @click="verifyMergeTarget"
                    >
                        {{ mergeValidating ? '校验中...' : '校验并计算合并金额' }}
                    </button>
                </div>
                <p v-if="mergeError" class="merge-error">{{ mergeError }}</p>
                <div v-else-if="mergePreview" class="merge-preview">
                    <div class="merge-preview-row">
                        <span>旧订单金额</span>
                        <span>¥{{ mergePreview.total }}</span>
                    </div>
                    <div class="merge-preview-row">
                        <span>本次下单金额</span>
                        <span>¥{{ payableTotal }}</span>
                    </div>
                    <div class="merge-preview-row" v-if="shippingAdjustmentEstimate < 0">
                        <span>合并运费返还</span>
                        <span style="color: #16a34a;">-¥{{ shippingRefundEstimate }}</span>
                    </div>
                    <div class="merge-preview-row" v-else-if="shippingAdjustmentEstimate > 0">
                        <span>合并追加运费</span>
                        <span style="color: #dc2626;">+¥{{ shippingExtraEstimate }}</span>
                    </div>
                    <div class="merge-preview-total">
                        <span>本次追加应付</span>
                        <span>¥{{ incrementalPayableEstimate }}</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- 右侧概览 -->
    <div>
        <div class="form-card summary-card">
            <h3 style="font-weight: bold; color: #374151; margin: 0 0 1rem 0;">订单概览</h3>
            <div class="order-item-list custom-scrollbar">
                <div v-for="item in cart" :key="item.id" class="summary-row">
                    <span class="summary-item-name">{{ item.name }} x{{ item.quantity }}</span>
                    <span style="color: #1f2937;">¥{{ item.price * item.quantity }}</span>
                </div>
            </div>
            <hr style="border: none; border-top: 1px dashed #eee; margin: 1rem 0;">
            <div class="summary-row"><span style="color: #666;">商品总额</span><span>¥{{ cartTotal }}</span></div>
            <div class="summary-row">
                <span style="color: #666;">运费 (满{{ store.freeShippingThreshold }}元包邮)</span>
                <span>¥{{ shippingFee }}</span>
            </div>
            <div style="margin: 0.75rem 0; padding: 0.75rem; background: #f9fafb; border: 1px dashed #d1d5db; border-radius: 8px;">
                <div style="font-size: 0.85rem; color: #4b5563; margin-bottom: 0.5rem;">优惠券</div>
                <div class="coupon-row">
                    <input
                        v-model.trim="couponCode"
                        class="input-field coupon-input"
                        maxlength="40"
                        placeholder="输入优惠券码"
                    >
                    <button
                        class="market-btn primary-action coupon-btn"
                        :disabled="isApplyingCoupon || !couponCode"
                        @click="applyCoupon"
                    >
                        {{ isApplyingCoupon ? '校验中...' : '使用' }}
                    </button>
                    <button
                        v-if="appliedCoupon"
                        class="market-btn btn-ghost coupon-btn"
                        @click="clearCoupon"
                    >
                        清除
                    </button>
                </div>
                <div v-if="appliedCoupon" style="margin-top: 0.5rem; font-size: 0.75rem; color: #16a34a;">
                    已应用: {{ appliedCoupon.code }} ({{ appliedCoupon.benefitText }})
                </div>
            </div>
            <div v-if="discountAmount > 0" class="summary-row">
                <span style="color: #666;">优惠减免</span>
                <span style="color: #16a34a;">-¥{{ discountAmount }}</span>
            </div>
            <div v-if="mergeForm.enabled && mergePreview" class="summary-row">
                <span style="color: #666;">旧订单金额</span>
                <span>¥{{ mergePreview.total }}</span>
            </div>
            <div v-if="mergeForm.enabled && mergePreview" class="summary-row">
                <span style="color: #666;">合并后总金额</span>
                <span>¥{{ mergedTotalEstimate }}</span>
            </div>
            <div v-if="mergeForm.enabled && mergePreview && shippingAdjustmentEstimate < 0" class="summary-row">
                <span style="color: #666;">合并运费返还</span>
                <span style="color: #16a34a;">-¥{{ shippingRefundEstimate }}</span>
            </div>
            <div v-if="mergeForm.enabled && mergePreview && shippingAdjustmentEstimate > 0" class="summary-row">
                <span style="color: #666;">合并追加运费</span>
                <span style="color: #dc2626;">+¥{{ shippingExtraEstimate }}</span>
            </div>
            <div class="total-row">
                <span style="font-weight: bold; color: #374151;">本次应付</span>
                <span class="total-price">¥{{ displayPayableTotal }}</span>
            </div>
            <button @click="submitOrder" :disabled="isSubmitting || isApplyingCoupon || mergeValidating" class="market-btn primary-action" style="width: 100%; margin-top: 1.5rem; padding: 0.75rem;">
                {{ isSubmitting ? '提交中...' : '提交订单' }}
            </button>
        </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, computed, ref, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useShopStore } from '@/stores/shopStore'
import { getAddressData } from '@/utils/chinaDivision'
import { resolveApiPath } from '@/utils/runtimePaths'

const store = useShopStore()
const router = useRouter()
const form = reactive({ name: '', phone: '', email: '', province: '', city: '', district: '', address: '' })
const isSubmitting = ref(false)
const addressData = ref([])
const loadingAddress = ref(false)
const couponCode = ref('')
const appliedCoupon = ref(null)
const isApplyingCoupon = ref(false)
const mergeForm = reactive({
    enabled: false,
    orderId: '',
    phoneLast4: ''
})
const mergePreview = ref(null)
const mergeError = ref('')
const mergeValidating = ref(false)

const roundMoney = (value) => Number((Number(value) || 0).toFixed(2))
const toShippingGroupKey = (value) => String(value || 'default').trim() || 'default'
const sumItemsAmount = (items = []) =>
    roundMoney((Array.isArray(items) ? items : []).reduce((sum, item) => {
        const quantity = Number(item?.quantity) || 0
        const price = Number(item?.price) || 0
        return sum + price * quantity
    }, 0))

const calculateShippingByItems = (items = [], productsTotal = 0) => {
    const totalProducts = roundMoney(productsTotal)
    if (totalProducts >= Number(store.freeShippingThreshold || 0)) return 0

    const groups = {}
    ;(Array.isArray(items) ? items : []).forEach((item) => {
        const key = toShippingGroupKey(item?.shippingTag)
        const fee = roundMoney(Number(item?.shippingCost) || 0)
        if (groups[key] === undefined || fee > groups[key]) {
            groups[key] = fee
        }
    })

    return roundMoney(Object.values(groups).reduce((sum, fee) => sum + Number(fee || 0), 0))
}

const cart = computed(() => store.state.cart)
const cartTotal = store.cartTotal
const shippingFee = store.shippingFee
const orderOriginalTotal = computed(() => Number((cartTotal.value + shippingFee.value).toFixed(2)))
const discountAmount = computed(() => Number(appliedCoupon.value?.discountAmount || 0))
const payableTotal = computed(() => Number(Math.max(0, orderOriginalTotal.value - discountAmount.value).toFixed(2)))
const oldOrderProductsTotal = computed(() => sumItemsAmount(mergePreview.value?.items || []))
const oldOrderShippingFee = computed(() => {
    if (!mergePreview.value) return 0
    return roundMoney(Math.max(0, Number(mergePreview.value.originalTotal || 0) - oldOrderProductsTotal.value))
})
const mergedShippingFeeEstimate = computed(() => {
    if (!mergeForm.enabled || !mergePreview.value) return shippingFee.value
    const mergedItems = [...(mergePreview.value.items || []), ...cart.value]
    const mergedProductsTotal = roundMoney(oldOrderProductsTotal.value + cartTotal.value)
    return calculateShippingByItems(mergedItems, mergedProductsTotal)
})
const shippingAdjustmentEstimate = computed(() => {
    if (!mergeForm.enabled || !mergePreview.value) return 0
    return roundMoney(mergedShippingFeeEstimate.value - oldOrderShippingFee.value - shippingFee.value)
})
const shippingRefundEstimate = computed(() => {
    return roundMoney(Math.max(0, -shippingAdjustmentEstimate.value))
})
const shippingExtraEstimate = computed(() => {
    return roundMoney(Math.max(0, shippingAdjustmentEstimate.value))
})
const incrementalPayableRawEstimate = computed(() => {
    if (!mergeForm.enabled || !mergePreview.value) return payableTotal.value
    return roundMoney(payableTotal.value + shippingAdjustmentEstimate.value)
})
const incrementalPayableEstimate = computed(() => {
    return roundMoney(Math.max(0, incrementalPayableRawEstimate.value))
})
const mergedTotalEstimate = computed(() => {
    if (!mergeForm.enabled || !mergePreview.value) return payableTotal.value
    return roundMoney(Math.max(0, Number(mergePreview.value.total || 0) + incrementalPayableRawEstimate.value))
})
const displayPayableTotal = computed(() => (mergeForm.enabled && mergePreview.value ? incrementalPayableEstimate.value : payableTotal.value))
const orderTotalForSubmit = computed(() => (mergeForm.enabled && mergePreview.value ? mergedTotalEstimate.value : payableTotal.value))

// 加载地址数据
onMounted(async () => {
    loadingAddress.value = true
    addressData.value = await getAddressData()
    loadingAddress.value = false
})

// 级联选择逻辑
const availableCities = computed(() => {
    const p = addressData.value.find(item => item.name === form.province)
    return p ? p.children : []
})
const availableDistricts = computed(() => {
    const c = availableCities.value.find(item => item.name === form.city)
    return c ? c.children : []
})
const onProvinceChange = () => { form.city = ''; form.district = '' }
const onCityChange = () => { form.district = '' }

// 校验逻辑
const validateForm = () => {
    if (!form.name.trim()) return '请填写收货人姓名'
    if (!/^1[3-9]\d{9}$/.test(form.phone)) return '请输入正确的11位手机号码'
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.email)) return '请输入正确的邮箱格式'
    if (!form.province || !form.city || !form.district) return '请选择完整的省市区'
    if (!form.address.trim()) return '请填写详细地址'
    return null
}

const clearCoupon = () => {
    appliedCoupon.value = null
    couponCode.value = ''
}

const applyCoupon = async () => {
    if (!couponCode.value) {
        store.showNotification('请输入优惠券码')
        return
    }
    if (cart.value.length === 0) {
        store.showNotification('购物车为空，无法使用优惠券')
        return
    }

    isApplyingCoupon.value = true
    const result = await store.previewCoupon({
        code: couponCode.value,
        orderAmount: orderOriginalTotal.value
    })
    isApplyingCoupon.value = false

    if (!result.ok) {
        store.showNotification(result.error || '优惠券不可用')
        return
    }

    appliedCoupon.value = result.data
    couponCode.value = result.data.code
    store.showNotification(`优惠券生效，立减 ¥${result.data.discountAmount}`)
}

watch([cartTotal, shippingFee], () => {
    if (!appliedCoupon.value) return
    appliedCoupon.value = null
    store.showNotification('订单金额已变化，请重新校验优惠券')
})

watch(() => mergeForm.enabled, (enabled) => {
    if (enabled) return
    mergePreview.value = null
    mergeError.value = ''
    mergeForm.orderId = ''
    mergeForm.phoneLast4 = ''
})

watch(() => [mergeForm.orderId, mergeForm.phoneLast4], () => {
    if (!mergeForm.enabled) return
    mergePreview.value = null
    mergeError.value = ''
})

const verifyMergeTarget = async () => {
    mergeError.value = ''
    mergePreview.value = null

    const orderId = mergeForm.orderId.trim()
    const phoneTail = mergeForm.phoneLast4.trim()
    if (!orderId) {
        mergeError.value = '请输入待合并订单号'
        return
    }
    if (!/^\d{4}$/.test(phoneTail)) {
        mergeError.value = '请输入手机号后四位'
        return
    }

    mergeValidating.value = true
    try {
        const params = new URLSearchParams({ phoneLast4: phoneTail })
        const response = await fetch(`${resolveApiPath(`/orders/${encodeURIComponent(orderId)}`)}?${params.toString()}`)
        const data = await response.json().catch(() => ({}))
        if (!response.ok) {
            mergeError.value = data.error || '待合并订单校验失败'
            return
        }

        if (![1, 2, 5].includes(Number(data.status))) {
            mergeError.value = '仅支持合并未发货订单'
            return
        }
        if (data.couponCode && appliedCoupon.value?.code) {
            mergeError.value = '旧订单已使用优惠券，请先清除本次优惠券再合并'
            return
        }

        mergePreview.value = data
    } catch (_) {
        mergeError.value = '网络错误，请稍后重试'
    } finally {
        mergeValidating.value = false
    }
}

const submitOrder = async () => {
    if (isSubmitting.value) return

    const error = validateForm()
    if (error) {
        store.showNotification(error)
        return
    }
    if (cart.value.length === 0) return
    if (mergeForm.enabled) {
        if (!mergeForm.orderId.trim()) {
            store.showNotification('请输入待合并订单号')
            return
        }
        if (!/^\d{4}$/.test(mergeForm.phoneLast4.trim())) {
            store.showNotification('请输入手机号后四位')
            return
        }
        if (!mergePreview.value || mergePreview.value.id !== mergeForm.orderId.trim()) {
            store.showNotification('请先校验待合并订单')
            return
        }
        if (mergePreview.value.couponCode && appliedCoupon.value?.code) {
            store.showNotification('旧订单已使用优惠券，请先清除本次优惠券再合并')
            return
        }
    }

    isSubmitting.value = true
    
    const now = new Date()
    const datePart = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}`
    const timePart = `${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}${String(now.getSeconds()).padStart(2, '0')}${String(now.getMilliseconds()).padStart(3, '0')}`
    const randPart = String(Math.floor(Math.random() * 10000)).padStart(4, '0')
    const id = `SOS-${datePart}-${timePart}-${randPart}`
    
    const orderData = {
        id,
        items: cart.value,
        contact: form,
        total: orderTotalForSubmit.value,
        couponCode: appliedCoupon.value?.code || '',
        mergeTarget: mergeForm.enabled ? {
            orderId: mergeForm.orderId.trim(),
            phoneLast4: mergeForm.phoneLast4.trim()
        } : null
    }

    const createdOrder = await store.createOrderBackend(orderData)
    
    isSubmitting.value = false
    
    if (createdOrder) {
        const finalOrderId = createdOrder.orderId || id
        store.setOrder({
            ...orderData,
            id: finalOrderId,
            total: createdOrder.total ?? orderTotalForSubmit.value,
            originalTotal: createdOrder.originalTotal ?? orderOriginalTotal.value,
            discountAmount: createdOrder.discountAmount ?? discountAmount.value,
            couponCode: createdOrder.couponCode || appliedCoupon.value?.code || null,
            mergeMeta: createdOrder.mergeMeta || null,
            payableNow: mergeForm.enabled && mergePreview.value ? incrementalPayableEstimate.value : (createdOrder.total ?? payableTotal.value)
        })
        store.clearCart()
        router.push('/payment')
    }
}
</script>

<style scoped>
.required { color: #ef4444; margin-left: 2px; }
.select-field { flex: 1; padding: 0.625rem; border: 1px solid #ddd; border-radius: var(--radius-md); outline: none; background: white; max-width: 32%; }
.checkout-layout { display: grid; grid-template-columns: 1fr; gap: 1.5rem; }
@media (min-width: 768px) { .checkout-layout { grid-template-columns: 2fr 1fr; } }
.form-card { background: white; padding: 1.5rem; border-radius: var(--radius-md); box-shadow: var(--shadow-sm); margin-bottom: 1.5rem; }
.section-title { font-size: 1.125rem; font-weight: bold; margin-bottom: 1rem; border-left: 4px solid var(--primary-color); padding-left: 0.75rem; }
.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
.col-span-2 { grid-column: span 2; }
.input-group label { display: block; font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 0.25rem; }
.input-field { width: 100%; padding: 0.625rem 1rem; border: 1px solid #ddd; border-radius: var(--radius-md); outline: none; transition: border-color 0.3s; }
.input-field:focus { border-color: var(--primary-color); }
textarea.input-field { resize: none; height: 6rem; }
.summary-card { position: sticky; top: 1.5rem; }
.order-item-list { max-height: 15rem; overflow-y: auto; margin-bottom: 1rem; padding-right: 0.5rem; }
.summary-row { display: flex; justify-content: space-between; font-size: 0.875rem; margin-bottom: 0.5rem; }
.total-row { display: flex; justify-content: space-between; align-items: flex-end; border-top: 1px solid #eee; padding-top: 1rem; margin-top: 1rem; }
.total-price { font-size: 1.5rem; font-weight: bold; color: var(--primary-color); }
.market-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.summary-item-name {
    width: 66%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.coupon-row { display: flex; gap: 0.5rem; }
.coupon-input { margin: 0; height: 2.25rem; flex: 1; }
.coupon-btn { white-space: nowrap; height: 2.25rem; }
.merge-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
}
.merge-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.82rem;
    color: #374151;
}
.merge-panel {
    margin-top: 0.8rem;
    padding: 0.8rem;
    border: 1px dashed #d1d5db;
    border-radius: 8px;
    background: #f9fafb;
}
.merge-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
}
.merge-actions {
    margin-top: 0.6rem;
}
.merge-error {
    margin: 0.55rem 0 0;
    font-size: 0.8rem;
    color: #dc2626;
}
.merge-preview {
    margin-top: 0.6rem;
    padding-top: 0.6rem;
    border-top: 1px solid #e5e7eb;
}
.merge-preview-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.84rem;
    color: #4b5563;
    margin-bottom: 0.35rem;
}
.merge-preview-total {
    display: flex;
    justify-content: space-between;
    font-size: 0.92rem;
    font-weight: 600;
    color: #1f2937;
}

@media (max-width: 639px) {
    .form-grid { grid-template-columns: 1fr; }
    .col-span-2 { grid-column: span 1; }
    .select-field { max-width: none; }
    .flex-row.gap-2 { flex-direction: column; }
    .summary-item-name { width: 58%; }
    .coupon-row { flex-wrap: wrap; }
    .coupon-input { flex-basis: 100%; }
    .coupon-btn { flex: 1; min-width: 108px; }
    .merge-grid { grid-template-columns: 1fr; }
    .merge-title-row { align-items: flex-start; flex-direction: column; }
}
</style>
