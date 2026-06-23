<template>
  <div class="home-view">
    <div class="page-header">
        <h2 class="page-title">{{ currentTypeName }}列表</h2>
        <span class="page-result-count">共找到 {{ filteredProducts.length }} 件宝物</span>
    </div>

    <div v-if="filteredProducts.length > 0" class="product-grid">
        <div
            v-for="item in filteredProducts"
            :key="item.id"
            class="product-card group sos-card sos-product-card sos-card--interactive"
            @click="$router.push(`/product/${item.id}`)"
        >
            <div class="card-image-wrapper sos-product-card__media">
                <picture>
                    <source v-if="item.imageMobile" media="(max-width: 639px)" :srcset="item.imageMobile">
                    <img :src="item.image" :alt="item.name" class="card-image">
                </picture>
                <div class="card-overlay">
                    <button class="market-btn sos-button sos-button--primary sos-button--sm" @click.stop="store.addToCart(item)">加入购物车</button>
                </div>
            </div>
            <div class="card-body sos-card__body">
                <div class="card-title-row sos-product-card__title-row">
                    <h3 class="card-title sos-product-card__title">{{ item.name }}</h3>
                    <div class="card-price-wrap">
                        <span class="card-price sos-product-card__price">¥{{ getDisplayPrice(item) }}</span>
                        <span v-if="hasDiscount(item)" class="card-price-original">¥{{ item.price }}</span>
                    </div>
                </div>
                <p class="card-desc sos-product-card__description">{{ item.desc }}</p>
                <div v-if="item.presaleMode === PRESALE_MODES.GOAL" class="card-presale-box sos-progress">
                    <div class="card-presale-head sos-progress__meta">
                        <span class="presale-chip presale-chip-goal sos-badge sos-badge--accent">进度预售</span>
                        <span class="card-presale-count">
                            {{ getPresaleProgress(item).paidCount }}/{{ getPresaleProgress(item).target }}
                        </span>
                    </div>
                    <div class="card-presale-track sos-progress__track">
                        <span class="sos-progress__fill" :style="{ width: `${getPresaleProgress(item).percent}%` }"></span>
                    </div>
                    <p class="card-presale-tip">
                        {{ getPresaleProgress(item).reached ? '已达标' : '订单持续累计中' }}
                    </p>
                </div>
                <div v-else-if="item.presaleMode === PRESALE_MODES.FIXED" class="card-presale-box sos-progress">
                    <div class="card-presale-head sos-progress__meta">
                        <span class="presale-chip presale-chip-fixed sos-badge sos-badge--accent">排期预售</span>
                    </div>
                    <p class="card-presale-tip">预售发货时间：{{ getFixedPresaleDateText(item) || '待设置' }}</p>
                </div>
                <div class="card-footer sos-card__footer">
                    <span class="badge-tag sos-badge sos-badge--accent">{{ item.category }}</span>
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
