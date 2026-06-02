<script setup lang="ts">
import { ref, reactive } from 'vue';
import type { Question, ContentBlock } from '@/types/exam';
import { api } from '@/services/api';
import { compressImage } from '@/utils/file';
import { useExamStore } from '@/stores/exam';

const props = defineProps<{
  questions: Question[];
}>();

// 2. 初始化 Store
const examStore = useExamStore();
// 使用 Map 精确追踪每一个内容块(Block)的上传状态
const uploadingState = ref<Map<ContentBlock, boolean>>(new Map());
// [新增] 使用 Map 存储本地预览图，Key是Block引用，Value是blobUrl
const blockPreviews = reactive(new Map<ContentBlock, string>());

// 激活的栏目
const activeCol = ref<'C1' | 'C2' | 'C3' | 'C4'>('C1');
const OPTION_KEYS = ['A', 'B', 'C', 'D'];

// 获取当前栏目的题目
const getColQuestions = (col: string) => {
  return props.questions.filter(q => q.column === col).sort((a, b) => a.no - b.no);
};

// 添加新题目
const addQuestion = () => {
  const maxNo = props.questions.reduce((max, q) => Math.max(max, q.no), 0);
  const newQ: Question = {
    id: `q_${Date.now()}`,
    no: maxNo + 1,
    column: activeCol.value,
    type: 'choice',
    typeLabel: '单选',
    score: 2,
    stemBlocks: [{ type: 'text', text: '' }],
    options: [
      { key: 'A', text: '' },
      { key: 'B', text: '' },
      { key: 'C', text: '' },
      { key: 'D', text: '' }
    ],
    answer: 'A',
    analysisBlocks: [{ type: 'text', text: '' }]
  };
  props.questions.push(newQ);
};

// 当题目类型改变时，自动调整相关配置
const handleTypeChange = (q: Question) => {
  if (q.type === 'judgment') {
    // 判断题：固定两个选项（正确/错误）
    q.options = [
      { key: 'true', text: '正确' },
      { key: 'false', text: '错误' }
    ];
    q.answer = 'true'; // 默认答案为正确
    q.typeLabel = '判断';
    
  } else if (q.type === 'choice') {
    q.options = [
      { key: 'A', text: '' },
      { key: 'B', text: '' },
      { key: 'C', text: '' },
      { key: 'D', text: '' }
    ];
    q.answer = 'A';
    q.typeLabel = '单选';
    
  } else if (q.type === 'multiple') {
    // 多选题
    q.options = [
      { key: 'A', text: '' },
      { key: 'B', text: '' },
      { key: 'C', text: '' },
      { key: 'D', text: '' }
    ];
    // 如果答案不是逗号分隔格式，转换为单个选项
    if (q.answer && !q.answer.includes(',')) {
      q.answer = q.answer; // 保持原样，但后续会作为单个答案
    } else if (!q.answer) {
      q.answer = 'A'; // 默认至少选一个
    }
    q.typeLabel = '多选';
    
  } else if (q.type === 'fill') {
    // 填空题：清除选项
    q.options = undefined;
    q.answer = '';
    q.typeLabel = '填空';
    
  }
};

// 多选题答案管理
const getMultipleAnswers = (answer: string): string[] => {
  if (!answer) return [];
  return answer.split(',').map(a => a.trim()).filter(a => a);
};

const setMultipleAnswer = (q: Question, key: string, checked: boolean) => {
  const answers = getMultipleAnswers(q.answer);
  if (checked) {
    if (!answers.includes(key)) {
      answers.push(key);
    }
  } else {
    const idx = answers.indexOf(key);
    if (idx > -1) {
      answers.splice(idx, 1);
    }
  }
  q.answer = answers.sort().join(',');
};

// 删除题目
const removeQuestion = (id: string) => {
  if (confirm('确定删除此题吗？')) {
    const idx = props.questions.findIndex(q => q.id === id);
    if (idx !== -1) props.questions.splice(idx, 1);
  }
};

// 内容块操作
const addBlock = (blocks: ContentBlock[], type: 'text'|'image'|'audio') => {
  if (type === 'text') blocks.push({ type: 'text', text: '' });
  if (type === 'image') blocks.push({ type: 'image', image: { src: '', alt: '' } });
  if (type === 'audio') blocks.push({ type: 'audio', audio: { src: '', title: '音频片段', durationText: '0:00' } });
};
const removeBlock = (blocks: ContentBlock[], idx: number) => {
  blocks.splice(idx, 1);
};

// --- 选项管理逻辑 ---

// 重新对选项进行 ABCD 编号
const reindexOptions = (q: Question) => {
  if (!q.options) return;
  q.options.forEach((opt, index) => {
    // 防止越界，虽然限制了4个，但为了安全加个 fallback
    opt.key = OPTION_KEYS[index] || '?';
  });
};

// 添加选项 (上限4个)
const addOption = (q: Question) => {
  if (!q.options) q.options = [];
  if (q.options.length >= 4) return; // 不允许超过4个

  q.options.push({ key: '', text: '' });
  reindexOptions(q);
};

// 删除选项
const removeOption = (q: Question, idx: number) => {
  if (!q.options) return;
  q.options.splice(idx, 1);
  reindexOptions(q);
  
  // 如果当前答案被删除了，重置答案
  const validKeys = q.options.map(o => o.key);
  if (!validKeys.includes(q.answer) && validKeys.length > 0) {
    q.answer = validKeys[0];
  }
};

// 上传辅助
const handleMediaUpload = async (e: Event, block: ContentBlock) => {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  
  // A. 开启该 Block 的 Loading
  uploadingState.value.set(block, true);

  // B. 本地预览
  const localUrl = URL.createObjectURL(file);

  // [修正] 将本地预览图存入 Map，UI渲染时优先取用
  blockPreviews.set(block, localUrl);

  // [临时] 同时也赋值给 src 确保基本逻辑（虽然下面会被服务端URL覆盖，但有了blockPreviews，UI不会受影响）
  if (block.type === 'image' && block.image) {
    block.image.src = localUrl;
  }
  if (block.type === 'audio' && block.audio) {
    block.audio.src = localUrl;
    // 自动填充标题
    if (!block.audio.title) block.audio.title = file.name.replace(/\.[^/.]+$/, "");
    
    // 计算音频时长 (利用本地文件)
    const audioObj = new Audio(localUrl);
    audioObj.onloadedmetadata = () => {
      const m = Math.floor(audioObj.duration / 60);
      const s = Math.floor(audioObj.duration % 60);
      if (block.audio) block.audio.durationText = `${m}:${s.toString().padStart(2, '0')}`;
    };
  }

  try {
    let fileToUpload = file;

    // C. 如果是图片，前端压缩；如果是音频，直接传原文件(后端压)
    if (block.type === 'image') {
      fileToUpload = await compressImage(file, 0.75);
    }
    
    // D. 上传
    const serverUrl = await api.uploadFile(fileToUpload);

    // E. 替换为服务器路径 (保证提交数据正确性)
    // [注意] 虽然这里把 block.image.src 变成了服务端URL，但因为模板中使用了 blockPreviews，
    // 所以用户看到的依然是上面生成的 localUrl，不会闪烁。
    if (block.type === 'image' && block.image) block.image.src = serverUrl;
    if (block.type === 'audio' && block.audio) block.audio.src = serverUrl;

    // F. 记录上传
    examStore.recordUpload(serverUrl);

  } catch (err) {
    console.error('Block upload failed', err);
    alert('上传失败');
  } finally {
    uploadingState.value.delete(block); // 关闭 Loading
    (e.target as HTMLInputElement).value = '';
  }
};
</script>

<template>
  <div class="panel">
    <!-- 顶部标签栏 -->
    <div class="tabs-header">
      <div class="tabs">
        <button v-for="c in ['C1','C2','C3','C4']" :key="c" 
          :class="{ active: activeCol === c }" @click="activeCol = c as any">
          {{ c }} 栏
        </button>
      </div>
      <div class="header-action">
        <button class="btn-primary-ghost" @click="addQuestion">
          <span class="icon">+</span> 添加
        </button>
      </div>
    </div>

    <!-- 题目列表滚动区 -->
    <div class="q-list">
      <div v-if="getColQuestions(activeCol).length === 0" class="empty-state">
        此栏还没有题目，点击右上角添加
      </div>

      <div v-for="q in getColQuestions(activeCol)" :key="q.id" class="q-card">
        
        <!-- 1. 题目头部：序号、类型、分数 -->
        <div class="q-header">
          <div class="q-badge">No.{{ q.no }}</div>
          
          <div class="q-meta-group">
            <div class="meta-item">
              <label>类型</label>
              <select v-model="q.type" @change="handleTypeChange(q)" class="modern-select">
                <option value="choice">单选题</option>
                <option value="multiple">多选题</option>
                <option value="judgment">判断题</option>
                <option value="fill">填空题</option>
              </select>
            </div>
            <div class="meta-item">
              <label>标签</label>
              <input type="text" v-model="q.typeLabel" class="modern-input short" placeholder="如:单选" />
            </div>
            <div class="meta-item">
              <label>分值</label>
              <input type="number" v-model.number="q.score" class="modern-input short" />
            </div>
          </div>

          <button class="btn-icon-del" title="删除题目" @click="removeQuestion(q.id)">
            🗑️
          </button>
        </div>

        <!-- 2. 题干内容区 -->
        <div class="section-area">
          <div class="section-title">题干 Stem</div>
          <div class="blocks-container">
            <div v-for="(b, bi) in q.stemBlocks" :key="bi" class="block-row">
              <!-- 类型图标 -->
              <div class="block-icon" :title="b.type">
                {{ b.type === 'text' ? 'T' : b.type === 'image' ? '🖼️' : '🎵' }}
              </div>
              
              <!-- 内容输入 -->
              <div class="block-body">
                <textarea v-if="b.type==='text'" v-model="b.text" rows="2" class="modern-textarea" placeholder="输入题目描述..."></textarea>
                
                <!-- 图片块: 仅上传 -->
                <div v-if="b.type==='image' && b.image" class="media-input-group">
                  <div v-if="!b.image.src && !blockPreviews.get(b)" class="upload-placeholder">
                      <label class="file-btn primary">
                        <span v-if="uploadingState.get(b)">⏳ 上传中...</span>
                        <span v-else>📄 点击上传图片</span>
                        <input type="file" accept="image/*" @change="(e) => handleMediaUpload(e, b)" />
                      </label>
                  </div>
                  <div v-else class="media-preview-row">
                      <!-- [修正] 优先显示 blockPreviews 中的本地图片 -->
                      <div class="media-preview" :style="{backgroundImage: `url(${blockPreviews.get(b) || b.image.src})`}"></div>
                      <div class="media-actions">
                          <label class="file-btn small">
                          {{ uploadingState.get(b) ? '处理中...' : '更换' }}
                          <input type="file" accept="image/*" @change="(e) => handleMediaUpload(e, b)" />
                          </label>
                      </div>
                  </div>
                </div>

                <!-- 音频块: 仅上传, 自动时长 -->
                <div v-if="b.type==='audio' && b.audio" class="media-input-group">
                    <div v-if="!b.audio.src && !blockPreviews.get(b)" class="upload-placeholder">
                      <label class="file-btn primary">
                        <span v-if="uploadingState.get(b)">⏳ 压缩中...</span>
                        <span v-else>🎵 点击上传音频</span>
                        <input type="file" accept="audio/*" @change="(e) => handleMediaUpload(e, b)" />
                      </label>
                    </div>
                    <div v-else class="media-preview-row">
                      <div class="audio-icon">🔊</div>
                      <input type="text" v-model="b.audio.title" class="modern-input title-input" placeholder="音频标题" />
                      <span class="duration-badge">{{ b.audio.durationText || '0:00' }}</span>
                      <label class="file-btn small">
                          {{ uploadingState.get(b) ? '处理中...' : '更换' }}
                          <input type="file" accept="audio/*" @change="(e) => handleMediaUpload(e, b)" />
                      </label>
                    </div>
                </div>
              </div>

              <button class="btn-block-del" @click="removeBlock(q.stemBlocks, bi)">×</button>
            </div>
          </div>

          <!-- 添加按钮组 -->
          <div class="action-bar">
            <button class="btn-pill" @click="addBlock(q.stemBlocks, 'text')">+ 文本</button>
            <button class="btn-pill" @click="addBlock(q.stemBlocks, 'image')">+ 图片</button>
            <button class="btn-pill" @click="addBlock(q.stemBlocks, 'audio')">+ 音频</button>
          </div>
        </div>

        <!-- 3. 答案/选项区 -->
        <div class="section-area answer-bg">
          <div class="section-title">答案 & 选项</div>
          
          <div v-if="q.type==='choice'" class="choice-container">
             <div class="correct-answer-row">
               <label>正确选项：</label>
               <div class="radio-group">
                 <label v-for="opt in q.options" :key="opt.key" class="radio-item" :class="{active: q.answer === opt.key}">
                   <input type="radio" :value="opt.key" v-model="q.answer" />
                   {{ opt.key }}
                 </label>
               </div>
             </div>

             <div class="options-list">
               <div v-for="(opt, oi) in q.options" :key="oi" class="option-row">
                  <div class="opt-key">{{ opt.key }}</div>
                  <input v-model="opt.text" class="modern-input" placeholder="选项内容" />
                  <button class="btn-icon-small" @click="removeOption(q, oi)">×</button>
               </div>
             </div>
             
             <!-- 仅当选项少于4个时显示添加按钮 -->
             <button v-if="q.options && q.options.length < 4" class="btn-dashed" @click="addOption(q)">+ 添加选项</button>
          </div>

          <div v-else-if="q.type==='multiple'" class="multiple-container">
             <div class="correct-answer-row">
               <label>正确答案（可多选）：</label>
               <div class="checkbox-group">
                 <label v-for="opt in q.options" :key="opt.key" 
                        class="checkbox-item" 
                        :class="{active: getMultipleAnswers(q.answer).includes(opt.key)}">
                   <input type="checkbox" 
                          :checked="getMultipleAnswers(q.answer).includes(opt.key)"
                          @change="setMultipleAnswer(q, opt.key, ($event.target as HTMLInputElement).checked)" />
                   {{ opt.key }}
                 </label>
               </div>
             </div>

             <div class="options-list">
               <div v-for="(opt, oi) in q.options" :key="oi" class="option-row">
                  <div class="opt-key">{{ opt.key }}</div>
                  <input v-model="opt.text" class="modern-input" placeholder="选项内容" />
                  <button class="btn-icon-small" @click="removeOption(q, oi)">×</button>
               </div>
             </div>
             
             <!-- 仅当选项少于4个时显示添加按钮 -->
             <button v-if="q.options && q.options.length < 4" class="btn-dashed" @click="addOption(q)">+ 添加选项</button>
          </div>

          <div v-else-if="q.type==='judgment'" class="judgment-container">
             <div class="correct-answer-row">
               <label>正确答案：</label>
               <div class="radio-group">
                 <label class="radio-item" :class="{active: q.answer === 'true'}">
                   <input type="radio" value="true" v-model="q.answer" />
                   ✓
                 </label>
                 <label class="radio-item" :class="{active: q.answer === 'false'}">
                   <input type="radio" value="false" v-model="q.answer" />
                   ✗
                 </label>
               </div>
             </div>
          </div>

          <div v-else class="fill-container">
             <label>标准答案：</label>
             <input type="text" v-model="q.answer" class="modern-input" placeholder="请输入填空题的正确答案" />
          </div>
        </div>

        <!-- 4. 解析区 -->
        <div class="section-area analysis-bg">
           <div class="section-title">解析 (选填)</div>
           <div v-for="(b, bi) in q.analysisBlocks" :key="bi" class="block-row">
              <div class="block-icon">💡</div>
              <div class="block-body">
                 <textarea v-if="b.type==='text'" v-model="b.text" rows="1" class="modern-textarea" placeholder="输入解析..."></textarea>
              </div>
              <button class="btn-block-del" @click="q.analysisBlocks?.splice(bi,1)">×</button>
           </div>
           <div class="action-bar" v-if="(!q.analysisBlocks || q.analysisBlocks.length === 0)">
              <button class="btn-pill small" @click="if(!q.analysisBlocks) q.analysisBlocks=[]; addBlock(q.analysisBlocks, 'text')">+ 添加解析</button>
           </div>
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
/* 容器 */
.panel {
  display: flex; flex-direction: column; 
  background: #f8fafc; /* 更柔和的背景灰 */
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0,0,0,0.03);
  min-height: 400px; /* 给一个最小高度保证视觉效果 */
}

/* 顶部 Tab 栏 */
.tabs-header {
  background: #fff;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
  height: 56px;
  flex-shrink: 0;
  border-radius: 12px 12px 0 0; /* 顶部圆角 */
}
.tabs { display: flex; gap: 4px; background: #f1f5f9; padding: 4px; border-radius: 8px; }
.tabs button {
  padding: 6px 16px;
  border: 0;
  background: transparent;
  color: #64748b;
  font-weight: 600;
  font-size: 13px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.tabs button:hover { color: #334155; }
.tabs button.active {
  background: #fff;
  color: #16a34a;
  box-shadow: 0 1px 3px rgba(0,0,0,0.05);
}
.header-action .btn-primary-ghost {
  border: 1px solid #16a34a;
  background: transparent;
  color: #16a34a;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  display: flex; align-items: center; gap: 4px;
  transition: all 0.2s;
  white-space: nowrap;
}
.header-action .btn-primary-ghost:hover {
  background: #f0fdf4;
}

/* 列表区 */
.q-list {
  padding: 24px;
  display: flex; flex-direction: column; gap: 24px;
}
.empty-state { text-align: center; color: #94a3b8; margin-top: 40px; font-size: 14px; }

/* 题目卡片 */
.q-card {
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.04);
  border: 1px solid transparent;
  transition: all 0.2s ease;
  overflow: hidden;
}
.q-card:hover {
  box-shadow: 0 8px 24px rgba(0,0,0,0.06);
  border-color: #e2e8f0;
}

/* 题目头部 */
.q-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 16px;
  background: #fff;
  border-bottom: 1px solid #f1f5f9;
}
.q-badge {
  background: #1e293b; color: #fff;
  font-size: 14px; font-weight: 800;
  padding: 4px 10px;
  border-radius: 6px;
  font-family: monospace;
}
.q-meta-group { display: flex; gap: 16px; align-items: center; }
.meta-item { display: flex; align-items: center; gap: 6px; }
.meta-item label { font-size: 12px; color: #94a3b8; font-weight: 600; text-transform: uppercase; }

/* 现代输入框 */
.modern-input, .modern-select, .modern-textarea {
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 13px;
  color: #334155;
  transition: border-color 0.2s;
  background: #fff;
}
.modern-input:focus, .modern-select:focus, .modern-textarea:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59,130,246,0.1);
}
.modern-input.short { width: 80px; }
.modern-input.mini { width: 50px; text-align: center; }
.modern-input.title-input { flex: 1; }
.modern-select { padding-right: 24px; cursor: pointer; }
.modern-textarea { width: 100%; resize: vertical; line-height: 1.5; }

/* 区域 */
.section-area { padding: 16px; border-bottom: 1px solid #f8fafc; }
.section-area:last-child { border-bottom: 0; }
.section-title { font-size: 11px; font-weight: 700; color: #cbd5e1; text-transform: uppercase; margin-bottom: 12px; letter-spacing: 0.5px; }

/* 题干块列表 */
.blocks-container { display: flex; flex-direction: column; gap: 12px; margin-bottom: 12px; }
.block-row { display: flex; gap: 10px; align-items: flex-start; }
.block-icon {
  width: 24px; height: 24px;
  background: #f1f5f9; color: #64748b;
  border-radius: 4px;
  display: flex; align-items: center; justify-content: center;
  font-size: 12px; font-weight: 700;
  flex-shrink: 0; margin-top: 4px;
}
.block-body { flex: 1; display: flex; flex-direction: column; gap: 8px; }

/* Media Input Styling */
.media-input-group { background: #f8fafc; padding: 10px; border-radius: 8px; border: 1px dashed #e2e8f0; }
.upload-placeholder { display: flex; justify-content: center; width: 100%; }
.media-preview-row { display: flex; align-items: center; gap: 10px; width: 100%; }

.audio-icon { 
  width: 32px; height: 32px; background: #e0f2fe; color: #0284c7; 
  border-radius: 50%; display: flex; align-items: center; justify-content: center; 
}
.duration-badge { font-size: 11px; font-weight: 600; color: #94a3b8; background: #f1f5f9; padding: 2px 6px; border-radius: 4px; }

/* 按钮样式 */
.file-btn {
  position: relative; overflow: hidden; cursor: pointer;
  display: inline-flex; align-items: center; justify-content: center;
  border-radius: 6px; font-weight: 600; transition: all 0.2s;
}
.file-btn.primary {
  background: #fff; border: 1px solid #e2e8f0; color: #64748b;
  padding: 8px 16px; font-size: 13px; width: 100%;
}
.file-btn.primary:hover { border-color: #cbd5e1; color: #334155; background: #f8fafc; }
.file-btn.small {
  font-size: 12px; padding: 4px 10px;
  background: #fff; border: 1px solid #e2e8f0; color: #64748b;
}
.file-btn.small:hover { color: #334155; border-color: #94a3b8; }

.file-btn input[type="file"] { position: absolute; left: 0; top: 0; opacity: 0; width: 100%; height: 100%; cursor: pointer; }
.media-preview { width: 60px; height: 60px; border-radius: 6px; background-size: cover; background-position: center; border: 1px solid #e2e8f0; }
.media-actions { display: flex; flex-direction: column; justify-content: center; }

.btn-icon-del { background: transparent; border: 0; cursor: pointer; font-size: 18px; opacity: 0.5; transition: opacity 0.2s; }
.btn-icon-del:hover { opacity: 1; }
.btn-block-del { width: 30px; height: 30px; border-radius: 50%; border: 0; background: #fee2e2; color: #ef4444; font-size: 16px; cursor: pointer; display: flex; align-items: center; justify-content: center; margin-top: 2px; opacity: 0.8; transition: opacity 0.2s; }
.btn-block-del:hover { opacity: 1; }
.btn-icon-small { border: 0; background: transparent; color: #94a3b8; font-size: 18px; cursor: pointer; }
.btn-icon-small:hover { color: #ef4444; }

.action-bar { display: flex; gap: 8px; }
.btn-pill {
  border: 1px dashed #cbd5e1;
  background: #fff; color: #64748b;
  padding: 6px 14px; border-radius: 20px;
  font-size: 12px; font-weight: 600;
  cursor: pointer; transition: all 0.2s;
}
.btn-pill:hover { border-color: #94a3b8; color: #334155; background: #f8fafc; }
.btn-pill.small { padding: 4px 10px; font-size: 11px; }

/* 答案区特殊样式 */
.answer-bg { background: #fdfbf7; /* 极淡的黄色背景，区分区域 */ }
.choice-container { display: flex; flex-direction: column; gap: 12px; }
.correct-answer-row { display: flex; align-items: center; gap: 12px; margin-bottom: 8px; }
.correct-answer-row label { font-size: 12px; font-weight: 700; color: #d97706; }

.radio-group { display: flex; gap: 8px; }
.radio-item {
  display: flex; align-items: center; justify-content: center;
  width: 32px; height: 32px; border-radius: 50%;
  border: 1px solid #e2e8f0; background: #fff;
  cursor: pointer; font-weight: 700; font-size: 13px; color: #64748b;
  transition: all 0.2s;
}
.radio-item input { display: none; }
.radio-item.active { background: #d97706; color: #fff; border-color: #d97706; box-shadow: 0 2px 5px rgba(217,119,6,0.3); }

.options-list { display: flex; flex-direction: column; gap: 8px; }
.option-row { display: flex; align-items: center; gap: 8px; }
.opt-key { width: 24px; text-align: center; font-weight: 700; color: #64748b; font-size: 13px; }
.btn-dashed {
  width: 100%; padding: 8px; border: 1px dashed #cbd5e1; background: transparent;
  color: #64748b; font-size: 12px; font-weight: 600; border-radius: 6px; cursor: pointer;
  margin-top: 8px; transition: all 0.2s;
}
.btn-dashed:hover { border-color: #94a3b8; color: #334155; }

/* 填空题 */
.fill-container { display: flex; align-items: center; gap: 12px; }
.fill-container label { font-size: 13px; font-weight: 700; color: #475569; }

/* 多选题 */
.multiple-container { display: flex; flex-direction: column; gap: 12px; }
.checkbox-group { display: flex; gap: 8px; flex-wrap: wrap; }
.checkbox-item {
  display: flex; align-items: center; justify-content: center;
  width: 32px; height: 32px; border-radius: 6px;
  border: 1px solid #e2e8f0; background: #fff;
  cursor: pointer; font-weight: 700; font-size: 13px; color: #64748b;
  transition: all 0.2s;
}
.checkbox-item input { display: none; }
.checkbox-item.active { 
  background: #3b82f6; 
  color: #fff; 
  border-color: #3b82f6; 
  box-shadow: 0 2px 5px rgba(59,130,246,0.3); 
}

/* 判断题 */
.judgment-container { display: flex; flex-direction: column; gap: 12px; }

/* 解析区 */
.analysis-bg { background: #f0fdf4; /* 极淡的绿色 */ }

/* Mobile Adaptation */
@media (max-width: 768px) {
  .tabs-header { padding: 0 8px; }
  .tabs button { padding: 6px 10px; }
  .header-action .btn-primary-ghost { padding: 6px 10px; }
  .header-action .btn-primary-ghost .icon { display: none; }
  
  .q-list { padding: 12px; }
  .q-header { flex-wrap: wrap; gap: 10px; position: relative; }
  
  /* Make Delete button large and absolute top-right */
  .btn-icon-del { 
    position: absolute; 
    top: 10px; 
    right: 10px; 
    font-size: 20px; 
    padding: 10px; /* larger tap target */
  }
  
  .q-meta-group { 
    width: 100%; 
    flex-wrap: wrap; 
    gap: 10px; 
    margin-top: 4px;
  }
  
  /* Inputs full width on mobile or specific layout */
  .meta-item {
    flex: 1 1 45%; /* 2 items per row approx */
    min-width: 100px;
  }
  .modern-input.short { width: 100%; }
  
  /* Block row needs to accommodate narrow screens */
  .block-row { gap: 6px; }
  .block-icon { margin-top: 8px; }
  .btn-block-del { width: 32px; height: 32px; flex-shrink: 0; }
  
  .fill-container { flex-direction: column; align-items: flex-start; gap: 4px; }
  .fill-container .modern-input { width: 100%; }
}
</style>