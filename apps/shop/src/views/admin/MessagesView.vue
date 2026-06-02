<template>
  <div class="panel">
    <div class="toolbar">
      <div class="filter-group">
        <button
          v-for="item in statusOptions"
          :key="item.value"
          class="filter-btn"
          :class="{ active: filters.status === item.value }"
          @click="changeFilter(item.value)"
        >
          {{ item.label }}
        </button>
      </div>
      <div class="toolbar-query">
        <input v-model.trim="filters.keyword" class="search-input" placeholder="按称呼/联系方式/订单号/内容搜索">
        <select v-model="filters.sortBy" class="form-select compact-select">
          <option value="created_at">按提交时间</option>
          <option value="handled_at">按处理时间</option>
          <option value="status">按状态</option>
          <option value="id">按ID</option>
        </select>
        <select v-model="filters.sortDir" class="form-select compact-select">
          <option value="desc">降序</option>
          <option value="asc">升序</option>
        </select>
        <select v-model.number="filters.pageSize" class="form-select compact-select">
          <option :value="10">10/页</option>
          <option :value="20">20/页</option>
          <option :value="50">50/页</option>
        </select>
        <button class="admin-btn btn-blue" @click="searchMessages">查询</button>
      </div>
    </div>

    <div v-if="loading" class="panel-body">
      <span class="text-sub">加载中...</span>
    </div>
    <div v-else class="table-container">
      <table class="data-table">
        <thead>
          <tr>
            <th style="width: 170px;">ID / 提交时间</th>
            <th style="width: 220px;">留言人</th>
            <th style="width: 180px;">关联订单</th>
            <th>留言内容</th>
            <th style="width: 170px; text-align: center;">状态</th>
            <th style="width: 130px; text-align: center;">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="message in messages" :key="message.id">
            <td>
              <div class="order-id">#{{ message.id }}</div>
              <div class="text-sub">{{ formatTime(message.created_at) }}</div>
            </td>
            <td>
              <div style="font-weight: 600;">{{ message.name }}</div>
              <div class="text-sub">{{ message.contact }}</div>
            </td>
            <td>
              <span v-if="message.orderId" class="order-id">{{ message.orderId }}</span>
              <span v-else class="text-sub">未关联订单</span>
            </td>
            <td>
              <div class="message-content">{{ message.content }}</div>
            </td>
            <td style="text-align: center;">
              <span :class="['status-badge', message.status === 1 ? 'status-3' : 'status-1']">
                {{ message.status === 1 ? '已处理' : '未处理' }}
              </span>
              <div class="text-sub">
                {{ message.handled_at ? `处理时间: ${formatTime(message.handled_at)}` : '尚未处理' }}
              </div>
            </td>
            <td style="text-align: center;">
              <button class="admin-btn btn-outline" style="font-size: 0.75rem;" @click="toggleStatus(message)">
                {{ message.status === 1 ? '标记未处理' : '标记已处理' }}
              </button>
            </td>
          </tr>
          <tr v-if="messages.length === 0">
            <td colspan="6" style="text-align: center; color: #9ca3af;">暂无留言数据</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="toolbar pagination-row">
      <span class="text-sub">共 {{ messagesMeta.total }} 条，第 {{ messagesMeta.page }} / {{ messagesMeta.totalPages }} 页</span>
      <button class="admin-btn btn-outline" :disabled="messagesMeta.page <= 1" @click="goPrevPage">上一页</button>
      <button class="admin-btn btn-outline" :disabled="messagesMeta.page >= messagesMeta.totalPages" @click="goNextPage">下一页</button>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { useShopStore } from '@/stores/shopStore'

const store = useShopStore()
const loading = ref(false)
const messages = computed(() => store.state.adminContactMessages || [])
const messagesMeta = computed(() => store.state.adminContactMessagesMeta || {
  page: 1,
  pageSize: 20,
  total: 0,
  totalPages: 1
})

const statusOptions = [
  { value: 'all', label: '全部' },
  { value: '0', label: '未处理' },
  { value: '1', label: '已处理' }
]

const filters = reactive({
  status: 'all',
  keyword: '',
  sortBy: 'created_at',
  sortDir: 'desc',
  pageSize: 20
})

const buildQuery = (page = 1) => ({
  status: filters.status,
  keyword: filters.keyword,
  sortBy: filters.sortBy,
  sortDir: filters.sortDir,
  page,
  pageSize: filters.pageSize
})

const loadMessages = async (page = 1) => {
  loading.value = true
  await store.fetchAdminContactMessages(buildQuery(page))
  loading.value = false
}

const changeFilter = (status) => {
  filters.status = status
  loadMessages(1)
}

const searchMessages = () => {
  loadMessages(1)
}

const toggleStatus = async (message) => {
  const nextStatus = Number(message.status) === 1 ? 0 : 1
  const success = await store.updateAdminContactMessageStatus(message.id, nextStatus)
  if (success) await loadMessages(messagesMeta.value.page)
}

const goPrevPage = () => {
  if (messagesMeta.value.page <= 1) return
  loadMessages(messagesMeta.value.page - 1)
}

const goNextPage = () => {
  if (messagesMeta.value.page >= messagesMeta.value.totalPages) return
  loadMessages(messagesMeta.value.page + 1)
}

const formatTime = (value) => {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '-'
  return date.toLocaleString()
}

onMounted(() => {
  loadMessages(1)
})
</script>

<style scoped>
.toolbar-query {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.compact-select {
  width: 110px;
  margin-bottom: 0;
  padding: 0.35rem 0.5rem;
}

.message-content {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
  color: #374151;
}

.pagination-row {
  justify-content: flex-end;
  gap: 0.5rem;
}

@media (max-width: 1023px) {
  .toolbar-query {
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
}
</style>
