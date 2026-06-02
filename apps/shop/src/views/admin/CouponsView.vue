<template>
  <div class="panel">
    <div class="toolbar">
      <h3 style="font-weight: bold; color: #374151; margin: 0;">优惠券管理</h3>
      <div class="toolbar-top-actions">
        <button class="admin-btn btn-outline" @click="loadCoupons">刷新</button>
        <button class="admin-btn btn-green" :disabled="selectedCount === 0" @click="exportSelectedCodes">导出所选券码</button>
      </div>
    </div>

    <div class="panel-body">
      <div class="create-box">
        <h4 class="box-title">批量创建优惠券</h4>
        <div class="create-grid">
          <div>
            <label class="form-label">名称</label>
            <input v-model.trim="createForm.name" class="form-input" placeholder="如：春日活动券">
          </div>
          <div>
            <label class="form-label">券码前缀</label>
            <input v-model.trim="createForm.prefix" class="form-input" placeholder="默认 CPN">
          </div>
          <div>
            <label class="form-label">批次号（可选）</label>
            <input v-model.trim="createForm.batchNo" class="form-input" placeholder="不填自动生成">
          </div>
          <div>
            <label class="form-label">数量</label>
            <input v-model.number="createForm.quantity" type="number" class="form-input" min="1" max="2000">
          </div>
          <div>
            <label class="form-label">使用门槛 (¥)</label>
            <input v-model.number="createForm.minSpend" type="number" class="form-input" min="0" step="0.01">
          </div>
          <div>
            <label class="form-label">优惠类型</label>
            <select v-model="createForm.discountType" class="form-select">
              <option value="amount">立减金额</option>
              <option value="percent">折扣比例</option>
            </select>
          </div>
          <div>
            <label class="form-label">{{ createForm.discountType === 'amount' ? '优惠金额 (¥)' : '折扣比例 (%)' }}</label>
            <input v-model.number="createForm.discountValue" type="number" class="form-input" min="0.01" step="0.01">
          </div>
          <div>
            <label class="form-label">最高优惠 (¥, 折扣券可选)</label>
            <input
              v-model.number="createForm.maxDiscount"
              type="number"
              class="form-input"
              min="0"
              step="0.01"
              :disabled="createForm.discountType !== 'percent'"
              placeholder="不填则不限制"
            >
          </div>
          <div>
            <label class="form-label">过期时间（可选）</label>
            <input v-model="createForm.expiresAt" type="datetime-local" class="form-input">
          </div>
        </div>
        <div style="margin-top: 0.5rem;">
          <button class="admin-btn btn-blue" :disabled="creating" @click="createBatch">
            {{ creating ? '创建中...' : '批量创建优惠券' }}
          </button>
        </div>
        <div v-if="latestBatchCodes.length > 0" class="latest-codes">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <strong>最近创建券码 ({{ latestBatchCodes.length }} 张)</strong>
            <button class="admin-btn btn-outline" @click="copyLatestCodes">复制全部</button>
          </div>
          <textarea :value="latestBatchCodes.join('\n')" readonly class="codes-textarea"></textarea>
        </div>
      </div>

      <div class="filter-row">
        <select v-model="filters.status" class="form-select" style="width: 120px; margin-bottom: 0;">
          <option value="all">全部状态</option>
          <option value="1">可用</option>
          <option value="0">已禁用</option>
          <option value="2">已使用</option>
        </select>
        <input v-model.trim="filters.batchNo" class="form-input" style="width: 180px; margin-bottom: 0;" placeholder="按批次号筛选">
        <input v-model.trim="filters.keyword" class="form-input" style="width: 220px; margin-bottom: 0;" placeholder="按券码或名称搜索">
        <select v-model="filters.sortBy" class="form-select" style="width: 120px; margin-bottom: 0;">
          <option value="created_at">按创建时间</option>
          <option value="expiresAt">按有效期</option>
          <option value="minSpend">按门槛</option>
          <option value="status">按状态</option>
          <option value="id">按ID</option>
        </select>
        <select v-model="filters.sortDir" class="form-select" style="width: 90px; margin-bottom: 0;">
          <option value="desc">降序</option>
          <option value="asc">升序</option>
        </select>
        <select v-model.number="filters.pageSize" class="form-select" style="width: 90px; margin-bottom: 0;">
          <option :value="10">10/页</option>
          <option :value="20">20/页</option>
          <option :value="50">50/页</option>
        </select>
        <button class="admin-btn btn-blue" @click="searchCoupons">查询</button>
      </div>

      <div v-if="loading" class="text-sub">加载中...</div>
      <div v-else class="table-container">
        <table class="data-table">
          <thead>
            <tr>
              <th class="checkbox-cell">
                <input ref="selectAllRef" type="checkbox" :checked="allSelected" :disabled="coupons.length === 0" @change="toggleSelectAll($event.target.checked)">
              </th>
              <th>券码</th>
              <th>批次</th>
              <th>规则</th>
              <th>状态</th>
              <th>有效期</th>
              <th>使用订单</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="coupon in coupons" :key="coupon.id">
              <td class="checkbox-cell">
                <input type="checkbox" :checked="selectedIds.has(coupon.id)" @change="toggleSelectOne(coupon.id, $event.target.checked)">
              </td>
              <td>
                <div style="font-family: monospace; font-weight: bold;">{{ coupon.code }}</div>
                <div class="text-sub">{{ coupon.name }}</div>
              </td>
              <td>{{ coupon.batchNo || '-' }}</td>
              <td>
                <div>门槛: ¥{{ coupon.minSpend }}</div>
                <div>力度: {{ coupon.benefitText }}</div>
              </td>
              <td>
                <span :class="['status-badge', statusClass(coupon)]">{{ statusText(coupon) }}</span>
              </td>
              <td>{{ coupon.expiresAt ? formatDate(coupon.expiresAt) : '长期有效' }}</td>
              <td>{{ coupon.usedOrderId || '-' }}</td>
              <td>
                <div style="display: flex; gap: 0.5rem; align-items: center;">
                  <button
                    v-if="coupon.status !== 2"
                    class="admin-btn btn-outline"
                    style="font-size: 0.75rem;"
                    @click="toggleStatus(coupon)"
                  >
                    {{ coupon.status === 1 ? '禁用' : '启用' }}
                  </button>
                  <button
                    v-if="coupon.status !== 2"
                    class="admin-btn btn-outline"
                    style="font-size: 0.75rem; color: #ef4444; border-color: #fca5a5;"
                    @click="removeCoupon(coupon)"
                  >
                    删除
                  </button>
                  <span v-else class="text-sub">已使用不可删</span>
                </div>
              </td>
            </tr>
            <tr v-if="coupons.length === 0">
              <td colspan="8" style="text-align: center; color: #9ca3af;">暂无优惠券数据</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="toolbar pagination-row">
        <span class="text-sub">共 {{ couponsMeta.total }} 张，第 {{ couponsMeta.page }} / {{ couponsMeta.totalPages }} 页</span>
        <button class="admin-btn btn-outline" :disabled="couponsMeta.page <= 1" @click="goPrevPage">上一页</button>
        <button class="admin-btn btn-outline" :disabled="couponsMeta.page >= couponsMeta.totalPages" @click="goNextPage">下一页</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useShopStore } from '@/stores/shopStore'

const store = useShopStore()
const loading = ref(false)
const creating = ref(false)
const selectAllRef = ref(null)
const selectedIds = ref(new Set())
const latestBatchCodes = ref([])

const coupons = computed(() => store.state.adminCoupons)
const couponsMeta = computed(() => store.state.adminCouponsMeta || { page: 1, pageSize: 20, total: 0, totalPages: 1 })
const selectedCount = computed(() => [...selectedIds.value].length)
const allSelected = computed(() => coupons.value.length > 0 && selectedCount.value === coupons.value.length)
const partiallySelected = computed(() => selectedCount.value > 0 && selectedCount.value < coupons.value.length)

const filters = reactive({
  status: 'all',
  batchNo: '',
  keyword: '',
  sortBy: 'created_at',
  sortDir: 'desc',
  pageSize: 20
})

const createForm = reactive({
  name: '活动优惠券',
  prefix: 'CPN',
  batchNo: '',
  quantity: 20,
  minSpend: 0,
  discountType: 'amount',
  discountValue: 10,
  maxDiscount: '',
  expiresAt: ''
})

const buildQuery = (page = 1) => ({
  status: filters.status,
  batchNo: filters.batchNo,
  keyword: filters.keyword,
  sortBy: filters.sortBy,
  sortDir: filters.sortDir,
  page,
  pageSize: filters.pageSize
})

const loadCoupons = async (page = 1) => {
  loading.value = true
  await store.fetchAdminCoupons(buildQuery(page))
  loading.value = false
}

const createBatch = async () => {
  if (!Number.isInteger(Number(createForm.quantity)) || Number(createForm.quantity) <= 0) {
    store.showNotification('优惠券数量必须是正整数')
    return
  }

  creating.value = true
  const result = await store.createCouponBatch({
    name: createForm.name,
    prefix: createForm.prefix,
    batchNo: createForm.batchNo,
    quantity: Number(createForm.quantity),
    minSpend: Number(createForm.minSpend || 0),
    discountType: createForm.discountType,
    discountValue: Number(createForm.discountValue),
    maxDiscount: createForm.discountType === 'percent' ? createForm.maxDiscount : '',
    expiresAt: createForm.expiresAt
  })
  creating.value = false

  if (!result) return
  latestBatchCodes.value = Array.isArray(result.codes) ? result.codes : []
  createForm.batchNo = result.batchNo || createForm.batchNo
  await loadCoupons(1)
}

const toggleSelectOne = (id, checked) => {
  const next = new Set(selectedIds.value)
  if (checked) next.add(id)
  else next.delete(id)
  selectedIds.value = next
}

const toggleSelectAll = (checked) => {
  if (!checked) {
    selectedIds.value = new Set()
    return
  }
  selectedIds.value = new Set(coupons.value.map((item) => item.id))
}

const formatDate = (value) => new Date(value).toLocaleString()

const statusText = (coupon) => {
  if (coupon.status === 2) return '已使用'
  if (coupon.isExpired) return '已过期'
  return coupon.status === 1 ? '可用' : '已禁用'
}

const statusClass = (coupon) => {
  if (coupon.status === 2) return 'status-3'
  if (coupon.isExpired) return 'status-0'
  return coupon.status === 1 ? 'status-2' : 'status-1'
}

const toggleStatus = async (coupon) => {
  const nextStatus = coupon.status === 1 ? 0 : 1
  const success = await store.updateCouponStatus(coupon.id, nextStatus)
  if (success) await loadCoupons(couponsMeta.value.page)
}

const removeCoupon = async (coupon) => {
  if (!confirm(`确定删除优惠券 ${coupon.code} 吗？`)) return
  const success = await store.deleteCoupon(coupon.id)
  if (!success) return
  const next = new Set(selectedIds.value)
  next.delete(coupon.id)
  selectedIds.value = next
  await loadCoupons(couponsMeta.value.page)
}

const searchCoupons = () => {
  loadCoupons(1)
}

const goPrevPage = () => {
  if (couponsMeta.value.page <= 1) return
  loadCoupons(couponsMeta.value.page - 1)
}

const goNextPage = () => {
  if (couponsMeta.value.page >= couponsMeta.value.totalPages) return
  loadCoupons(couponsMeta.value.page + 1)
}

const exportSelectedCodes = () => {
  const selected = coupons.value.filter((item) => selectedIds.value.has(item.id))
  if (selected.length === 0) {
    store.showNotification('请先选择优惠券')
    return
  }

  const headers = ['券码', '名称', '批次号', '门槛', '优惠规则', '状态', '有效期', '使用订单']
  const rows = selected.map((item) => [
    item.code,
    item.name,
    item.batchNo || '',
    item.minSpend,
    item.benefitText,
    statusText(item),
    item.expiresAt ? formatDate(item.expiresAt) : '',
    item.usedOrderId || ''
  ])

  const escape = (value) => {
    const text = String(value ?? '').replace(/"/g, '""')
    return /[",\n]/.test(text) ? `"${text}"` : text
  }

  const csv = [headers.map(escape).join(','), ...rows.map((row) => row.map(escape).join(','))].join('\n')
  const blob = new Blob([`\uFEFF${csv}`], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `coupons-${Date.now()}.csv`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

const copyLatestCodes = async () => {
  if (latestBatchCodes.value.length === 0) return
  await navigator.clipboard.writeText(latestBatchCodes.value.join('\n'))
  store.showNotification('券码已复制到剪贴板')
}

watch([coupons, partiallySelected], () => {
  if (selectAllRef.value) {
    selectAllRef.value.indeterminate = partiallySelected.value
  }
}, { immediate: true })

watch(coupons, (newList) => {
  const valid = new Set(newList.map((item) => item.id))
  const next = new Set()
  selectedIds.value.forEach((id) => {
    if (valid.has(id)) next.add(id)
  })
  selectedIds.value = next
})

onMounted(loadCoupons)
</script>

<style scoped>
.create-box {
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1rem;
  background: #fafafa;
}

.box-title {
  margin: 0 0 0.75rem 0;
  color: #374151;
}

.toolbar-top-actions {
  display: flex;
  gap: 0.5rem;
}

.create-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 0.75rem;
}

@media (min-width: 1100px) {
  .create-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

.latest-codes {
  margin-top: 0.75rem;
  padding: 0.75rem;
  border: 1px dashed #9ca3af;
  border-radius: 8px;
  background: #fff;
}

.codes-textarea {
  width: 100%;
  min-height: 130px;
  margin-top: 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-family: monospace;
  font-size: 12px;
  padding: 0.5rem;
}

.filter-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  flex-wrap: wrap;
  margin-bottom: 0.75rem;
}

.filter-row .form-select,
.filter-row .form-input {
  margin-bottom: 0;
}

.pagination-row {
  justify-content: flex-end;
  gap: 0.5rem;
}

.checkbox-cell {
  width: 42px;
  text-align: center;
}

button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

@media (max-width: 1023px) {
  .toolbar-top-actions {
    width: 100%;
    flex-wrap: wrap;
  }

  .pagination-row {
    justify-content: space-between;
    flex-wrap: wrap;
  }
}

@media (max-width: 639px) {
  .toolbar-top-actions .admin-btn {
    flex: 1;
  }

  .filter-row {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .filter-row .form-select,
  .filter-row .form-input,
  .filter-row .admin-btn {
    width: 100% !important;
  }

  .filter-row .admin-btn {
    grid-column: 1 / -1;
  }

  .latest-codes > div {
    flex-wrap: wrap;
    gap: 0.5rem;
  }
}
</style>
