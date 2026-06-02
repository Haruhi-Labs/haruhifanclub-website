<template>
  <div>
    <div class="stats-grid">
        <div class="stat-card yellow">
            <div class="stat-label">待人工核销</div>
            <div class="stat-value">{{ loading ? '-' : summary.pendingVerify }} <span class="stat-unit">单</span></div>
        </div>
        <div class="stat-card blue">
            <div class="stat-label">待发货</div>
            <div class="stat-value">{{ loading ? '-' : summary.pendingShipment }} <span class="stat-unit">单</span></div>
        </div>
        <div class="stat-card green">
            <div class="stat-label">总销售额</div>
            <div class="stat-value">{{ loading ? '-' : formatCurrency(summary.totalSales) }}</div>
        </div>
        <div class="stat-card purple">
            <div class="stat-label">总订单数</div>
            <div class="stat-value">{{ loading ? '-' : summary.totalOrders }}</div>
        </div>
    </div>
    <div class="panel">
        <div class="panel-body">
            <div style="display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; margin-bottom: 1rem;">
                <h3 style="font-weight: bold; color: #374151; margin: 0;">快捷操作</h3>
                <button class="admin-btn btn-outline" :disabled="loading" @click="loadSummary">
                    {{ loading ? '刷新中...' : '刷新数据' }}
                </button>
            </div>
            <div v-if="errorMessage" class="text-sub" style="margin-bottom: 1rem; color: #b91c1c;">
                {{ errorMessage }}
            </div>
            <div style="display: flex; gap: 1rem;">
                <button class="admin-btn btn-blue" style="padding: 0.75rem 1.5rem;" @click="$router.push('/admin/orders')">
                    <i class="fa fa-check-double"></i> 去审核收款
                </button>
                <button class="admin-btn btn-green" style="padding: 0.75rem 1.5rem;">
                    <i class="fa fa-file-excel"></i> 导出今日发货单
                </button>
            </div>
        </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, reactive, ref } from 'vue'
import { buildAdminAuthHeaders, clearAdminToken } from '@/utils/adminAuth'
import { resolveApiPath, resolveAppPath } from '@/utils/runtimePaths'

const loading = ref(false)
const errorMessage = ref('')
const summary = reactive({
    pendingVerify: 0,
    pendingShipment: 0,
    totalSales: 0,
    totalOrders: 0
})

const formatCurrency = (value) => `¥${Number(value || 0).toLocaleString('zh-CN')}`

const loadSummary = async () => {
    loading.value = true
    errorMessage.value = ''

    try {
        const res = await fetch(resolveApiPath('/admin/dashboard-summary'), {
            headers: buildAdminAuthHeaders()
        })

        if (res.status === 401) {
            clearAdminToken()
            if (typeof window !== 'undefined') {
                window.location.href = resolveAppPath('admin/login')
            }
            return
        }

        const data = await res.json().catch(() => ({}))
        if (!res.ok) {
            errorMessage.value = data.error || '看板数据加载失败'
            return
        }

        summary.pendingVerify = Number(data.pendingVerify) || 0
        summary.pendingShipment = Number(data.pendingShipment) || 0
        summary.totalSales = Number(data.totalSales) || 0
        summary.totalOrders = Number(data.totalOrders) || 0
    } catch (err) {
        errorMessage.value = '看板数据加载失败'
    } finally {
        loading.value = false
    }
}

onMounted(() => {
    loadSummary()
})
</script>
