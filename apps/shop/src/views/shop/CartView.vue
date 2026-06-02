<template>
  <div>
    <div v-if="store.state.cart.length === 0" class="cart-empty">
        <i class="fa fa-shopping-basket cart-icon"></i>
        <p style="color: #666; margin-bottom: 1.5rem;">购物车空空如也，快去选购吧！</p>
        <button class="market-btn primary-action" @click="$router.push('/')">去逛逛</button>
    </div>
    <div v-else>
        <!-- Desktop: table layout (hidden on mobile) -->
        <div class="cart-table-wrapper cart-desktop">
            <table class="cart-table">
                <thead>
                    <tr><th style="padding-left: 1.5rem;">商品信息</th><th class="text-center">单价</th><th class="text-center">数量</th><th class="text-center">小计</th><th class="text-center">操作</th></tr>
                </thead>
                <tbody>
                    <tr v-for="(item, index) in store.state.cart" :key="item.id">
                        <td style="padding-left: 1.5rem;">
                            <div class="product-cell">
                                <img :src="item.image" class="thumb-img">
                                <span style="font-weight: bold; color: #374151;">{{ item.name }}</span>
                            </div>
                        </td>
                        <td class="text-center" style="color: #4b5563;">
                            <div style="display: inline-flex; flex-direction: column; align-items: center; line-height: 1.2;">
                                <span>¥{{ item.price }}</span>
                                <span
                                    v-if="Number(item.originalPrice) > Number(item.price)"
                                    style="font-size: 0.75rem; color: #9ca3af; text-decoration: line-through;"
                                >
                                    ¥{{ item.originalPrice }}
                                </span>
                            </div>
                        </td>
                        <td class="text-center">
                            <div class="quantity-control" style="display: inline-flex;">
                                <button @click="updateQuantity(index, -1)" class="qty-btn" style="width: 2rem; height: 2rem;">-</button>
                                <span class="qty-input" style="width: 2.5rem; height: 2rem; line-height: 2rem; padding: 0;">{{ item.quantity }}</span>
                                <button @click="updateQuantity(index, 1)" class="qty-btn" style="width: 2rem; height: 2rem;">+</button>
                            </div>
                        </td>
                        <td class="text-center" style="font-weight: bold; color: var(--primary-color);">¥{{ item.price * item.quantity }}</td>
                        <td class="text-center">
                            <button @click="store.removeFromCart(index)" style="color: #f87171; background: none; border: none; cursor: pointer;"><i class="fa fa-trash"></i></button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- Mobile: card layout (hidden on desktop) -->
        <div class="cart-mobile">
            <div v-for="(item, index) in store.state.cart" :key="item.id" class="cart-card">
                <div class="cart-card-top">
                    <img :src="item.image" class="cart-card-img">
                    <div class="cart-card-info">
                        <span class="cart-card-name">{{ item.name }}</span>
                        <div class="cart-card-price">
                            <span>¥{{ item.price }}</span>
                            <span v-if="Number(item.originalPrice) > Number(item.price)" class="cart-card-original">¥{{ item.originalPrice }}</span>
                        </div>
                    </div>
                    <button @click="store.removeFromCart(index)" class="cart-card-delete"><i class="fa fa-trash"></i></button>
                </div>
                <div class="cart-card-bottom">
                    <div class="quantity-control" style="display: inline-flex;">
                        <button @click="updateQuantity(index, -1)" class="qty-btn">-</button>
                        <span class="qty-input" style="width: 2.5rem; line-height: 2rem; padding: 0;">{{ item.quantity }}</span>
                        <button @click="updateQuantity(index, 1)" class="qty-btn">+</button>
                    </div>
                    <span class="cart-card-subtotal">¥{{ item.price * item.quantity }}</span>
                </div>
            </div>
        </div>

        <div class="cart-footer">
            <div style="font-size: 0.875rem; color: #666;"><i class="fa fa-info-circle mr-1"></i> 满{{ store.freeShippingThreshold }}元包邮，当前运费: ¥{{ store.shippingFee }}</div>
            <div class="cart-footer-right">
                <div style="text-align: right;">
                    <span style="color: #666; margin-right: 0.5rem;">合计 (不含运费):</span>
                    <span style="font-size: 1.5rem; font-weight: bold; color: #1f2937;">¥{{ store.cartTotal }}</span>
                </div>
                <button class="market-btn primary-action" style="padding: 0.5rem 2rem; font-size: 1.125rem;" @click="$router.push('/checkout')">
                    去结算 <i class="fa fa-arrow-right ml-2" style="font-size: 0.875rem;"></i>
                </button>
            </div>
        </div>
    </div>
  </div>
</template>

<script setup>
import { useShopStore } from '@/stores/shopStore'
const store = useShopStore()

const updateQuantity = (index, delta) => {
    const item = store.state.cart[index]
    const newQty = item.quantity + delta
    if (newQty > 0) item.quantity = newQty
}
</script>

<style scoped>
/* Mobile card layout: hidden on desktop */
.cart-mobile { display: none; }

/* Desktop table: shown by default */
.cart-desktop { display: block; }

.cart-footer-right { display: flex; align-items: center; gap: 1rem; }

@media (max-width: 767px) {
    .cart-desktop { display: none; }
    .cart-mobile { display: flex; flex-direction: column; gap: 0.75rem; margin-bottom: 1.5rem; }
    .cart-footer-right { flex-direction: column; align-items: stretch; text-align: center; }

    .cart-card {
        background: white;
        border-radius: var(--radius-md);
        box-shadow: var(--shadow-sm);
        padding: 1rem;
    }
    .cart-card-top {
        display: flex;
        align-items: flex-start;
        gap: 0.75rem;
    }
    .cart-card-img {
        width: 4rem;
        height: 4rem;
        object-fit: cover;
        border-radius: 4px;
        background: #eee;
        flex-shrink: 0;
    }
    .cart-card-info {
        flex: 1;
        min-width: 0;
    }
    .cart-card-name {
        font-weight: bold;
        color: #374151;
        display: block;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .cart-card-price {
        color: #4b5563;
        font-size: 0.875rem;
        margin-top: 0.25rem;
    }
    .cart-card-original {
        font-size: 0.75rem;
        color: #9ca3af;
        text-decoration: line-through;
        margin-left: 0.5rem;
    }
    .cart-card-delete {
        color: #f87171;
        background: none;
        border: none;
        cursor: pointer;
        padding: 0.5rem;
        min-width: 44px;
        min-height: 44px;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .cart-card-bottom {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-top: 0.75rem;
        padding-top: 0.75rem;
        border-top: 1px solid #f3f4f6;
    }
    .cart-card-subtotal {
        font-weight: bold;
        color: var(--primary-color);
        font-size: 1.05rem;
    }
}
</style>
