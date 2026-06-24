<template>
  <div class="tab-content">
    <div class="tab-header">
      <h3 class="tab-title">奖品列表</h3>
      <button @click="openPrizeModal()" class="btn-primary-sm">
        <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        添加奖品
      </button>
    </div>
    <div class="table-wrapper">
      <table class="data-table">
        <thead class="table-head">
          <tr>
            <th class="th-sort">排序</th>
            <th class="th-cell">排序ID</th>
            <th class="th-cell">图片</th>
            <th class="th-cell">名称</th>
            <th class="th-cell">积分/库存</th>
            <th class="th-cell">属性</th>
            <th class="th-cell th-right">操作</th>
          </tr>
        </thead>
        <tbody class="table-body">
          <tr
            v-for="(prize, index) in store.prizes"
            :key="prize.id"
            class="table-row"
            draggable="true"
            @dragstart="onDragStart($event, index, 'prizes')"
            @dragover.prevent="onDragOver($event, index)"
            @drop="onDrop($event, index, 'prizes')"
            :class="{ 'dragging': draggedItemIndex === index }"
          >
            <td class="td-drag">
              <svg class="icon-drag" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16"></path></svg>
            </td>
            <td class="td-order">#{{ prize.displayOrder }}</td>
            <td class="td-cell">
              <div class="thumb-square">
                <img :src="prize.image" class="thumb-img">
              </div>
            </td>
            <td class="td-cell td-bold">{{ prize.name }}</td>
            <td class="td-cell">
              <div class="cell-col">
                <span class="cell-mono">{{ prize.points }} PT</span>
                <span class="cell-stock" :class="prize.stock > 0 ? 'text-success' : 'text-danger'">Stock: {{ prize.stock }}</span>
              </div>
            </td>
            <td class="td-cell td-cell-xs">
              <div class="attr-tags">
                <span class="attr-tag-gray">{{ prize.category }}</span>
                <span class="attr-tag-color" :style="{ backgroundColor: prize.color || 'var(--sos-text-tertiary)', textShadow: '0 1px 2px rgba(0,0,0,0.3)' }">{{ prize.rarity }}</span>
                <span class="attr-tag-outline">{{ prize.size }}</span>
              </div>
            </td>
            <td class="td-actions">
              <button @click="openPrizeModal(prize)" class="link-edit">编辑</button>
              <button @click="confirmDeletePrize(prize)" class="link-delete">删除</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <Teleport to="body">
      <!-- ================= Prize Edit/Add Modal ================= -->
      <Transition name="fade">
        <div v-if="showPrizeModal" class="modal-overlay">
          <div class="modal-prize">
            <!-- Left: Image Uploader & Cropper -->
            <div class="prize-modal-left">
              <div v-if="cropperImage" class="cropper-area"
                   ref="cropperContainer"
                   @mousedown="startDrag"
                   @touchstart.prevent="startDrag"
                   @wheel.prevent="handleWheel"
              >
                <img :src="cropperImage" ref="imageElement" class="cropper-image" :style="{ transform: `translate(${imgState.x}px, ${imgState.y}px) scale(${imgState.scale})` }">
                <div class="cropper-mask" :style="maskStyle"></div>
                <div class="cropper-frame" :style="cropFrameStyle"></div>
              </div>
              <div v-else class="prize-upload-area">
                <div class="prize-upload-placeholder">
                  <img v-if="prizeForm.image" :src="prizeForm.image" class="thumb-img">
                  <span v-else class="prize-upload-plus">+</span>
                </div>
                <p class="prize-upload-hint">先选择右侧的 Size，再上传图片进行裁切</p>
                <label class="btn-upload">
                  上传图片 <input type="file" accept="image/*" class="hidden-input" @change="onFileSelect">
                </label>
              </div>
              <div v-if="cropperImage" class="cropper-actions">
                <button @click="confirmCrop" class="btn-crop-confirm">确认裁切</button>
                <button @click="cancelCrop" class="btn-crop-cancel">取消</button>
              </div>
            </div>
            <!-- Right: Form Fields -->
            <div class="prize-modal-right">
              <h2 class="modal-title">{{ isEditing ? '编辑奖品' : '新增奖品' }}</h2>
              <div class="modal-form-space">
                <div>
                  <label class="form-label">名称</label>
                  <input v-model="prizeForm.name" type="text" class="form-input form-input-bold">
                </div>
                <div>
                  <label class="form-label">描述</label>
                  <textarea v-model="prizeForm.description" rows="2" class="form-textarea-sm"></textarea>
                </div>
                <div class="form-grid-2col-equal">
                  <div>
                    <label class="form-label">积分</label>
                    <input v-model.number="prizeForm.points" type="number" class="form-input form-input-mono">
                  </div>
                  <div>
                    <label class="form-label">库存</label>
                    <input v-model.number="prizeForm.stock" type="number" class="form-input form-input-mono">
                  </div>
                </div>
                <div>
                  <label class="form-label">尺寸 (影响裁切比例)</label>
                  <div class="size-grid">
                    <button v-for="s in sizeOptions" :key="s.value" @click="changeSize(s.value)" :class="prizeForm.size === s.value ? 'opt-active' : 'opt-inactive'" class="size-btn">{{ s.label }}</button>
                  </div>
                </div>
                <div class="form-grid-2col-equal">
                  <div>
                    <label class="form-label">分类</label>
                    <select v-model="prizeForm.category" class="form-select">
                      <option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option>
                    </select>
                  </div>
                  <div>
                    <label class="form-label">稀有度</label>
                    <select v-model="prizeForm.rarity" class="form-select">
                      <option>UR</option><option>SSR</option><option>SR</option><option>Rare</option><option>Uncommon</option><option>Common</option>
                    </select>
                  </div>
                </div>
                <div>
                  <label class="form-label">主题色</label>
                  <div class="color-picker-row">
                    <input v-model="prizeForm.color" type="color" class="color-picker">
                    <input v-model="prizeForm.color" type="text" class="form-input form-input-mono form-input-upper">
                  </div>
                </div>
                <div class="modal-footer-btns-sm">
                  <button @click="showPrizeModal = false" class="btn-cancel">取消</button>
                  <button @click="submitPrize" :disabled="isSubmitting" class="btn-submit">{{ isSubmitting ? '提交中...' : '保存' }}</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, reactive, computed, nextTick } from 'vue';
import { useMainStore } from '@/stores/main';

const store = useMainStore();

// Prize State
const showPrizeModal = ref(false);
const isEditing = ref(false);
const isSubmitting = ref(false);
const prizeForm = reactive({ id: null, name: '', description: '', points: 0, stock: 0, category: 'anime', rarity: 'SSR', color: '#ECC94B', size: 'normal', image: '' });
const sizeOptions = [{ label: 'Normal', value: 'normal', ratio: 1, ratioText: '1:1' }, { label: 'Large', value: 'large', ratio: 1, ratioText: '1:1' }, { label: 'Wide', value: 'wide', ratio: 2, ratioText: '2:1' }, { label: 'Tall', value: 'tall', ratio: 0.5, ratioText: '1:2' }];
const categories = [{ id: 'anime', name: '动漫周边' }, { id: 'paint', name: '绘画用具' }, { id: 'computer', name: '电脑配件' }, { id: 'book', name: '书籍' }, { id: 'peripheral', name: '外设' }, { id: 'instrument', name: '乐器' }, { id: 'game', name: '游戏' }];

// Image Cropper State
const cropperImage = ref(null);
const cropperContainer = ref(null);
const imageElement = ref(null);
const imgState = reactive({ x: 0, y: 0, scale: 1 });
const dragStart = { x: 0, y: 0, imgX: 0, imgY: 0 };
const isDragging = ref(false);

// ================= Prize Actions =================

const openPrizeModal = (prize = null) => {
    cropperImage.value = null;
    if (prize) {
        isEditing.value = true;
        Object.assign(prizeForm, JSON.parse(JSON.stringify(prize)));
    } else {
        isEditing.value = false;
        Object.assign(prizeForm, {
            id: null, name: '', description: '', points: 0, stock: 1, category: 'anime',
            rarity: 'SSR', color: '#ECC94B', size: 'normal', image: ''
        });
    }
    showPrizeModal.value = true;
};

const confirmDeletePrize = async (prize) => {
    if (confirm(`确认删除 "${prize.name}" 吗？`)) {
        await store.deletePrize(prize.id);
    }
};

const submitPrize = async () => {
    isSubmitting.value = true;
    try {
        if (isEditing.value) {
            await store.updatePrize(prizeForm.id, prizeForm);
        } else {
            await store.addPrize(prizeForm);
        }
        showPrizeModal.value = false;
    } catch (e) {
        alert('操作失败');
    } finally {
        isSubmitting.value = false;
    }
};

// ================= Image Cropper Logic =================

const getAspectRatio = () => {
    const opt = sizeOptions.find(s => s.value === prizeForm.size);
    return opt ? opt.ratio : 1;
};

const onFileSelect = (e) => {
    const file = e.target.files[0];
    if (!file) return;
    const url = URL.createObjectURL(file);
    cropperImage.value = url;
    imgState.x = 0;
    imgState.y = 0;
    imgState.scale = 1;
    nextTick(() => {
        fitImage();
    });
};

const changeSize = (newSize) => {
    prizeForm.size = newSize;
    if (cropperImage.value) {
        fitImage();
    }
};

const fitImage = () => {
    imgState.x = 0;
    imgState.y = 0;
    imgState.scale = 0.5;
};

const cropFrameStyle = computed(() => {
    if (!cropperContainer.value) return {};
    const containerW = cropperContainer.value.clientWidth || 300;
    const containerH = cropperContainer.value.clientHeight || 300;
    const ratio = getAspectRatio();

    const margin = 40;
    const maxW = containerW - margin;
    const maxH = containerH - margin;
    let w, h;

    if (maxW / maxH > ratio) {
        h = maxH;
        w = h * ratio;
    } else {
        w = maxW;
        h = w / ratio;
    }

    return {
        width: `${w}px`,
        height: `${h}px`,
        left: `${(containerW - w) / 2}px`,
        top: `${(containerH - h) / 2}px`
    };
});

const startDrag = (e) => {
    isDragging.value = true;
    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const clientY = e.touches ? e.touches[0].clientY : e.clientY;
    dragStart.x = clientX;
    dragStart.y = clientY;
    dragStart.imgX = imgState.x;
    dragStart.imgY = imgState.y;

    window.addEventListener('mousemove', onDrag);
    window.addEventListener('mouseup', stopDrag);
    window.addEventListener('touchmove', onDrag, { passive: false });
    window.addEventListener('touchend', stopDrag);
};

const onDrag = (e) => {
    if (!isDragging.value) return;
    e.preventDefault();
    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const clientY = e.touches ? e.touches[0].clientY : e.clientY;
    const dx = clientX - dragStart.x;
    const dy = clientY - dragStart.y;
    imgState.x = dragStart.imgX + dx;
    imgState.y = dragStart.imgY + dy;
};

const stopDrag = () => {
    isDragging.value = false;
    window.removeEventListener('mousemove', onDrag);
    window.removeEventListener('mouseup', stopDrag);
    window.removeEventListener('touchmove', onDrag);
    window.removeEventListener('touchend', stopDrag);
};

const handleWheel = (e) => {
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    imgState.scale *= delta;
};

const confirmCrop = () => {
    if (!imageElement.value) return;

    const containerW = cropperContainer.value.clientWidth;
    const containerH = cropperContainer.value.clientHeight;
    const ratio = getAspectRatio();

    const margin = 40;
    const maxW = containerW - margin;
    const maxH = containerH - margin;
    let cropW, cropH;
    if (maxW / maxH > ratio) {
        cropH = maxH;
        cropW = cropH * ratio;
    } else {
        cropW = maxW;
        cropH = cropW / ratio;
    }

    const cropLeft = (containerW - cropW) / 2;
    const cropTop = (containerH - cropH) / 2;

    const outputWidth = 800;
    const outputHeight = outputWidth / ratio;

    const canvas = document.createElement('canvas');
    canvas.width = outputWidth;
    canvas.height = outputHeight;
    const ctx = canvas.getContext('2d');

    const scaleFactor = outputWidth / cropW;

    const imgObj = imageElement.value;
    const renderW = imgObj.naturalWidth * imgState.scale;
    const renderH = imgObj.naturalHeight * imgState.scale;

    const containerCenterX = containerW / 2;
    const containerCenterY = containerH / 2;

    const imgCenterX = containerCenterX + imgState.x;
    const imgCenterY = containerCenterY + imgState.y;

    const imgLeftScreen = imgCenterX - renderW / 2;
    const imgTopScreen = imgCenterY - renderH / 2;

    const drawX = (imgLeftScreen - cropLeft) * scaleFactor;
    const drawY = (imgTopScreen - cropTop) * scaleFactor;
    const drawW = renderW * scaleFactor;
    const drawH = renderH * scaleFactor;

    ctx.drawImage(imgObj, drawX, drawY, drawW, drawH);

    const base64 = canvas.toDataURL('image/webp', 0.8);
    prizeForm.image = base64;
    cropperImage.value = null;
};

const cancelCrop = () => {
    cropperImage.value = null;
};

// ================= Sorting / Drag and Drop Actions =================
const draggedItemIndex = ref(null);

const onDragStart = (e, index, type) => {
    draggedItemIndex.value = index;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', index);
    e.dataTransfer.setData('type', type);
};

const onDragOver = (e, index) => {
    // Just allow drop
};

const onDrop = async (e, dropIndex, type) => {
    const dragIndex = parseInt(e.dataTransfer.getData('text/plain'), 10);
    const dragType = e.dataTransfer.getData('type');

    draggedItemIndex.value = null;

    if (dragType !== type || dragIndex === dropIndex) return;

    let list = [...store.prizes];

    const [movedItem] = list.splice(dragIndex, 1);
    list.splice(dropIndex, 0, movedItem);

    const ids = list.map(item => item.id);

    store.prizes = list;
    await store.reorderPrizes(ids);
};
</script>

<style scoped>
.icon-sm {
  width: 1rem;
  height: 1rem;
}

.icon-drag {
  width: 1.25rem;
  height: 1.25rem;
}

.icon-drag:hover {
  color: var(--sos-text-primary);
}

/* ==================== Tab Content ==================== */
.tab-content {
  padding: 1.5rem;
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.tab-title {
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1.75rem;
}

/* ==================== Shared Table Styles ==================== */
.table-wrapper {
  overflow-x: auto;
}

.data-table {
  width: 100%;
  text-align: left;
  border-collapse: collapse;
}

.table-head {
  background-color: var(--sos-bg-subtle);
  font-size: 0.75rem;
  line-height: 1rem;
  text-transform: uppercase;
  color: var(--sos-text-secondary);
}

.table-body {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.th-sort {
  padding: 0.75rem;
  padding-left: 1rem;
  font-weight: 700;
  width: 3rem;
}

.th-cell {
  padding: 0.75rem;
  font-weight: 700;
}

.th-right {
  text-align: right;
  padding-right: 1rem;
}

.table-row {
  border-bottom: 1px solid var(--sos-bg-muted);
  transition: color 150ms, background-color 150ms, border-color 150ms;
}

.table-row:hover {
  background-color: rgba(249,250,251,0.8);
}

/* ==================== Drag State ==================== */
.dragging {
  opacity: 0.5;
}

/* ==================== Table Cells ==================== */
.td-drag {
  padding: 0.75rem;
  padding-left: 1rem;
  cursor: grab;
  color: var(--sos-text-tertiary);
}

.td-drag:active {
  cursor: grabbing;
}

.td-order {
  padding: 0.75rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: var(--sos-text-tertiary);
}

.td-cell {
  padding: 0.75rem;
}

.td-cell-xs {
  font-size: 0.75rem;
  line-height: 1rem;
}

.td-bold {
  font-weight: 700;
}

.td-actions {
  padding: 0.75rem;
  text-align: right;
  padding-right: 1rem;
}

.cell-col {
  display: flex;
  flex-direction: column;
}

.cell-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.cell-stock {
  font-size: 0.75rem;
  line-height: 1rem;
}

.thumb-square {
  width: 3rem;
  height: 3rem;
  border-radius: 0.25rem;
  border: 1px solid var(--sos-border-default);
  overflow: hidden;
  background-color: var(--sos-bg-muted);
  position: relative;
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* ==================== Attribute Tags ==================== */
.attr-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.attr-tag-gray {
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  background-color: var(--sos-bg-muted);
  color: var(--sos-text-secondary);
}

.attr-tag-color {
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  color: var(--sos-bg-surface);
}

.attr-tag-outline {
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  border: 1px solid var(--sos-border-default);
}

/* ==================== Action Links ==================== */
.link-edit {
  color: #2563eb;
  margin-right: 0.75rem;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}

.link-edit:hover {
  text-decoration: underline;
}

.link-delete {
  color: var(--sos-danger);
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}

.link-delete:hover {
  text-decoration: underline;
}

/* ==================== Utility Colors ==================== */
.text-success { color: var(--sos-success); }

.text-danger { color: var(--sos-danger); }

/* ==================== Option Active/Inactive ==================== */
.opt-active {
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  border-color: var(--sos-text-primary);
}

.opt-inactive {
  background-color: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
  border-color: var(--sos-border-default);
}

.opt-inactive:hover {
  border-color: var(--sos-text-tertiary);
}

/* ==================== Buttons ==================== */
.btn-primary-sm {
  padding: 0.5rem 1rem;
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  border-radius: 0.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  transition: all 150ms;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-primary-sm:hover {
  background-color: var(--sos-text-primary);
}

/* ==================== Modal Overlay ==================== */
.modal-overlay {
  position: fixed;
  top: 0; right: 0; bottom: 0; left: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0,0,0,0.8);
  backdrop-filter: blur(4px);
  padding: 1rem;
}

.modal-title {
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
  font-family: "Noto Serif SC", serif;
  margin-bottom: 1.5rem;
}

.modal-form-space > * + * {
  margin-top: 1rem;
}

/* ==================== Form Elements ==================== */
.form-label {
  display: block;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  color: var(--sos-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.25rem;
}

.form-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--sos-border-default);
  border-radius: 0.25rem;
}

.form-input-bold {
  font-weight: 700;
}

.form-input-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.form-input-upper {
  text-transform: uppercase;
}

.form-textarea-sm {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--sos-border-default);
  border-radius: 0.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.form-select {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--sos-border-default);
  border-radius: 0.25rem;
  background-color: var(--sos-bg-surface);
}

.form-grid-2col-equal {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
}

.modal-footer-btns-sm {
  padding-top: 1rem;
  display: flex;
  gap: 0.75rem;
}

.btn-cancel {
  flex: 1;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  border: 1px solid var(--sos-border-default);
  border-radius: 0.5rem;
  font-weight: 700;
  color: var(--sos-text-secondary);
}

.btn-cancel:hover {
  background-color: var(--sos-bg-subtle);
}

.btn-submit {
  flex: 1;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  border-radius: 0.5rem;
  font-weight: 700;
}

.btn-submit:hover {
  background-color: var(--sos-text-primary);
}

.btn-submit:disabled {
  opacity: 0.5;
}

/* ==================== Prize Modal ==================== */
.modal-prize {
  background-color: var(--sos-bg-surface);
  border-radius: 0.75rem;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  width: 100%;
  max-width: 56rem;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

@media (min-width: 768px) {
  .modal-prize {
    flex-direction: row;
  }
}

.prize-modal-left {
  width: 100%;
  background-color: var(--sos-bg-muted);
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid var(--sos-border-default);
  position: relative;
}

@media (min-width: 768px) {
  .prize-modal-left {
    width: 50%;
    border-bottom: 0;
    border-right: 1px solid var(--sos-border-default);
  }
}

.prize-modal-right {
  width: 100%;
  padding: 1.5rem;
  overflow-y: auto;
}

@media (min-width: 768px) {
  .prize-modal-right {
    width: 50%;
  }
}

/* Cropper */
.cropper-area {
  position: relative;
  width: 100%;
  height: 300px;
  overflow: hidden;
  cursor: move;
  touch-action: none;
  display: flex;
  align-items: center;
  justify-content: center;
}

@media (min-width: 768px) {
  .cropper-area {
    height: 100%;
  }
}

.cropper-image {
  position: absolute;
  max-width: none;
  user-select: none;
  pointer-events: none;
  transition: transform 75ms;
  transform-origin: center;
}

.cropper-mask {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  pointer-events: none;
  z-index: 10;
}

.cropper-frame {
  position: absolute;
  z-index: 20;
  border: 2px solid var(--sos-bg-surface);
  pointer-events: none;
  box-shadow: 0 0 0 9999px rgba(0,0,0,0.6);
}

.cropper-actions {
  position: absolute;
  bottom: 1rem;
  width: 100%;
  padding-left: 1.5rem;
  padding-right: 1.5rem;
  z-index: 40;
  display: flex;
  gap: 0.5rem;
}

.btn-crop-confirm {
  flex: 1;
  background-color: var(--sos-success);
  color: var(--sos-bg-surface);
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  border-radius: 0.25rem;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.btn-crop-confirm:hover {
  background-color: #15803d;
}

.btn-crop-cancel {
  padding-left: 1rem;
  padding-right: 1rem;
  background-color: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  border-radius: 0.25rem;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.btn-crop-cancel:hover {
  background-color: var(--sos-bg-muted);
}

/* Prize Upload */
.prize-upload-area {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  text-align: center;
}

.prize-upload-placeholder {
  width: 10rem;
  height: 10rem;
  background-color: var(--sos-bg-surface);
  border: 2px dashed var(--sos-border-strong);
  border-radius: 0.5rem;
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.prize-upload-plus {
  color: var(--sos-border-strong);
  font-size: 2.25rem;
  line-height: 2.5rem;
}

.prize-upload-hint {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-text-secondary);
  margin-bottom: 1rem;
}

.btn-upload {
  padding: 0.5rem 1.5rem;
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  border-radius: 0.5rem;
  cursor: pointer;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.btn-upload:hover {
  background-color: var(--sos-text-primary);
}

.hidden-input {
  display: none;
}

/* Size Grid */
.size-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.5rem;
}

.size-btn {
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  border: 1px solid;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  transition: all 150ms;
}

/* Color Picker */
.color-picker-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.color-picker {
  width: 2.5rem;
  height: 2.5rem;
  border: none;
  padding: 0;
  background-color: transparent;
  cursor: pointer;
}

.bg-checkerboard {
  background-image: 
    linear-gradient(45deg, var(--sos-border-strong) 25%, transparent 25%), 
    linear-gradient(-45deg, var(--sos-border-strong) 25%, transparent 25%), 
    linear-gradient(45deg, transparent 75%, var(--sos-border-strong) 75%), 
    linear-gradient(-45deg, transparent 75%, var(--sos-border-strong) 75%);
  background-size: 20px 20px;
  background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
}

/* Fade transition for Vue <Transition name="fade"> */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
