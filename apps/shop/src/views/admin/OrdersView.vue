<template>
  <div class="panel">
    <div class="toolbar">
      <div class="filter-group">
        <button
          v-for="status in statusOptions"
          :key="status.value"
          class="filter-btn"
          :class="{ active: filterStatus === status.value }"
          @click="changeFilter(status.value)"
        >
          {{ status.label }}
        </button>
        <span class="filter-separator">|</span>
        <button
          class="filter-btn"
          :class="{ active: filterItemType === 'spot' }"
          @click="changeItemTypeFilter('spot')"
        >仅现货</button>
        <button
          class="filter-btn"
          :class="{ active: filterItemType === 'presale' }"
          @click="changeItemTypeFilter('presale')"
        >仅预售</button>
      </div>
      <div class="toolbar-query">
        <input v-model.trim="keyword" class="search-input" placeholder="按订单号/收货人/手机号搜索">
        <select v-model="sortBy" class="form-select compact-select">
          <option value="created_at">按时间</option>
          <option value="total">按金额</option>
          <option value="status">按状态</option>
          <option value="id">按订单号</option>
        </select>
        <select v-model="sortDir" class="form-select compact-select">
          <option value="desc">降序</option>
          <option value="asc">升序</option>
        </select>
        <select v-model.number="pageSize" class="form-select compact-select">
          <option :value="10">10/页</option>
          <option :value="20">20/页</option>
          <option :value="50">50/页</option>
        </select>
        <button class="admin-btn btn-blue" @click="searchOrders">查询</button>
      </div>
      <div class="toolbar-actions">
        <span class="text-sub">已选 {{ selectedCount }} 单</span>
        <button class="admin-btn btn-outline" :disabled="selectedCount === 0" @click="clearSelection">清空选择</button>
        <button class="admin-btn btn-outline" @click="selectPendingUnexported" title="勾选所有尚未导出发货单的待发货订单">
          <i class="fa fa-check-square-o"></i> 未导出待发货
        </button>
        <button class="admin-btn btn-outline" @click="selectAllPending" title="勾选所有待发货订单">
          <i class="fa fa-check-square-o"></i> 全部待发货
        </button>
        <button class="admin-btn btn-green" :disabled="selectedCount === 0" @click="exportSelectedOrders">
          <i class="fa fa-download"></i> 导出所选
        </button>
        <button class="admin-btn btn-green" :disabled="selectedCount === 0" @click="exportSpotOrders" title="导出所选订单中的现货商品（预售商品不导出，纯预售订单被跳过）">
          <i class="fa fa-download"></i> 导出现货订单
        </button>
        <div class="presale-export-group">
          <select v-model="presaleExportProductId" class="form-select compact-select presale-export-select">
            <option value="all">全部预售商品</option>
            <option v-for="p in presaleProductOptions" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
          <button class="admin-btn btn-green" :disabled="selectedCount === 0" @click="exportPresaleOrders" title="导出所选订单中的预售商品（仅导出选定的预售商品）">
            <i class="fa fa-download"></i> 导出预售订单
          </button>
        </div>
        <input type="file" ref="importFileRef" accept=".csv" style="display:none" @change="handleImportFile">
        <button class="admin-btn btn-outline" @click="$refs.importFileRef.click()" title="导入带有物流单号的CSV文件，自动匹配包裹并发货">
          <i class="fa fa-upload"></i> 导入发货单
        </button>
      </div>
    </div>

    <div v-if="importResult" class="import-result-panel">
      <div class="import-result-header">
        <strong>导入结果</strong>
        <button class="admin-btn btn-outline" style="padding:2px 8px;font-size:0.75rem;" @click="importResult = null">关闭</button>
      </div>
      <div class="import-result-summary">
        成功 <strong>{{ importResult.success }}</strong> 单，跳过 {{ importResult.skipped }} 单，失败 {{ importResult.errors?.length || 0 }} 单
      </div>
      <div v-if="importResult.details?.length" class="import-result-details">
        <div v-for="(d, i) in importResult.details" :key="i" class="import-detail-line">{{ d }}</div>
      </div>
      <div v-if="importResult.errors?.length" class="import-result-errors">
        <div v-for="(e, i) in importResult.errors" :key="i" class="import-error-line">{{ e }}</div>
      </div>
    </div>

    <div class="table-container">
      <table class="data-table">
        <thead>
          <tr>
            <th class="checkbox-cell">
              <input
                ref="selectAllRef"
                type="checkbox"
                :checked="pageAllSelected"
                :disabled="orders.length === 0"
                @change="toggleSelectAllPage($event.target.checked)"
              >
            </th>
            <th>订单号 / 时间</th>
            <th>商品概览</th>
            <th>收货信息</th>
            <th>金额</th>
            <th style="text-align: center;">状态</th>
            <th style="text-align: center;">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="order in orders" :key="order.id">
            <td class="checkbox-cell">
              <input type="checkbox" :checked="isOrderSelected(order.id)" @change="toggleSelectOrder(order.id, $event.target.checked)">
            </td>
            <td>
              <div class="order-id">{{ order.id }}</div>
              <div class="text-sub">{{ new Date(order.created_at).toLocaleString() }}</div>
              <span v-if="order.orderType === 'mixed'" class="order-type-badge type-mixed">混合订单</span>
              <span v-else-if="order.orderType === 'presale'" class="order-type-badge type-presale">预售订单</span>
              <div v-if="getExportTags(order).length" class="export-tags">
                <span v-for="tag in getExportTags(order)" :key="tag.key" class="export-tag" :class="tag.cls">{{ tag.label }}</span>
              </div>
              <div v-if="order.mergeMeta" class="merge-order-brief">
                <div><strong>合并单（第 {{ getMergeCount(order.mergeMeta) }} 次）</strong></div>
                <div class="merge-kv-row">
                  <span>构成订单</span>
                  <span>{{ (order.mergeMeta.parts || []).length }} 笔</span>
                </div>
                <div v-for="part in getMergeParts(order.mergeMeta)" :key="part.orderId">
                  {{ part.orderId }}: ¥{{ part.amount }}
                </div>
                <div v-if="(order.mergeMeta.parts || []).length > 4">...</div>
                <div class="merge-kv-row">
                  <span>先前订单</span>
                  <span>¥{{ getMergeSourceAmount(order.mergeMeta, order.total) }}</span>
                </div>
                <div class="merge-kv-row">
                  <span>本次应付</span>
                  <span>¥{{ getMergeIncrementalPayable(order.mergeMeta, order.total) }}</span>
                </div>
                <div class="merge-kv-row">
                  <span>订单总金额</span>
                  <span>¥{{ getMergeTotalAmount(order.mergeMeta, order.total) }}</span>
                </div>
                <div class="merge-kv-row">
                  <span>邮费减免</span>
                  <span style="color: #16a34a;">-¥{{ getMergeShippingDiscount(order.mergeMeta) }}</span>
                </div>
                <div v-if="getMergeShippingExtra(order.mergeMeta) > 0" class="merge-kv-row">
                  <span>邮费补差</span>
                  <span style="color: #dc2626;">+¥{{ getMergeShippingExtra(order.mergeMeta) }}</span>
                </div>
              </div>
            </td>
            <td>
              <template v-if="order.subOrders && order.subOrders.length > 0">
                <div v-for="sub in order.subOrders" :key="sub.subKey" class="sub-order-block">
                  <div class="sub-order-header">
                    <span class="sub-order-label">{{ sub.label }}</span>
                    <span v-if="sub.shipped" class="sub-shipped-badge"><i class="fa fa-check"></i> 已发货</span>
                    <span v-else class="sub-pending-badge">待发货</span>
                  </div>
                  <div v-for="(item, idx) in sub.items" :key="idx" class="item-row">
                    {{ item.name }} <span style="color: #9ca3af;">x{{ item.quantity }}</span>
                  </div>
                  <div v-if="sub.shipped && sub.trackingNo" class="text-sub" style="font-size: 0.72rem;">
                    {{ sub.trackingCompany }} {{ sub.trackingNo }}
                  </div>
                </div>
              </template>
              <template v-else>
                <div v-for="(item, idx) in order.items" :key="idx" class="item-row">
                  {{ item.name }} <span style="color: #9ca3af;">x{{ item.quantity }}</span>
                  <span v-if="item.isPresale" class="item-presale-tag">预售</span>
                </div>
              </template>
            </td>
            <td style="font-size: 0.85rem;">
              <div><strong>{{ order.contact.name }}</strong> {{ order.contact.phone }}</div>
              <div class="text-sub">{{ order.contact.email || '-' }}</div>
              <div class="text-sub">{{ order.contact.province }}{{ order.contact.city }}{{ order.contact.district }}</div>
              <div class="text-sub" style="max-width: 200px;">{{ order.contact.addressDetail }}</div>
            </td>
            <td style="font-weight: bold;">¥{{ order.total }}</td>
            <td style="text-align: center;">
              <span :class="['status-badge', 'status-' + order.status]">{{ getStatusLabel(order.status) }}</span>
            </td>
            <td style="text-align: center;">
              <div style="display: flex; flex-direction: column; gap: 0.4rem; align-items: center;">
                <div style="display: flex; gap: 0.5rem; justify-content: center;">
                  <button class="admin-btn btn-outline" style="font-size: 0.75rem;" @click="openEditContact(order)">修改收货</button>
                  <template v-if="order.status === 1">
                    <button class="admin-btn btn-blue" style="font-size: 0.75rem;" @click="updateStatus(order.id, 2)">收款</button>
                    <button class="admin-btn btn-outline" style="font-size: 0.75rem; color: #ef4444;" @click="updateStatus(order.id, 0)">取消</button>
                  </template>
                  <template v-if="order.status === 5">
                    <button class="admin-btn btn-blue" style="font-size: 0.75rem;" @click="updateStatus(order.id, 2)">确认收款</button>
                    <button class="admin-btn btn-outline order-delete-btn" style="font-size: 0.75rem;" @click="deletePendingOrder(order)">删除</button>
                  </template>
                  <template v-if="order.status === 2 && (!order.subOrders || order.subOrders.length === 0)">
                    <button class="admin-btn btn-green" style="font-size: 0.75rem;" @click="openShip(order)">发货</button>
                  </template>
                  <span v-if="order.status === 0" style="color: #999; font-size: 0.75rem;">已取消 (库存已回滚)</span>
                  <span v-if="order.status === 3 && (!order.subOrders || order.subOrders.length === 0)" style="color: #10b981; font-size: 0.75rem;">已发货</span>
                </div>
                <!-- Sub-order shipping buttons -->
                <template v-if="order.status === 2 && order.subOrders && order.subOrders.length > 0">
                  <div v-for="sub in order.subOrders" :key="sub.subKey" class="sub-ship-row">
                    <template v-if="!sub.shipped">
                      <button class="admin-btn btn-green" style="font-size: 0.72rem;" @click="openSubShip(order.id, sub)">
                        发货: {{ sub.label }}
                      </button>
                    </template>
                    <span v-else style="color: #10b981; font-size: 0.72rem;">
                      <i class="fa fa-check"></i> {{ sub.label }}
                    </span>
                  </div>
                </template>
                <template v-if="order.status === 3 && order.subOrders && order.subOrders.length > 0">
                  <span style="color: #10b981; font-size: 0.75rem;">全部已发货</span>
                </template>
              </div>
            </td>
          </tr>
          <tr v-if="orders.length === 0">
            <td colspan="7" style="text-align: center; color: #9ca3af;">暂无订单数据</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="toolbar pagination-row">
      <span class="text-sub">共 {{ ordersMeta.total }} 单，第 {{ ordersMeta.page }} / {{ ordersMeta.totalPages }} 页</span>
      <button class="admin-btn btn-outline" :disabled="ordersMeta.page <= 1" @click="goPrevPage">上一页</button>
      <button class="admin-btn btn-outline" :disabled="ordersMeta.page >= ordersMeta.totalPages" @click="goNextPage">下一页</button>
    </div>

    <!-- Ship modal (for orders without sub-orders) -->
    <div v-if="shipModal.show" class="modal-overlay">
      <div class="modal-card">
        <h3 class="modal-title">订单发货</h3>
        <label class="form-label">快递单号 (选填，可自动识别快递公司)</label>
        <input v-model="shipModal.no" type="text" class="form-input" placeholder="留空则不回填运单号">
        <div v-if="detectedCompany" style="margin-top: 0.5rem; font-size: 0.85rem; color: #047857;">
          <i class="fa fa-check-circle"></i> 识别为: {{ detectedCompany }}
        </div>
        <div class="modal-actions">
          <button @click="shipModal.show = false" class="admin-btn btn-outline">取消</button>
          <button @click="confirmShip" class="admin-btn btn-blue">确认发货</button>
        </div>
      </div>
    </div>

    <!-- Sub-order ship modal -->
    <div v-if="subShipModal.show" class="modal-overlay">
      <div class="modal-card">
        <h3 class="modal-title">子订单发货: {{ subShipModal.label }}</h3>
        <div class="sub-ship-items">
          <div v-for="(item, idx) in subShipModal.items" :key="idx" class="item-row">
            {{ item.name }} <span style="color: #9ca3af;">x{{ item.quantity }}</span>
          </div>
        </div>
        <label class="form-label">快递单号 (选填，可自动识别快递公司)</label>
        <input v-model="subShipModal.no" type="text" class="form-input" placeholder="留空则不回填运单号">
        <div v-if="subDetectedCompany" style="margin-top: 0.5rem; font-size: 0.85rem; color: #047857;">
          <i class="fa fa-check-circle"></i> 识别为: {{ subDetectedCompany }}
        </div>
        <div class="modal-actions">
          <button @click="subShipModal.show = false" class="admin-btn btn-outline">取消</button>
          <button @click="confirmSubShip" class="admin-btn btn-blue">确认发货</button>
        </div>
      </div>
    </div>

    <div v-if="editContactModal.show" class="modal-overlay">
      <div class="modal-card">
        <h3 class="modal-title">修改收货信息</h3>
        <div class="contact-edit-grid">
          <div>
            <label class="form-label">收货人姓名</label>
            <input v-model.trim="editContactModal.form.name" type="text" class="form-input">
          </div>
          <div>
            <label class="form-label">手机号</label>
            <input v-model.trim="editContactModal.form.phone" type="tel" maxlength="11" class="form-input">
          </div>
          <div class="col-span-2">
            <label class="form-label">邮箱</label>
            <input v-model.trim="editContactModal.form.email" type="email" class="form-input">
          </div>
          <div>
            <label class="form-label">省</label>
            <input v-model.trim="editContactModal.form.province" type="text" class="form-input">
          </div>
          <div>
            <label class="form-label">市</label>
            <input v-model.trim="editContactModal.form.city" type="text" class="form-input">
          </div>
          <div class="col-span-2">
            <label class="form-label">区/县</label>
            <input v-model.trim="editContactModal.form.district" type="text" class="form-input">
          </div>
          <div class="col-span-2">
            <label class="form-label">详细地址</label>
            <input v-model.trim="editContactModal.form.addressDetail" type="text" class="form-input">
          </div>
        </div>
        <p v-if="editContactModal.error" class="text-danger">{{ editContactModal.error }}</p>
        <div class="modal-actions">
          <button @click="closeEditContactModal" class="admin-btn btn-outline">取消</button>
          <button @click="confirmEditContact" class="admin-btn btn-blue" :disabled="editContactModal.saving">
            {{ editContactModal.saving ? '保存中...' : '保存' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, reactive, watch } from 'vue'
import { useShopStore } from '@/stores/shopStore'

const store = useShopStore()
const filterStatus = ref('all')
const filterItemType = ref('')
const orders = computed(() => store.state.adminOrders)
const ordersMeta = computed(() => store.state.adminOrdersMeta || { page: 1, pageSize: 20, total: 0, totalPages: 1 })
const selectedOrderIds = ref(new Set())
const selectAllRef = ref(null)
const keyword = ref('')
const sortBy = ref('created_at')
const sortDir = ref('desc')
const pageSize = ref(20)

const presaleExportProductId = ref('all')
const importFileRef = ref(null)
const importResult = ref(null)

const presaleProductOptions = computed(() => {
  const map = new Map()
  for (const order of orders.value) {
    for (const item of (order.items || [])) {
      if (item.isPresale && item.id && !map.has(item.id)) {
        map.set(item.id, { id: item.id, name: item.name || `商品${item.id}` })
      }
    }
  }
  return [...map.values()]
})

const statusOptions = [
  { value: 'all', label: '全部' },
  { value: 1, label: '待付款' },
  { value: 5, label: '待确认' },
  { value: 2, label: '待发货' },
  { value: 3, label: '已发货' },
  { value: 0, label: '已取消' }
]

// --- Cross-page selection ---
const selectedCount = computed(() => selectedOrderIds.value.size)

const pageVisibleIds = computed(() => new Set(orders.value.map(o => o.id)))
const pageSelectedCount = computed(() => {
  let count = 0
  pageVisibleIds.value.forEach(id => { if (selectedOrderIds.value.has(id)) count++ })
  return count
})
const pageAllSelected = computed(() => orders.value.length > 0 && pageSelectedCount.value === orders.value.length)
const pagePartiallySelected = computed(() => pageSelectedCount.value > 0 && pageSelectedCount.value < orders.value.length)

const clearSelection = () => {
  selectedOrderIds.value = new Set()
}

const isOrderSelected = (orderId) => selectedOrderIds.value.has(orderId)

const toggleSelectOrder = (orderId, checked) => {
  const next = new Set(selectedOrderIds.value)
  if (checked) next.add(orderId)
  else next.delete(orderId)
  selectedOrderIds.value = next
}

const toggleSelectAllPage = (checked) => {
  const next = new Set(selectedOrderIds.value)
  if (checked) {
    orders.value.forEach(o => next.add(o.id))
  } else {
    orders.value.forEach(o => next.delete(o.id))
  }
  selectedOrderIds.value = next
}

const selectPendingUnexported = async () => {
  const ids = await store.fetchOrderIds(2, undefined, { notFullyExported: true })
  if (ids.length === 0) {
    store.showNotification('没有未导出的待发货订单')
    return
  }
  const next = new Set(selectedOrderIds.value)
  ids.forEach(id => next.add(id))
  selectedOrderIds.value = next
  store.showNotification(`已勾选 ${ids.length} 单未导出待发货订单`)
}

const selectAllPending = async () => {
  const ids = await store.fetchOrderIds(2)
  if (ids.length === 0) {
    store.showNotification('没有待发货订单')
    return
  }
  const next = new Set(selectedOrderIds.value)
  ids.forEach(id => next.add(id))
  selectedOrderIds.value = next
  store.showNotification(`已勾选 ${ids.length} 单待发货订单`)
}

// --- Data loading ---
const buildListFilters = (pageOverride = null) => ({
  keyword: keyword.value,
  sortBy: sortBy.value,
  sortDir: sortDir.value,
  page: pageOverride ?? ordersMeta.value.page ?? 1,
  pageSize: pageSize.value,
  hasPresale: filterItemType.value === 'presale' ? true : undefined,
  hasSpot: filterItemType.value === 'spot' ? true : undefined
})

const loadOrders = async (page = 1) => {
  await store.fetchAdminOrders(filterStatus.value, buildListFilters(page))
}

const changeFilter = (status) => {
  filterStatus.value = status
  clearSelection()
  loadOrders(1)
}

const changeItemTypeFilter = (type) => {
  filterItemType.value = filterItemType.value === type ? '' : type
  clearSelection()
  loadOrders(1)
}

onMounted(() => {
  loadOrders(1)
})

const searchOrders = () => {
  clearSelection()
  loadOrders(1)
}

const goPrevPage = () => {
  if (ordersMeta.value.page <= 1) return
  loadOrders(ordersMeta.value.page - 1)
}

const goNextPage = () => {
  if (ordersMeta.value.page >= ordersMeta.value.totalPages) return
  loadOrders(ordersMeta.value.page + 1)
}

const getStatusLabel = (s) => (['已取消', '待付款', '待发货', '已发货', '已完成', '待确认'][s] || '未知')
const getExportTags = (order) => {
  if (order.exported && order.orderType === 'spot') {
    return [{ key: 'all', label: '已导出', cls: 'tag-done' }]
  }
  // For mixed/presale orders, show granular tags
  const tags = []
  const items = order.items || []
  const hasSpot = items.some(i => !i.isPresale)
  const presaleProductMap = new Map()
  for (const item of items) {
    if (item.isPresale && item.id && !presaleProductMap.has(Number(item.id))) {
      presaleProductMap.set(Number(item.id), item.name || `商品${item.id}`)
    }
  }
  const exportedPresale = Array.isArray(order.presaleExportedProducts) ? order.presaleExportedProducts : []

  // Check if fully exported
  const spotDone = !hasSpot || order.spotExported || order.exported
  const presaleDone = presaleProductMap.size === 0 || [...presaleProductMap.keys()].every(pid => exportedPresale.includes(pid))
  if (spotDone && presaleDone && (hasSpot || presaleProductMap.size > 0)) {
    return [{ key: 'all', label: '已全部导出', cls: 'tag-done' }]
  }

  if (hasSpot && (order.spotExported || order.exported)) {
    tags.push({ key: 'spot', label: '现货', cls: 'tag-spot' })
  }
  for (const [pid, name] of presaleProductMap) {
    if (exportedPresale.includes(pid)) {
      tags.push({ key: `p-${pid}`, label: name, cls: 'tag-presale' })
    }
  }
  return tags
}

const getMergeParts = (mergeMeta = null) => {
  const parts = Array.isArray(mergeMeta?.parts) ? mergeMeta.parts : []
  return parts.slice(0, 4)
}
const getMergeCount = (mergeMeta = null) => {
  const parsed = Number(mergeMeta?.mergeCount)
  if (Number.isInteger(parsed) && parsed > 0) return parsed
  const history = Array.isArray(mergeMeta?.history) ? mergeMeta.history : []
  return history.length || 1
}
const toMoney = (value) => Number((Number(value) || 0).toFixed(2))
const getMergeShippingAdjustment = (mergeMeta = null) => {
  const parsed = Number(mergeMeta?.shippingAdjustment)
  if (Number.isFinite(parsed)) return toMoney(parsed)
  const source = Number(mergeMeta?.sourceShippingFee) || 0
  const appended = Number(mergeMeta?.appendedShippingFee) || 0
  const merged = Number(mergeMeta?.mergedShippingFee) || 0
  return toMoney(merged - source - appended)
}
const getMergeShippingDiscount = (mergeMeta = null) =>
  toMoney(Math.max(0, -getMergeShippingAdjustment(mergeMeta)))
const getMergeShippingExtra = (mergeMeta = null) =>
  toMoney(Math.max(0, getMergeShippingAdjustment(mergeMeta)))
const getMergeIncrementalPayable = (mergeMeta = null, orderTotal = 0) => {
  const parsed = Number(mergeMeta?.incrementalPayable)
  if (Number.isFinite(parsed)) return toMoney(parsed)
  const appendedAmount = Number(mergeMeta?.appendedAmount)
  if (Number.isFinite(appendedAmount)) {
    return toMoney(Math.max(0, appendedAmount + getMergeShippingAdjustment(mergeMeta)))
  }
  const total = getMergeTotalAmount(mergeMeta, orderTotal)
  const source = getMergeSourceAmount(mergeMeta, orderTotal)
  return toMoney(Math.max(0, total - source))
}
const getMergeTotalAmount = (mergeMeta = null, orderTotal = 0) => {
  const parsed = Number(mergeMeta?.mergedAmount)
  if (Number.isFinite(parsed)) return toMoney(parsed)
  return toMoney(orderTotal)
}
const getMergeSourceAmount = (mergeMeta = null, orderTotal = 0) => {
  const parsed = Number(mergeMeta?.sourceAmount)
  if (Number.isFinite(parsed)) return toMoney(parsed)
  const appendedAmount = Number(mergeMeta?.appendedAmount)
  if (Number.isFinite(appendedAmount)) {
    const total = getMergeTotalAmount(mergeMeta, orderTotal)
    const incremental = toMoney(Math.max(0, appendedAmount + getMergeShippingAdjustment(mergeMeta)))
    return toMoney(Math.max(0, total - incremental))
  }
  return toMoney(orderTotal)
}

const updateStatus = async (id, status) => {
  if (status === 0 && !confirm('取消订单将自动回滚库存，确定吗？')) return
  await store.updateOrderStatus(id, status, {}, filterStatus.value, buildListFilters())
}

const deletePendingOrder = async (order) => {
  if (!order || order.status !== 5) return
  const shouldDelete = confirm(`确认删除订单 ${order.id}？该操作将回滚库存，仅用于清理异常或测试订单。`)
  if (!shouldDelete) return
  await store.deleteAdminOrder(order.id, filterStatus.value, buildListFilters())
}

// --- Select-all checkbox indeterminate state ---
watch([orders, pagePartiallySelected], () => {
  if (selectAllRef.value) {
    selectAllRef.value.indeterminate = pagePartiallySelected.value
  }
}, { immediate: true })

// --- Ship modal (regular orders) ---
const shipModal = reactive({ show: false, id: null, no: '' })
const openShip = (order) => {
  shipModal.no = ''
  shipModal.id = order.id
  shipModal.show = true
}

const detectCompany = (no) => {
  if (!no) return ''
  const n = no.trim().toUpperCase()
  if (n.startsWith('SF')) return '顺丰速运'
  if (n.startsWith('YT')) return '圆通速递'
  if (n.startsWith('JT')) return '极兔速递'
  if (n.startsWith('JD')) return '京东快递'
  if (/^E[A-Z]\d/.test(n)) return 'EMS'
  if (/^46\d/.test(n)) return '韵达快递'
  if (/^77\d/.test(n)) return '申通快递'
  if (/^7[56]\d/.test(n)) return '中通快递'
  if (/^(268|368|468|568)\d/.test(n)) return '申通快递'
  if (/^(31|32|33|34)\d/.test(n)) return '韵达快递'
  if (/^(70|56)\d/.test(n)) return '百世快递'
  return '未知快递'
}

const detectedCompany = computed(() => detectCompany(shipModal.no))

const confirmShip = async () => {
  const no = shipModal.no.trim()
  const tracking = no ? { trackingCompany: detectCompany(no), trackingNo: no } : {}
  const success = await store.updateOrderStatus(shipModal.id, 3, tracking, filterStatus.value, buildListFilters())
  if (success) shipModal.show = false
}

// --- Sub-order ship modal ---
const subShipModal = reactive({ show: false, orderId: '', subKey: '', label: '', items: [], no: '' })
const subDetectedCompany = computed(() => detectCompany(subShipModal.no))

const openSubShip = (orderId, sub) => {
  subShipModal.orderId = orderId
  subShipModal.subKey = sub.subKey
  subShipModal.label = sub.label
  subShipModal.items = sub.items || []
  subShipModal.no = ''
  subShipModal.show = true
}

const confirmSubShip = async () => {
  const no = subShipModal.no.trim()
  const tracking = no ? { trackingCompany: detectCompany(no), trackingNo: no } : {}
  const result = await store.shipSubOrder(subShipModal.orderId, subShipModal.subKey, tracking)
  if (result) {
    subShipModal.show = false
    await loadOrders(ordersMeta.value.page)
  }
}

// --- Edit contact modal ---
const editContactModal = reactive({
  show: false,
  id: '',
  saving: false,
  error: '',
  form: {
    name: '',
    phone: '',
    email: '',
    province: '',
    city: '',
    district: '',
    addressDetail: ''
  }
})

const closeEditContactModal = () => {
  editContactModal.show = false
  editContactModal.saving = false
  editContactModal.error = ''
}

const openEditContact = (order) => {
  if (!order) return
  editContactModal.id = order.id
  editContactModal.error = ''
  editContactModal.form.name = String(order.contact?.name || '')
  editContactModal.form.phone = String(order.contact?.phone || '')
  editContactModal.form.email = String(order.contact?.email || '')
  editContactModal.form.province = String(order.contact?.province || '')
  editContactModal.form.city = String(order.contact?.city || '')
  editContactModal.form.district = String(order.contact?.district || '')
  editContactModal.form.addressDetail = String(order.contact?.addressDetail || '')
  editContactModal.show = true
}

const confirmEditContact = async () => {
  if (!editContactModal.id || editContactModal.saving) return
  editContactModal.error = ''

  if (!editContactModal.form.name.trim()) {
    editContactModal.error = '请填写收货人姓名'
    return
  }
  if (!/^1[3-9]\d{9}$/.test(editContactModal.form.phone.trim())) {
    editContactModal.error = '手机号格式错误'
    return
  }
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(editContactModal.form.email.trim())) {
    editContactModal.error = '邮箱格式错误'
    return
  }
  if (!editContactModal.form.province.trim() || !editContactModal.form.city.trim() || !editContactModal.form.district.trim()) {
    editContactModal.error = '省市区信息不完整'
    return
  }
  if (!editContactModal.form.addressDetail.trim()) {
    editContactModal.error = '详细地址不能为空'
    return
  }

  editContactModal.saving = true
  const success = await store.updateAdminOrderContact(
    editContactModal.id,
    {
      name: editContactModal.form.name.trim(),
      phone: editContactModal.form.phone.trim(),
      email: editContactModal.form.email.trim(),
      province: editContactModal.form.province.trim(),
      city: editContactModal.form.city.trim(),
      district: editContactModal.form.district.trim(),
      addressDetail: editContactModal.form.addressDetail.trim()
    },
    filterStatus.value,
    buildListFilters()
  )
  editContactModal.saving = false
  if (success) {
    closeEditContactModal()
  } else {
    editContactModal.error = '修改失败，请检查输入或稍后重试'
  }
}

// --- Export ---
const toCsvCell = (value) => {
  if (value === null || value === undefined) return ''
  const text = String(value).replace(/"/g, '""')
  return /[",\n]/.test(text) ? `"${text}"` : text
}

const toAddressText = (contact = {}) => `${contact.province || ''}${contact.city || ''}${contact.district || ''}${contact.addressDetail || ''}`
const toItemsText = (items = []) => items.map((item) => `${item.name} x${item.quantity}`).join('；')

const downloadCsv = (rows, headers, filenamePrefix) => {
  const csv = [
    headers.map(toCsvCell).join(','),
    ...rows.map((row) => row.map(toCsvCell).join(','))
  ].join('\n')

  const now = new Date()
  const stamp = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}-${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}`
  const blob = new Blob([`\uFEFF${csv}`], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${filenamePrefix}-${stamp}.csv`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

const orderToRow = (order, items = null) => [
  order.id,
  new Date(order.created_at).toLocaleString(),
  getStatusLabel(order.status),
  order.total,
  order.contact?.name || '',
  order.contact?.phone || '',
  order.contact?.email || '',
  toAddressText(order.contact),
  toItemsText(items || order.items),
  order.trackingCompany || '',
  order.trackingNo || ''
]

const exportSelectedOrders = async () => {
  if (selectedCount.value === 0) {
    alert('请先选择要导出的订单')
    return
  }

  const currentPageSelected = orders.value.filter(o => selectedOrderIds.value.has(o.id))

  const headers = ['订单号', '下单时间', '状态', '金额', '收货人', '联系电话', '邮箱', '收货地址', '商品明细', '物流公司', '物流单号']
  const rows = currentPageSelected.map((order) => orderToRow(order))

  const offPageIds = [...selectedOrderIds.value].filter(id => !pageVisibleIds.value.has(id))
  if (offPageIds.length > 0) {
    const saved = { orders: [...store.state.adminOrders], meta: { ...store.state.adminOrdersMeta } }
    for (const id of offPageIds) {
      await store.fetchAdminOrders('all', { keyword: id, page: 1, pageSize: 1 })
      const found = store.state.adminOrders.find(o => o.id === id)
      if (found) rows.push(orderToRow(found))
    }
    store.state.adminOrders = saved.orders
    store.state.adminOrdersMeta = saved.meta
  }

  downloadCsv(rows, headers, 'orders')

  const exportedIds = [...selectedOrderIds.value]
  await store.markOrdersExported(exportedIds)

  await loadOrders(ordersMeta.value.page)
  store.showNotification(`已导出 ${rows.length} 单并标记为已导出`)
}

// --- Export spot orders ---
const exportSpotOrders = async () => {
  if (selectedCount.value === 0) {
    alert('请先选择要导出的订单')
    return
  }

  // Collect all selected orders (cross-page)
  const allSelectedOrders = []
  const currentPageSelected = orders.value.filter(o => selectedOrderIds.value.has(o.id))
  allSelectedOrders.push(...currentPageSelected)

  const offPageIds = [...selectedOrderIds.value].filter(id => !pageVisibleIds.value.has(id))
  if (offPageIds.length > 0) {
    const saved = { orders: [...store.state.adminOrders], meta: { ...store.state.adminOrdersMeta } }
    for (const id of offPageIds) {
      await store.fetchAdminOrders('all', { keyword: id, page: 1, pageSize: 1 })
      const found = store.state.adminOrders.find(o => o.id === id)
      if (found) allSelectedOrders.push(found)
    }
    store.state.adminOrders = saved.orders
    store.state.adminOrdersMeta = saved.meta
  }

  // Filter: skip pure presale orders, export only spot items from each order
  const headers = ['订单号', '下单时间', '状态', '金额', '收货人', '联系电话', '邮箱', '收货地址', '商品明细(现货)', '物流公司', '物流单号']
  const rows = []
  const pureSpotIds = []
  const mixedIds = []

  for (const order of allSelectedOrders) {
    const spotItems = (order.items || []).filter(i => !i.isPresale)
    if (spotItems.length === 0) continue // pure presale, skip

    rows.push(orderToRow(order, spotItems))

    if (order.orderType === 'mixed' || order.hasPresaleItems) {
      mixedIds.push(order.id)
    } else {
      pureSpotIds.push(order.id)
    }
  }

  if (rows.length === 0) {
    store.showNotification('所选订单中没有包含现货商品的订单')
    return
  }

  downloadCsv(rows, headers, 'spot-orders')

  // Mark: pure spot -> exported, mixed -> spotExported
  if (pureSpotIds.length > 0) {
    await store.markOrdersExported(pureSpotIds)
  }
  if (mixedIds.length > 0) {
    await store.markOrdersSpotExported(mixedIds)
  }

  await loadOrders(ordersMeta.value.page)
  store.showNotification(`已导出 ${rows.length} 单现货订单`)
}

// --- Export presale orders ---
const exportPresaleOrders = async () => {
  if (selectedCount.value === 0) {
    alert('请先选择要导出的订单')
    return
  }

  const filterProductId = presaleExportProductId.value
  const filterById = filterProductId !== 'all' ? Number(filterProductId) : null

  // Collect all selected orders (cross-page)
  const allSelectedOrders = []
  const currentPageSelected = orders.value.filter(o => selectedOrderIds.value.has(o.id))
  allSelectedOrders.push(...currentPageSelected)

  const offPageIds = [...selectedOrderIds.value].filter(id => !pageVisibleIds.value.has(id))
  if (offPageIds.length > 0) {
    const saved = { orders: [...store.state.adminOrders], meta: { ...store.state.adminOrdersMeta } }
    for (const id of offPageIds) {
      await store.fetchAdminOrders('all', { keyword: id, page: 1, pageSize: 1 })
      const found = store.state.adminOrders.find(o => o.id === id)
      if (found) allSelectedOrders.push(found)
    }
    store.state.adminOrders = saved.orders
    store.state.adminOrdersMeta = saved.meta
  }

  const headers = ['订单号', '下单时间', '状态', '金额', '收货人', '联系电话', '邮箱', '收货地址', '商品明细(预售)', '物流公司', '物流单号']
  const rows = []

  for (const order of allSelectedOrders) {
    const presaleItems = (order.items || []).filter(i =>
      i.isPresale && (filterById === null || Number(i.id) === filterById)
    )
    if (presaleItems.length === 0) continue

    rows.push(orderToRow(order, presaleItems))
  }

  if (rows.length === 0) {
    store.showNotification('所选订单中没有匹配的预售商品')
    return
  }

  const suffix = filterById !== null
    ? presaleProductOptions.value.find(p => p.id === filterById)?.name || '预售'
    : '预售'
  downloadCsv(rows, headers, `presale-${suffix}`)

  // Collect exported product IDs and order IDs for marking
  const exportedProductIds = []
  const exportedOrderIds = []
  for (const order of allSelectedOrders) {
    const presaleItems = (order.items || []).filter(i =>
      i.isPresale && (filterById === null || Number(i.id) === filterById)
    )
    if (presaleItems.length === 0) continue
    exportedOrderIds.push(order.id)
    for (const item of presaleItems) {
      if (!exportedProductIds.includes(Number(item.id))) {
        exportedProductIds.push(Number(item.id))
      }
    }
  }

  if (exportedOrderIds.length > 0 && exportedProductIds.length > 0) {
    await store.markPresaleExported(exportedOrderIds, exportedProductIds)
  }

  await loadOrders(ordersMeta.value.page)
  store.showNotification(`已导出 ${rows.length} 单预售订单`)
}

// --- Import tracking CSV ---
const handleImportFile = async (e) => {
  const file = e.target.files?.[0]
  if (!file) return
  // Reset so same file can be re-selected
  e.target.value = ''

  importResult.value = null
  store.showNotification('正在导入发货单...')
  const result = await store.importTracking(file)
  if (!result) {
    store.showNotification('导入失败')
    return
  }
  if (result.error) {
    store.showNotification(`导入失败: ${result.error}`)
    return
  }
  importResult.value = result
  await loadOrders(ordersMeta.value.page)
  store.showNotification(`导入完成: 成功 ${result.success} 单`)
}
</script>

<style scoped>
.import-result-panel {
  margin: 0.5rem 0;
  padding: 0.6rem 0.8rem;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  background: #f9fafb;
  font-size: 0.8rem;
}

.import-result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.4rem;
}

.import-result-summary {
  margin-bottom: 0.3rem;
}

.import-result-details,
.import-result-errors {
  max-height: 150px;
  overflow-y: auto;
  font-size: 0.75rem;
  line-height: 1.6;
}

.import-detail-line {
  color: #047857;
}

.import-error-line {
  color: #dc2626;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.toolbar-query {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.pagination-row {
  justify-content: flex-end;
  gap: 0.5rem;
}

.checkbox-cell {
  width: 3rem;
  text-align: center;
}

.compact-select {
  width: 110px;
  margin-bottom: 0;
  padding: 0.35rem 0.5rem;
}

button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.order-delete-btn {
  color: #ef4444;
  border-color: #fca5a5;
}

.export-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
  margin-top: 0.25rem;
}

.export-tag {
  display: inline-block;
  padding: 0 5px;
  border-radius: 3px;
  font-size: 0.63rem;
  font-weight: 600;
  line-height: 1.5;
  white-space: nowrap;
}

.export-tag::before {
  content: '\2713 ';
}

.tag-done {
  background: #ecfdf5;
  color: #047857;
  border: 1px solid #a7f3d0;
}

.tag-spot {
  background: #eff6ff;
  color: #1d4ed8;
  border: 1px solid #bfdbfe;
}

.tag-presale {
  background: #ede9fe;
  color: #5b21b6;
  border: 1px solid #c4b5fd;
}

.order-type-badge {
  display: inline-block;
  margin-top: 0.25rem;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 0.68rem;
  font-weight: 600;
}

.type-mixed {
  background: #fef3c7;
  color: #92400e;
  border: 1px solid #fcd34d;
}

.type-presale {
  background: #ede9fe;
  color: #5b21b6;
  border: 1px solid #c4b5fd;
}

.item-presale-tag {
  display: inline-block;
  margin-left: 0.3rem;
  padding: 0 4px;
  border-radius: 3px;
  font-size: 0.65rem;
  font-weight: 600;
  background: #ede9fe;
  color: #7c3aed;
  vertical-align: middle;
}

.presale-export-group {
  display: inline-flex;
  align-items: center;
  gap: 0;
}

.presale-export-group .presale-export-select {
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
  border-right: none;
  width: auto;
  max-width: 150px;
}

.presale-export-group .admin-btn {
  border-top-left-radius: 0;
  border-bottom-left-radius: 0;
  white-space: nowrap;
}

.filter-separator {
  color: #cbd5e1;
  margin: 0 0.15rem;
  user-select: none;
}

.sub-order-block {
  padding: 0.35rem 0.45rem;
  margin-bottom: 0.35rem;
  border-radius: 5px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
}

.sub-order-header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-bottom: 0.2rem;
}

.sub-order-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: #475569;
}

.sub-shipped-badge {
  font-size: 0.68rem;
  color: #10b981;
  font-weight: 600;
}

.sub-pending-badge {
  font-size: 0.68rem;
  color: #f59e0b;
  font-weight: 600;
}

.sub-ship-row {
  width: 100%;
}

.sub-ship-items {
  padding: 0.4rem 0.5rem;
  margin-bottom: 0.5rem;
  border-radius: 5px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
}

.merge-order-brief {
  margin-top: 0.35rem;
  padding: 0.35rem 0.45rem;
  border-radius: 6px;
  background: #eff6ff;
  border: 1px solid #bfdbfe;
  color: #1e3a8a;
  font-size: 0.73rem;
  line-height: 1.45;
}

.merge-kv-row {
  display: flex;
  justify-content: space-between;
  gap: 0.5rem;
}

.contact-edit-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.6rem;
}

.contact-edit-grid .form-input {
  margin-bottom: 0.2rem;
}

.col-span-2 {
  grid-column: span 2;
}

.text-danger {
  margin: 0.4rem 0 0;
  font-size: 0.85rem;
  color: #dc2626;
}

@media (max-width: 1023px) {
  .toolbar-query,
  .toolbar-actions {
    width: 100%;
  }

  .toolbar-query .search-input {
    width: 100%;
    max-width: 100%;
    flex: 1 1 100%;
  }

  .pagination-row {
    justify-content: space-between;
    flex-wrap: wrap;
  }
}

@media (max-width: 639px) {
  .toolbar-query {
    gap: 0.4rem;
  }

  .compact-select {
    width: calc(33.333% - 0.3rem);
    min-width: 0;
  }

  .toolbar-actions {
    flex-wrap: wrap;
  }

  .toolbar-actions .admin-btn {
    flex: 1;
  }

  .checkbox-cell {
    width: 2.25rem;
  }

  .contact-edit-grid {
    grid-template-columns: 1fr;
  }

  .col-span-2 {
    grid-column: span 1;
  }
}
</style>
