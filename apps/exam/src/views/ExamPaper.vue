<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useExamStore } from '@/stores/exam';
import { useAudioStore } from '@/stores/audio';
import { useCameraRig } from '@/composables/useCameraRig';
import QuestionRenderer from '@/components/QuestionRenderer.vue';
import { toPng } from 'html-to-image';
import QRCode from 'qrcode';
import { usePageMeta, canonicalUrl } from '@haruhi/seo';
import type { Question } from '@/types/exam';


const props = defineProps<{
  isHaruhi?: boolean;
  id?: string;
}>();

const route = useRoute();
const router = useRouter();
const examStore = useExamStore();
const audioStore = useAudioStore();
const { rigEl, rig, rigStyle, isMobile, measureRig, onRigDown, onRigMove, onRigUp } = useCameraRig();

const mode = ref<'preview' | 'solve' | 'marked'>('preview');
// 试卷数据是否已就绪（store.paper 初始为内置 mock，须待 loadExam 完成后才能用于页面 meta）
const metaReady = ref(false);
const side = ref<'front' | 'back'>('front');
const activeColumn = ref<'C1' | 'C2' | 'C3' | 'C4'>('C1');
const viewColumn = ref<'C1' | 'C2' | 'C3' | 'C4'>('C1'); 
const solveInnerEl = ref<HTMLElement | null>(null);

const nameModal = reactive({ on: false, value: '' });
const shareModal = reactive({ on: false, src: '' });
const overviewModal = reactive({ on: false, src: '' });
const lightbox = reactive({ on: false, src: '', title: '', x: 0, y: 0, z: 1 });
const isGeneratingImage = ref(false);

const qrCodeUrl = ref(''); 

const exportEl = ref<HTMLElement | null>(null);
const overviewExportEl = ref<HTMLElement | null>(null);
const nameInputEl = ref<HTMLInputElement | null>(null);

const scoreDash = ref(0);
const scoreText = computed(() => `${examStore.result.score}分`);

const BASE_WIDTH = 420;
const appScale = ref(1);

const goUpLevel = () => {
  const currentPath = route.path;
  const segments = currentPath.split('/').filter(p => p);
  
  if (segments.length >= 2) {
    // 移除最后两级
    segments.pop(); // 移除倒数第一级
    segments.pop(); // 移除倒数第二级
    const upPath = '/' + segments.join('/');
    router.push(upPath);
  } else {
    // 如果层级不足两级（比如 /home），直接回根目录
    router.push('/');
  }
};

const updateAppScale = () => {
  const ww = window.innerWidth;
  if (ww < BASE_WIDTH) {
    appScale.value = ww / BASE_WIDTH;
  } else {
    appScale.value = 1;
  }
};

const questionsByColumn = computed(() => {
  const qs = examStore.paper.questions || [];
  return {
    C1: qs.filter((q: Question) => q.column === 'C1'),
    C2: qs.filter((q: Question) => q.column === 'C2'),
    C3: qs.filter((q: Question) => q.column === 'C3'),
    C4: qs.filter((q: Question) => q.column === 'C4'),
  };
});

const colIndex = (c: string) => {
  const map: Record<string, number> = { C1: 1, C2: 2, C3: 3, C4: 4 };
  return map[c] || 1;
};

const isLastColumn = computed(() => activeColumn.value === 'C4');

const columnComplete = (col: string) => {
  const groups = questionsByColumn.value;
  const key = col as keyof typeof groups;
  const qs = groups[key] || [];
  if (qs.length === 0) return true;
  
  for (const q of qs) {
    if (q.type === 'choice' && !examStore.answers[q.id]) return false;
    if (q.type === 'fill' && !examStore.answers[q.id]) return false;
  }
  return true;
};

const getQuestions = (col: string) => {
  const key = col as keyof typeof questionsByColumn.value;
  return questionsByColumn.value[key] || [];
};

const clamp = (v: number, min: number, max: number) => Math.max(min, Math.min(max, v));

const panToViewColumn = (c: string) => {
  if (!isMobile.value) return;
  if (mode.value === 'solve') return;

  const winW = rigEl.value?.parentElement?.clientWidth || window.innerWidth;
  const margin = 2;
  const focusLeft = (c === 'C1' || c === 'C3');
  let targetX = 0;

  if (focusLeft) {
    targetX = (rig.spreadW / 2) - (winW / 2) + margin;
  } else {
    targetX = (winW / 2) - margin - (rig.spreadW / 2);
  }

  rig.panX = clamp(Math.round(targetX), rig.minPanX, rig.maxPanX);
  rig.panY = 0;
};

const applyMobileView = () => {
  if (!isMobile.value) return;
  if (mode.value === 'solve') return;
  
  if (viewColumn.value === 'C1' || viewColumn.value === 'C2') {
    side.value = 'front';
  } else {
    side.value = 'back';
  }
  
  nextTick(() => panToViewColumn(viewColumn.value));
};

watch(viewColumn, () => nextTick(applyMobileView));
watch(mode, () => nextTick(applyMobileView));

watch(() => rig.spreadW, (val) => {
  if (val > 0 && isMobile.value) {
    setTimeout(applyMobileView, 50);
  }
});

watch(isMobile, (val) => {
  nextTick(() => {
    if (val) {
      applyMobileView();
    } else {
      rig.panX = 0;
      rig.panY = 0;
    }
  });
});

const switchMobileColumn = () => {
  if (!isMobile.value) return;
  audioStore.stop();
  const map: Record<string, 'C1' | 'C2' | 'C3' | 'C4'> = { C1:'C2', C2:'C1', C3:'C4', C4:'C3' };
  viewColumn.value = map[viewColumn.value];
  applyMobileView();
};

const mobileSwitchText = computed(() => (viewColumn.value==='C1'||viewColumn.value==='C3') ? '看右栏' : '看左栏');
const mobileNextDir = computed(() => (viewColumn.value==='C1'||viewColumn.value==='C3') ? 'right' : 'left');

const toggleSide = () => {
  if (mode.value === 'solve') return;
  side.value = side.value === 'front' ? 'back' : 'front';
  
  if (isMobile.value) {
     viewColumn.value = side.value === 'front' ? 'C1' : 'C3';
  }
  audioStore.stop();
};

const openNameModal = () => {
  nameModal.value = examStore.userName;
  nameModal.on = true;
  nextTick(() => nameInputEl.value?.focus());
};

const confirmName = (skip = false) => {
  if (!skip) examStore.userName = nameModal.value.slice(0, 20);
  nameModal.on = false;
};

const startExam = () => {
  audioStore.stop();
  mode.value = 'solve';
  activeColumn.value = 'C1';
  viewColumn.value = 'C1';
  nextTick(() => {
    if (solveInnerEl.value) solveInnerEl.value.scrollTop = 0;
  });
};

const jumpToPreview = () => {
  audioStore.stop();
  mode.value = 'preview';
  viewColumn.value = activeColumn.value;
  if (isMobile.value) nextTick(applyMobileView);
};

const goNextColumn = () => {
  const map: Record<string, 'C1' | 'C2' | 'C3' | 'C4'> = { C1:'C2', C2:'C3', C3:'C4', C4:'C1' };
  const nxt = map[activeColumn.value];
  if (!nxt) return;
  
  audioStore.stop();
  if (nxt === 'C3') side.value = 'back'; 
  
  activeColumn.value = nxt;
  viewColumn.value = nxt;
  nextTick(() => {
    if (solveInnerEl.value) solveInnerEl.value.scrollTop = 0;
  });
};

const submit = () => {
  examStore.submitPaper();
  mode.value = 'marked';
  side.value = 'front';
  viewColumn.value = 'C1'; 
  if (isMobile.value) nextTick(applyMobileView);
  audioStore.stop();
  
  scoreDash.value = 180;
  const start = performance.now();
  const dur = 520;
  const tick = () => {
    const t = Math.max(0, Math.min(1, (performance.now() - start) / dur));
    scoreDash.value = 180 - 180 * t;
    if (t < 1) requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
};

const resetAll = () => {
  examStore.resetExam();
  mode.value = 'preview';
  side.value = 'front';
  activeColumn.value = 'C1';
  viewColumn.value = 'C1';
  if (isMobile.value) nextTick(applyMobileView);
};

const waitForAssets = async (el: HTMLElement) => {
  await document.fonts.ready;
  const imgs = Array.from(el.querySelectorAll('img'));
  await Promise.all(imgs.map((img: HTMLImageElement) => {
    if (img.complete) return Promise.resolve();
    return new Promise(r => { img.onload = r; img.onerror = r; });
  }));
};

const generateOverview = async () => {
  if (!overviewExportEl.value || isGeneratingImage.value) return;
  isGeneratingImage.value = true;
  overviewModal.src = '';
  overviewModal.on = true;

  try {
    const baseUrl = window.location.origin;
    const targetId = props.isHaruhi ? 'haruhi' : (props.id || examStore.paper.id);
    const url = `${baseUrl}/exam/?highlight=${targetId}`;
    
    qrCodeUrl.value = await QRCode.toDataURL(url, { 
      margin: 1, 
      width: 150,
      color: {
        dark: examStore.currentLevel?.color || '#000000',
        light: '#ffffff'
      }
    });
  } catch (e) {
    console.warn('QR Code generation failed, fallback to config image if available', e);
    qrCodeUrl.value = examStore.paper.config.qrCodeImg || ''; 
  }
  
  await nextTick();
  await waitForAssets(overviewExportEl.value);
  await new Promise(r => setTimeout(r, 100)); 
  
  try {
    const url = await toPng(overviewExportEl.value, { pixelRatio: 3, backgroundColor: '#ffffff' });
    overviewModal.src = url;
  } catch(e) {
    alert('生成失败，请重试');
    overviewModal.on = false;
  } finally {
    isGeneratingImage.value = false;
  }
};

const shareImage = async () => {
  if (!exportEl.value || isGeneratingImage.value) return;
  isGeneratingImage.value = true;
  shareModal.src = '';
  shareModal.on = true;
  
  await nextTick();
  await waitForAssets(exportEl.value);
  await new Promise(r => setTimeout(r, 100));
  
  try {
    const url = await toPng(exportEl.value, { pixelRatio: 2, backgroundColor: '#e3dccb' });
    shareModal.src = url;
  } catch(e) {
    alert('生成失败，请重试');
    shareModal.on = false;
  } finally {
    isGeneratingImage.value = false;
  }
};

const saveImage = (src: string, name: string) => {
  const a = document.createElement('a');
  a.download = name;
  a.href = src;
  a.click();
};

const openImage = (img: { src: string; title?: string }) => {
  if (!img || !img.src) return;
  audioStore.stop();
  lightbox.on = true;
  lightbox.src = img.src;
  lightbox.title = img.title || '';
  lightbox.x = 0; lightbox.y = 0; lightbox.z = 1;
};
const closeImage = () => { lightbox.on = false; };
const lbDown = (e: PointerEvent) => {
};

onMounted(() => {
  updateAppScale();
  window.addEventListener('resize', updateAppScale, { passive: true });

  examStore.loadExam(props.id, props.isHaruhi).then(() => {
    metaReady.value = true;
    // 如果没有被锁定，才执行后续 DOM 逻辑
    if (examStore.accessStatus === 'allow') {
      nextTick(() => {
        measureRig();
        if (!examStore.userName) openNameModal();
        if (isMobile.value) setTimeout(applyMobileView, 100);
      });
    }
  });
});

onUnmounted(() => {
  window.removeEventListener('resize', updateAppScale);
});

// 页面 meta：仅自定义试卷（/exam/:id）在数据加载成功后设置；
// 内置卷 /haruhi 走路由级静态 title，加载中/出错/被锁定时返回 null 保留静态兜底。
usePageMeta(() => {
  if (props.isHaruhi || !props.id) return null;
  if (!metaReady.value || examStore.loading || examStore.error || examStore.accessStatus !== 'allow') {
    return null;
  }
  const cfg = examStore.paper.config;
  const title = cfg.title || cfg.paperTitle;
  return {
    title: `${title} · 春日试卷中心`,
    description: cfg.paperSubtitle || `来挑战《${title}》`,
    canonical: canonicalUrl(`/exam/${props.id}`),
    ogType: 'website',
  };
});
</script>

<template>
  <div v-if="examStore.loading" style="height:100vh; display:flex; align-items:center; justify-content:center; color:#666;">
    试卷加载中...
  </div>
  
  <!-- 🚫 拦截显示 UI：当试卷被锁定或审核中 -->
  <div v-else-if="examStore.accessStatus !== 'allow'" class="desk access-denied-view">
     <div class="wood"></div>
     <div class="vignette"></div>
     
     <div class="access-card">
        <div class="icon-area">
           <svg v-if="examStore.accessStatus === 'pending'" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color:#f59e0b"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
           <svg v-else width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color:#ef4444"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
        </div>
        <div class="title">
           {{ examStore.accessStatus === 'pending' ? '试卷审核中' : '试卷已锁定' }}
        </div>
        <div class="reason">
           {{ examStore.accessStatus === 'pending' ? '系统正在处理' : '试卷存在异常' }}
        </div>
        <div class="desc">
           {{ examStore.accessStatus === 'pending' ? '该试卷仍在审核中' : '该试卷可能存在违规内容或被判定为垃圾信息，若为误判可联系管理员邮箱haruhiism15532@outlook.com' }}
        </div>
        <div class="actions">
           <a href="#" @click.prevent="goUpLevel" class="ui" style="text-decoration: none;">返回首页</a>
        </div>
     </div>
  </div>

  <div v-else-if="examStore.error" style="height:100vh; display:flex; align-items:center; justify-content:center; flex-direction:column; gap:10px;">
    <div>{{ examStore.error }}</div>
    <a href="#" @click.prevent="goUpLevel" class="ui" style="text-decoration: none;">返回首页</a>
  </div>
  
  <div v-else class="desk">
    <div class="wood"></div>
    <div class="depth"></div>
    <div class="vignette"></div>

    <div class="decor pencil"></div>
    <div class="decor paperclip"></div>
    <div class="decor note"></div>

    <!-- 🔙 移动端左上角简约返回按钮 -->
    <div class="m-back-nav" v-if="isMobile" @click="goUpLevel">
       <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
         <path d="M15 18l-6-6 6-6"/>
       </svg>
    </div>

    <div class="topbar">
      <div class="t">
        <a href="#" @click.prevent="goUpLevel" class="title home-link" style="text-decoration:none; color:inherit; display:flex; align-items:center; gap:4px;">
           <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" stroke-linejoin="round" style="transform: translateY(1px);">
              <path d="M15 18l-6-6 6-6"/>
           </svg>
           试卷中心
        </a>
        <div class="sub">
          <span v-if="mode==='marked'">已批改（点击右侧按钮分享结果）</span>
          <span v-else-if="mode==='solve' && examStore.result.submitted">批改详情：可上下滚动查看全部题目</span>
          <span v-else-if="mode==='solve'">正在作答：第 {{ colIndex(activeColumn) }} 栏</span>
          <span v-else>阅览态：点击试卷开始考试</span>
        </div>
      </div>
      <div class="btns">
         <button class="ui ghost" v-if="!examStore.result.submitted" @click="openNameModal">改姓名</button>
         <button class="ui ghost" v-if="mode!=='solve'" @click="toggleSide">{{ side==='front'?'背面':'正面' }}</button>
         <button class="ui red" v-if="mode==='solve'" @click="submit">交卷</button>
         <button class="ui" v-if="mode==='marked'" @click="mode='solve'">阅卷</button>
         <button class="ui green" v-if="mode==='marked'" @click="generateOverview" :disabled="isGeneratingImage">分享结果</button>
         <button class="ui green" v-if="mode==='marked'" @click="shareImage" :disabled="isGeneratingImage">完整试卷</button>
         <button class="ui ghost" v-if="mode==='marked'" @click="resetAll">重做</button>
      </div>
    </div>

    <svg width="0" height="0" style="position:absolute">
      <filter id="inkTurb" x="-20%" y="-20%" width="140%" height="140%">
        <feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" stitchTiles="stitch" result="n" />
        <feDisplacementMap in="SourceGraphic" in2="n" scale="0.7" xChannelSelector="R" yChannelSelector="G" />
      </filter>
    </svg>

    <!-- Name Modal -->
    <transition name="fade">
      <div class="name-overlay" v-if="nameModal.on" @click.self="confirmName(true)">
        <div class="name-card">
          <div class="hd">请输入姓名</div>
          <div class="bd">
            <input class="name-input" ref="nameInputEl" v-model="nameModal.value" maxlength="20" placeholder="例如：凉宫春日" @keydown.enter.prevent="confirmName()" />
            <div class="name-tip">留下你的姓名吧！之后也可以重新设置。</div>
          </div>
          <div class="name-actions">
            <button class="ui ghost" @click="confirmName(true)">先不填</button>
            <button class="ui" @click="confirmName()">确定</button>
          </div>
        </div>
      </div>
    </transition>

    <div class="stage" :class="{ solving: mode==='solve' }">
      
      <!-- Mobile Hint -->
      <transition name="fade">
        <div class="m-start-hint" v-if="isMobile && mode==='preview'" @click="startExam">
          <div class="bubble"><span class="dot"></span><span>点击试卷开始考试</span></div>
        </div>
      </transition>

      <div class="paper-viewport" :class="{ mobile: isMobile }" @click="mode==='preview' && startExam()">
        <div class="rig" ref="rigEl" :class="{ dragging: rig.dragging }" :style="rigStyle"
             @pointerdown="onRigDown" @pointermove="onRigMove" @pointerup="onRigUp" @pointercancel="onRigUp">
             
             <div class="lift"></div>
             <div class="spread">
               <div class="spread-edge"></div>
               <div class="sheet" :class="{ flipped: side === 'back' }">
                 
                 <!-- Front -->
                 <div class="side front">
                   <div class="gutterShade" :style="{ width: rigStyle['--gutter'], opacity: 0 }"></div>
                   <div class="spread-grid" :style="{ gap: rigStyle['--gutter'] }">
                     <div class="panel">
                       <div class="panel-inner">
                         <div class="paper-header">
                           <div class="h1">{{ examStore.paper.config.paperTitle }}</div>
                           <div class="h2">{{ examStore.paper.config.paperMeta }}</div>
                           <div class="fields">
                             <div class="field"><span>姓名</span><div class="line"><span class="handname">{{ examStore.userName }}</span></div></div>
                             <div class="field"><span>考号</span><div class="line"><span class="handmeta">{{ examStore.examNo }}</span></div></div>
                             <div class="field"><span>班级</span><div class="line"><span class="handmeta">{{ examStore.className }}</span></div></div>
                             <div class="field"><span>日期</span><div class="line"><span class="handmeta">{{ examStore.examDateText }}</span></div></div>
                           </div>
                           <!-- Score Display -->
                           <div class="score" v-if="mode==='marked' || (mode==='solve' && examStore.result.submitted)">
                             <svg viewBox="0 0 200 150" width="200" height="150" style="overflow:visible">
                               <text x="100" y="80" text-anchor="middle" class="score-text" font-family='"Bradley Hand","Comic Sans MS",cursive' font-size="60" font-weight="800" fill="rgba(200,23,30,.92)">{{ scoreText }}</text>
                               <path d="M40 90 C70 100, 130 100, 160 88" fill="none" stroke="rgba(200,23,30,.72)" stroke-width="4" stroke-linecap="round" :style="{strokeDasharray: 180, strokeDashoffset: scoreDash}" />
                             </svg>
                           </div>
                         </div>
                         <QuestionRenderer :questions="getQuestions('C1')" :readonly="mode!=='solve'" @openimg="openImage" />
                         <div class="footer"><span>第 1 页（正面左）</span><span>—</span></div>
                       </div>
                     </div>
                     <div class="panel">
                       <div class="panel-inner">
                         <QuestionRenderer :questions="getQuestions('C2')" :readonly="mode!=='solve'" @openimg="openImage" />
                         <div class="footer"><span>第 1 页（正面右）</span><span>—</span></div>
                       </div>
                     </div>
                   </div>
                 </div>

                 <!-- Back -->
                 <div class="side back">
                    <div class="gutterShade" :style="{ width: rigStyle['--gutter'], opacity: 0 }"></div>
                    <div class="spread-grid" :style="{ gap: rigStyle['--gutter'] }">
                      <div class="panel">
                        <div class="panel-inner">
                          <QuestionRenderer :questions="getQuestions('C3')" :readonly="mode!=='solve'" @openimg="openImage" />
                          <div class="footer"><span>第 2 页（背面左）</span><span>—</span></div>
                        </div>
                      </div>
                      <div class="panel">
                        <div class="panel-inner">
                          <QuestionRenderer :questions="getQuestions('C4')" :readonly="mode!=='solve'" @openimg="openImage" />
                          <div class="footer"><span>第 2 页（背面右）</span><span>—</span></div>
                        </div>
                      </div>
                    </div>
                 </div>

               </div>
             </div>
        </div>
        
        <!-- Mobile Pager -->
        <div class="m-pager" v-if="isMobile && mode!=='solve'">
          <button class="m-pager-btn" @click.stop="switchMobileColumn">
             <span class="m-ico" :class="mobileNextDir">
               <svg viewBox="0 0 24 24" width="18" height="18" fill="none"><path d="M9 6 L15 12 L9 18" stroke="rgba(0,0,0,.78)" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round"/></svg>
             </span>
             <span class="m-txt">{{ mobileSwitchText }}</span>
          </button>
        </div>

      </div>

      <!-- Solve Modal -->
      <transition name="pop">
        <div class="solve-overlay" v-if="mode==='solve'">
           <div class="solve-modal">
             <div class="solve-head">
               <div class="meta">
                 <div class="a">{{ examStore.result.submitted ? '批改详情' : `作答窗口 · 第 ${colIndex(activeColumn)} 栏` }}</div>
                 <div class="b" v-if="!examStore.result.submitted">{{ columnComplete(activeColumn) ? '本栏已完成，可翻页' : '请完成本栏所有题目' }}</div>
               </div>
               <div>
                 <button class="ui ghost" v-if="examStore.result.submitted" @click="mode='marked'">返回</button>
                 <button class="ui ghost" v-else @click="jumpToPreview">回到阅览</button>
               </div>
             </div>
             <div class="solve-paper">
               <div class="solve-inner" ref="solveInnerEl">
                 <!-- 批改详情 -->
                 <template v-if="examStore.result.submitted">
                    <div class="paper-header">
                      <div class="h1">{{ examStore.paper.config.paperSubtitle }}</div>
                      <div class="score"><text class="score-text">{{ scoreText }}</text></div>
                    </div>
                    <div v-for="col in ['C1','C2','C3','C4']" :key="col">
                       <div class="detail-sep"><span class="k">{{ col }} 栏</span></div>
                       <QuestionRenderer :questions="getQuestions(col)" :readonly="true" @openimg="openImage" />
                    </div>
                 </template>
                 <!-- 作答 -->
                 <template v-else>
                    <!-- Header (Only C1) -->
                    <div class="paper-header" v-if="activeColumn==='C1'">
                      <div class="h1">{{ examStore.paper.config.paperSubtitle }}</div>
                      <div class="h2">{{ examStore.paper.config.paperMeta }}</div>
                      <div class="fields">
                        <div class="field"><span>姓名</span><div class="line"><span class="handname">{{ examStore.userName }}</span></div></div>
                        <div class="field"><span>考号</span><div class="line"><span class="handmeta">{{ examStore.examNo }}</span></div></div>
                        <div class="field"><span>班级</span><div class="line"><span class="handmeta">{{ examStore.className }}</span></div></div>
                        <div class="field"><span>日期</span><div class="line"><span class="handmeta">{{ examStore.examDateText }}</span></div></div>
                      </div>
                      <div class="notice">提示：右侧滚动条可上下滚动阅览完整题目。</div>
                    </div>
                    <QuestionRenderer :questions="getQuestions(activeColumn)" :readonly="false" @openimg="openImage" />
                 </template>
               </div>
               
               <!-- Hints -->
               <div class="page-hint" :class="{ on: !examStore.result.submitted && columnComplete(activeColumn), done: isLastColumn }" v-if="!examStore.result.submitted">
                  <div class="left"><span class="txt">{{ isLastColumn ? '题目已答完' : '本栏已完成' }}</span></div>
                  <button class="act primary" @click="isLastColumn ? submit() : goNextColumn()">{{ isLastColumn ? '交卷' : '下一栏' }}</button>
               </div>
             </div>
           </div>
        </div>
      </transition>

    </div>

    <!-- Export Layer (Hidden) -->
    <div class="export-layer">
      <div ref="exportEl" class="export-container">
         <div class="export-header-deco">
           <div class="e-title">{{ examStore.paper.config.exportHeaderTitle }}</div>
           <div class="e-sub">{{ examStore.paper.config.exportHeaderSub }}</div>
           <div class="e-score-stamp">
             <div class="s-val">{{ examStore.result.score }}</div>
             <div class="s-label">分</div>
           </div>
         </div>
         <!-- Sheets C1+C2 / C3+C4 -->
         <div class="export-sheet-wrap" v-for="(pair, pi) in [['C1','C2'], ['C3','C4']]" :key="pi">
            <div class="export-sheet-label">>>> {{ pi===0?'正面 (Front)':'背面 (Back)' }}</div>
            <div class="export-sheet">
               <div class="export-grid">
                  <div class="panel" v-for="col in pair" :key="col">
                    <div class="panel-inner">
                       <!-- 补全信息栏 -->
                       <div class="paper-header" v-if="col==='C1'">
                          <div class="h1">{{ examStore.paper.config.paperSubtitle }}</div>
                          <div class="fields">
                             <div class="field"><span>姓名</span><div class="line"><span class="handname">{{ examStore.userName }}</span></div></div>
                             <div class="field"><span>考号</span><div class="line"><span class="handmeta">{{ examStore.examNo }}</span></div></div>
                             <div class="field"><span>班级</span><div class="line"><span class="handmeta">{{ examStore.className }}</span></div></div>
                             <div class="field"><span>日期</span><div class="line"><span class="handmeta">{{ examStore.examDateText }}</span></div></div>
                          </div>
                       </div>
                       <QuestionRenderer :questions="getQuestions(col)" :readonly="true" />
                    </div>
                  </div>
               </div>
            </div>
         </div>
         <div class="export-footer-deco">
           <div>{{ examStore.paper.config.exportOrgName }}</div>
           <div style="font-size:11px; opacity:0.7; margin-top:4px;">{{ examStore.paper.config.exportFooterText }}</div>
         </div>
      </div>
    </div>

    <!-- Overview Export Layer -->
    <div class="overview-export-layer">
      <div ref="overviewExportEl" class="overview-card-export" v-if="examStore.currentLevel">
         <div class="ov-bg-deco"></div>
         <div class="ov-sketch-container">
             <img :src="examStore.currentLevel.sketch" crossorigin="anonymous" class="ov-sketch-img" />
         </div>
         <div class="ov-header-row">
            <div class="ov-char-container">
               <div class="ov-char-img-box" :style="{borderColor: examStore.currentLevel.color}">
                  <img :src="examStore.currentLevel.img" crossorigin="anonymous" class="ov-char-img" />
               </div>
               <div class="ov-level-badge" :style="{backgroundColor: examStore.currentLevel.color}">
                  {{ examStore.currentLevel.name }}
               </div>
               <div class="ov-score-row">
                  <span class="label">最终得分</span>
                  <span class="num" :style="{color: examStore.currentLevel.color}">{{ examStore.result.score }}</span>
                  <span class="unit">分</span>
               </div>
            </div>
         </div>
         <div class="ov-content">
            <div class="ov-comment-box">
               <div class="qt">“</div>
               <div class="txt">{{ examStore.currentLevel.comment }}</div>
               <div class="qt r">”</div>
            </div>
            <div class="ov-info-row">
               <div class="ov-user-info">
                  <div class="u-name">{{ examStore.userName || '考生' }}</div>
                  <div class="u-meta">{{ examStore.examDateText }} · {{ examStore.className }}</div>
               </div>
               <div class="ov-qr">
                  <img :src="qrCodeUrl" width="150" height="150" />
                  <span>{{ examStore.paper.config.qrCodeText }}</span>
               </div>
            </div>
         </div>
         <div class="ov-footer">{{ examStore.paper.config.exportFooterText }}</div>
      </div>
    </div>

    <!-- Modals (Share / Overview) -->
    <transition name="fade">
      <div class="share-overlay" v-if="shareModal.on" @click.self="shareModal.on=false">
         <div class="share-card">
            <div class="hd"><span>完整试卷</span><span class="sub">长按保存或点击下载</span></div>
            <div class="bd"><img v-if="shareModal.src" :src="shareModal.src" /><div v-else class="loading">生成中...</div></div>
            <div class="ft"><button class="ui ghost" @click="shareModal.on=false">关闭</button><button class="ui green" @click="saveImage(shareModal.src, 'paper.png')">下载</button></div>
         </div>
      </div>
    </transition>
    <transition name="fade">
      <div class="overview-overlay" v-if="overviewModal.on" @click.self="overviewModal.on=false">
         <div class="overview-modal-card">
            <div class="ov-hd"><div class="t">成绩概览</div><div class="s">恭喜完成考试！</div></div>
            <div class="ov-bd"><img v-if="overviewModal.src" :src="overviewModal.src" class="ov-img" /><div v-else class="loading-state"><span class="spin"></span><span>分析中...</span></div></div>
            <div class="ov-ft"><button class="ui ghost" @click="overviewModal.on=false">关闭</button><button class="ui" @click="shareImage">完整试卷</button><button class="ui green" :disabled="!overviewModal.src" @click="saveImage(overviewModal.src, 'overview.png')">保存</button></div>
         </div>
      </div>
    </transition>

    <!-- Lightbox -->
    <transition name="fade">
      <div class="lightbox" v-if="lightbox.on" @click="closeImage">
        <div class="lb-card" @click.stop>
          <div class="lb-top">
            <span style="white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">{{ lightbox.title || '查看图片' }}</span>
            <button class="ui ghost" style="padding:8px 12px" @click="closeImage">关闭</button>
          </div>
          <div class="lb-body" @pointerdown="lbDown">
            <img :src="lightbox.src" :alt="lightbox.title||'image'">
            <div class="lb-hint">双击放大 / 拖拽移动</div>
          </div>
        </div>
      </div>
    </transition>

  </div>
</template>