<template>
    <div class="editor-root">
        <!-- 头部工具栏 -->
        <div class="editor-header">
            <div class="editor-header-left">
                <h1 class="editor-title serif-font">{{ isEditMode ? '编辑内容' : '内容投稿' }}</h1>
                <!-- Admin 状态切换 -->
                <div v-if="store.isAdmin"
                     class="admin-toggle"
                     :class="localAdminState ? 'admin-toggle--on' : 'admin-toggle--off'"
                     @click="toggleAdminState"
                     title="点击切换管理员权限模式">
                    {{ localAdminState ? 'ADMIN MODE ON' : 'ADMIN MODE OFF' }}
                </div>
                <button v-else @click="verifyAdmin" class="admin-login-btn">
                    管理员登录
                </button>
            </div>

            <div class="editor-header-right">
                <!-- 格式化工具栏 -->
                <div class="format-toolbar" v-if="focusedBlockType === 'paragraph'">
                    <button @mousedown.prevent="insertFormat('bold')" class="format-btn format-btn--bold" title="加粗 (Ctrl/Cmd + B)">B</button>
                    <button @mousedown.prevent="insertFormat('italic')" class="format-btn format-btn--italic" title="斜体 (Ctrl/Cmd + I)">I</button>
                    <button @mousedown.prevent="insertFormat('underline')" class="format-btn format-btn--underline" title="下划线 (Ctrl/Cmd + U)">U</button>
                    <button @mousedown.prevent="insertFormat('strikethrough')" class="format-btn format-btn--strikethrough" title="删除线 (Ctrl/Cmd + Shift + S)">S</button>
                    <button @mousedown.prevent="insertLink" class="format-btn format-btn--link" title="插入链接">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="icon-sm">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M13.19 8.688a4.5 4.5 0 0 1 1.242 7.244l-4.5 4.5a4.5 4.5 0 0 1-6.364-6.364l1.757-1.757m13.35-.622 1.757-1.757a4.5 4.5 0 0 0-6.364-6.364l-4.5 4.5a4.5 4.5 0 0 0 1.242 7.244" />
                        </svg>
                    </button>
                </div>

                <!-- 预览按钮 -->
                <button @click="showPreview = true" class="preview-btn">
                    <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>
                    预览
                </button>

                <!-- 发布按钮 -->
                <button @click="publishArticle" class="publish-btn">
                    {{ isEditMode ? '保存修改' : '发布内容' }}
                </button>
            </div>
        </div>

        <div class="editor-layout">
            <!-- 左侧边栏：元数据设置 -->
            <div class="metadata-panel">
                <!-- ID 显示 (仅编辑模式) -->
                <div v-if="isEditMode" class="edit-id-banner">
                    正在编辑文章 ID: #{{ route.query.id }}
                </div>

                <!-- 头图上传 (回归双图片逻辑) -->
                <div>
                    <label class="field-label">头图 (Cover Image)</label>
                    <div class="cover-upload-zone">
                        <div v-if="editorForm.image" class="cover-preview-wrapper">
                             <!-- 显示裁剪后的图片 -->
                             <img :src="editorForm.image" class="cover-preview-img">
                             <div class="cover-overlay">
                                <button @click.stop="removeHeaderImage" class="cover-action-btn cover-action-btn--remove">移除</button>
                                <!-- 只有当存在原图时，才允许重新裁剪 -->
                                <button v-if="editorForm.originalImage" @click.stop="reCropImage" class="cover-action-btn cover-action-btn--recrop">重新裁剪</button>
                             </div>
                        </div>
                        <div v-else class="cover-placeholder">
                             <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="cover-placeholder-icon"><path stroke-linecap="round" stroke-linejoin="round" d="m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z" /></svg>
                             <span class="cover-placeholder-text">点击上传头图</span>
                             <span class="cover-placeholder-hint">将自动保存原图与裁剪图</span>
                        </div>
                        <input type="file" accept="image/*" class="cover-file-input" @change="(e) => handleImageUpload(e, 'header')">
                    </div>
                    <p v-if="editorForm.originalImage" class="original-saved-hint">
                        <svg class="icon-xs" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg>
                        已保存高清原图 (用于正文页头图)
                    </p>
                </div>

                <!-- 类型选择 -->
                <div>
                    <label class="field-label">类型</label>
                    <div class="type-selector">
                        <button @click="editorForm.type = 'article'" class="type-btn" :class="editorForm.type === 'article' ? 'type-btn--active' : 'type-btn--inactive'">文章</button>
                        <button
                            @click="selectNewsType"
                            class="type-btn type-btn--news"
                            :class="[
                                editorForm.type === 'news' ? 'type-btn--active' : 'type-btn--inactive',
                                !localAdminState ? 'type-btn--disabled' : ''
                            ]"
                            :disabled="!localAdminState"
                        >
                            新闻 <span v-if="!localAdminState" class="type-admin-hint">(限管理员)</span>
                        </button>
                    </div>
                </div>

                <!-- 标题 -->
                <div>
                    <label class="field-label">标题 <span class="required-mark">*</span></label>
                    <input v-model="editorForm.title" type="text" class="input-title" placeholder="请输入标题...">
                </div>

                <!-- 副标题 -->
                <div>
                    <label class="field-label">副标题</label>
                    <input v-model="editorForm.subtitle" type="text" class="input-field" placeholder="可选副标题...">
                </div>

                <!-- 角标 (仅管理员) -->
                <div v-if="localAdminState">
                    <label class="field-label">角标 <span class="admin-badge">ADMIN</span></label>
                    <input v-model="editorForm.headerNote" type="text" class="input-field" placeholder="例如: 置顶 / 独家...">
                    <p class="field-hint">留空则在置顶时自动显示"置顶"</p>
                </div>

                <!-- 时间 -->
                <div>
                    <label class="field-label">时间</label>
                    <input v-model="editorForm.date" type="text" :disabled="editorForm.type === 'article'" :class="{'input-field--disabled': editorForm.type === 'article'}" class="input-field input-field--mono">
                </div>

                <!-- 作者 -->
                <div v-if="editorForm.type === 'article'">
                    <label class="field-label">作者 <span class="required-mark">*</span></label>
                    <input v-model="editorForm.author" type="text" class="input-field" placeholder="作者署名">
                </div>

                <!-- 参与者 (News 类型特有) -->
                <div v-if="editorForm.type === 'news'" class="participants-section">
                    <label class="field-label">参与者记录</label>
                    <div v-for="(p, idx) in editorForm.participants" :key="idx" class="participant-item">
                        <div class="participant-fields">
                            <input v-model="p.name" placeholder="姓名" class="participant-input participant-input--name">
                            <input v-model="p.project" placeholder="项目" class="participant-input">
                            <input v-model="p.role" placeholder="职务" class="participant-input">
                        </div>
                        <button @click="editorForm.participants.splice(idx, 1)" class="participant-remove">移除</button>
                    </div>
                    <button @click="editorForm.participants.push({name:'', project:'', role:''})" class="participant-add">+ 添加</button>
                </div>

                <!-- 标签 -->
                <div>
                    <label class="field-label">标签</label>
                    <input v-model="editorForm.tagsInput" type="text" class="input-field" placeholder="例如: 吹奏部, 公告">
                </div>

                <!-- 置顶设置 (仅管理员) -->
                <div v-if="localAdminState" class="pin-toggle">
                    <input type="checkbox" id="pinCheck" v-model="editorForm.isPinned" class="pin-checkbox">
                    <label for="pinCheck" class="pin-label">置顶显示</label>
                </div>
                <div v-if="localAdminState && editorForm.isPinned" class="pin-order">
                    <label class="field-label">置顶权重</label>
                    <input v-model="editorForm.pinOrder" type="number" class="pin-order-input">
                </div>
            </div>

            <!-- 右侧：内容编辑区 -->
            <div class="content-panel">
                <!-- 摘要 -->
                <div class="summary-section">
                    <label class="section-label">首页摘要</label>
                    <textarea
                        v-model="editorForm.summary"
                        v-auto-resize
                        rows="3"
                        class="summary-textarea"
                        placeholder="输入首页卡片展示的摘要内容..."
                    ></textarea>
                </div>

                <!-- 正文编辑器 -->
                <label class="section-label">正文内容</label>
                <div class="content-blocks">
                    <div v-for="(block, idx) in editorForm.content" :key="idx" class="block-item">
                        <!-- 块操作工具栏 -->
                        <div class="block-toolbar">
                            <button @click="moveBlock(idx, -1)" class="block-action-btn">&uarr;</button>
                            <button @click="moveBlock(idx, 1)" class="block-action-btn">&darr;</button>
                            <button @click="editorForm.content.splice(idx, 1)" class="block-action-btn block-action-btn--delete">&times;</button>
                        </div>

                        <!-- 1. 段落 (Paragraph) -->
                        <div v-if="block.type === 'paragraph'" class="block-paragraph">
                            <textarea
                                v-if="focusedBlockIndex === idx"
                                v-model="block.text"
                                :ref="(el) => setBlockRef(el, idx)"
                                v-auto-resize
                                @blur="handleBlockBlur"
                                @keydown="handleParagraphKeydown($event, idx)"
                                @keydown.enter.exact.prevent="addParagraphAfter(idx)"
                                rows="1"
                                class="paragraph-textarea"
                                placeholder="输入段落... (Enter 新段落, Shift+Enter 换行)"
                            ></textarea>
                            <!-- 非聚焦状态：渲染 Markdown 预览 -->
                            <div
                                v-else
                                class="paragraph-preview rendered-content"
                                :class="{'paragraph-preview--empty': !block.text}"
                                v-html="renderBlockMarkdown(block.text) || '点击输入段落...'"
                                @click="activateBlock(idx, 'paragraph')"
                            ></div>
                        </div>

                        <!-- 2. 标题 (Heading) -->
                        <input
                            v-if="block.type === 'heading'"
                            v-model="block.text"
                            :ref="(el) => setBlockRef(el, idx)"
                            @focus="handleBlockFocus(idx, 'heading')"
                            @blur="handleBlockBlur"
                            @keydown.enter.prevent="addParagraphAfter(idx)"
                            class="block-heading-input"
                            placeholder="输入标题..."
                        >

                        <!-- 3. 图片 (Image) -->
                        <div v-if="block.type === 'image'" class="block-image" @click="activateBlock(idx, 'image')">
                            <div v-if="block.src" class="block-image-preview">
                                <img :src="block.src" class="block-image-img">
                                <button @click.stop="block.src = ''" class="block-image-remove">✕</button>
                            </div>
                            <div v-else class="block-image-placeholder">
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="block-image-placeholder-icon"><path stroke-linecap="round" stroke-linejoin="round" d="m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z" /></svg>
                                <span class="block-image-placeholder-text">点击上传图片</span>
                                <input type="file" accept="image/*" class="block-image-file-input" @change="(e) => handleImageUpload(e, 'block', idx)">
                            </div>
                            <input v-model="block.caption" class="block-image-caption" placeholder="图片说明 (可选)" @focus="handleBlockFocus(idx, 'image')">
                        </div>

                        <!-- 4. 公式 (Math) -->
                        <div v-if="block.type === 'math'" class="block-math" @click="activateBlock(idx, 'math')">
                            <input v-model="block.expression" class="block-math-input" placeholder="LaTeX 公式 (e.g. E=mc^2)" @focus="handleBlockFocus(idx, 'math')">
                            <input v-model="block.caption" class="block-math-caption" placeholder="公式说明 (可选)" @focus="handleBlockFocus(idx, 'math')">
                            <div class="block-math-preview">预览: $$ {{ block.expression }} $$</div>
                        </div>
                    </div>

                    <!-- 添加块按钮 -->
                    <div class="add-block-bar">
                        <button @click="addBlock('paragraph')" class="add-block-btn">+ 段落</button>
                        <button @click="addBlock('heading')" class="add-block-btn">+ 标题</button>
                        <button @click="addBlock('image')" class="add-block-btn">+ 图片</button>
                        <button @click="addBlock('math')" class="add-block-btn">+ 公式</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- ======================= 实时预览层 ======================= -->
        <Transition name="fade">
            <div v-if="showPreview" class="preview-overlay">
                <!-- Preview Header -->
                <div class="preview-header">
                    <div class="preview-header-left">
                        <h2 class="preview-header-title serif-font">实时预览</h2>
                        <div class="preview-tabs">
                            <button @click="previewTab = 'card'" :class="previewTab === 'card' ? 'preview-tab--active' : 'preview-tab--inactive'" class="preview-tab">NewsCard</button>
                            <button @click="previewTab = 'modal'" :class="previewTab === 'modal' ? 'preview-tab--active' : 'preview-tab--inactive'" class="preview-tab">弹窗模式</button>
                            <button @click="previewTab = 'page'" :class="previewTab === 'page' ? 'preview-tab--active' : 'preview-tab--inactive'" class="preview-tab">正文阅读页</button>
                        </div>
                    </div>
                    <button @click="showPreview = false" class="preview-close-btn">
                        <svg class="icon-md" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
                    </button>
                </div>

                <!-- Preview Content Area -->
                <div class="preview-content-area">

                    <!-- 1. Card Preview -->
                    <div v-if="previewTab === 'card'" class="preview-card-container">
                        <NewsCard :article="previewArticleData" class="pointer-events-none" />
                        <p class="preview-card-hint">提示: 使用裁剪后的图片 (editorForm.image)</p>
                    </div>

                    <!-- 2. Modal Preview -->
                    <div v-if="previewTab === 'modal'" class="preview-modal-container">
                        <div class="preview-modal-inner">
                            <!-- Header Info -->
                            <div class="preview-modal-header">
                                <div class="preview-modal-tags">
                                    <span v-for="tag in previewArticleData.tags" :key="tag" class="preview-modal-tag">{{ tag }}</span>
                                    <span v-if="previewArticleData.type === 'news'" class="preview-modal-tag--news">News</span>
                                </div>
                                <h2 class="preview-modal-title serif-font">{{ previewArticleData.title }}</h2>
                                <div class="preview-modal-meta">
                                    <span>{{ previewArticleData.date }}</span>
                                    <span v-if="previewArticleData.type !== 'news'" class="preview-modal-author">By {{ previewArticleData.author || '凉宫春日应援团' }}</span>
                                </div>
                            </div>
                            <!-- Image (Using Cropped) -->
                            <div v-if="previewArticleData.image" class="preview-modal-image">
                                <img :src="previewArticleData.image" class="preview-modal-image-img">
                            </div>
                            <!-- Content -->
                             <div class="preview-modal-content">
                                <div v-for="(block, idx) in previewArticleData.content" :key="idx">
                                    <p v-if="block.type === 'paragraph'" v-html="renderBlockMarkdown(block.text)"></p>
                                    <h3 v-else-if="block.type === 'heading'" class="preview-modal-heading">{{ block.text }}</h3>
                                    <div v-else-if="block.type === 'math'" class="preview-modal-math">$$ {{ block.expression }} $$</div>
                                    <div v-else-if="block.type === 'image'" class="preview-modal-block-image"><img :src="block.src" class="preview-modal-block-image-img"></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- 3. Page Preview (Simplified BlogDetail) -->
                    <div v-if="previewTab === 'page'" class="preview-page-container">
                         <!-- Header Image Area (Using Original Image if Available) -->
                         <div class="preview-page-hero">
                             <div v-if="previewArticleData.image" class="preview-page-hero-image">
                                 <!-- 优先使用原图，否则使用裁剪图 -->
                                 <img :src="previewArticleData.originalImage || previewArticleData.image" class="preview-page-hero-img">
                                 <div class="preview-page-hero-overlay"></div>
                             </div>
                             <div class="preview-page-hero-text">
                                 <h1 class="preview-page-hero-title serif-font">{{ previewArticleData.title }}</h1>
                                 <p class="preview-page-hero-subtitle">{{ previewArticleData.subtitle }}</p>
                             </div>
                         </div>
                         <!-- Body -->
                         <div class="preview-page-body">
                             <div class="preview-page-main">
                                 <div v-for="(block, idx) in previewArticleData.content" :key="idx" class="preview-page-block">
                                    <p v-if="block.type === 'paragraph'" v-html="renderBlockMarkdown(block.text)" class="preview-page-paragraph"></p>
                                    <h3 v-else-if="block.type === 'heading'" class="preview-page-heading">{{ block.text }}</h3>
                                    <div v-else-if="block.type === 'image'" class="preview-page-image">
                                        <img :src="block.src" class="preview-page-image-img">
                                        <p v-if="block.caption" class="preview-page-image-caption">{{ block.caption }}</p>
                                    </div>
                                    <div v-else-if="block.type === 'math'" class="preview-page-math">$$ {{ block.expression }} $$</div>
                                 </div>
                             </div>
                             <div class="preview-page-sidebar">
                                 <div class="preview-page-sidebar-title">M E T A</div>
                                 <div class="preview-page-sidebar-item"><span class="preview-page-sidebar-label">Author</span><span class="preview-page-sidebar-value">{{ previewArticleData.author }}</span></div>
                                 <div class="preview-page-sidebar-item"><span class="preview-page-sidebar-label">Date</span><span class="preview-page-sidebar-value">{{ previewArticleData.date }}</span></div>
                             </div>
                         </div>
                    </div>
                </div>
            </div>
        </Transition>

        <!-- 图片裁剪弹窗 -->
        <div v-if="showCropper" class="cropper-overlay" @click.self="cancelCrop">
            <div class="cropper-dialog">
                <div class="cropper-header">
                    <h3 class="cropper-title">裁剪头图</h3>
                    <button @click="cancelCrop" class="cropper-close">&times;</button>
                </div>

                <div class="cropper-canvas"
                     @mousedown="startDrag"
                     @mousemove="onDrag"
                     @mouseup="endDrag"
                     @mouseleave="endDrag"
                     @wheel.prevent="handleWheel"
                >
                    <!-- 裁剪窗口覆盖层 -->
                    <div class="cropper-mask">
                        <div class="cropper-mask-bg"></div>
                        <!-- 镂空的裁剪区域：比例 2.5:1 (e.g., 500x200) -->
                        <div class="cropper-window"
                             :style="{ width: cropWindowSize.w + 'px', height: cropWindowSize.h + 'px' }"
                        >
                            <!-- 网格线 -->
                            <div class="cropper-grid">
                                <div class="cropper-grid-col"></div>
                                <div class="cropper-grid-col"></div>
                                <div class="cropper-grid-col cropper-grid-col--last"></div>
                                <div class="cropper-grid-row"></div>
                                <div class="cropper-grid-row cropper-grid-row--second"></div>
                            </div>
                        </div>
                    </div>

                    <!-- 被移动的图片 -->
                    <img
                        ref="cropperImg"
                        :src="tempImageSrc"
                        class="cropper-image"
                        :style="{
                            transform: `translate(${cropPos.x}px, ${cropPos.y}px) scale(${cropScale})`
                        }"
                        @load="initCropper"
                        draggable="false"
                    >
                </div>

                <div class="cropper-controls">
                    <div class="cropper-scale-row">
                        <span class="cropper-scale-label">缩放</span>
                        <input type="range" v-model.number="cropScale" :min="minCropScale" max="3" step="0.01" class="cropper-scale-slider">
                        <span class="cropper-scale-value">{{ (cropScale * 100).toFixed(0) }}%</span>
                    </div>
                    <div class="cropper-actions">
                        <button @click="cancelCrop" class="cropper-cancel-btn">取消</button>
                        <button @click="confirmCrop" class="cropper-confirm-btn">确认裁剪</button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { reactive, ref, watch, nextTick, onMounted, onBeforeUnmount, computed } from 'vue';
import { useRouter, useRoute, onBeforeRouteLeave } from 'vue-router';
import { useMainStore } from '@/stores/main';
import NewsCard from '@/components/NewsCard.vue';

const store = useMainStore();
const router = useRouter();
const route = useRoute();
const blockRefs = ref({});

const isEditMode = ref(false);
const editId = ref(null);
const isDirty = ref(false);

// Admin state logic
const localAdminState = ref(false); // 本地状态，用于 UI 绑定

// Preview state
const showPreview = ref(false);
const previewTab = ref('card');

// --- Cropper State ---
const showCropper = ref(false);
const tempImageSrc = ref('');
const cropScale = ref(1);
const minCropScale = ref(0.1);
const cropPos = reactive({ x: 0, y: 0 });
const isDragging = ref(false);
const dragStart = reactive({ x: 0, y: 0 });
const imgNatural = reactive({ w: 0, h: 0 });
const cropWindowSize = { w: 500, h: 200 };

const toggleAdminState = () => {
    if (store.isAdmin) {
        localAdminState.value = !localAdminState.value;
    }
};

const verifyAdmin = async () => {
    // 统一 JWT 登录：用户名 + 密码，校验 news 权限
    const user = prompt("请输入管理员用户名:");
    if (!user) return;
    const pwd = prompt("请输入管理员密码:");
    if (!pwd) return;
    const r = await store.loginAdmin(user.trim(), pwd);
    if (r.ok) {
        localAdminState.value = true;
    } else {
        alert(r.error || "用户名或密码错误，或该账号无新闻站管理权限");
    }
};

const selectNewsType = () => {
    if (localAdminState.value) {
        editorForm.type = 'news';
    } else {
        alert("需要管理员权限才能发布新闻");
    }
};

// --- Form State ---
const getBeijingDate = () => new Date().toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric', timeZone: 'Asia/Shanghai' });

const editorForm = reactive({
    type: 'article',
    title: '',
    subtitle: '',
    headerNote: '',
    summary: '',
    date: getBeijingDate(),
    author: '',
    image: '',
    originalImage: '', // 新增: 存储原图
    // 新增：头图在原图坐标中的裁切中心（0~1 归一化）
    coverFocalX: null,
    coverFocalY: null,
    tagsInput: '',
    isPinned: false,
    pinOrder: 0,
    participants: [],
    content: [{ type: 'paragraph', text: '' }]
});

// 计算用于预览的数据对象
const previewArticleData = computed(() => {
    const tags = editorForm.tagsInput.split(/[,，\s]+/).filter(t => t.trim());
    // [Fix] 过滤无效参与者，且如果为空则设为 null，防止 NewsCard 渲染空容器
    const validParticipants = editorForm.participants.filter(p => p.name && p.name.trim());

    return {
        ...editorForm,
        id: 9999, // Mock ID
        tags,
        // 如果没有单独上传封面，尝试取正文第一张图
        image: editorForm.image || (editorForm.content.find(b => b.type === 'image' && b.src)?.src) || '',
        participants: validParticipants.length > 0 ? validParticipants : null
    };
});

// --- Lifecycle ---
const handleBeforeUnload = (e) => {
    if (isDirty.value) {
        e.preventDefault();
        e.returnValue = '';
        return '';
    }
};

onBeforeRouteLeave((to, from, next) => {
    if (isDirty.value) {
        const answer = window.confirm('您有未保存的修改，确定要离开吗？');
        if (answer) {
            next();
        } else {
            next(false);
        }
    } else {
        next();
    }
});

onMounted(async () => {
    window.addEventListener('beforeunload', handleBeforeUnload);

    // 1. 同步 Admin 状态
    if (store.isAdmin) {
        localAdminState.value = true;
    }

    // 2. 处理编辑模式数据加载
    if (route.query.id) {
        editId.value = parseInt(route.query.id);
        isEditMode.value = true;

        // 使用 fetchArticleById 直接获取文章，支持待审核文章
        const target = await store.fetchArticleById(editId.value);

        if (target) {
            // 如果成功获取到（说明有权限或已发布）
            if (store.isAdmin) localAdminState.value = true;

            // 填充表单
            editorForm.type = target.type || 'article';
            editorForm.title = target.title;
            editorForm.subtitle = target.subtitle || '';
            editorForm.headerNote = target.headerNote || '';
            editorForm.summary = target.summary || '';
            editorForm.date = target.date;
            editorForm.author = target.author || '';
            editorForm.image = target.image || '';
            editorForm.originalImage = target.originalImage || '';
            editorForm.coverFocalX = typeof target.coverFocalX === 'number' ? target.coverFocalX : null;
            editorForm.coverFocalY = typeof target.coverFocalY === 'number' ? target.coverFocalY : null;
            editorForm.isPinned = !!target.isPinned;
            editorForm.pinOrder = target.pinOrder || 0;
            editorForm.tagsInput = (target.tags || []).join(', ');
            editorForm.participants = JSON.parse(JSON.stringify(target.participants || []));
            editorForm.content = JSON.parse(JSON.stringify(target.content || [{ type: 'paragraph', text: '' }]));
        } else {
            alert('无法加载文章：文章不存在或权限不足');
            router.push('/admin');
        }
    }

    nextTick(() => {
        watch(editorForm, () => isDirty.value = true, { deep: true });
    });
});

onBeforeUnmount(() => {
    window.removeEventListener('beforeunload', handleBeforeUnload);
});


// --- Block Logic & Utils ---

const vAutoResize = {
    mounted(el) {
        el.style.height = el.scrollHeight + 'px';
        el.addEventListener('input', () => {
            el.style.height = 'auto';
            el.style.height = el.scrollHeight + 'px';
        });
    },
    updated(el) {
        el.style.height = 'auto';
        el.style.height = el.scrollHeight + 'px';
    }
};

const setBlockRef = (el, idx) => { if (el) blockRefs.value[idx] = el; };
const focusedBlockIndex = ref(-1);
const focusedBlockType = ref(null);

const handleBlockBlur = () => setTimeout(() => { focusedBlockIndex.value = -1; focusedBlockType.value = null; }, 100);

const handleBlockFocus = (idx, type) => {
    focusedBlockIndex.value = idx;
    focusedBlockType.value = type;
};

const activateBlock = (idx, type) => {
    focusedBlockIndex.value = idx;
    focusedBlockType.value = type;
    nextTick(() => blockRefs.value[idx]?.focus());
};

const addParagraphAfter = (idx) => {
    editorForm.content.splice(idx + 1, 0, { type: 'paragraph', text: '' });
    nextTick(() => activateBlock(idx + 1, 'paragraph'));
};

const addBlock = (type) => {
    editorForm.content.push({ type, text: '', src: '', caption: '', expression: '' });
    nextTick(() => activateBlock(editorForm.content.length - 1, type));
};

const moveBlock = (index, dir) => {
    if ((index === 0 && dir === -1) || (index === editorForm.content.length - 1 && dir === 1)) return;
    [editorForm.content[index], editorForm.content[index + dir]] = [editorForm.content[index + dir], editorForm.content[index]];
};

const escapeHtml = (str) => str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');

const parseInlineStyles = (html) => html
    .replace(/\*\*(.*?)\*\*/g, '<b>$1</b>')
    .replace(/\*(.*?)\*/g, '<i>$1</i>')
    .replace(/__(.*?)__/g, '<u>$1</u>')
    .replace(/~~(.*?)~~/g, '<s class="inline-strikethrough">$1</s>')
    // 链接解析
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" class="inline-link" onclick="event.stopPropagation()">$1</a>');

const renderBlockMarkdown = (text) => parseInlineStyles(escapeHtml(text || ''));

const handleParagraphKeydown = (e, idx) => {
    const isMeta = e.metaKey || e.ctrlKey;
    if (!isMeta) return;

    if (e.key === 'b') { e.preventDefault(); insertFormat('bold'); }
    else if (e.key === 'i') { e.preventDefault(); insertFormat('italic'); }
    else if (e.key === 'u') { e.preventDefault(); insertFormat('underline'); }
    else if (e.key === 's' && e.shiftKey) { e.preventDefault(); insertFormat('strikethrough'); }
};

const insertFormat = (formatType) => {
    if (focusedBlockIndex.value === -1) return;
    const block = editorForm.content[focusedBlockIndex.value];
    if (block.type !== 'paragraph') return;
    const textarea = blockRefs.value[focusedBlockIndex.value];
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const text = block.text;
    let wrapper = '';
    switch(formatType) {
        case 'bold': wrapper = '**'; break;
        case 'italic': wrapper = '*'; break;
        case 'underline': wrapper = '__'; break;
        case 'strikethrough': wrapper = '~~'; break;
    }
    const selectedText = text.substring(start, end);
    const beforeText = text.substring(0, start);
    const afterText = text.substring(end);
    block.text = beforeText + wrapper + selectedText + wrapper + afterText;
    nextTick(() => {
        textarea.focus();
        textarea.setSelectionRange(start + wrapper.length, end + wrapper.length);
    });
};

const insertLink = () => {
    if (focusedBlockIndex.value === -1) return;
    const block = editorForm.content[focusedBlockIndex.value];
    if (block.type !== 'paragraph') return;
    const textarea = blockRefs.value[focusedBlockIndex.value];
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const text = block.text;
    const selectedText = text.substring(start, end);

    const url = prompt("请输入链接地址:", "https://");
    if (!url) return;

    const insertText = `[${selectedText || '链接'}](${url})`;
    block.text = text.substring(0, start) + insertText + text.substring(end);

    nextTick(() => {
        textarea.focus();
        textarea.setSelectionRange(start + insertText.length, start + insertText.length);
    });
};

// --- Image Handling ---

const compressImage = (file, maxWidth = 1920, quality = 0.85) => {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.readAsDataURL(file);
        reader.onload = (e) => {
            const img = new Image();
            img.src = e.target.result;
            img.onload = () => {
                const canvas = document.createElement('canvas');
                let width = img.width;
                let height = img.height;

                if (width > maxWidth) {
                    height *= maxWidth / width;
                    width = maxWidth;
                }

                canvas.width = width;
                canvas.height = height;
                const ctx = canvas.getContext('2d');
                ctx.drawImage(img, 0, 0, width, height);
                resolve(canvas.toDataURL('image/webp', quality));
            };
            img.onerror = reject;
        };
        reader.onerror = reject;
    });
};



const handleImageUpload = async (event, type, index = null) => {
    const file = event.target.files[0];
    if (!file) return;
    if (file.size > 20 * 1024 * 1024) { return alert("图片大小不能超过 20MB"); }

    try {
        if (type === 'header') {
            // For Header: Compress to High-Res WebP (Original but Optimized)
            // Using 3840px (4K) limit and 0.95 quality
            const highResWebP = await compressImage(file, 3840, 0.95);
            tempImageSrc.value = highResWebP;
            editorForm.originalImage = highResWebP;
            showCropper.value = true;
        } else if (type === 'block' && index !== null) {
            // For Content: Compress to Standard WebP
            const compressedDataUrl = await compressImage(file, 1920, 0.85);
            editorForm.content[index].src = compressedDataUrl;
        }
    } catch (error) {
        console.error("Image processing failed:", error);
        alert("图片处理失败，请重试");
    }
    event.target.value = '';
};

const removeHeaderImage = () => {
    editorForm.image = '';
    editorForm.originalImage = '';
};

const reCropImage = () => {
    if (editorForm.originalImage) {
        tempImageSrc.value = editorForm.originalImage;
        showCropper.value = true;
    }
};

// --- Cropper Logic ---

const initCropper = (e) => {
    const img = e.target;
    imgNatural.w = img.naturalWidth;
    imgNatural.h = img.naturalHeight;

    const minScaleW = cropWindowSize.w / imgNatural.w;
    const minScaleH = cropWindowSize.h / imgNatural.h;
    minCropScale.value = Math.max(minScaleW, minScaleH);
    cropScale.value = minCropScale.value > 0.5 ? minCropScale.value : 0.5;

    cropPos.x = 0;
    cropPos.y = 0;
    clampPos();
};

const clampPos = () => {
    if (cropScale.value < minCropScale.value) {
        cropScale.value = minCropScale.value;
    }

    const s = cropScale.value;
    const currentW = imgNatural.w * s;
    const currentH = imgNatural.h * s;

    const limitX = (currentW - cropWindowSize.w) / 2;
    const limitY = (currentH - cropWindowSize.h) / 2;

    cropPos.x = Math.max(-limitX, Math.min(limitX, cropPos.x));
    cropPos.y = Math.max(-limitY, Math.min(limitY, cropPos.y));
};

const startDrag = (e) => {
    isDragging.value = true;
    dragStart.x = e.clientX - cropPos.x;
    dragStart.y = e.clientY - cropPos.y;
};

const onDrag = (e) => {
    if (!isDragging.value) return;
    cropPos.x = e.clientX - dragStart.x;
    cropPos.y = e.clientY - dragStart.y;
    clampPos();
};

const endDrag = () => {
    isDragging.value = false;
};

const handleWheel = (e) => {
    const delta = e.deltaY > 0 ? -0.05 : 0.05;
    const newScale = Math.max(minCropScale.value, Math.min(3, cropScale.value + delta));
    cropScale.value = newScale;
    clampPos();
};

watch(cropScale, () => { clampPos(); });

const cancelCrop = () => {
    showCropper.value = false;
    tempImageSrc.value = '';
};

// 确认裁剪：生成新图片，并记录裁剪框在原图上的几何中心
const confirmCrop = () => {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');

    // 输出尺寸 1000x400 (2.5:1)
    const outputW = 1000;
    const outputH = outputW * (cropWindowSize.h / cropWindowSize.w);

    canvas.width = outputW;
    canvas.height = outputH;

    const img = new Image();

    img.onload = () => {
        const naturalW = img.naturalWidth || imgNatural.w || 1;
        const naturalH = img.naturalHeight || imgNatural.h || 1;

        const s = cropScale.value;
        const dx = cropPos.x;
        const dy = cropPos.y;

        const sourceW = cropWindowSize.w / s;
        const sourceH = cropWindowSize.h / s;

        // 计算源坐标（与之前一致）
        const sourceX = (naturalW / 2) - (sourceW / 2) - (dx / s);
        const sourceY = (naturalH / 2) - (sourceH / 2) - (dy / s);

        ctx.drawImage(img, sourceX, sourceY, sourceW, sourceH, 0, 0, outputW, outputH);

        const focalX = (naturalW / 2) - (dx / s);
        const focalY = (naturalH / 2) - (dy / s);

        const clamp01 = (v, max) => {
            if (!max || !isFinite(max)) return 0.5;
            const t = v / max;
            if (!isFinite(t) || isNaN(t)) return 0.5;
            return Math.min(1, Math.max(0, t));
        };

        const coverFocalX = clamp01(focalX, naturalW);
        const coverFocalY = clamp01(focalY, naturalH);

        editorForm.coverFocalX = Number(coverFocalX.toFixed(6));
        editorForm.coverFocalY = Number(coverFocalY.toFixed(6));

        // 保存裁剪图（保持你原来的逻辑）
        editorForm.image = canvas.toDataURL('image/webp', 0.8);
        showCropper.value = false;
    };

    img.onerror = (err) => {
        console.error('裁剪图片加载失败:', err);
        alert('裁剪图片加载失败，请重试');
    };

    img.src = tempImageSrc.value;
};


const publishArticle = async () => {
    if (!editorForm.title.trim()) return alert('请输入标题');
    if (editorForm.type === 'article' && !editorForm.author.trim()) return alert('文章模式下，作者为必填项');

    // 使用 localAdminState 判断权限
    if (!localAdminState.value) {
        editorForm.isPinned = false;
        editorForm.headerNote = '';
        editorForm.type = 'article';
    }

    const tags = editorForm.tagsInput.split(/[,，\s]+/).filter(t => t.trim());
    let coverImage = editorForm.image;
    if (!coverImage) {
         const firstImageBlock = editorForm.content.find(b => b.type === 'image' && b.src);
         coverImage = firstImageBlock ? firstImageBlock.src : '';
    }

    // [Fix] 过滤无效参与者，且如果为空则设为 null
    const validParticipants = editorForm.participants.filter(p => p.name && p.name.trim());

    const payload = {
        ...JSON.parse(JSON.stringify(editorForm)),
        tags: tags,
        image: coverImage,
        originalImage: editorForm.originalImage,
        // 如果没有有效参与者，传递 null，这样 NewsCard 不会渲染空框
        participants: validParticipants.length > 0 ? validParticipants : null
    };
    delete payload.tagsInput;

    if (isEditMode.value) {
        const success = await store.updateArticle(editId.value, payload);
        if (success) {
            isDirty.value = false;
            router.push('/admin');
        } else {
            alert('更新失败');
        }
    } else {
        const newArticleData = { id: Date.now(), ...payload };
        const result = await store.addArticle(newArticleData);
        if (result) {
            isDirty.value = false;
            if (result.status === 'pending') {
                alert('投稿成功！文章已进入审核队列。');
                router.push('/');
            } else {
                router.push('/');
            }
        } else {
            alert('发布失败，请重试');
        }
    }
};
</script>

<style scoped>
/* ===== Animation ===== */
.editor-root {
    animation: slideUpFadeIn 0.5s ease-out both;
}

@keyframes slideUpFadeIn {
    from {
        opacity: 0;
        transform: translateY(1rem);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

/* ===== Editor Header ===== */
.editor-header {
    margin-bottom: 2rem;
    border-bottom: 2px solid #000;
    padding-bottom: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: sticky;
    top: 0;
    background-color: #fff;
    z-index: 20;
    padding-top: 1rem;
}

.editor-header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
}

.editor-title {
    font-size: 1.875rem;
    line-height: 2.25rem;
    font-weight: 900;
}

.admin-toggle {
    font-size: 0.75rem;
    line-height: 1rem;
    padding: 0.25rem 0.5rem;
    font-weight: 700;
    cursor: pointer;
    transition: color 0.15s, background-color 0.15s;
}

.admin-toggle--on {
    background-color: #000;
    color: #fff;
}

.admin-toggle--off {
    background-color: #e5e7eb;
    color: #6b7280;
}

.admin-login-btn {
    font-size: 0.75rem;
    line-height: 1rem;
    border: 1px solid #d1d5db;
    color: #9ca3af;
    padding: 0.25rem 0.5rem;
    transition: border-color 0.15s, color 0.15s;
}

.admin-login-btn:hover {
    border-color: #000;
    color: #000;
}

.editor-header-right {
    display: flex;
    gap: 0.75rem;
}

/* ===== Format Toolbar ===== */
.format-toolbar {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    border-right: 1px solid #e5e7eb;
    padding-right: 0.75rem;
    margin-right: 0.25rem;
}

.format-btn {
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
    font-family: "Noto Serif SC", serif;
}

.format-btn:hover {
    background-color: #f3f4f6;
}

.format-btn--bold {
    font-weight: 700;
}

.format-btn--italic {
    font-style: italic;
}

.format-btn--underline {
    text-decoration: underline;
}

.format-btn--strikethrough {
    text-decoration: line-through;
    text-decoration-color: #9ca3af;
}

.format-btn--link {
    color: #4b5563;
}

/* ===== Icon Sizes ===== */
.icon-xs {
    width: 0.75rem;
    height: 0.75rem;
}

.icon-sm {
    width: 1rem;
    height: 1rem;
}

.icon-md {
    width: 1.5rem;
    height: 1.5rem;
}

/* ===== Preview & Publish Buttons ===== */
.preview-btn {
    font-size: 0.875rem;
    line-height: 1.25rem;
    background-color: #f3f4f6;
    color: #000;
    padding: 0.5rem 1rem;
    border-radius: 0.25rem;
    font-weight: 700;
    transition: background-color 0.15s;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.preview-btn:hover {
    background-color: #e5e7eb;
}

.publish-btn {
    background-color: #000;
    color: #fff;
    padding: 0.5rem 1.5rem;
    font-size: 0.875rem;
    line-height: 1.25rem;
    font-weight: 700;
    transition: background-color 0.15s;
}

.publish-btn:hover {
    background-color: #1f2937;
}

/* ===== Editor Layout (Grid) ===== */
.editor-layout {
    display: grid;
    grid-template-columns: 1fr;
    gap: 3rem;
    padding-bottom: 5rem;
}

@media (min-width: 768px) {
    .editor-layout {
        grid-template-columns: 1fr 2fr;
    }
}

/* ===== Metadata Panel (Left Sidebar) ===== */
.metadata-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

.edit-id-banner {
    background-color: #fefce8;
    padding: 0.5rem;
    font-size: 0.75rem;
    line-height: 1rem;
    color: #854d0e;
    border: 1px solid #fde68a;
    border-radius: 0.25rem;
}

/* ===== Field Label ===== */
.field-label {
    display: block;
    font-size: 0.75rem;
    line-height: 1rem;
    font-weight: 700;
    color: #9ca3af;
    text-transform: uppercase;
    margin-bottom: 0.5rem;
}

.required-mark {
    color: #ef4444;
}

.admin-badge {
    color: #000;
    background-color: #e5e7eb;
    padding: 0 0.25rem;
    font-size: 10px;
    border-radius: 0.25rem;
    margin-left: 0.25rem;
}

.field-hint {
    font-size: 10px;
    color: #9ca3af;
    margin-top: 0.25rem;
}

/* ===== Cover Upload Zone ===== */
.cover-upload-zone {
    border: 2px dashed #e5e7eb;
    padding: 0.25rem;
    border-radius: 0.25rem;
    transition: border-color 0.15s;
    position: relative;
    background-color: rgba(249, 250, 251, 0.5);
}

.cover-upload-zone:hover {
    border-color: #000;
}

.cover-preview-wrapper {
    position: relative;
}

.cover-preview-img {
    width: 100%;
    height: 10rem;
    object-fit: cover;
    border-radius: 0.25rem;
}

.cover-overlay {
    position: absolute;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.4);
    opacity: 0;
    transition: opacity 0.15s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    z-index: 10;
    pointer-events: none;
}

.cover-preview-wrapper:hover .cover-overlay {
    opacity: 1;
}

.cover-action-btn {
    padding: 0.25rem 0.75rem;
    font-size: 0.75rem;
    line-height: 1rem;
    border-radius: 0.25rem;
    font-weight: 700;
    pointer-events: auto;
    z-index: 20;
    position: relative;
}

.cover-action-btn--remove {
    background-color: #ef4444;
    color: #fff;
}

.cover-action-btn--remove:hover {
    background-color: #dc2626;
}

.cover-action-btn--recrop {
    background-color: #3b82f6;
    color: #fff;
}

.cover-action-btn--recrop:hover {
    background-color: #2563eb;
}

.cover-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem 0;
    color: #9ca3af;
}

.cover-upload-zone:hover .cover-placeholder {
    color: #000;
}

.cover-placeholder-icon {
    width: 2rem;
    height: 2rem;
    margin-bottom: 0.5rem;
}

.cover-placeholder-text {
    font-size: 0.75rem;
    line-height: 1rem;
    font-weight: 700;
}

.cover-placeholder-hint {
    font-size: 10px;
    margin-top: 0.25rem;
    opacity: 0.6;
}

.cover-file-input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
    z-index: 0;
}

.original-saved-hint {
    font-size: 10px;
    color: #16a34a;
    margin-top: 0.25rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
}

/* ===== Type Selector ===== */
.type-selector {
    display: flex;
    border: 1px solid #000;
}

.type-btn {
    flex: 1;
    padding: 0.5rem 0;
    font-size: 0.875rem;
    line-height: 1.25rem;
    font-weight: 700;
    transition: background-color 0.15s, color 0.15s;
}

.type-btn--news {
    border-left: 1px solid #000;
}

.type-btn--active {
    background-color: #000;
    color: #fff;
}

.type-btn--inactive:hover {
    background-color: #f3f4f6;
}

.type-btn--disabled {
    opacity: 0.4;
    cursor: not-allowed;
    background-color: #f3f4f6;
    color: #9ca3af;
}

.type-admin-hint {
    font-size: 10px;
    font-weight: 400;
    transform: scale(0.75);
    display: inline-block;
    transform-origin: left;
}

/* ===== Input Fields ===== */
.input-title {
    width: 100%;
    border-bottom: 1px solid #e5e7eb;
    padding: 0.5rem 0;
    font-size: 1.25rem;
    line-height: 1.75rem;
    font-family: "Noto Serif SC", serif;
    font-weight: 700;
    outline: none;
    transition: border-color 0.15s;
}

.input-title:focus {
    border-color: #000;
}

.input-field {
    width: 100%;
    border-bottom: 1px solid #e5e7eb;
    padding: 0.5rem 0;
    font-size: 0.875rem;
    line-height: 1.25rem;
    outline: none;
    transition: border-color 0.15s;
}

.input-field:focus {
    border-color: #000;
}

.input-field--mono {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.input-field--disabled {
    background-color: #f9fafb;
    color: #6b7280;
    cursor: not-allowed;
}

/* ===== Participants ===== */
.participants-section {
    background-color: #f9fafb;
    padding: 1rem;
    border-radius: 0.25rem;
    border: 1px solid #f3f4f6;
}

.participant-item {
    margin-bottom: 0.75rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid #e5e7eb;
}

.participant-item:last-child {
    border-bottom: 0;
    padding-bottom: 0;
}

.participant-fields {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.5rem;
}

.participant-input {
    background-color: transparent;
    border-bottom: 1px solid #d1d5db;
    font-size: 0.75rem;
    line-height: 1rem;
    padding: 0.25rem 0;
    outline: none;
}

.participant-input:focus {
    border-color: #000;
}

.participant-input--name {
    font-size: 0.875rem;
    line-height: 1.25rem;
}

.participant-remove {
    font-size: 10px;
    color: #f87171;
    margin-top: 0.25rem;
}

.participant-remove:hover {
    color: #dc2626;
}

.participant-add {
    width: 100%;
    margin-top: 0.5rem;
    border: 1px dashed #d1d5db;
    padding: 0.25rem 0;
    font-size: 0.75rem;
    line-height: 1rem;
    color: #6b7280;
    transition: border-color 0.15s, color 0.15s;
}

.participant-add:hover {
    border-color: #000;
    color: #000;
}

/* ===== Pin Toggle ===== */
.pin-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
    padding: 0.5rem;
    background-color: #f9fafb;
    border-radius: 0.25rem;
}

.pin-checkbox {
    width: 1rem;
    height: 1rem;
    accent-color: #000;
}

.pin-label {
    font-size: 0.875rem;
    line-height: 1.25rem;
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.pin-order {
    margin-top: 0.5rem;
    padding-left: 1.5rem;
}

.pin-order-input {
    width: 5rem;
    border-bottom: 1px solid #e5e7eb;
    padding: 0.25rem 0;
    font-size: 0.875rem;
    line-height: 1.25rem;
    outline: none;
}

.pin-order-input:focus {
    border-color: #000;
}

/* ===== Content Panel (Right) ===== */
.content-panel {
    /* spans 2 cols on md via grid */
}

/* ===== Summary Section ===== */
.summary-section {
    margin-bottom: 2rem;
}

.section-label {
    display: block;
    font-size: 0.75rem;
    line-height: 1rem;
    font-weight: 700;
    color: #9ca3af;
    text-transform: uppercase;
    margin-bottom: 0.5rem;
    border-bottom: 1px solid #f3f4f6;
    padding-bottom: 0.5rem;
}

.summary-textarea {
    width: 100%;
    background-color: #f9fafb;
    padding: 1rem;
    font-size: 0.875rem;
    line-height: 1.625;
    outline: none;
    border: 1px solid transparent;
    border-radius: 0.25rem;
    resize: none;
    overflow: hidden;
}

.summary-textarea:focus {
    border-color: #e5e7eb;
}

/* ===== Content Blocks ===== */
.content-blocks {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    min-height: 500px;
}

.block-item {
    position: relative;
    border: 1px dashed transparent;
    border-radius: 0.25rem;
    padding: 1rem;
    transition: border-color 0.15s, background-color 0.15s;
}

.block-item:hover {
    border-color: #e5e7eb;
    background-color: #f9fafb;
}

/* ===== Block Toolbar ===== */
.block-toolbar {
    display: none;
    position: absolute;
    right: 0.5rem;
    top: 0.5rem;
    gap: 0.25rem;
    z-index: 10;
}

.block-item:hover .block-toolbar {
    display: flex;
}

.block-action-btn {
    padding: 0.25rem;
    background-color: #fff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    border-radius: 0.25rem;
    color: #9ca3af;
}

.block-action-btn:hover {
    color: #000;
}

.block-action-btn--delete {
    color: #fca5a5;
}

.block-action-btn--delete:hover {
    color: #ef4444;
}

/* ===== Paragraph Block ===== */
.block-paragraph {
    position: relative;
    min-height: 32px;
}

.paragraph-textarea {
    width: 100%;
    background-color: transparent;
    resize: none;
    outline: none;
    color: #1f2937;
    line-height: 1.625;
    font-family: "Noto Serif SC", serif;
    font-size: 1.125rem;
    overflow: hidden;
}

.paragraph-textarea::placeholder {
    color: #e5e7eb;
}

.paragraph-preview {
    width: 100%;
    font-size: 1.125rem;
    line-height: 1.625;
    font-family: "Noto Serif SC", serif;
    cursor: text;
    color: #1f2937;
    white-space: pre-wrap;
    word-wrap: break-word;
    min-height: 28px;
}

.paragraph-preview--empty {
    color: #d1d5db;
    font-style: italic;
}

/* ===== Heading Block ===== */
.block-heading-input {
    width: 100%;
    background-color: transparent;
    outline: none;
    font-weight: 700;
    font-family: "Noto Serif SC", serif;
    font-size: 1.5rem;
    line-height: 2rem;
}

.block-heading-input::placeholder {
    color: #d1d5db;
}

/* ===== Image Block ===== */
.block-image {
    background-color: #f9fafb;
    padding: 1rem;
    border-radius: 0.25rem;
    text-align: center;
    position: relative;
}

.block-image-preview {
    position: relative;
}

.block-image-img {
    margin-top: 0.5rem;
    margin-left: auto;
    margin-right: auto;
    max-height: 16rem;
    object-fit: contain;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    background-color: #fff;
}

.block-image-remove {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    background-color: rgba(255, 255, 255, 0.8);
    border-radius: 9999px;
    padding: 0.25rem;
    color: #ef4444;
    opacity: 0;
    transition: opacity 0.15s;
}

.block-image-preview:hover .block-image-remove {
    opacity: 1;
}

.block-image-placeholder {
    border: 2px dashed #d1d5db;
    border-radius: 0.25rem;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    transition: border-color 0.15s;
    cursor: pointer;
    position: relative;
}

.block-image-placeholder:hover {
    border-color: #000;
}

.block-image-placeholder-icon {
    width: 2rem;
    height: 2rem;
    color: #9ca3af;
    margin-bottom: 0.5rem;
}

.block-image-placeholder-text {
    font-size: 0.875rem;
    line-height: 1.25rem;
    color: #6b7280;
    font-weight: 700;
}

.block-image-file-input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
}

.block-image-caption {
    width: 100%;
    background-color: transparent;
    font-size: 0.75rem;
    line-height: 1rem;
    text-align: center;
    font-style: italic;
    color: #6b7280;
    outline: none;
    margin-top: 0.5rem;
}

/* ===== Math Block ===== */
.block-math {
    background-color: #f9fafb;
    padding: 1rem;
    border-left: 4px solid #000;
}

.block-math-input {
    width: 100%;
    background-color: transparent;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 0.875rem;
    line-height: 1.25rem;
    outline: none;
    margin-bottom: 0.5rem;
}

.block-math-caption {
    width: 100%;
    background-color: transparent;
    font-size: 0.75rem;
    line-height: 1rem;
    color: #6b7280;
    outline: none;
}

.block-math-preview {
    text-align: center;
    color: #9ca3af;
    font-size: 0.75rem;
    line-height: 1rem;
    margin-top: 0.5rem;
}

/* ===== Add Block Bar ===== */
.add-block-bar {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    margin-top: 2rem;
    padding-top: 1rem;
    border-top: 1px dashed #e5e7eb;
}

.add-block-btn {
    padding: 0.25rem 0.75rem;
    font-size: 0.75rem;
    line-height: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.25rem;
    transition: background-color 0.15s, color 0.15s, border-color 0.15s;
}

.add-block-btn:hover {
    background-color: #000;
    color: #fff;
    border-color: #000;
}

/* ===== Preview Overlay ===== */
.preview-overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    background-color: #f3f4f6;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.preview-header {
    background-color: #fff;
    border-bottom: 1px solid #e5e7eb;
    padding: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    z-index: 10;
}

.preview-header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
}

.preview-header-title {
    font-weight: 700;
    font-size: 1.25rem;
    line-height: 1.75rem;
}

.preview-tabs {
    display: flex;
    background-color: #f3f4f6;
    border-radius: 0.25rem;
    padding: 0.25rem;
    font-size: 0.875rem;
    line-height: 1.25rem;
    font-weight: 700;
}

.preview-tab {
    padding: 0.375rem 1rem;
    border-radius: 0.25rem;
    transition: all 0.15s;
}

.preview-tab--active {
    background-color: #fff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    color: #000;
}

.preview-tab--inactive {
    color: #9ca3af;
}

.preview-tab--inactive:hover {
    color: #4b5563;
}

.preview-close-btn {
    color: #6b7280;
    padding: 0.5rem;
}

.preview-close-btn:hover {
    color: #000;
}

/* ===== Preview Content Area ===== */
.preview-content-area {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
    display: flex;
    justify-content: center;
    background-color: #f3f4f6;
}

/* ===== Card Preview ===== */
.preview-card-container {
    width: 360px;
    height: fit-content;
}

.pointer-events-none {
    pointer-events: none;
}

.preview-card-hint {
    text-align: center;
    color: #9ca3af;
    font-size: 0.75rem;
    line-height: 1rem;
    margin-top: 1rem;
}

/* ===== Modal Preview ===== */
.preview-modal-container {
    width: 100%;
    max-width: 42rem;
    background-color: #fff;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
    border: 1px solid #000;
    min-height: 600px;
    display: flex;
    flex-direction: column;
}

.preview-modal-inner {
    padding: 1.5rem;
}

@media (min-width: 768px) {
    .preview-modal-inner {
        padding: 2.5rem;
    }
}

.preview-modal-header {
    margin-bottom: 1.5rem;
}

.preview-modal-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
}

.preview-modal-tag {
    background-color: #000;
    color: #fff;
    font-size: 0.75rem;
    line-height: 1rem;
    padding: 0.25rem 0.5rem;
}

.preview-modal-tag--news {
    border: 1px solid #000;
    color: #000;
    font-size: 0.75rem;
    line-height: 1rem;
    padding: 0.25rem 0.5rem;
    text-transform: uppercase;
}

.preview-modal-title {
    font-size: 1.875rem;
    line-height: 2.25rem;
    font-weight: 700;
    margin-bottom: 1rem;
    line-height: 1.25;
}

@media (min-width: 768px) {
    .preview-modal-title {
        font-size: 2.25rem;
        line-height: 2.5rem;
    }
}

.preview-modal-meta {
    display: flex;
    align-items: center;
    font-size: 0.875rem;
    line-height: 1.25rem;
    color: #6b7280;
    gap: 1rem;
    margin-bottom: 0.5rem;
}

.preview-modal-author {
    color: #000;
    font-weight: 700;
}

.preview-modal-image {
    margin-bottom: 2rem;
}

.preview-modal-image-img {
    width: 100%;
    height: auto;
}

.preview-modal-content {
    color: #1f2937;
    font-family: "Noto Serif SC", serif;
    line-height: 2;
    text-align: justify;
    font-size: 1.125rem;
}

.preview-modal-heading {
    font-weight: 700;
    font-size: 1.25rem;
    line-height: 1.75rem;
    margin-top: 1rem;
    margin-bottom: 0.5rem;
}

.preview-modal-math {
    margin: 1rem 0;
    padding: 1rem;
    background-color: #f9fafb;
    text-align: center;
}

.preview-modal-block-image {
    margin: 1rem 0;
}

.preview-modal-block-image-img {
    width: 100%;
    max-height: 12rem;
    object-fit: cover;
}

/* ===== Page Preview ===== */
.preview-page-container {
    width: 100%;
    max-width: 64rem;
    background-color: #fff;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
    min-height: 100vh;
}

.preview-page-hero {
    position: relative;
    width: 100%;
    height: 400px;
    background-color: #d1d5db;
    overflow: hidden;
}

.preview-page-hero-image {
    position: absolute;
    inset: 0;
}

.preview-page-hero-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.preview-page-hero-overlay {
    position: absolute;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.3);
}

.preview-page-hero-text {
    position: absolute;
    bottom: 2.5rem;
    left: 2.5rem;
    color: #fff;
    z-index: 10;
}

.preview-page-hero-title {
    font-size: 3rem;
    line-height: 1;
    font-weight: 900;
    margin-bottom: 1rem;
}

.preview-page-hero-subtitle {
    font-size: 1.25rem;
    line-height: 1.75rem;
    font-family: "Noto Sans SC", sans-serif;
}

.preview-page-body {
    padding: 3rem;
    display: grid;
    grid-template-columns: repeat(12, 1fr);
    gap: 2rem;
}

@media (min-width: 768px) {
    .preview-page-body {
        padding: 5rem;
    }
}

.preview-page-main {
    grid-column: span 9;
    max-width: none;
    font-family: "Noto Serif SC", serif;
    font-size: 1.25rem;
}

.preview-page-block {
    margin-bottom: 2rem;
}

.preview-page-paragraph {
    text-align: justify;
    line-height: 2;
}

.preview-page-heading {
    font-size: 1.875rem;
    line-height: 2.25rem;
    font-weight: 700;
    margin-top: 3rem;
    margin-bottom: 1.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #e5e7eb;
}

.preview-page-image {
    margin: 2rem 0;
}

.preview-page-image-img {
    width: 100%;
    border-radius: 0.25rem;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    border: 1px solid #e5e7eb;
}

.preview-page-image-caption {
    text-align: center;
    font-size: 0.875rem;
    line-height: 1.25rem;
    color: #6b7280;
    margin-top: 0.5rem;
}

.preview-page-math {
    padding: 2rem;
    background-color: #f9fafb;
    text-align: center;
    font-family: "Noto Serif SC", serif;
    font-size: 1.25rem;
    line-height: 1.75rem;
}

.preview-page-sidebar {
    grid-column: span 3;
    border-left: 1px solid #e5e7eb;
    padding-left: 2rem;
    padding-top: 2rem;
}

.preview-page-sidebar-title {
    font-size: 0.75rem;
    line-height: 1rem;
    font-weight: 700;
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 1rem;
}

.preview-page-sidebar-item {
    margin-bottom: 1rem;
}

.preview-page-sidebar-label {
    display: block;
    color: #9ca3af;
    font-size: 0.75rem;
    line-height: 1rem;
}

.preview-page-sidebar-value {
    font-weight: 700;
}

/* ===== Cropper Overlay ===== */
.cropper-overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    background-color: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    user-select: none;
}

.cropper-dialog {
    background-color: #fff;
    width: 100%;
    max-width: 42rem;
    border-radius: 0.5rem;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    max-height: 90vh;
}

.cropper-header {
    padding: 1rem;
    border-bottom: 1px solid #f3f4f6;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.cropper-title {
    font-size: 1.125rem;
    line-height: 1.75rem;
    font-weight: 700;
}

.cropper-close {
    color: #9ca3af;
}

.cropper-close:hover {
    color: #000;
}

.cropper-canvas {
    position: relative;
    background-color: #f3f4f6;
    overflow: hidden;
    flex: 1;
    min-height: 400px;
    cursor: move;
    display: flex;
    align-items: center;
    justify-content: center;
}

.cropper-mask {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 10;
}

.cropper-mask-bg {
    position: absolute;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
}

.cropper-window {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    border: 2px solid #fff;
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.5);
}

.cropper-grid {
    position: absolute;
    inset: 0;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-template-rows: repeat(3, 1fr);
    opacity: 0.3;
}

.cropper-grid-col {
    border-right: 1px solid rgba(255, 255, 255, 0.5);
}

.cropper-grid-col--last {
    border-right: 1px solid transparent;
}

.cropper-grid-row {
    border-bottom: 1px solid rgba(255, 255, 255, 0.5);
    grid-column: span 3;
    grid-row-start: 1;
}

.cropper-grid-row--second {
    grid-row-start: 2;
}

.cropper-image {
    position: absolute;
    transition: transform 75ms;
    transform-origin: center;
    max-width: none;
}

.cropper-controls {
    padding: 1.5rem;
    background-color: #fff;
    border-top: 1px solid #f3f4f6;
}

.cropper-scale-row {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
}

.cropper-scale-label {
    font-size: 0.75rem;
    line-height: 1rem;
    font-weight: 700;
    color: #9ca3af;
}

.cropper-scale-slider {
    flex: 1;
    accent-color: #000;
}

.cropper-scale-value {
    font-size: 0.75rem;
    line-height: 1rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    color: #6b7280;
    width: 2rem;
}

.cropper-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
}

.cropper-cancel-btn {
    padding: 0.5rem 1.5rem;
    font-size: 0.875rem;
    line-height: 1.25rem;
    font-weight: 700;
    color: #6b7280;
}

.cropper-cancel-btn:hover {
    color: #000;
}

.cropper-confirm-btn {
    padding: 0.5rem 1.5rem;
    font-size: 0.875rem;
    line-height: 1.25rem;
    font-weight: 700;
    background-color: #000;
    color: #fff;
}

.cropper-confirm-btn:hover {
    background-color: #1f2937;
}

/* ===== Inline Styles for Rendered Content ===== */
.rendered-content :deep(b) { font-weight: bold; }
.rendered-content :deep(i) { font-style: italic; }
.rendered-content :deep(u) { text-decoration: underline; }
.rendered-content :deep(s) { text-decoration: line-through; }

.rendered-content :deep(.inline-strikethrough) {
    text-decoration: line-through;
}

.rendered-content :deep(.inline-link) {
    color: #2563eb;
}

.rendered-content :deep(.inline-link:hover) {
    text-decoration: underline;
}

/* ===== Transition ===== */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
