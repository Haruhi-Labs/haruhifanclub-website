<template>
    <div class="query-box">
        <h2 class="query-title">订单查询</h2>
        <div class="query-form-row" style="margin-bottom: 0.75rem;">
            <input v-model="queryId" type="text" placeholder="请输入订单号 (如: SOS...)" class="input-field" style="flex-grow: 1;">
            <input
                v-model="phoneLast4"
                type="text"
                placeholder="手机号后四位"
                class="input-field"
                style="width: 9rem;"
                maxlength="4"
                inputmode="numeric"
            >
            <button @click="query" class="market-btn primary-action" style="padding: 0 1.5rem;">查询</button>
        </div>
        <p style="margin: 0 0 1.5rem 0; color: #9ca3af; font-size: 0.8rem;">
            为保护隐私，查询需同时验证下单手机号后四位
        </p>
        <p style="margin: 0 0 1.5rem 0; color: #9ca3af; font-size: 0.8rem;">
            查询到订单后，也可进行收货信息的修改。
        </p>

        <div v-if="error" style="background: #fef2f2; padding: 1.5rem; border-radius: var(--radius-md); text-align: center; color: #dc2626;">
            <i class="fa fa-exclamation-circle" style="font-size: 1.5rem; margin-bottom: 0.5rem;"></i>
            <p>{{ error }}</p>
        </div>

        <div v-else-if="order" class="query-result">
            <div class="result-header">
                <span class="result-order-id">订单: {{ order.id }}</span>
                <span :style="statusStyle">{{ statusText }}</span>
            </div>
            <div v-if="order.mergeMeta" class="merge-meta-card">
                <div class="merge-meta-title">合并订单信息</div>
                <div class="merge-meta-row">
                    <span>合并次数</span>
                    <span>{{ mergeCount }}</span>
                </div>
                <div class="merge-meta-subtitle">构成订单（旧单号 / 原金额）</div>
                <div v-for="part in mergeParts" :key="part.orderId" class="merge-meta-row">
                    <span>{{ part.orderId }}</span>
                    <span>¥{{ part.amount }}</span>
                </div>
                <div v-if="latestMergeHistory" class="merge-meta-subtitle">最近一次合并</div>
                <div v-if="latestMergeHistory" class="merge-meta-row">
                    <span>旧单号</span>
                    <span>{{ latestMergeHistory.sourceOrderId }}</span>
                </div>
                <div v-if="latestMergeHistory" class="merge-meta-row">
                    <span>新加订单号</span>
                    <span>{{ latestMergeHistory.appendedOrderId }}</span>
                </div>
                <div v-if="latestMergeHistory" class="merge-meta-row">
                    <span>本次合并金额</span>
                    <span>¥{{ latestMergeHistory.sourceAmount }} + ¥{{ latestMergeHistory.appendedAmount }}</span>
                </div>
                <div class="merge-meta-row merge-meta-total">
                    <span>合并后总金额</span>
                    <span>¥{{ order.mergeMeta.mergedAmount || order.total }}</span>
                </div>
                <div v-if="mergeShippingAdjustment < 0" class="merge-meta-row">
                    <span>运费返还</span>
                    <span style="color: #16a34a;">-¥{{ mergeShippingRefund }}</span>
                </div>
                <div v-else-if="mergeShippingAdjustment > 0" class="merge-meta-row">
                    <span>追加运费</span>
                    <span style="color: #dc2626;">+¥{{ mergeShippingExtra }}</span>
                </div>
            </div>
            <div class="result-body">
                <div style="width: 100%; text-align: left;">
                    <!-- 收件信息 -->
                    <div style="padding: 0.75rem; background: #f9fafb; border-radius: 8px; margin-bottom: 0.75rem; font-size: 0.875rem; color: #374151;">
                        <div class="contact-header-row">
                            <strong style="font-size: 0.82rem; color: #6b7280;">收货信息</strong>
                            <button
                                v-if="canEditContact && !editingContact"
                                class="market-btn btn-ghost contact-edit-btn"
                                @click="openContactEditor"
                            >
                                修改
                            </button>
                        </div>
                        <div style="margin-bottom: 0.35rem;">
                            <i class="fa fa-user" style="width: 1rem; color: #9ca3af;"></i>
                            <strong>{{ order.contact.name }}</strong>
                            <span style="margin-left: 0.75rem; color: #6b7280;">{{ order.contact.phone }}</span>
                        </div>
                        <div v-if="order.contact.email" style="margin-bottom: 0.35rem;">
                            <i class="fa fa-envelope" style="width: 1rem; color: #9ca3af;"></i>
                            <span style="color: #6b7280;">{{ order.contact.email }}</span>
                        </div>
                        <div>
                            <i class="fa fa-map-marker-alt" style="width: 1rem; color: #9ca3af;"></i>
                            <span style="color: #6b7280;">{{ order.contact.province }}{{ order.contact.city }}{{ order.contact.district }} {{ order.contact.addressDetail }}</span>
                        </div>
                    </div>

                    <div v-if="editingContact" class="contact-edit-card">
                        <div class="contact-edit-grid">
                            <input v-model.trim="contactForm.name" type="text" class="input-field" placeholder="收货人姓名">
                            <input v-model.trim="contactForm.phone" type="tel" class="input-field" placeholder="手机号" maxlength="11">
                            <input v-model.trim="contactForm.email" type="email" class="input-field col-span-2" placeholder="邮箱">
                            <input v-model.trim="contactForm.province" type="text" class="input-field" placeholder="省">
                            <input v-model.trim="contactForm.city" type="text" class="input-field" placeholder="市">
                            <input v-model.trim="contactForm.district" type="text" class="input-field col-span-2" placeholder="区/县">
                            <input v-model.trim="contactForm.addressDetail" type="text" class="input-field col-span-2" placeholder="详细地址">
                        </div>
                        <p v-if="contactEditError" class="contact-edit-error">{{ contactEditError }}</p>
                        <div class="contact-edit-actions">
                            <button class="market-btn btn-ghost" @click="cancelContactEditor">取消</button>
                            <button class="market-btn primary-action" :disabled="savingContact" @click="saveContact">
                                {{ savingContact ? '保存中...' : '保存收货信息' }}
                            </button>
                        </div>
                        <p style="margin: 0.6rem 0 0; font-size: 0.75rem; color: #9ca3af;">
                            仅未发货订单可自助修改，已发货订单请联系管理员处理。
                        </p>
                    </div>

                    <!-- 商品明细 -->
                    <div v-for="item in order.items" :key="item.id" class="query-item-row">
                        <span class="query-item-name">{{ item.name }} x{{ item.quantity }}</span>
                        <span style="color: #6b7280;">&yen;{{ item.price * item.quantity }}</span>
                    </div>
                    <div class="query-total-row">
                        <span>{{ order.mergeMeta ? '本次追加应付' : '合计（含运费）' }}</span>
                        <span style="color: #dc2626;">&yen;{{ order.mergeMeta ? mergePayableNow : order.total }}</span>
                    </div>
                    <div v-if="order.mergeMeta" class="query-subtotal-row">
                        <span>旧订单金额</span>
                        <span>&yen;{{ order.mergeMeta.sourceAmount }}</span>
                    </div>
                    <div v-if="order.mergeMeta" class="query-subtotal-row">
                        <span>合并总金额</span>
                        <span>&yen;{{ order.mergeMeta.mergedAmount || order.total }}</span>
                    </div>

                    <!-- 下单时间 -->
                    <div style="font-size: 0.8rem; color: #9ca3af; padding-top: 0.25rem;">
                        <i class="fa fa-clock"></i> 下单时间: {{ new Date(order.created_at).toLocaleString() }}
                    </div>

                    <!-- 物流信息 -->
                    <div v-if="order.status === 3 || order.status === 4" style="margin-top: 0.75rem; padding: 0.75rem; border-radius: 8px;" :style="order.trackingNo ? { background: '#eff6ff' } : { background: '#fefce8' }">
                        <p v-if="order.trackingNo" style="color: #1d4ed8; font-size: 0.875rem;">
                            <i class="fa fa-truck"></i>
                            {{ order.trackingCompany }}: {{ order.trackingNo }}
                        </p>
                        <p v-else style="color: #92400e; font-size: 0.875rem; line-height: 1.5;">
                            <i class="fa fa-info-circle"></i>
                            未回填快递单号，请在快递小程序中查询，如有问题可联系我们
                        </p>
                    </div>
                </div>
            </div>
        </div>

        <div v-else style="background: #f9fafb; padding: 1.5rem; border-radius: var(--radius-md); text-align: center; color: #9ca3af;">
            <i class="fa fa-search" style="font-size: 1.5rem; margin-bottom: 0.5rem;"></i>
            <p>请输入订单号查询发货状态</p>
        </div>
    </div>
</template>

<script setup>
import { ref, computed, onMounted, reactive } from 'vue'
import { useRoute } from 'vue-router'
import { resolveApiPath } from '@/utils/runtimePaths'

const route = useRoute()
const queryId = ref('')
const phoneLast4 = ref('')
const order = ref(null)
const error = ref(null)
const editingContact = ref(false)
const savingContact = ref(false)
const contactEditError = ref('')
const contactForm = reactive({
    name: '',
    phone: '',
    email: '',
    province: '',
    city: '',
    district: '',
    addressDetail: ''
})

onMounted(() => {
    if (route.query.id) {
        queryId.value = route.query.id
    }
    if (route.query.phoneLast4) {
        phoneLast4.value = String(route.query.phoneLast4)
    }
    if (queryId.value && /^\d{4}$/.test(phoneLast4.value)) {
        query()
    }
})

const statusMap = {
    0: { text: '已取消', bg: '#fee2e2', color: '#dc2626' },
    1: { text: '待付款', bg: '#fef3c7', color: '#d97706' },
    2: { text: '待发货', bg: '#d1fae5', color: '#047857' },
    3: { text: '已发货', bg: '#dbeafe', color: '#1d4ed8' },
    4: { text: '已完成', bg: '#f3f4f6', color: '#374151' },
    5: { text: '待确认', bg: '#fce7f3', color: '#be185d' },
}

const statusText = computed(() => order.value ? (statusMap[order.value.status]?.text || '未知') : '')
const statusStyle = computed(() => {
    const s = order.value ? statusMap[order.value.status] : null
    return s ? { fontSize: '0.75rem', background: s.bg, color: s.color, padding: '2px 8px', borderRadius: '4px' } : {}
})
const mergeParts = computed(() => {
    const parts = order.value?.mergeMeta?.parts
    return Array.isArray(parts) ? parts : []
})
const latestMergeHistory = computed(() => {
    const history = order.value?.mergeMeta?.history
    if (!Array.isArray(history) || history.length === 0) return null
    return history[history.length - 1]
})
const mergeCount = computed(() => {
    const count = Number(order.value?.mergeMeta?.mergeCount)
    if (Number.isInteger(count) && count > 0) return count
    const history = order.value?.mergeMeta?.history
    if (Array.isArray(history) && history.length > 0) return history.length
    const parts = order.value?.mergeMeta?.parts
    if (Array.isArray(parts) && parts.length > 1) return parts.length - 1
    return 1
})
const mergeShippingAdjustment = computed(() => {
    const raw = Number(order.value?.mergeMeta?.shippingAdjustment)
    if (Number.isFinite(raw)) return Number(raw.toFixed(2))

    const source = Number(order.value?.mergeMeta?.sourceShippingFee) || 0
    const appended = Number(order.value?.mergeMeta?.appendedShippingFee) || 0
    const merged = Number(order.value?.mergeMeta?.mergedShippingFee) || 0
    return Number((merged - source - appended).toFixed(2))
})
const mergeShippingRefund = computed(() => Number(Math.max(0, -mergeShippingAdjustment.value).toFixed(2)))
const mergeShippingExtra = computed(() => Number(Math.max(0, mergeShippingAdjustment.value).toFixed(2)))
const mergePayableNow = computed(() => {
    const fromMeta = Number(order.value?.mergeMeta?.incrementalPayable)
    if (Number.isFinite(fromMeta)) return Number(fromMeta.toFixed(2))
    const appended = Number(order.value?.mergeMeta?.appendedAmount) || 0
    return Number(Math.max(0, appended + mergeShippingAdjustment.value).toFixed(2))
})
const canEditContact = computed(() => {
    const status = Number(order.value?.status)
    return [1, 2, 5].includes(status)
})

const fillContactForm = (contact = {}) => {
    contactForm.name = String(contact.name || '')
    contactForm.phone = String(contact.phone || '')
    contactForm.email = String(contact.email || '')
    contactForm.province = String(contact.province || '')
    contactForm.city = String(contact.city || '')
    contactForm.district = String(contact.district || '')
    contactForm.addressDetail = String(contact.addressDetail || '')
}

const openContactEditor = () => {
    if (!order.value || !canEditContact.value) return
    fillContactForm(order.value.contact || {})
    contactEditError.value = ''
    editingContact.value = true
}

const cancelContactEditor = () => {
    editingContact.value = false
    contactEditError.value = ''
}

const saveContact = async () => {
    if (!order.value || !canEditContact.value || savingContact.value) return
    contactEditError.value = ''

    if (!contactForm.name.trim()) {
        contactEditError.value = '请填写收货人姓名'
        return
    }
    if (!/^1[3-9]\d{9}$/.test(contactForm.phone.trim())) {
        contactEditError.value = '手机号格式错误'
        return
    }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(contactForm.email.trim())) {
        contactEditError.value = '邮箱格式错误'
        return
    }
    if (!contactForm.province.trim() || !contactForm.city.trim() || !contactForm.district.trim()) {
        contactEditError.value = '省市区信息不完整'
        return
    }
    if (!contactForm.addressDetail.trim()) {
        contactEditError.value = '请填写详细地址'
        return
    }

    const payload = {
        phoneLast4: phoneLast4.value.trim(),
        contact: {
            name: contactForm.name.trim(),
            phone: contactForm.phone.trim(),
            email: contactForm.email.trim(),
            province: contactForm.province.trim(),
            city: contactForm.city.trim(),
            district: contactForm.district.trim(),
            addressDetail: contactForm.addressDetail.trim()
        }
    }

    savingContact.value = true
    try {
        const res = await fetch(resolveApiPath(`/orders/${order.value.id}/contact`), {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        })
        const data = await res.json().catch(() => ({}))
        if (!res.ok) {
            contactEditError.value = data.error || '修改失败'
            return
        }
        order.value = data
        phoneLast4.value = payload.contact.phone.slice(-4)
        editingContact.value = false
    } catch (_) {
        contactEditError.value = '网络错误，请稍后重试'
    } finally {
        savingContact.value = false
    }
}

const query = async () => {
    if (!queryId.value.trim()) return
    if (!/^\d{4}$/.test(phoneLast4.value.trim())) {
        error.value = '请输入手机号后四位数字'
        order.value = null
        return
    }
    error.value = null
    order.value = null
    editingContact.value = false
    contactEditError.value = ''
    try {
        const params = new URLSearchParams({ phoneLast4: phoneLast4.value.trim() })
        const res = await fetch(`${resolveApiPath(`/orders/${queryId.value.trim()}`)}?${params.toString()}`)
        if (!res.ok) {
            const data = await res.json()
            error.value = data.error || '查询失败'
            return
        }
        order.value = await res.json()
    } catch (e) {
        error.value = '网络错误，请稍后重试'
    }
}
</script>

<style scoped>
.query-title {
    font-size: 1.5rem;
    font-weight: bold;
    text-align: center;
    color: #1f2937;
    margin-bottom: 2rem;
}

.query-form-row {
    display: flex;
    flex-direction: row;
    gap: 1rem;
}

.result-order-id {
    font-weight: bold;
    color: #374151;
}

.merge-meta-card {
    margin-bottom: 0.75rem;
    padding: 0.75rem;
    border-radius: 8px;
    background: #eff6ff;
    border: 1px solid #bfdbfe;
}

.merge-meta-title {
    font-size: 0.82rem;
    color: #1d4ed8;
    font-weight: 600;
    margin-bottom: 0.4rem;
}

.merge-meta-subtitle {
    margin: 0.5rem 0 0.35rem;
    font-size: 0.76rem;
    color: #1d4ed8;
    font-weight: 600;
}

.merge-meta-row {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    font-size: 0.82rem;
    color: #1f2937;
    margin-bottom: 0.3rem;
}

.merge-meta-total {
    padding-top: 0.25rem;
    margin-top: 0.25rem;
    border-top: 1px dashed #bfdbfe;
    font-weight: 600;
}

.query-item-row {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.5rem 0;
    border-bottom: 1px solid #f3f4f6;
}

.query-item-name {
    color: #374151;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 72%;
}

.query-total-row {
    display: flex;
    justify-content: space-between;
    padding: 0.75rem 0;
    font-weight: bold;
}

.query-subtotal-row {
    display: flex;
    justify-content: space-between;
    padding: 0.15rem 0;
    font-size: 0.78rem;
    color: #6b7280;
}

.contact-header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
}

.contact-edit-btn {
    padding: 0.2rem 0.7rem;
    font-size: 0.75rem;
}

.contact-edit-card {
    border: 1px dashed #d1d5db;
    border-radius: 8px;
    padding: 0.75rem;
    margin-bottom: 0.75rem;
    background: #fff;
}

.contact-edit-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
}

.col-span-2 {
    grid-column: span 2;
}

.contact-edit-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.6rem;
}

.contact-edit-error {
    margin: 0.6rem 0 0;
    font-size: 0.8rem;
    color: #dc2626;
}

@media (max-width: 639px) {
    .query-title {
        margin-bottom: 1.25rem;
        font-size: 1.25rem;
    }
    .query-form-row {
        flex-direction: column;
        gap: 0.75rem;
    }
    .query-form-row .input-field {
        width: 100% !important;
    }
    .query-form-row .market-btn {
        width: 100%;
        padding: 0.625rem !important;
    }
    .query-item-row {
        flex-direction: column;
        gap: 0.2rem;
    }
    .query-item-name {
        max-width: 100%;
        white-space: normal;
        word-break: break-word;
    }
    .contact-edit-grid {
        grid-template-columns: 1fr;
    }
    .col-span-2 {
        grid-column: span 1;
    }
    .contact-edit-actions {
        justify-content: stretch;
    }
    .contact-edit-actions .market-btn {
        flex: 1;
    }
}
</style>
