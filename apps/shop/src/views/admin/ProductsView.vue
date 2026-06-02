<template>
  <div class="panel">
    <div class="toolbar">
        <h3 style="font-weight: bold; color: #374151; margin: 0;">商品库管理</h3>
        <button class="admin-btn btn-green" @click="openModal()">
            <i class="fa fa-plus"></i> 新增商品
        </button>
    </div>
    <div class="table-container">
        <table class="data-table">
            <thead>
                <tr><th style="width:2rem"></th><th>ID</th><th>图片</th><th>名称</th><th>分类</th><th>价格</th><th>库存</th><th>预售</th><th>发货组</th><th>运费</th><th>操作</th></tr>
            </thead>
            <tbody>
                <tr
                    v-for="(p, idx) in products" :key="p.id"
                    :class="{ 'row-drag-over': rowDragOverIdx === idx }"
                    @dragover.prevent="rowDragOverIdx = idx"
                    @dragleave="rowDragOverIdx = -1"
                    @drop.prevent="onRowDrop(idx)"
                >
                    <td class="drag-handle" draggable="true" @dragstart="onRowDragStart(idx, $event)" @dragend="onRowDragEnd"><i class="fa fa-grip-vertical"></i></td>
                    <td class="text-sub">#{{ p.id }}</td>
                    <td><img :src="p.image" style="width: 40px; height: 40px; object-fit: cover; border-radius: 4px;"></td>
                    <td style="font-weight: 500;">{{ p.name }}</td>
                    <td><span class="status-badge status-2">{{ p.category }}</span></td>
                    <td>
                        <div style="font-weight: bold;">¥{{ getDisplayPrice(p) }}</div>
                        <div v-if="hasDiscount(p)" class="text-sub" style="text-decoration: line-through;">原价 ¥{{ p.price }}</div>
                    </td>
                    <td :style="{color: p.stock < 10 ? 'red' : 'inherit', fontWeight: 'bold'}">{{ p.stock }}</td>
                    <td class="text-sub">
                        <div class="presale-table-main">{{ getPresaleModeLabel(p) }}</div>
                        <div v-if="getPresaleSummary(p)" class="presale-table-sub">{{ getPresaleSummary(p) }}</div>
                    </td>
                    <td class="text-sub">{{ p.shippingTag }}</td>
                    <td class="text-sub">¥{{ p.shippingCost }}</td>
                    <td>
                        <button class="admin-btn btn-blue" style="font-size: 0.75rem; margin-right: 0.5rem;" @click="openModal(p)">编辑</button>
                        <button class="admin-btn btn-outline" style="font-size: 0.75rem; color: #ef4444; border-color: #fca5a5;" @click="store.deleteProduct(p.id)">删除</button>
                    </td>
                </tr>
            </tbody>
        </table>
    </div>

    <!-- 编辑/新增 弹窗 (大) -->
    <div v-if="showModal" class="modal-overlay">
        <div class="modal-card product-modal-card">
            <h3 class="modal-title">{{ isEdit ? '编辑商品' : '新增商品' }}</h3>
            <div class="form-grid product-form-grid">

                <!-- 第一行：基础信息 -->
                <div><label class="form-label">名称</label><input v-model="form.name" class="form-input"></div>
                <div><label class="form-label">分类名</label><input v-model="form.category" class="form-input" placeholder="如: 立牌、卡贴、折扇"></div>
                <div>
                    <label class="form-label">发货标签 (分组)</label>
                    <select v-model="form.shippingTag" class="form-select">
                        <option value="深圳">深圳</option>
                        <option value="上海">上海</option>
                        <option value="柔造">柔造</option>
                        <option value="蚌埠">蚌埠</option>
                    </select>
                </div>

                <!-- 第二行：价格库存 -->
                <div><label class="form-label">价格 (¥)</label><input v-model.number="form.price" class="form-input" type="number"></div>
                <div><label class="form-label">折扣价 (¥)</label><input v-model="form.discountPrice" class="form-input" type="number" min="0" step="0.01" placeholder="留空表示不打折"></div>
                <div v-if="!isEdit"><label class="form-label">库存</label><input v-model.number="form.stock" class="form-input" type="number"></div>
                <div v-else>
                    <label class="form-label">当前库存</label>
                    <input :value="form.stock" class="form-input input-readonly" type="text" readonly>
                </div>

                <!-- 第三行：预售配置 -->
                <div class="full-span presale-config-wrap">
                    <div class="presale-config-head">
                        <label class="form-label" style="margin: 0;">预售设置</label>
                        <span class="presale-config-hint">支持进度达标开做与固定预售日期两种模式</span>
                    </div>
                    <div class="presale-config-grid">
                        <div>
                            <label class="form-label">预售模式</label>
                            <select v-model="form.presaleMode" class="form-select">
                                <option :value="PRESALE_MODES.NONE">普通现货</option>
                                <option :value="PRESALE_MODES.GOAL">目标达标预售</option>
                                <option :value="PRESALE_MODES.FIXED">固定日期预售</option>
                            </select>
                        </div>
                        <div v-if="form.presaleMode === PRESALE_MODES.GOAL">
                            <label class="form-label">目标支付件数</label>
                            <input v-model.number="form.presaleGoalTarget" class="form-input" type="number" min="1" step="1" placeholder="如 100">
                        </div>
                        <template v-if="form.presaleMode === PRESALE_MODES.FIXED">
                            <div>
                                <label class="form-label">固定日期类型</label>
                                <select v-model="form.presaleFixedDateType" class="form-select">
                                    <option :value="PRESALE_FIXED_DATE_TYPES.MONTH_START">某月初</option>
                                    <option :value="PRESALE_FIXED_DATE_TYPES.MONTH_END">某月底</option>
                                    <option :value="PRESALE_FIXED_DATE_TYPES.DATE">具体日期</option>
                                </select>
                            </div>
                            <div>
                                <label class="form-label">固定日期值</label>
                                <input
                                    v-model="form.presaleFixedDateValue"
                                    class="form-input"
                                    :type="fixedDateInputType"
                                    :placeholder="form.presaleFixedDateType === PRESALE_FIXED_DATE_TYPES.DATE ? 'YYYY-MM-DD' : 'YYYY-MM'"
                                >
                            </div>
                        </template>
                    </div>
                </div>

                <div v-if="isEdit" class="full-span presale-config-wrap">
                    <div class="presale-config-head">
                        <label class="form-label" style="margin: 0;">数量调整（增减）</label>
                        <span class="presale-config-hint">填正数表示增加，负数表示减少，留空或 0 表示不调整</span>
                    </div>
                    <div class="presale-config-grid">
                        <div>
                            <label class="form-label">库存调整</label>
                            <input v-model.number="form.stockAdjustDelta" class="form-input" type="number" step="1" placeholder="如 +30 或 -10">
                            <div class="adjust-current-text">当前库存：{{ form.stock }}</div>
                        </div>
                        <div>
                            <label class="form-label">预售进度调整</label>
                            <input
                                v-model.number="form.presalePaidAdjustDelta"
                                class="form-input"
                                type="number"
                                step="1"
                                placeholder="如 +20 或 -5"
                                :disabled="form.presaleMode !== PRESALE_MODES.GOAL"
                            >
                            <div class="adjust-current-text" v-if="form.presaleMode === PRESALE_MODES.GOAL">
                                当前进度：{{ form.presalePaidCount }}
                            </div>
                            <div class="adjust-current-text" v-else>仅目标达标预售支持进度调整</div>
                        </div>
                    </div>
                </div>

                <!-- 第四行：运费 -->
                <div><label class="form-label">单品运费 (¥)</label><input v-model.number="form.shippingCost" class="form-input" type="number"></div>

                <!-- 第五行：主图上传 -->
                <div class="full-span">
                    <label class="form-label">商品主图</label>
                    <div class="main-image-row">
                        <div v-if="form.image" class="thumb-group">
                            <div class="thumb-item">
                                <img :src="form.image" class="thumb-img thumb-desktop">
                                <span class="thumb-label">桌面端</span>
                            </div>
                            <div v-if="form.imageMobile" class="thumb-item">
                                <img :src="form.imageMobile" class="thumb-img thumb-mobile">
                                <span class="thumb-label">移动端</span>
                            </div>
                        </div>
                        <input type="file" @change="e => handleUpload(e, 'image')" accept="image/*">
                    </div>
                </div>

                <!-- 第六行：描述 -->
                <div class="full-span">
                    <label class="form-label">短描述 (列表页显示)</label>
                    <input v-model="form.desc" class="form-input">
                </div>

                <!-- 第七行：参数表 (Key-Value 编辑) -->
                <div class="full-span" style="background: #f9fafb; padding: 1rem; border-radius: 8px;">
                    <label class="form-label" style="font-weight: bold;">规格参数表</label>
                    <div v-for="(spec, idx) in form.specs" :key="idx" style="display: flex; gap: 0.5rem; margin-bottom: 0.5rem;">
                        <input v-model="spec.key" placeholder="参数名" class="form-input" style="flex:1; margin:0;">
                        <input v-model="spec.val" placeholder="参数值" class="form-input" style="flex:2; margin:0;">
                        <button @click="removeSpec(idx)" class="admin-btn btn-outline" style="color: red; padding: 0 0.5rem;">×</button>
                    </div>
                    <button @click="addSpec" class="admin-btn btn-outline" style="font-size: 0.75rem;">+ 添加参数</button>
                </div>

                <!-- 第八行：详情页文案 -->
                <div class="full-span">
                    <label class="form-label">详情页长文案</label>
                    <textarea v-model="form.detailText" class="form-input" style="height: 100px;"></textarea>
                </div>

                <!-- 第九行：详情多图上传 -->
                <div class="full-span">
                    <label class="form-label">详情页图片组 <span style="font-weight: normal; color: #999; font-size: 0.75rem;">拖拽可排序</span></label>
                    <div class="detail-images-sortable">
                        <div
                            v-for="(img, idx) in form.detailImages" :key="img"
                            class="detail-img-item"
                            draggable="true"
                            :class="{ 'drag-over': dragOverIdx === idx }"
                            @dragstart="onDragStart(idx, $event)"
                            @dragover.prevent="onDragOver(idx)"
                            @dragleave="onDragLeave"
                            @drop.prevent="onDrop(idx)"
                            @dragend="onDragEnd"
                        >
                            <img :src="img" style="width: 80px; height: 80px; object-fit: cover; border-radius: 4px;">
                            <button @click="removeDetailImg(idx)" class="detail-img-remove">×</button>
                        </div>
                    </div>
                    <input type="file" @change="e => handleUpload(e, 'detail')" accept="image/*" multiple>
                </div>

            </div>
            <div class="modal-actions">
                <button @click="showModal = false" class="admin-btn btn-outline">取消</button>
                <button @click="save" class="admin-btn btn-blue">保存</button>
            </div>
        </div>
    </div>

    <!-- 裁切弹窗 -->
    <div v-if="showCropModal" class="modal-overlay" style="z-index: 1100;">
        <div class="modal-card crop-modal-card">
            <h3 class="modal-title">
                {{ cropStep === 1 ? '步骤 1/2：裁切桌面端头图 (6:5)' : '步骤 2/2：裁切移动端头图 (1:1.12)' }}
            </h3>
            <div class="crop-container">
                <img ref="cropImageEl" :src="cropImageSrc">
            </div>
            <div class="modal-actions">
                <button @click="cancelCrop" class="admin-btn btn-outline">取消</button>
                <button v-if="cropStep === 2" @click="skipMobileCrop" class="admin-btn btn-outline">跳过移动端</button>
                <button @click="confirmCrop" class="admin-btn btn-blue" :disabled="cropUploading">
                    {{ cropUploading ? '上传中...' : '确认裁切' }}
                </button>
            </div>
        </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, reactive, nextTick, watch, onBeforeUnmount } from 'vue'
import { useShopStore } from '@/stores/shopStore'
import Cropper from 'cropperjs'
import 'cropperjs/dist/cropper.css'

const store = useShopStore()
const products = computed(() => store.state.products)
const showModal = ref(false)
const isEdit = ref(false)
const DETAIL_IMAGE_COMPRESSION_OPTIONS = Object.freeze({
    maxDimension: 2000,
    quality: 0.82
})
const HEADER_IMAGE_COMPRESSION_CONFIG = Object.freeze({
    desktop: {
        maxWidth: 1800,
        maxHeight: 1800,
        quality: 0.84
    },
    mobile: {
        maxWidth: 1500,
        maxHeight: 1700,
        quality: 0.82
    },
    original: {
        quality: 0.88
    }
})
const PRESALE_MODES = Object.freeze({
    NONE: 'none',
    GOAL: 'goal',
    FIXED: 'fixed'
})
const PRESALE_FIXED_DATE_TYPES = Object.freeze({
    MONTH_START: 'month_start',
    MONTH_END: 'month_end',
    DATE: 'date'
})
const PRESALE_FIXED_DATE_TYPE_VALUES = new Set(Object.values(PRESALE_FIXED_DATE_TYPES))
const PRESALE_MONTH_PATTERN = /^\d{4}-(0[1-9]|1[0-2])$/
const PRESALE_DATE_PATTERN = /^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$/

// 表单初始状态
const initialForm = {
    id: null, name: '', price: 0, category: '', stock: 100,
    discountPrice: '',
    image: '', imageMobile: '', imageOriginal: '', desc: '',
    specs: [], detailText: '', detailImages: [],
    shippingTag: '深圳', shippingCost: 0,
    presaleMode: PRESALE_MODES.NONE,
    presaleGoalTarget: '',
    presaleFixedDateType: PRESALE_FIXED_DATE_TYPES.MONTH_START,
    presaleFixedDateValue: '',
    presalePaidCount: 0,
    stockAdjustDelta: 0,
    presalePaidAdjustDelta: 0
}

const form = reactive({ ...initialForm })

onMounted(() => { store.fetchProducts() })

const openModal = (product = null) => {
    if (product) {
        isEdit.value = true
        const p = JSON.parse(JSON.stringify(product))
        Object.assign(form, p)
        form.discountPrice = p.discountPrice ?? ''
        form.imageMobile = p.imageMobile || ''
        form.imageOriginal = p.imageOriginal || ''
        form.presaleMode = p.presaleMode || PRESALE_MODES.NONE
        form.presaleGoalTarget = Number(p.presaleGoalTarget) > 0 ? Number(p.presaleGoalTarget) : ''
        form.presaleFixedDateType = p.presaleFixedDateType || PRESALE_FIXED_DATE_TYPES.MONTH_START
        form.presaleFixedDateValue = p.presaleFixedDateValue || ''
        form.presalePaidCount = Number(p.presalePaidCount) || 0
        form.stockAdjustDelta = 0
        form.presalePaidAdjustDelta = 0
        if (!form.specs) form.specs = []
        if (!form.detailImages) form.detailImages = []
    } else {
        isEdit.value = false
        Object.assign(form, JSON.parse(JSON.stringify(initialForm)))
    }
    showModal.value = true
}

// --- 裁切相关 ---
const showCropModal = ref(false)
const cropStep = ref(1) // 1=桌面端6:5, 2=移动端1:1.12
const cropImageSrc = ref('')
const cropImageEl = ref(null)
const cropUploading = ref(false)
let cropperInstance = null
let pendingFileInput = null
let pendingImageOriginalUrl = ''

const destroyCropper = () => {
    if (cropperInstance) {
        cropperInstance.destroy()
        cropperInstance = null
    }
}

onBeforeUnmount(() => { destroyCropper() })

const initCropper = (aspectRatio) => {
    destroyCropper()
    nextTick(() => {
        if (!cropImageEl.value) return
        cropperInstance = new Cropper(cropImageEl.value, {
            aspectRatio,
            viewMode: 1,
            autoCropArea: 1,
            responsive: true,
            restore: false
        })
    })
}

watch(showCropModal, (v) => {
    if (!v) destroyCropper()
})

watch(() => form.presaleMode, (mode) => {
    if (mode !== PRESALE_MODES.GOAL) {
        form.presaleGoalTarget = ''
        form.presalePaidAdjustDelta = 0
    }
    if (mode !== PRESALE_MODES.FIXED) {
        form.presaleFixedDateType = PRESALE_FIXED_DATE_TYPES.MONTH_START
        form.presaleFixedDateValue = ''
    }
})

watch(() => form.presaleFixedDateType, (type) => {
    if (form.presaleMode !== PRESALE_MODES.FIXED) return
    if (!PRESALE_FIXED_DATE_TYPE_VALUES.has(type)) {
        form.presaleFixedDateType = PRESALE_FIXED_DATE_TYPES.MONTH_START
        form.presaleFixedDateValue = ''
        return
    }
    const value = String(form.presaleFixedDateValue || '').trim()
    if (!value) return
    if (type === PRESALE_FIXED_DATE_TYPES.DATE && PRESALE_MONTH_PATTERN.test(value)) {
        form.presaleFixedDateValue = `${value}-01`
        return
    }
    if (
        (type === PRESALE_FIXED_DATE_TYPES.MONTH_START || type === PRESALE_FIXED_DATE_TYPES.MONTH_END)
        && PRESALE_DATE_PATTERN.test(value)
    ) {
        form.presaleFixedDateValue = value.slice(0, 7)
    }
})

const fixedDateInputType = computed(() => (
    form.presaleFixedDateType === PRESALE_FIXED_DATE_TYPES.DATE ? 'date' : 'month'
))

const encodeCanvasToWebp = (canvas, quality) => (
    new Promise((resolve) => {
        canvas.toBlob((blob) => resolve(blob), 'image/webp', quality)
    })
)

const getCroppedBlob = async (step) => {
    if (!cropperInstance) return null
    const isDesktopCrop = step === 1
    const sizeConfig = isDesktopCrop
        ? HEADER_IMAGE_COMPRESSION_CONFIG.desktop
        : HEADER_IMAGE_COMPRESSION_CONFIG.mobile

    let canvas = cropperInstance.getCroppedCanvas({
        maxWidth: sizeConfig.maxWidth,
        maxHeight: sizeConfig.maxHeight,
        imageSmoothingEnabled: true,
        imageSmoothingQuality: 'high'
    })
    if (!canvas) return null

    const blob = await encodeCanvasToWebp(canvas, sizeConfig.quality)
    if (!blob) return null
    if (String(blob.type || '').toLowerCase() !== 'image/webp') return null
    return blob
}

const uploadBlob = async (blob, suffix) => {
    if (String(blob?.type || '').toLowerCase() !== 'image/webp') return null
    const file = new File([blob], `crop-${suffix}-${Date.now()}.webp`, { type: 'image/webp' })
    return await store.uploadImage(file, { convertToWebp: false })
}

const handleUpload = async (e, type) => {
    const file = e.target.files[0]
    if (!file) return

    if (type === 'detail') {
        const files = Array.from(e.target.files)
        e.target.value = ''
        for (const f of files) {
            const url = await store.uploadImage(f, DETAIL_IMAGE_COMPRESSION_OPTIONS)
            if (url) form.detailImages.push(url)
        }
        return
    }

    const originalUrl = await store.uploadImage(file, {
        purpose: 'original',
        convertToWebp: true,
        quality: HEADER_IMAGE_COMPRESSION_CONFIG.original.quality
    })
    if (!originalUrl) {
        e.target.value = ''
        return
    }
    pendingImageOriginalUrl = originalUrl

    // 主图 → 打开裁切弹窗
    const reader = new FileReader()
    reader.onload = (ev) => {
        cropImageSrc.value = ev.target.result
        cropStep.value = 1
        showCropModal.value = true
        nextTick(() => initCropper(6 / 5))
    }
    reader.readAsDataURL(file)
    pendingFileInput = e.target
}

const confirmCrop = async () => {
    cropUploading.value = true
    try {
        const blob = await getCroppedBlob(cropStep.value)
        if (!blob) {
            store.showNotification('WebP 编码失败，请更换浏览器后重试')
            return
        }
        if (String(blob.type || '').toLowerCase() !== 'image/webp') {
            store.showNotification('当前环境不支持 WebP 编码，请使用最新版 Chrome/Edge/Safari')
            return
        }

        if (cropStep.value === 1) {
            // 桌面端裁切完成 → 上传 → 进入步骤2
            const url = await uploadBlob(blob, 'desktop')
            if (!url) return
            form.image = url
            if (pendingImageOriginalUrl) form.imageOriginal = pendingImageOriginalUrl
            // 进入移动端裁切
            cropStep.value = 2
            destroyCropper()
            await nextTick()
            initCropper(1 / 1.12)
        } else {
            // 移动端裁切完成 → 上传 → 关闭弹窗
            const url = await uploadBlob(blob, 'mobile')
            if (!url) return
            form.imageMobile = url
            closeCropModal()
        }
    } finally {
        cropUploading.value = false
    }
}

const skipMobileCrop = () => {
    form.imageMobile = ''
    closeCropModal()
}

const cancelCrop = () => {
    closeCropModal()
}

const closeCropModal = () => {
    showCropModal.value = false
    cropImageSrc.value = ''
    destroyCropper()
    pendingImageOriginalUrl = ''
    if (pendingFileInput) {
        pendingFileInput.value = ''
        pendingFileInput = null
    }
}

// --- 商品行拖拽排序 ---
const rowDragFromIdx = ref(-1)
const rowDragOverIdx = ref(-1)

const onRowDragStart = (idx, e) => {
    rowDragFromIdx.value = idx
    e.dataTransfer.effectAllowed = 'move'
}
const onRowDragEnd = () => {
    rowDragFromIdx.value = -1
    rowDragOverIdx.value = -1
}
const onRowDrop = async (toIdx) => {
    const fromIdx = rowDragFromIdx.value
    rowDragFromIdx.value = -1
    rowDragOverIdx.value = -1
    if (fromIdx === -1 || fromIdx === toIdx) return
    const list = [...store.state.products]
    const [moved] = list.splice(fromIdx, 1)
    list.splice(toIdx, 0, moved)
    // 乐观更新 UI
    store.state.products = list
    // 持久化
    const order = list.map((p, i) => ({ id: p.id, sortOrder: i }))
    await store.reorderProducts(order)
}

// --- 详情图拖拽排序 ---
const dragFromIdx = ref(-1)
const dragOverIdx = ref(-1)

const onDragStart = (idx, e) => {
    dragFromIdx.value = idx
    e.dataTransfer.effectAllowed = 'move'
}
const onDragOver = (idx) => { dragOverIdx.value = idx }
const onDragLeave = () => { dragOverIdx.value = -1 }
const onDrop = (idx) => {
    const from = dragFromIdx.value
    if (from !== -1 && from !== idx) {
        const item = form.detailImages.splice(from, 1)[0]
        form.detailImages.splice(idx, 0, item)
    }
    dragFromIdx.value = -1
    dragOverIdx.value = -1
}
const onDragEnd = () => {
    dragFromIdx.value = -1
    dragOverIdx.value = -1
}

const addSpec = () => form.specs.push({ key: '', val: '' })
const removeSpec = (idx) => form.specs.splice(idx, 1)
const removeDetailImg = (idx) => form.detailImages.splice(idx, 1)

const hasDiscount = (product) => {
    const base = Number(product?.price)
    const discount = Number(product?.discountPrice)
    return Number.isFinite(discount) && discount > 0 && (!Number.isFinite(base) || discount < base)
}

const getDisplayPrice = (product) => (hasDiscount(product) ? Number(product.discountPrice) : Number(product.price))
const parseAdjustDelta = (value) => {
    if (value === '' || value === null || value === undefined) return 0
    const numeric = Number(value)
    if (!Number.isInteger(numeric)) return null
    return numeric
}
const getPresaleModeLabel = (product) => {
    const mode = String(product?.presaleMode || '').trim().toLowerCase()
    if (mode === PRESALE_MODES.GOAL) return '达标预售'
    if (mode === PRESALE_MODES.FIXED) return '固定预售'
    return '现货'
}
const getPresaleSummary = (product) => {
    const mode = String(product?.presaleMode || '').trim().toLowerCase()
    if (mode === PRESALE_MODES.GOAL) {
        const progress = store.getPresaleProgress(product)
        if (!progress.target) return ''
        return `${progress.paidCount}/${progress.target}${progress.reached ? '（已达标）' : ''}`
    }
    if (mode === PRESALE_MODES.FIXED) {
        return store.formatFixedPresaleDate(product)
    }
    return ''
}

const save = async () => {
    const payload = { ...form }
    const stockAdjustDelta = isEdit.value ? parseAdjustDelta(payload.stockAdjustDelta) : 0
    const presalePaidAdjustDelta = isEdit.value ? parseAdjustDelta(payload.presalePaidAdjustDelta) : 0
    if (isEdit.value && (stockAdjustDelta === null || presalePaidAdjustDelta === null)) {
        store.showNotification('调整数量必须为整数')
        return
    }

    delete payload.presalePaidCount
    delete payload.stockAdjustDelta
    delete payload.presalePaidAdjustDelta
    delete payload.presalePaidCountBase
    delete payload.presalePaidOffset

    if (payload.discountPrice === '' || payload.discountPrice === null || payload.discountPrice === undefined) {
        payload.discountPrice = null
    } else {
        payload.discountPrice = Number(payload.discountPrice)
    }

    if (payload.discountPrice !== null) {
        if (!Number.isFinite(payload.discountPrice) || payload.discountPrice <= 0) {
            store.showNotification('折扣价必须大于0')
            return
        }
        if (payload.discountPrice >= Number(payload.price)) {
            store.showNotification('折扣价需小于原价')
            return
        }
    }

    payload.presaleMode = String(payload.presaleMode || PRESALE_MODES.NONE).trim().toLowerCase()
    if (payload.presaleMode === PRESALE_MODES.GOAL) {
        const target = Number.parseInt(payload.presaleGoalTarget, 10)
        if (!Number.isInteger(target) || target <= 0) {
            store.showNotification('预售目标数量必须为大于0的整数')
            return
        }
        payload.presaleGoalTarget = target
        payload.presaleFixedDateType = ''
        payload.presaleFixedDateValue = ''
    } else if (payload.presaleMode === PRESALE_MODES.FIXED) {
        const fixedDateType = String(payload.presaleFixedDateType || '').trim().toLowerCase()
        const fixedDateValue = String(payload.presaleFixedDateValue || '').trim()
        if (!PRESALE_FIXED_DATE_TYPE_VALUES.has(fixedDateType)) {
            store.showNotification('请选择固定预售日期类型')
            return
        }
        if (!fixedDateValue) {
            store.showNotification('请填写固定预售日期')
            return
        }
        if (
            (fixedDateType === PRESALE_FIXED_DATE_TYPES.MONTH_START || fixedDateType === PRESALE_FIXED_DATE_TYPES.MONTH_END)
            && !PRESALE_MONTH_PATTERN.test(fixedDateValue)
        ) {
            store.showNotification('固定预售月份格式应为 YYYY-MM')
            return
        }
        if (fixedDateType === PRESALE_FIXED_DATE_TYPES.DATE && !PRESALE_DATE_PATTERN.test(fixedDateValue)) {
            store.showNotification('固定预售日期格式应为 YYYY-MM-DD')
            return
        }
        payload.presaleGoalTarget = 0
        payload.presaleFixedDateType = fixedDateType
        payload.presaleFixedDateValue = fixedDateValue
    } else {
        payload.presaleMode = PRESALE_MODES.NONE
        payload.presaleGoalTarget = 0
        payload.presaleFixedDateType = ''
        payload.presaleFixedDateValue = ''
    }
    const effectivePresalePaidAdjustDelta = payload.presaleMode === PRESALE_MODES.GOAL ? presalePaidAdjustDelta : 0

    let success = false
    if (isEdit.value) {
        delete payload.stock
        success = await store.updateProduct(form.id, payload)
        if (success && (stockAdjustDelta !== 0 || effectivePresalePaidAdjustDelta !== 0)) {
            success = await store.adjustProductMetrics(form.id, {
                stockDelta: stockAdjustDelta,
                presalePaidDelta: effectivePresalePaidAdjustDelta
            })
        }
    } else {
        success = await store.addProduct(payload)
    }
    if (success) showModal.value = false
}
</script>

<style scoped>
.form-grid { display: grid; }
.product-modal-card {
    width: min(960px, 94vw);
    max-height: 90vh;
    overflow-y: auto;
}
.product-form-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1rem;
}
.full-span {
    grid-column: 1 / -1;
}
.main-image-row {
    display: flex;
    gap: 1rem;
    align-items: center;
}
.input-readonly {
    background: #f8fafc;
    color: #64748b;
}
.adjust-current-text {
    margin-top: 0.35rem;
    font-size: 0.75rem;
    color: #64748b;
}
.presale-config-wrap .form-input:disabled {
    background: #f1f5f9;
    color: #94a3b8;
    cursor: not-allowed;
}

/* 商品行拖拽 */
.drag-handle {
    cursor: grab;
    color: #aaa;
    text-align: center;
    user-select: none;
}
.drag-handle:active { cursor: grabbing; }
.row-drag-over td {
    background-color: #eff6ff;
}

.thumb-group {
    display: flex;
    gap: 0.75rem;
    align-items: flex-end;
}
.thumb-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
}
.thumb-img {
    object-fit: cover;
    border: 1px solid #ddd;
    border-radius: 4px;
}
.thumb-desktop {
    width: 72px;
    height: 60px;
}
.thumb-mobile {
    width: 54px;
    height: 60px;
}
.thumb-label {
    font-size: 0.675rem;
    color: #888;
}

/* 详情图排序 */
.detail-images-sortable {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 0.5rem;
}
.detail-img-item {
    position: relative;
    cursor: grab;
    border: 2px solid transparent;
    border-radius: 4px;
    transition: border-color 0.15s;
}
.detail-img-item:active { cursor: grabbing; }
.detail-img-item.drag-over {
    border-color: #3b82f6;
}
.detail-img-remove {
    position: absolute;
    top: 0;
    right: 0;
    background: red;
    color: white;
    border: none;
    width: 20px;
    height: 20px;
    cursor: pointer;
    line-height: 20px;
    text-align: center;
    padding: 0;
    border-radius: 0 4px 0 4px;
}

.presale-config-wrap {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 10px;
    padding: 0.85rem 1rem 1rem;
}
.presale-config-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.85rem;
}
.presale-config-hint {
    font-size: 0.75rem;
    color: #64748b;
}
.presale-config-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
}
.presale-table-main {
    font-weight: 600;
    color: #334155;
}
.presale-table-sub {
    font-size: 0.75rem;
    color: #64748b;
}

/* 裁切弹窗 */
.crop-modal-card {
    width: min(720px, 94vw);
    max-height: 90vh;
    overflow-y: auto;
}
.crop-container {
    max-height: 60vh;
    overflow: hidden;
    background: #f0f0f0;
    border-radius: 4px;
}
.crop-container img {
    display: block;
    max-width: 100%;
}

@media (max-width: 1023px) {
    .product-form-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .presale-config-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }
}

@media (max-width: 639px) {
    .product-form-grid {
        grid-template-columns: 1fr;
        gap: 0.75rem;
    }
    .main-image-row {
        flex-direction: column;
        align-items: flex-start;
        gap: 0.5rem;
    }
    .presale-config-head {
        flex-direction: column;
        align-items: flex-start;
        gap: 0.35rem;
    }
    .presale-config-grid {
        grid-template-columns: 1fr;
    }
}
</style>
