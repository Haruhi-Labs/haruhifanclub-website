<template>
  <div class="tab-content">
    <div class="tab-header">
      <h3 class="tab-title">活动列表</h3>
      <button @click="openActivityModal()" class="btn-primary-sm">
        <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        添加活动
      </button>
    </div>
    <div class="table-wrapper">
      <table class="data-table">
        <thead class="table-head">
          <tr>
            <th class="th-sort">排序</th>
            <th class="th-cell">排序ID</th>
            <th class="th-cell">封面</th>
            <th class="th-cell">活动名称</th>
            <th class="th-cell">状态/类型</th>
            <th class="th-cell">奖励规则</th>
            <th class="th-cell th-right">操作</th>
          </tr>
        </thead>
        <tbody class="table-body">
          <tr
            v-for="(act, index) in store.activities"
            :key="act.id"
            class="table-row"
            draggable="true"
            @dragstart="onDragStart($event, index, 'activities')"
            @dragover.prevent="onDragOver($event, index)"
            @drop="onDrop($event, index, 'activities')"
            :class="{ 'dragging': draggedItemIndex === index }"
          >
            <td class="td-drag">
              <svg class="icon-drag" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16"></path></svg>
            </td>
            <td class="td-order">#{{ act.displayOrder }}</td>
            <td class="td-cell">
              <div class="thumb-rect">
                <img :src="act.image" class="thumb-img">
              </div>
            </td>
            <td class="td-cell">
              <div class="cell-title">{{ act.title }}</div>
              <div class="cell-subtitle">{{ act.intro }}</div>
            </td>
            <td class="td-cell td-cell-xs">
              <div class="cell-col-gap1">
                <span class="cell-status-bold" :class="act.status === 'Activate' ? 'text-success' : 'text-muted'">{{ act.status }}</span>
                <span class="cell-type">{{ act.type }}</span>
              </div>
            </td>
            <td class="td-cell td-cell-xs">
              <div class="cell-points">+{{ act.pointsPerAction }} PT</div>
              <div class="cell-pool">Pool: {{ act.totalPoints }}</div>
            </td>
            <td class="td-actions">
              <button @click="openActivityModal(act)" class="link-edit">编辑</button>
              <button @click="confirmDeleteActivity(act)" class="link-delete">删除</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <Teleport to="body">
      <!-- ================= Activity Edit/Add Modal ================= -->
      <Transition name="fade">
        <div v-if="showActivityModal" class="modal-overlay">
          <div class="modal-activity">
            <h2 class="modal-title">{{ isEditingActivity ? '编辑活动' : '新增活动' }}</h2>
            <div class="modal-form-space">
              <div class="form-grid-2col">
                <div>
                  <label class="form-label">活动标题</label>
                  <input v-model="activityForm.title" type="text" class="form-input form-input-bold">
                </div>
                <div>
                  <label class="form-label">类型 (e.g. VIDEO, OFFLINE)</label>
                  <input v-model="activityForm.type" type="text" class="form-input form-input-upper-mono">
                </div>
              </div>
              <div>
                <label class="form-label">简短介绍</label>
                <input v-model="activityForm.intro" type="text" class="form-input">
              </div>
              <div>
                <label class="form-label">详细描述 (支持 Markdown 粗体 **text** 和 URL)</label>
                <textarea v-model="activityForm.detail" rows="6" class="form-textarea"></textarea>
              </div>
              <div>
                <label class="form-label">活动封面图 (上传后自动转Base64)</label>
                <input type="file" accept="image/*" @change="onActivityImageSelect" class="form-file-input">
                <div v-if="activityForm.image" class="activity-image-preview">
                  <img :src="activityForm.image" class="thumb-img">
                </div>
              </div>
              <div class="form-grid-3col">
                <div>
                  <label class="form-label">总奖池积分</label>
                  <input v-model.number="activityForm.totalPoints" type="number" class="form-input form-input-mono">
                </div>
                <div>
                  <label class="form-label">单次奖励积分</label>
                  <input v-model.number="activityForm.pointsPerAction" type="number" class="form-input form-input-mono">
                </div>
                <div>
                  <label class="form-label">动作名称 (e.g. 参与瓜分)</label>
                  <input v-model="activityForm.actionName" type="text" class="form-input">
                </div>
              </div>
              <div>
                <label class="form-label">状态</label>
                <select v-model="activityForm.status" class="form-select">
                  <option value="Activate">Activate (进行中)</option>
                  <option value="Ended">Ended (已结束)</option>
                  <option value="Coming">Coming (即将开始)</option>
                </select>
              </div>
              <div class="modal-footer-btns">
                <button @click="showActivityModal = false" class="btn-cancel">取消</button>
                <button @click="submitActivity" :disabled="isSubmitting" class="btn-submit">
                  {{ isSubmitting ? '保存中...' : '保存活动' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue';
import { useMainStore } from '@/stores/main';

const store = useMainStore();

// Activity State
const showActivityModal = ref(false);
const isEditingActivity = ref(false);
const isSubmitting = ref(false);
const activityForm = reactive({
    id: null,
    title: '',
    intro: '',
    detail: '',
    image: '',
    totalPoints: 0,
    actionName: '',
    pointsPerAction: 0,
    status: 'Activate',
    type: 'VIDEO'
});

// ================= Activity Actions =================

const openActivityModal = (activity = null) => {
    if (activity) {
        isEditingActivity.value = true;
        Object.assign(activityForm, JSON.parse(JSON.stringify(activity)));
    } else {
        isEditingActivity.value = false;
        Object.assign(activityForm, {
            id: null, title: '', intro: '', detail: '', image: '', totalPoints: 0,
            actionName: '', pointsPerAction: 0, status: 'Activate', type: 'VIDEO'
        });
    }
    showActivityModal.value = true;
};

const onActivityImageSelect = (e) => {
    const file = e.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (evt) => {
        activityForm.image = evt.target.result;
    };
    reader.readAsDataURL(file);
};

const submitActivity = async () => {
    isSubmitting.value = true;
    try {
        if (isEditingActivity.value) {
            await store.updateActivity(activityForm.id, activityForm);
        } else {
            await store.addActivity(activityForm);
        }
        showActivityModal.value = false;
    } catch (e) {
        alert('操作失败');
    } finally {
        isSubmitting.value = false;
    }
};

const confirmDeleteActivity = async (activity) => {
    if (confirm(`确认删除活动 "${activity.title}" 吗？`)) {
        await store.deleteActivity(activity.id);
    }
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

    let list = [...store.activities];

    const [movedItem] = list.splice(dragIndex, 1);
    list.splice(dropIndex, 0, movedItem);

    const ids = list.map(item => item.id);

    // Optimistic update
    store.activities = list;
    await store.reorderActivities(ids);
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
  color: #000;
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
  background-color: #f9fafb;
  font-size: 0.75rem;
  line-height: 1rem;
  text-transform: uppercase;
  color: #6b7280;
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
  border-bottom: 1px solid #f3f4f6;
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
  color: #9ca3af;
}

.td-drag:active {
  cursor: grabbing;
}

.td-order {
  padding: 0.75rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: #9ca3af;
}

.td-cell {
  padding: 0.75rem;
}

.td-cell-xs {
  font-size: 0.75rem;
  line-height: 1rem;
}

.td-actions {
  padding: 0.75rem;
  text-align: right;
  padding-right: 1rem;
}

/* ==================== Cell Content ==================== */
.cell-title {
  font-weight: 700;
}

.cell-subtitle {
  font-size: 0.75rem;
  line-height: 1rem;
  color: #9ca3af;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 1;
  max-width: 20rem;
}

.cell-col-gap1 {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.cell-status-bold {
  font-weight: 700;
}

.cell-type {
  color: #6b7280;
}

.cell-points {
  color: #9333ea;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-weight: 700;
}

.cell-pool {
  color: #9ca3af;
}

/* ==================== Thumbnail ==================== */
.thumb-rect {
  width: 4rem;
  height: 2.5rem;
  border-radius: 0.25rem;
  border: 1px solid #e5e7eb;
  overflow: hidden;
  background-color: #f3f4f6;
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
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
  color: #ef4444;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}

.link-delete:hover {
  text-decoration: underline;
}

/* ==================== Utility Colors ==================== */
.text-success { color: #16a34a; }

.text-muted { color: #9ca3af; }

/* ==================== Buttons ==================== */
.btn-primary-sm {
  padding: 0.5rem 1rem;
  background-color: #000;
  color: #fff;
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
  background-color: #1f2937;
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

/* ==================== Activity Modal ==================== */
.modal-activity {
  background-color: #fff;
  border-radius: 0.75rem;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  width: 100%;
  max-width: 56rem;
  max-height: 90vh;
  overflow-y: auto;
  padding: 2rem;
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
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.25rem;
}

.form-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.25rem;
}

.form-input-bold {
  font-weight: 700;
}

.form-input-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.form-input-upper-mono {
  text-transform: uppercase;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.form-textarea {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-family: "Noto Sans SC", sans-serif;
}

.form-select {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.25rem;
  background-color: #fff;
}

.form-file-input {
  display: block;
  width: 100%;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #6b7280;
}

.form-file-input::file-selector-button {
  margin-right: 1rem;
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  border: 0;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  background-color: #000;
  color: #fff;
}

.form-file-input::file-selector-button:hover {
  background-color: #1f2937;
}

.form-grid-2col {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1.5rem;
}

@media (min-width: 768px) {
  .form-grid-2col {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

.form-grid-3col {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1rem;
}

@media (min-width: 768px) {
  .form-grid-3col {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

/* Activity Image Preview */
.activity-image-preview {
  margin-top: 0.5rem;
  height: 8rem;
  width: 100%;
  background-color: #f3f4f6;
  border-radius: 0.25rem;
  overflow: hidden;
  position: relative;
  border: 1px solid #e5e7eb;
}

/* ==================== Modal Footer Buttons ==================== */
.modal-footer-btns {
  padding-top: 1.5rem;
  display: flex;
  gap: 0.75rem;
}

.btn-cancel {
  flex: 1;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  font-weight: 700;
  color: #6b7280;
}

.btn-cancel:hover {
  background-color: #f9fafb;
}

.btn-submit {
  flex: 1;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  background-color: #000;
  color: #fff;
  border-radius: 0.5rem;
  font-weight: 700;
}

.btn-submit:hover {
  background-color: #1f2937;
}

.btn-submit:disabled {
  opacity: 0.5;
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
