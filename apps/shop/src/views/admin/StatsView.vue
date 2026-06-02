<template>
  <div>
    <div class="stats-grid">
        <div class="stat-card green">
            <div class="stat-label">周期销售额</div>
            <div class="stat-value">{{ formatCurrency(trendSummary.salesAmount) }}</div>
        </div>
        <div class="stat-card blue">
            <div class="stat-label">周期订单量</div>
            <div class="stat-value">{{ trendSummary.orderCount }} <span class="stat-unit">单</span></div>
        </div>
        <div class="stat-card yellow">
            <div class="stat-label">客单价</div>
            <div class="stat-value">{{ formatCurrency(trendSummary.avgOrderAmount) }}</div>
        </div>
        <div class="stat-card purple">
            <div class="stat-label">整体转化率</div>
            <div class="stat-value">{{ formatPercent(conversion.overallConversion) }}</div>
        </div>
    </div>

    <div class="panel">
        <div class="toolbar">
            <div class="filter-group">
                <button
                    v-for="option in trendPeriodOptions"
                    :key="option.value"
                    class="filter-btn"
                    :class="{ active: trendPeriod === option.value }"
                    @click="setTrendPeriod(option.value)"
                >
                    {{ option.label }}
                </button>
            </div>

            <div v-if="trendPeriod === 'custom'" class="date-picker-group">
                <input v-model="customRange.startDate" type="date" class="date-input">
                <span>至</span>
                <input v-model="customRange.endDate" type="date" class="date-input">
                <button class="admin-btn btn-blue" @click="loadTrendAndConversion">查询</button>
            </div>
            <div v-else class="text-sub">{{ trendRangeText }}</div>
        </div>
        <div class="panel-body">
            <div class="chart-header">
                <h3 style="margin: 0; color: #374151;">销售趋势（销售额 + 订单量）</h3>
                <span class="text-sub">
                    销售口径：已支付、待发货、已发货、已完成
                    <span v-if="usingMockTrend" style="margin-left: 0.5rem; color: #b45309;">(当前为模拟数据)</span>
                </span>
            </div>

            <div v-if="trendLoading" class="empty-tip">趋势数据加载中...</div>
            <div v-else-if="trendPoints.length === 0" class="empty-tip">
                <div>当前周期暂无销售数据</div>
                <button class="admin-btn btn-outline" style="margin-top: 0.75rem;" @click="useMockTrendData">
                    加载模拟趋势数据
                </button>
            </div>
            <div v-else class="chart-wrap">
                <canvas ref="trendCanvasRef"></canvas>
            </div>
        </div>
    </div>

    <div class="panel">
        <div class="toolbar">
            <div style="font-weight: bold; color: #374151;">商品销售报表</div>
            <div class="filter-group">
                <button
                    v-for="option in productPeriodOptions"
                    :key="option.value"
                    class="filter-btn"
                    :class="{ active: productPeriod === option.value }"
                    @click="setProductPeriod(option.value)"
                >
                    {{ option.label }}
                </button>
            </div>
        </div>
        <div class="panel-body">
            <div class="text-sub" style="margin-bottom: 1rem;">{{ productRangeText }}</div>
            <div v-if="productLoading" class="empty-tip">商品报表加载中...</div>
            <div v-else-if="productItems.length === 0" class="empty-tip">当前周期暂无商品销量数据</div>
            <div v-else class="table-container">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th style="width: 6rem;">排名</th>
                            <th>商品</th>
                            <th style="text-align: right;">销量</th>
                            <th style="text-align: right;">销售额</th>
                            <th style="text-align: right;">销量占比</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="(item, idx) in productItems" :key="item.productId || item.name">
                            <td>
                                <span :class="['rank-badge', idx < 3 ? `rank-${idx + 1}` : 'rank-other']">
                                    {{ idx + 1 }}
                                </span>
                            </td>
                            <td>{{ item.name }}</td>
                            <td style="text-align: right; font-weight: 600;">{{ item.quantity }}</td>
                            <td style="text-align: right;">{{ formatCurrency(item.amount) }}</td>
                            <td style="text-align: right;">{{ formatPercent(getProductShare(item.quantity)) }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>

    <div class="panel">
        <div class="panel-body">
            <h3 style="margin: 0 0 0.5rem 0; color: #374151;">转化率漏斗</h3>
            <div class="text-sub" style="margin-bottom: 1rem;">{{ conversionRangeText }}</div>
            <div v-if="trendLoading" class="empty-tip">转化数据加载中...</div>
            <div v-else class="table-container">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>环节</th>
                            <th style="text-align: right;">访问人数(UV)</th>
                            <th style="text-align: right;">环节转化率</th>
                            <th>转化进度</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="step in conversionSteps" :key="step.key">
                            <td>{{ step.label }}</td>
                            <td style="text-align: right;">{{ step.visitors }}</td>
                            <td style="text-align: right;">{{ formatPercent(step.conversionRate) }}</td>
                            <td style="min-width: 180px;">
                                <div class="progress-bar-bg">
                                    <div
                                        class="progress-bar-fill"
                                        :style="{ width: `${Math.max(0, Math.min(100, step.conversionRate || 0))}%` }"
                                    ></div>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import Chart from 'chart.js/auto'
import { buildAdminAuthHeaders, clearAdminToken } from '@/utils/adminAuth'
import { apiBasePath, resolveApiPath, resolveAppPath } from '@/utils/runtimePaths'

const trendPeriodOptions = [
    { value: '7', label: '近7天' },
    { value: '30', label: '近30天' },
    { value: '90', label: '近90天' },
    { value: 'all', label: '全部' },
    { value: 'custom', label: '自定义' }
]

const productPeriodOptions = [
    { value: 'week', label: '本周销量' },
    { value: 'month', label: '本月销量' },
    { value: 'all', label: '全时期销量' }
]

const trendPeriod = ref('7')
const trendLoading = ref(false)
const usingMockTrend = ref(false)
const customRange = reactive({ startDate: '', endDate: '' })
const trendResult = ref({
    points: [],
    summary: { salesAmount: 0, orderCount: 0, avgOrderAmount: 0 },
    startDate: null,
    endDate: null
})

const productPeriod = ref('week')
const productLoading = ref(false)
const productResult = ref({ items: [], totalQuantity: 0, totalAmount: 0, startDate: null, endDate: null })

const conversion = ref({ steps: [], overallConversion: 0, startDate: null, endDate: null })
const trendCanvasRef = ref(null)
let trendChart = null

const trendPoints = computed(() => trendResult.value.points || [])
const trendSummary = computed(() => trendResult.value.summary || { salesAmount: 0, orderCount: 0, avgOrderAmount: 0 })
const productItems = computed(() => productResult.value.items || [])
const conversionSteps = computed(() => conversion.value.steps || [])

const formatCurrency = (val) => `¥${Number(val || 0).toLocaleString('zh-CN')}`
const formatPercent = (val) => `${Number(val || 0).toFixed(2)}%`
const formatRangeText = (startDate, endDate) => (startDate && endDate ? `${startDate} ~ ${endDate}` : '全时间范围')

const trendRangeText = computed(() => `统计区间：${formatRangeText(trendResult.value.startDate, trendResult.value.endDate)}`)
const productRangeText = computed(() => `统计区间：${formatRangeText(productResult.value.startDate, productResult.value.endDate)}`)
const conversionRangeText = computed(() => `统计区间：${formatRangeText(conversion.value.startDate, conversion.value.endDate)}`)

const toDateInput = (date) => {
    const y = date.getFullYear()
    const m = String(date.getMonth() + 1).padStart(2, '0')
    const d = String(date.getDate()).padStart(2, '0')
    return `${y}-${m}-${d}`
}

const resetCustomRangeToLast30Days = () => {
    const end = new Date()
    end.setHours(0, 0, 0, 0)
    const start = new Date(end)
    start.setDate(start.getDate() - 29)
    customRange.startDate = toDateInput(start)
    customRange.endDate = toDateInput(end)
}

const buildTrendQuery = () => {
    const params = new URLSearchParams()
    params.set('period', trendPeriod.value)
    if (trendPeriod.value === 'custom') {
        params.set('startDate', customRange.startDate)
        params.set('endDate', customRange.endDate)
    }
    return params.toString()
}

const getDateWithOffset = (offset) => {
    const date = new Date()
    date.setHours(0, 0, 0, 0)
    date.setDate(date.getDate() + offset)
    return date
}

const buildMockTrendPoints = (days = 7) => {
    const points = []
    for (let i = 0; i < days; i++) {
        const day = getDateWithOffset(-days + 1 + i)
        const base = 600 + i * 110
        const wave = (i % 3) * 140
        const orderCount = 6 + (i % 4) * 2 + i
        points.push({
            date: toDateInput(day),
            salesAmount: base + wave,
            orderCount
        })
    }
    return points
}

const useMockTrendData = async () => {
    const points = buildMockTrendPoints(7)
    const summary = points.reduce(
        (acc, item) => {
            acc.salesAmount += item.salesAmount
            acc.orderCount += item.orderCount
            return acc
        },
        { salesAmount: 0, orderCount: 0 }
    )
    summary.avgOrderAmount = summary.orderCount > 0 ? Number((summary.salesAmount / summary.orderCount).toFixed(2)) : 0

    trendResult.value = {
        period: 'mock',
        startDate: points[0]?.date || null,
        endDate: points[points.length - 1]?.date || null,
        points,
        summary
    }
    usingMockTrend.value = true
    await renderTrendChart()
}

const readJsonResponse = async (res, fallbackMessage) => {
    const contentType = String(res.headers.get('content-type') || '')
    const text = await res.text()

    if (res.status === 401) {
        clearAdminToken()
        if (typeof window !== 'undefined') window.location.href = resolveAppPath('admin/login')
        throw new Error('登录已失效，请重新登录')
    }

    if (contentType.includes('text/html') || text.trim().startsWith('<!DOCTYPE') || text.trim().startsWith('<html')) {
        throw new Error(`统计接口返回了 HTML 页面，请确认后端已启动并已重启到最新代码（包含 ${apiBasePath}/admin/stats/* 接口）`)
    }

    let data
    try {
        data = text ? JSON.parse(text) : {}
    } catch {
        throw new Error(fallbackMessage)
    }

    if (!res.ok) {
        throw new Error(data?.error || fallbackMessage)
    }
    return data
}

const renderTrendChart = async () => {
    await nextTick()
    if (!trendCanvasRef.value) return

    if (trendChart) {
        trendChart.destroy()
        trendChart = null
    }

    const labels = trendPoints.value.map((item) => item.date.slice(5))
    const salesData = trendPoints.value.map((item) => item.salesAmount)
    const orderData = trendPoints.value.map((item) => item.orderCount)

    trendChart = new Chart(trendCanvasRef.value, {
        type: 'line',
        data: {
            labels,
            datasets: [
                {
                    type: 'line',
                    label: '销售额',
                    data: salesData,
                    borderColor: '#2563eb',
                    backgroundColor: 'rgba(37, 99, 235, 0.15)',
                    yAxisID: 'y',
                    tension: 0.3,
                    fill: true
                },
                {
                    type: 'bar',
                    label: '订单量',
                    data: orderData,
                    yAxisID: 'y1',
                    backgroundColor: 'rgba(16, 185, 129, 0.45)',
                    borderColor: '#10b981',
                    borderWidth: 1
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            interaction: { mode: 'index', intersect: false },
            scales: {
                y: {
                    beginAtZero: true,
                    position: 'left',
                    title: { display: true, text: '销售额(元)' }
                },
                y1: {
                    beginAtZero: true,
                    position: 'right',
                    grid: { drawOnChartArea: false },
                    title: { display: true, text: '订单量(单)' }
                }
            }
        }
    })
}

const loadTrendAndConversion = async () => {
    if (trendPeriod.value === 'custom') {
        if (!customRange.startDate || !customRange.endDate) {
            alert('请选择完整的开始和结束日期')
            return
        }
        if (customRange.startDate > customRange.endDate) {
            alert('开始日期不能晚于结束日期')
            return
        }
    }

    trendLoading.value = true
    let loaded = false
    try {
        const query = buildTrendQuery()
        const [trendRes, conversionRes] = await Promise.all([
            fetch(`${resolveApiPath('/admin/stats/sales-trend')}?${query}`, { headers: buildAdminAuthHeaders() }),
            fetch(`${resolveApiPath('/admin/stats/conversion')}?${query}`, { headers: buildAdminAuthHeaders() })
        ])
        const trendJson = await readJsonResponse(trendRes, '销售趋势加载失败')
        const conversionJson = await readJsonResponse(conversionRes, '转化率加载失败')

        trendResult.value = trendJson
        usingMockTrend.value = false
        conversion.value = conversionJson
        loaded = true
    } catch (err) {
        console.error(err)
        alert(err.message || '统计数据加载失败')
    } finally {
        trendLoading.value = false
    }

    if (!loaded) return
    await nextTick()
    if (trendPoints.value.length > 0) await renderTrendChart()
    else if (trendChart) {
        trendChart.destroy()
        trendChart = null
    }
}

const loadProductReport = async () => {
    productLoading.value = true
    try {
        const res = await fetch(`${resolveApiPath('/admin/stats/product-sales')}?period=${productPeriod.value}`, {
            headers: buildAdminAuthHeaders()
        })
        const json = await readJsonResponse(res, '商品报表加载失败')
        productResult.value = json
    } catch (err) {
        console.error(err)
        alert(err.message || '商品报表加载失败')
    } finally {
        productLoading.value = false
    }
}

const setTrendPeriod = async (period) => {
    trendPeriod.value = period
    usingMockTrend.value = false
    if (period === 'custom') {
        if (!customRange.startDate || !customRange.endDate) resetCustomRangeToLast30Days()
        return
    }
    await loadTrendAndConversion()
}

const setProductPeriod = async (period) => {
    productPeriod.value = period
    await loadProductReport()
}

const getProductShare = (quantity) => {
    const total = Number(productResult.value.totalQuantity || 0)
    if (total <= 0) return 0
    return (Number(quantity || 0) / total) * 100
}

onMounted(async () => {
    await Promise.all([loadTrendAndConversion(), loadProductReport()])
})

onBeforeUnmount(() => {
    if (trendChart) {
        trendChart.destroy()
        trendChart = null
    }
})
</script>

<style scoped>
.chart-wrap {
    height: 340px;
    position: relative;
}

.empty-tip {
    padding: 2rem 1rem;
    text-align: center;
    color: #9ca3af;
    background: #f9fafb;
    border-radius: 8px;
}

@media (max-width: 1023px) {
    .chart-wrap {
        height: 300px;
    }
}

@media (max-width: 639px) {
    .chart-wrap {
        height: 240px;
    }

    .date-picker-group {
        width: 100%;
        flex-wrap: wrap;
        justify-content: space-between;
        gap: 0.4rem;
    }

    .date-input {
        flex: 1;
        min-width: 0;
    }

    .date-picker-group .admin-btn {
        width: 100%;
        margin-top: 0.2rem;
    }
}
</style>
