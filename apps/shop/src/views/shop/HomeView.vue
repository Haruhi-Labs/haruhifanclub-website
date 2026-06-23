<template>
  <div class="home-view">
    <div class="page-header">
        <h2 class="page-title">{{ currentTypeName }}列表</h2>
        <span style="font-size: 0.875rem; color: #666;">共找到 {{ filteredProducts.length }} 件宝物</span>
    </div>

    <div v-if="filteredProducts.length > 0" class="product-grid">
        <div v-for="item in filteredProducts" :key="item.id" class="product-card group" @click="$router.push(`/product/${item.id}`)">
            <div class="card-image-wrapper">
                <picture>
                    <source v-if="item.imageMobile" media="(max-width: 639px)" :srcset="item.imageMobile">
                    <img :src="item.image" :alt="item.name" class="card-image">
                </picture>
                <div class="card-overlay">
                    <button class="market-btn" @click.stop="store.addToCart(item)">加入购物车</button>
                </div>
            </div>
            <div class="card-body">
                <div class="card-title-row">
                    <h3 class="card-title">{{ item.name }}</h3>
                    <div class="card-price-wrap">
                        <span class="card-price">¥{{ getDisplayPrice(item) }}</span>
                        <span v-if="hasDiscount(item)" class="card-price-original">¥{{ item.price }}</span>
                    </div>
                </div>
                <p class="card-desc">{{ item.desc }}</p>
                <div v-if="item.presaleMode === PRESALE_MODES.GOAL" class="card-presale-box">
                    <div class="card-presale-head">
                        <span class="presale-chip presale-chip-goal">进度预售</span>
                        <span class="card-presale-count">
                            {{ getPresaleProgress(item).paidCount }}/{{ getPresaleProgress(item).target }}
                        </span>
                    </div>
                    <div class="card-presale-track">
                        <span :style="{ width: `${getPresaleProgress(item).percent}%` }"></span>
                    </div>
                    <p class="card-presale-tip">
                        {{ getPresaleProgress(item).reached ? '已达标' : '订单持续累计中' }}
                    </p>
                </div>
                <div v-else-if="item.presaleMode === PRESALE_MODES.FIXED" class="card-presale-box">
                    <div class="card-presale-head">
                        <span class="presale-chip presale-chip-fixed">排期预售</span>
                    </div>
                    <p class="card-presale-tip">预售发货时间：{{ getFixedPresaleDateText(item) || '待设置' }}</p>
                </div>
                <div class="card-footer">
                    <span class="badge-tag">{{ item.category }}</span>
                    <small class="stock-label">
                        {{ item.presaleMode === PRESALE_MODES.NONE ? `库存: ${item.stock}` : '预售商品' }}
                    </small>
                </div>
            </div>
        </div>
    </div>
    <div v-else style="padding: 4rem; text-align: center; color: #666;">
        <i class="fa fa-spinner fa-spin" style="font-size: 2rem; margin-bottom: 1rem;"></i>
        <p>正在读取商品数据...</p>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted } from 'vue'
import { useShopStore } from '@/stores/shopStore'

const store = useShopStore()
const PRESALE_MODES = Object.freeze({
    NONE: 'none',
    GOAL: 'goal',
    FIXED: 'fixed'
})
const PRODUCT_POLLING_INTERVAL_MS = 20000
let pollingTimer = null

onMounted(() => {
    store.fetchProducts()
    pollingTimer = window.setInterval(() => {
        store.fetchProducts()
    }, PRODUCT_POLLING_INTERVAL_MS)
})

onUnmounted(() => {
    if (pollingTimer) {
        clearInterval(pollingTimer)
        pollingTimer = null
    }
})

const filteredProducts = computed(() => {
    const currentType = store.state.currentType
    if (currentType === 'all') return store.state.products
    return store.state.products.filter(p => p.category === currentType)
})

const currentTypeName = computed(() => store.state.currentType === 'all' ? '全部' : store.state.currentType)

const hasDiscount = (item) => store.hasProductDiscount(item)
const getDisplayPrice = (item) => store.resolveProductPrice(item)
const getPresaleProgress = (item) => store.getPresaleProgress(item)
const getFixedPresaleDateText = (item) => store.formatFixedPresaleDate(item)
</script>
