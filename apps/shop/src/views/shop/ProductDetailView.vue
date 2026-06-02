<template>
  <div v-if="product">
    <div class="breadcrumb">
        <span @click="$router.push('/')">首页</span> &gt; 
        <span>{{ product.category }}</span> &gt; 
        <span style="color: var(--text-main); font-weight: bold;">详情</span>
    </div>

    <div class="detail-card">
        <div class="detail-layout">
            <div
                class="detail-image-box detail-main-image"
                role="button"
                tabindex="0"
                aria-label="查看商品主图大图"
                @click="openMainImagePreview"
                @keydown.enter.prevent="openMainImagePreview"
                @keydown.space.prevent="openMainImagePreview"
            >
                <picture>
                    <source v-if="product.imageMobile" media="(max-width: 767px)" :srcset="product.imageMobile">
                    <img :src="product.image" alt="商品主图">
                </picture>
            </div>
            <div class="detail-info">
                <div>
                    <h1 class="detail-title">{{ product.name }}</h1>
                    <p class="detail-desc">{{ product.desc }}</p>
                    <div class="price-box">
                        <span style="font-size: 0.875rem; color: var(--primary-color);">应援价</span>
                        <span class="price-current">¥{{ getDisplayPrice(product) }}</span>
                        <span v-if="hasDiscount(product)" class="price-original">¥{{ product.price }}</span>
                        <span class="tag-stock" :class="{ 'tag-stock-presale': isPresaleProduct }">
                            {{ isPresaleProduct ? '预售' : '现货' }}
                        </span>
                    </div>
                    <div v-if="isPresaleProduct" class="detail-presale-card">
                        <div class="detail-presale-title">
                            <i class="fa fa-hourglass-half"></i>
                            <span>预售信息</span>
                        </div>
                        <template v-if="product.presaleMode === PRESALE_MODES.GOAL">
                            <div class="detail-presale-row">
                                <span class="detail-presale-label">开做条件</span>
                                <span class="detail-presale-value">支付件数达到 {{ presaleProgress.target }}</span>
                            </div>
                            <div class="detail-presale-progress-track">
                                <span :style="{ width: `${presaleProgress.percent}%` }"></span>
                            </div>
                            <div class="detail-presale-progress-meta">
                                <span>已支付 {{ presaleProgress.paidCount }} / {{ presaleProgress.target }}</span>
                                <span>{{ presaleProgress.percent }}%</span>
                            </div>
                            <p class="detail-presale-note">
                                {{ presaleProgress.reached ? '已达到开做目标，正在推进生产。' : '进度会随着已支付订单实时更新。' }}
                            </p>
                        </template>
                        <template v-else-if="product.presaleMode === PRESALE_MODES.FIXED">
                            <div class="detail-presale-row">
                                <span class="detail-presale-label">开做时间</span>
                                <span class="detail-presale-value">{{ fixedPresaleDateText || '待设置' }}</span>
                            </div>
                            <p class="detail-presale-note">请留意后续发货通知。</p>
                        </template>
                    </div>
                    <!-- [修改] 动态渲染参数表 -->
                    <div style="margin-bottom: 2rem;">
                        <h3 style="font-size: 0.875rem; color: #999; margin-bottom: 0.75rem; text-transform: uppercase;">规格参数</h3>
                        <table class="specs-table" v-if="product.specs && product.specs.length">
                            <tbody>
                                <tr v-for="(spec, idx) in product.specs" :key="idx">
                                    <td>{{ spec.key }}</td>
                                    <td>{{ spec.val }}</td>
                                </tr>
                            </tbody>
                        </table>
                        <p v-else style="font-size: 0.8rem; color: #ccc;">暂无规格参数</p>
                    </div>
                </div>
                <div>
                    <div class="flex-row items-center gap-4 detail-qty-row" style="margin-bottom: 1.5rem;">
                        <span style="font-size: 0.875rem; color: #666;">数量</span>
                        <div class="quantity-control">
                            <button @click="quantity > 1 ? quantity-- : null" class="qty-btn">-</button>
                            <input type="text" :value="quantity" class="qty-input" readonly aria-label="购买数量">
                            <button @click="quantity++" class="qty-btn">+</button>
                        </div>
                        <span style="font-size: 0.75rem; color: #999;">
                            {{ isPresaleProduct ? '预售商品（不受库存限制）' : `库存 (${product.stock}件)` }}
                        </span>
                    </div>
                    
                    <div v-if="product.shippingCost > 0" style="margin-bottom: 1rem; font-size: 0.75rem; color: #666;">
                        <i class="fa fa-truck"></i> 基础运费: ¥{{ product.shippingCost }} 
                    </div>

                    <div class="action-row">
                        <button class="market-btn primary action-btn-group" style="padding: 0.75rem;" @click="buyNow">
                            <i class="fa fa-shopping-bag mr-2"></i> 立即购买
                        </button>
                        <button class="market-btn outline-blue action-btn-group" style="padding: 0.75rem;" @click="addToCart">
                            <i class="fa fa-cart-plus mr-2"></i> 加入购物车
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <div class="content-card">
        <div class="tabs-header">
            <div class="tab-item active">商品详情</div>
        </div>
        <div class="tab-content">
            <!-- [修改] 渲染详情文案和详情图 -->
            <div class="detail-text" v-if="product.detailText">
                <p style="white-space: pre-wrap;">{{ product.detailText }}</p>
            </div>
            <div class="detail-images">
                <img v-for="(img, idx) in product.detailImages" :key="idx" :src="img" alt="详情图">
            </div>
            <div v-if="!product.detailText && (!product.detailImages || product.detailImages.length===0)" style="text-align:center; color:#ccc; padding: 2rem;">
                暂无图文详情
            </div>
        </div>
    </div>

    <div v-if="mainImagePreview.show" class="detail-image-preview-overlay" @click.self="closeMainImagePreview">
        <div class="detail-image-preview-card">
            <button class="detail-image-preview-close" @click="closeMainImagePreview" aria-label="关闭预览">×</button>
            <img :src="mainImagePreview.url" :alt="`${product.name} 主图`" class="detail-image-preview-image">
        </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useShopStore } from '@/stores/shopStore'

const route = useRoute()
const router = useRouter()
const store = useShopStore()
const quantity = ref(1)
const PRESALE_MODES = Object.freeze({
    NONE: 'none',
    GOAL: 'goal',
    FIXED: 'fixed'
})
const PRODUCT_POLLING_INTERVAL_MS = 20000
let pollingTimer = null
const mainImagePreview = ref({
    show: false,
    url: ''
})

const openMainImagePreview = () => {
    const imageUrl = String(product.value?.imageOriginal || product.value?.image || '').trim()
    if (!imageUrl) return
    mainImagePreview.value = {
        show: true,
        url: imageUrl
    }
}

const closeMainImagePreview = () => {
    mainImagePreview.value.show = false
}

const onEscapeClosePreview = (event) => {
    if (event.key !== 'Escape') return
    if (!mainImagePreview.value.show) return
    closeMainImagePreview()
}

// 确保进入详情页时有数据
onMounted(() => {
    store.fetchProducts()
    pollingTimer = window.setInterval(() => {
        store.fetchProducts()
    }, PRODUCT_POLLING_INTERVAL_MS)
    window.addEventListener('keydown', onEscapeClosePreview)
})

onUnmounted(() => {
    if (pollingTimer) {
        clearInterval(pollingTimer)
        pollingTimer = null
    }
    window.removeEventListener('keydown', onEscapeClosePreview)
})

const product = computed(() => store.state.products.find(p => p.id == route.params.id))
const isPresaleProduct = computed(() => store.isPresaleProduct(product.value))
const presaleProgress = computed(() => store.getPresaleProgress(product.value))
const fixedPresaleDateText = computed(() => store.formatFixedPresaleDate(product.value))
const hasDiscount = (item) => store.hasProductDiscount(item)
const getDisplayPrice = (item) => store.resolveProductPrice(item)

const addToCart = () => {
    if(product.value) store.addToCart(product.value, quantity.value)
}

const buyNow = () => {
    addToCart()
    router.push('/cart')
}
</script>

<style scoped>
.detail-main-image {
    cursor: zoom-in;
}

.detail-main-image:focus-visible {
    outline: 2px solid #2563eb;
    outline-offset: 2px;
}

.detail-image-preview-overlay {
    position: fixed;
    inset: 0;
    z-index: 160;
    background: rgba(0, 0, 0, 0.72);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
}

.detail-image-preview-card {
    width: min(92vw, 520px);
    background: #fff;
    border-radius: 12px;
    padding: 1rem;
    text-align: center;
    position: relative;
}

.detail-image-preview-close {
    position: absolute;
    right: 0.75rem;
    top: 0.75rem;
    border: none;
    width: 2rem;
    height: 2rem;
    border-radius: 9999px;
    background: #111827;
    color: #fff;
    cursor: pointer;
    font-size: 1.1rem;
    line-height: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: opacity 0.2s ease;
}

.detail-image-preview-close:hover {
    opacity: 0.85;
}

.detail-image-preview-title {
    font-size: 0.95rem;
    color: #111827;
    font-weight: 700;
    margin: 0.25rem 0 0.75rem;
    padding-right: 2.5rem;
}

.detail-image-preview-image {
    width: 100%;
    max-height: min(78vh, 680px);
    object-fit: contain;
    border-radius: 10px;
    background: #fff;
}

.detail-image-preview-tip {
    margin-top: 0.6rem;
    font-size: 0.8rem;
    color: #6b7280;
}

@media (max-width: 639px) {
    .detail-qty-row {
        flex-wrap: wrap;
        gap: 0.6rem;
        align-items: center;
    }
}
</style>
