<script setup lang="ts">
import { computed, ref, nextTick } from 'vue';
import type { LevelConfig } from '@/types/exam';
import { api } from '@/services/api';
import { compressImage } from '@/utils/file';
import { useExamStore } from '@/stores/exam';

const props = defineProps<{
  levels: LevelConfig[];
}>();

const examStore = useExamStore();
const uploadingStates = ref<Record<string, boolean>>({}); // 记录上传状态
const previewMap = ref<Record<string, string>>({}); // 本地预览图映射表

// --- 预设资源库 ---
const PRESET_AVATARS = [
  { name: '默认-元气少女', url: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='50' fill='%23fef3c7'/><path d='M30,40 Q50,60 70,40' fill='none' stroke='%23d97706' stroke-width='3' stroke-linecap='round'/><circle cx='35' cy='45' r='5' fill='%23d97706'/><circle cx='65' cy='45' r='5' fill='%23d97706'/><path d='M45,60 Q50,65 55,60' fill='none' stroke='%23d97706' stroke-width='3'/></svg>" },
  { name: '默认-眼镜学霸', url: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><rect width='100' height='100' fill='%23e0e7ff'/><circle cx='35' cy='45' r='12' fill='none' stroke='%233730a3' stroke-width='2'/><circle cx='65' cy='45' r='12' fill='none' stroke='%233730a3' stroke-width='2'/><line x1='47' y1='45' x2='53' y2='45' stroke='%233730a3' stroke-width='2'/><path d='M45,70 Q50,70 55,70' fill='none' stroke='%233730a3' stroke-width='2'/></svg>" },
  { name: '默认-文艺青年', url: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='50' fill='%23dcfce7'/><circle cx='35' cy='45' r='11' fill='none' stroke='%2315803d' stroke-width='2'/><circle cx='65' cy='45' r='11' fill='none' stroke='%2315803d' stroke-width='2'/><path d='M46,45 L54,45' stroke='%2315803d' stroke-width='2'/><path d='M42,70 Q50,75 58,70' fill='none' stroke='%2315803d' stroke-width='2' stroke-linecap='round'/></svg>" },
  { name: '默认-猫系卖萌', url: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='50' fill='%23fce7f3'/><path d='M30,45 Q35,40 40,45' fill='none' stroke='%23be185d' stroke-width='3' stroke-linecap='round'/><path d='M60,45 Q65,40 70,45' fill='none' stroke='%23be185d' stroke-width='3' stroke-linecap='round'/><path d='M45,65 Q50,70 55,65' fill='none' stroke='%23be185d' stroke-width='3' stroke-linecap='round'/><path d='M20,60 L30,62' stroke='%23be185d' stroke-width='2' opacity='0.5'/><path d='M80,60 L70,62' stroke='%23be185d' stroke-width='2' opacity='0.5'/></svg>" },
  { name: '默认-佛系咸鱼', url: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><rect width='100' height='100' fill='%23f3f4f6'/><path d='M30,50 L42,50' stroke='%23374151' stroke-width='3' stroke-linecap='round'/><path d='M58,50 L70,50' stroke='%23374151' stroke-width='3' stroke-linecap='round'/><path d='M82,25 L92,15 M85,35 L90,30' stroke='%23374151' stroke-width='2' opacity='0.6'/><path d='M40,70 Q50,70 60,70' fill='none' stroke='%23374151' stroke-width='3' stroke-linecap='round'/></svg>" },
  { name: '默认-自然森系', url: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='50' fill='%23ecfccb'/><path d='M50,20 L50,10 M50,20 Q30,30 20,50 M50,20 Q70,30 80,50' stroke='%2365a30d' stroke-width='3' fill='none'/><circle cx='35' cy='55' r='4' fill='%233f6212'/><circle cx='65' cy='55' r='4' fill='%233f6212'/><path d='M45,75 Q50,80 55,75' stroke='%233f6212' stroke-width='2' fill='none'/></svg>" },
  { name: '默认-运动健将', url: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='50' fill='%23ffedd5'/><rect x='20' y='25' width='60' height='12' fill='%23ef4444' rx='4'/><circle cx='35' cy='55' r='5' fill='%231f2937'/><circle cx='65' cy='55' r='5' fill='%231f2937'/><path d='M35,75 Q50,85 65,75' fill='none' stroke='%231f2937' stroke-width='2' stroke-linecap='round'/></svg>" },
  { name: 'Notion-Leo', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Leo" },
  { name: 'Notion-Micah', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Micah" },
  { name: 'Notion-Felix', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Felix" },
  { name: 'Notion-Aneka', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Aneka" },
  { name: 'Notion-Zoe', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Zoe" },
  { name: 'Notion-Chris', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Christopher" },
  { name: 'Notion-Mason', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Mason" },
  { name: 'Notion-Jude', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Jude" },
  { name: 'Notion-Avery', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Avery" },
  { name: 'Notion-Riley', url: "https://api.dicebear.com/9.x/notionists/svg?seed=Riley" },
];

const PRESET_SKETCHES = [
  // --- 京阿尼/凉宫系列 ---
  { name: '凉宫春日', url: new URL('../../assets/images/haruhi.webp', import.meta.url).href },
  { name: '长门有希', url: new URL('../../assets/images/nagato.webp', import.meta.url).href },
  { name: '朝比奈实玖瑠', url: new URL('../../assets/images/mikuru.webp', import.meta.url).href },
  { name: '阿虚', url: new URL('../../assets/images/kyon.webp', import.meta.url).href },
  { name: '古泉一树', url: new URL('../../assets/images/itsuki.webp', import.meta.url).href },
  { name: '朝仓凉子', url: new URL('../../assets/images/asakura.webp', import.meta.url).href },
  { name: '鹤屋学姐', url: new URL('../../assets/images/tsuruya.webp', import.meta.url).href },
  { name: '喜绿江美里', url: new URL('../../assets/images/kimidori.webp', import.meta.url).href },
  { name: '谷口', url: new URL('../../assets/images/taniguchi.webp', import.meta.url).href },
  { name: '学生会长', url: new URL('../../assets/images/president.webp', import.meta.url).href },
  { name: '虚妹', url: new URL('../../assets/images/imouto.webp', import.meta.url).href },
  // --- 轻音少女 ---
  { name: '平泽唯', url: new URL('../../assets/images/平泽唯.webp', import.meta.url).href },
  { name: '秋山澪', url: new URL('../../assets/images/秋山澪.webp', import.meta.url).href },
  { name: '田井中律', url: new URL('../../assets/images/田井中律.webp', import.meta.url).href },
  { name: '琴吹紬', url: new URL('../../assets/images/琴吹紬.webp', import.meta.url).href },
  { name: '中野梓', url: new URL('../../assets/images/中野梓.webp', import.meta.url).href },
  // --- 吹响吧！上低音号 ---
  { name: '黄前久美子', url: new URL('../../assets/images/黄前久美子.webp', import.meta.url).href },
  { name: '高坂丽奈', url: new URL('../../assets/images/竞速.webp', import.meta.url).href }, 
  // --- 冰菓 ---
  { name: '千反田爱茹', url: new URL('../../assets/images/千反田.webp', import.meta.url).href },
  { name: '伊原摩耶花', url: new URL('../../assets/images/伊原摩耶花.webp', import.meta.url).href },
  // --- 中二病 ---
  { name: '小鸟游六花', url: new URL('../../assets/images/六花.webp', import.meta.url).href },
  // --- 紫罗兰 ---
  { name: '薇尔莉特', url: new URL('../../assets/images/薇尔莉特.webp', import.meta.url).href },
  // --- 幸运星 ---
  { name: '泉此方', url: new URL('../../assets/images/泉此方.webp', import.meta.url).href },
  // --- Clannad ---
  { name: '古河渚', url: new URL('../../assets/images/古河渚.webp', import.meta.url).href },
  // --- 甘城光辉 ---
  { name: '千斗五十铃', url: new URL('../../assets/images/千斗五十铃.webp', import.meta.url).href },
  // --- 境界的彼方 ---
  { name: '栗山未来', url: new URL('../../assets/images/栗山未来.webp', import.meta.url).href },
  // --- 龙女仆 ---
  { name: '托尔', url: new URL('../../assets/images/托尔.webp', import.meta.url).href },
  // --- 玉子市场 ---
  { name: '北白川玉子', url: new URL('../../assets/images/玉子.webp', import.meta.url).href },
  // --- 其他/场景 ---
  { name: '浮岛', url: new URL('../../assets/images/浮岛.webp', import.meta.url).href },
  { name: '柳', url: new URL('../../assets/images/柳.webp', import.meta.url).href },
  { name: '山与人间', url: new URL('../../assets/images/山与人间.webp', import.meta.url).href },
  { name: '维多利亚', url: new URL('../../assets/images/维多利亚.webp', import.meta.url).href },
  { name: '伊鲁席尔', url: new URL('../../assets/images/伊鲁席尔.webp', import.meta.url).href },
  { name: '雨后', url: new URL('../../assets/images/雨后.webp', import.meta.url).href },
];

const BAD_IMAGE_URL = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='100' height='100'><rect width='100' height='100' fill='%23a1a1aa'/><text x='50' y='50' text-anchor='middle' fill='white' font-size='12'>灰底图</text></svg>";
const QR_PLACEHOLDER = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='80' height='80'><rect width='80' height='80' fill='%23eee'/><text x='40' y='40' text-anchor='middle' dominant-baseline='middle' fill='%23ccc' font-size='12' font-weight='bold'>QR</text></svg>";

const guideImoutoAvatar = new URL('../../assets/images/imouto-profile.webp', import.meta.url).href;
const guideImoutoSketch = new URL('../../assets/images/imouto.webp', import.meta.url).href;

const customAvatars = ref<{name: string, url: string}[]>([]);
const customSketches = ref<{name: string, url: string}[]>([]);

const conflictErrors = computed(() => {
  const errors: Record<number, string> = {};
  props.levels.forEach((lv1, idx1) => {
    if (lv1.min === undefined || lv1.min === null || lv1.max === undefined || lv1.max === null) return;
    if (lv1.min > lv1.max) {
      errors[idx1] = '最低分不能高于最高分';
      return;
    }
    for (let idx2 = 0; idx2 < props.levels.length; idx2++) {
      if (idx1 === idx2) continue;
      const lv2 = props.levels[idx2];
      if (lv2.min === undefined || lv2.min === null || lv2.max === undefined || lv2.max === null) continue;
      const isOverlap = Math.max(lv1.min, lv2.min) <= Math.min(lv1.max, lv2.max);
      if (isOverlap) {
        errors[idx1] = `与等级 #${idx2 + 1} 分数区间重叠`;
        break;
      }
    }
  });
  return errors;
});

const validate = () => {
  if (Object.keys(conflictErrors.value).length > 0) {
    alert('检测到等级配置存在分数冲突，请修正后再提交！');
    return false;
  }
  return true;
};

defineExpose({ validate });

const addLevel = async () => {
  const newId = `lv_${Date.now()}`;
  props.levels.push({
    id: newId, min: 0, max: 100, name: '新等级', color: '#333333', img: '', sketch: '', comment: '请输入评语...'
  });
  await nextTick();
  const el = document.getElementById(`level-card-${newId}`);
  if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' });
};

const insertLevel = async (index: number) => {
  const newId = `lv_${Date.now()}`;
  props.levels.splice(index + 1, 0, {
    id: newId, min: 0, max: 100, name: '新等级', color: '#333333', img: '', sketch: '', comment: '请输入评语...'
  });
  await nextTick();
  const el = document.getElementById(`level-card-${newId}`);
  if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' });
};

const resetToGeneric = () => {
  props.levels.splice(0, props.levels.length);
  props.levels.push(
    { id: `lv_gen_${Date.now()}_1`, min: 0, max: 59, name: '再接再厉', color: '#64748b', img: '', sketch: '', comment: '看来这次试题有些难度。别气馁，查漏补缺，下次一定能取得更好的成绩！' },
    { id: `lv_gen_${Date.now()}_2`, min: 60, max: 79, name: '成绩合格', color: '#059669', img: '', sketch: '', comment: '恭喜你通过了测试！你的基础知识掌握得不错，继续保持这份热情吧。' },
    { id: `lv_gen_${Date.now()}_3`, min: 80, max: 100, name: '出类拔萃', color: '#d97706', img: '', sketch: '', comment: '太出色了！近乎完美的表现证明了你的实力。你是最棒的！' }
  );
  isUnlocked.value = true;
};

const isUnlocked = ref(false);
const isDefaultTemplate = computed(() => props.levels.length > 0 && props.levels[0].id === 'taniguchi');
const showLockOverlay = computed(() => isDefaultTemplate.value && !isUnlocked.value);

const removeLevel = (index: number) => {
  if (confirm('确定删除这个等级吗？')) props.levels.splice(index, 1);
};

// 获取预览图的辅助函数：优先使用本地Blob URL，否则使用数据模型中的URL
const getPreviewUrl = (id: string, field: string, modelUrl: string) => {
  return previewMap.value[`${id}_${field}`] || modelUrl;
};

// [新增] 专门用于处理用户手动切换图片选择的函数
// 作用：更新数据的同时，清除之前可能存在的“强制本地预览”状态
const selectImage = (level: LevelConfig, field: 'img' | 'sketch', url: string) => {
  // 1. 更新数据模型
  level[field] = url;
  
  // 2. 清除该位置的强制预览（如果之前上传过图片，previewMap里会有值）
  // 只有删除了 previewMap 中的条目，getPreviewUrl 才会回退到使用 level[field]
  const key = `${level.id}_${field}`;
  if (previewMap.value[key]) {
    delete previewMap.value[key];
  }
};

const handleFileUpload = async (e: Event, level: LevelConfig, field: 'img' | 'sketch') => {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;

  const uploadKey = `${level.id}_${field}`;
  uploadingStates.value[uploadKey] = true; // 开启 Loading

  try {
    // A. 立即显示本地预览
    const localUrl = URL.createObjectURL(file);
    previewMap.value[uploadKey] = localUrl;
    
    // [临时] 也更新到 level 方便调试，但最终展示会优先取 previewMap
    level[field] = localUrl;
    
    // 同步更新到自定义列表(UI显示用)
    const newItem = { name: `自定义上传`, url: localUrl };
    if (field === 'img') customAvatars.value.push(newItem);
    else customSketches.value.push(newItem);

    // B. 图片前端压缩 (转WebP, 0.75质量)
    const compressedFile = await compressImage(file, 0.75);

    // C. 上传到服务器
    const serverUrl = await api.uploadFile(compressedFile);

    // D. 上传成功，将服务端路径保存到数据对象(level)
    level[field] = serverUrl;
    
    // E. 记录到 Store
    examStore.recordUpload(serverUrl);

  } catch (err) {
    console.error('Upload failed', err);
    alert('上传失败，请重试');
  } finally {
    uploadingStates.value[uploadKey] = false; // 关闭 Loading
    target.value = ''; // 清空 input，允许再次选择同一文件
  }
};
</script>

<template>
  <div class="panel">
    <!-- 顶部引导区 -->
    <div class="guide-box">
      <div class="guide-title">📢 配置建议与效果预览 (手机端左右滑动查看)</div>
      <div class="guide-row">
        <!-- Case 1: Sample (Imouto) -->
        <div class="guide-card">
          <div class="tag">样例</div>
          <div class="guide-preview-wrapper">
             <div class="preview-scale-wrapper">
               <div class="overview-card-export">
                  <div class="ov-bg-deco"></div>
                  <!-- 图片层 -->
                  <div class="ov-sketch-container">
                      <img :src="guideImoutoSketch" class="ov-sketch-img" crossorigin="anonymous" />
                  </div>
                  <div class="ov-header-row">
                      <div class="ov-char-container">
                        <div class="ov-char-img-box" style="border-color: #ff8a65">
                            <img :src="guideImoutoAvatar" class="ov-char-img" crossorigin="anonymous" />
                        </div>
                        <div class="ov-level-badge" style="background-color: #ff8a65">
                            虚妹级
                        </div>
                        <div class="ov-score-row">
                            <span class="label">最终得分</span>
                            <span class="num" style="color: #ff8a65">35</span>
                            <span class="unit">分</span>
                        </div>
                      </div>
                  </div>
                  <div class="ov-content">
                      <div class="ov-comment-box">
                        <div class="qt">“</div>
                        <div class="txt">阿虚～阿虚～这个分数是不是有点太孩子气了？如果不加油的话，就比三味线还笨了哦！</div>
                        <div class="qt r">”</div>
                      </div>
                      <div class="ov-info-row">
                        <div class="ov-user-info">
                            <div class="u-name">用户名</div>
                            <div class="u-meta">考试日期 · 试卷名称</div>
                        </div>
                        <div class="ov-qr">
                            <img :src="QR_PLACEHOLDER" />
                            <span>扫码挑战</span>
                        </div>
                      </div>
                  </div>
                  <div class="ov-footer">Generated by Haruyuki.cn</div>
               </div>
             </div>
          </div>
          <div class="guide-desc">
            配置合适的等级非常重要，这些等级会在用户交卷时根据得分显示，并用于生成结果分享卡片。有趣的文案、合适的图片可以提升试卷的传播效果
          </div>
        </div>

        <!-- Case 2: 预设图 -->
        <div class="guide-card recommend">
          <div class="guide-preview-wrapper">
             <div class="preview-scale-wrapper">
               <div class="overview-card-export">
                  <div class="ov-bg-deco"></div>
                  <!-- 图片层 -->
                  <div class="ov-sketch-container">
                      <img :src="PRESET_SKETCHES[11].url" class="ov-sketch-img" crossorigin="anonymous" />
                  </div>
                  <div class="ov-header-row">
                      <div class="ov-char-container">
                        <div class="ov-char-img-box" style="border-color: #d97706">
                            <img :src="PRESET_AVATARS[0].url" class="ov-char-img" crossorigin="anonymous" />
                        </div>
                        <div class="ov-level-badge" style="background-color: #d97706">
                            再接再厉
                        </div>
                        <div class="ov-score-row">
                            <span class="label">最终得分</span>
                            <span class="num" style="color: #d97706">65</span>
                            <span class="unit">分</span>
                        </div>
                      </div>
                  </div>
                  <div class="ov-content">
                      <div class="ov-comment-box">
                        <div class="qt">“</div>
                        <div class="txt">正在朝着更高的目标稳步迈进哦，不要气馁！</div>
                        <div class="qt r">”</div>
                      </div>
                      <div class="ov-info-row">
                        <div class="ov-user-info">
                            <div class="u-name">用户名</div>
                            <div class="u-meta">考试日期 · 试卷名称</div>
                        </div>
                        <div class="ov-qr">
                            <img :src="QR_PLACEHOLDER" />
                            <span>扫码挑战</span>
                        </div>
                      </div>
                  </div>
                  <div class="ov-footer">Generated by Haruyuki.cn</div>
               </div>
             </div>
          </div>
          <div class="guide-desc">
            我们提供了一组预设图片，如果你不知道使用什么图片比较好，可以从中选择哦。
          </div>
        </div>

        <!-- Case 3: 不合适 -->
        <div class="guide-card warning">
          <div class="guide-preview-wrapper">
             <div class="preview-scale-wrapper">
               <div class="overview-card-export">
                  <div class="ov-bg-deco"></div>
                  <!-- 图片层: 灰底错误示范 -->
                  <div class="ov-sketch-container">
                      <img :src="BAD_IMAGE_URL" class="ov-sketch-img" crossorigin="anonymous" style="mix-blend-mode: multiply;" />
                  </div>
                  <div class="ov-header-row">
                      <div class="ov-char-container">
                        <div class="ov-char-img-box" style="border-color: #333">
                            <img :src="PRESET_AVATARS[1].url" class="ov-char-img" crossorigin="anonymous" />
                        </div>
                        <div class="ov-level-badge" style="background-color: #333">
                            效果不佳
                        </div>
                        <div class="ov-score-row">
                            <span class="label">最终得分</span>
                            <span class="num" style="color: #333">30</span>
                            <span class="unit">分</span>
                        </div>
                      </div>
                  </div>
                  <div class="ov-content">
                      <div class="ov-comment-box">
                        <div class="qt">“</div>
                        <div class="txt">自定义图片尽量以纯白或透明为背景，并减少两侧留白，否则将会影响展示效果。</div>
                        <div class="qt r">”</div>
                      </div>
                      <div class="ov-info-row">
                        <div class="ov-user-info">
                            <div class="u-name">用户名</div>
                            <div class="u-meta">考试日期 · 试卷名称</div>
                        </div>
                        <div class="ov-qr">
                            <img :src="QR_PLACEHOLDER" />
                            <span>扫码挑战</span>
                        </div>
                      </div>
                  </div>
                  <div class="ov-footer">Generated by Haruyuki.cn</div>
               </div>
             </div>
          </div>
          <div class="guide-desc">
            自定义背景图片尽量以纯白或透明为背景，否则将会影响展示效果。
          </div>
        </div>
      </div>
    </div>

    <!-- 列表容器，包含遮罩层 -->
    <div class="list-container">
      
      <!-- 锁定遮罩 -->
      <transition name="fade">
        <div v-if="showLockOverlay" class="lock-overlay">
          <div class="lock-content">
             <div class="lock-icon">🔒</div>
             <h3>等级待配置</h3>
             <p>当前加载的是样卷数据。等级配置会用来生成分享卡片，对于试卷传播非常重要，强烈建议进行编辑，否则默认将会使用这套凉宫春日主题的等级配置。</p>
             
             <button class="ui green big-btn" @click="resetToGeneric">
               编辑并初始化
             </button>
          </div>
        </div>
      </transition>

      <!-- 等级列表 (被模糊层) -->
      <div class="level-list" :class="{ blurred: showLockOverlay }">
        <div 
          v-for="(lv, idx) in levels" 
          :key="lv.id" 
          :id="`level-card-${lv.id}`"
          class="level-card"
        >
          <div class="lv-header" :style="{ borderLeftColor: lv.color }">
            <span class="lv-idx">等级 #{{ idx + 1 }}</span>
            <div class="header-actions">
              <button class="action-btn insert" @click="insertLevel(idx)" title="在下方插入新等级">
                <span class="icon">+</span> 插入
              </button>
              <button class="action-btn del" @click="removeLevel(idx)" title="删除此等级">
                删除
              </button>
            </div>
          </div>
          
          <div class="lv-body-flex">
            <!-- 左侧：表单配置 -->
            <div class="form-area">
              <div class="row" style="flex-wrap: wrap;">
                <div 
                  class="form-group small" 
                  :class="{ 'error-input': !!conflictErrors[idx] }"
                >
                  <label>最低分</label>
                  <input v-model.number="lv.min" type="number" />
                </div>
                <div 
                  class="form-group small"
                  :class="{ 'error-input': !!conflictErrors[idx] }"
                >
                  <label>最高分</label>
                  <input v-model.number="lv.max" type="number" />
                </div>
                <div class="form-group name-group">
                  <label>等级名称</label>
                  <input v-model="lv.name" type="text" placeholder="例如：阿虚级" />
                </div>
                <div class="form-group small">
                  <label>主题色</label>
                  <input v-model="lv.color" type="color" class="color-picker" />
                </div>
                
                <!-- 错误提示信息 -->
                <div v-if="conflictErrors[idx]" class="error-msg-banner">
                  ⚠️ {{ conflictErrors[idx] }}
                </div>
              </div>

              <!-- 图片配置: 头像 -->
              <div class="row">
                <div class="form-group">
                  <label>
                    头像图片
                    <span v-if="uploadingStates[`${lv.id}_img`]" class="loading-tag">上传中...</span>
                  </label>
                  
                  <!-- 响应式图片选择器容器 -->
                  <div class="responsive-picker">
                    <!-- [移动端] 左侧固定上传按钮 -->
                    <div class="upload-option square mobile-only">
                      <input type="file" accept="image/*" @change="(e) => handleFileUpload(e, lv, 'img')" />
                      <span class="upload-icon">+</span>
                    </div>

                    <!-- 滚动/网格列表区域 -->
                    <div class="scroll-track">
                      <!-- 预设列表 -->
                      <div 
                        v-for="p in PRESET_AVATARS" 
                        :key="p.name"
                        class="img-option"
                        :class="{ active: lv.img === p.url }"
                        @click="selectImage(lv, 'img', p.url)"
                        :title="p.name"
                      >
                        <img :src="p.url" />
                      </div>
                      
                      <!-- 用户上传列表 (仅当前有效) -->
                      <div 
                        v-for="c in customAvatars" 
                        :key="c.name"
                        class="img-option custom"
                        :class="{ active: lv.img === c.url }"
                        @click="selectImage(lv, 'img', c.url)"
                        :title="c.name"
                      >
                        <img :src="c.url" />
                        <span class="custom-badge">自</span>
                      </div>

                      <!-- [桌面端] 尾部上传按钮 (流式布局) -->
                      <div class="upload-option desktop-only">
                        <input type="file" accept="image/*" @change="(e) => handleFileUpload(e, lv, 'img')" />
                        <span class="text">上传新图</span>
                      </div>
                    </div>

                    <!-- [移动端] 右侧引导箭头 -->
                    <div class="scroll-hint-arrow mobile-only">→</div>
                  </div>

                </div>
              </div>

              <!-- 图片配置: 背景 -->
              <div class="row">
                <div class="form-group">
                  <label>
                    背景图片
                    <span v-if="uploadingStates[`${lv.id}_sketch`]" class="loading-tag">上传中...</span>
                  </label>
                  
                  <!-- 响应式图片选择器容器 -->
                  <div class="responsive-picker">
                    <!-- [移动端] 左侧固定上传按钮 -->
                    <div class="upload-option square mobile-only">
                      <input type="file" accept="image/*" @change="(e) => handleFileUpload(e, lv, 'sketch')" />
                      <span class="upload-icon">+</span>
                    </div>

                    <!-- 滚动/网格列表区域 -->
                    <div class="scroll-track">
                      <!-- 预设列表 -->
                      <div 
                        v-for="p in PRESET_SKETCHES" 
                        :key="p.name"
                        class="img-option"
                        :class="{ active: lv.sketch === p.url }"
                        @click="selectImage(lv, 'sketch', p.url)"
                        :title="p.name"
                      >
                        <img :src="p.url" />
                      </div>

                      <!-- 用户上传列表 -->
                      <div 
                        v-for="c in customSketches" 
                        :key="c.name"
                        class="img-option custom"
                        :class="{ active: lv.sketch === c.url }"
                        @click="selectImage(lv, 'sketch', c.url)"
                        :title="c.name"
                      >
                        <img :src="c.url" />
                        <span class="custom-badge">自</span>
                      </div>

                      <!-- [桌面端] 尾部上传按钮 (流式布局) -->
                      <div class="upload-option desktop-only">
                        <input type="file" accept="image/*" @change="(e) => handleFileUpload(e, lv, 'sketch')" />
                        <span class="text">上传新图</span>
                      </div>
                    </div>
                    
                    <!-- [移动端] 右侧引导箭头 -->
                    <div class="scroll-hint-arrow mobile-only">→</div>
                  </div>

                </div>
              </div>

              <div class="form-group">
                <label>文案</label>
                <textarea v-model="lv.comment" rows="2"></textarea>
              </div>
            </div>

            <!-- 右侧：真实渲染预览 -->
            <div class="preview-area">
               <div class="preview-label">导出效果预览</div>
               
               <div class="preview-scale-wrapper">
                 <div class="overview-card-export">
                    <div class="ov-bg-deco"></div>
                    <div class="ov-sketch-container">
                        <img v-if="getPreviewUrl(lv.id, 'sketch', lv.sketch)" :src="getPreviewUrl(lv.id, 'sketch', lv.sketch)" crossorigin="anonymous" class="ov-sketch-img" />
                    </div>
                    <div class="ov-header-row">
                        <div class="ov-char-container">
                          <div class="ov-char-img-box" :style="{borderColor: lv.color}">
                              <img v-if="getPreviewUrl(lv.id, 'img', lv.img)" :src="getPreviewUrl(lv.id, 'img', lv.img)" crossorigin="anonymous" class="ov-char-img" />
                              <div v-else class="ov-avatar-empty">?</div>
                          </div>
                          <div class="ov-level-badge" :style="{backgroundColor: lv.color}">
                              {{ lv.name || '称号' }}
                          </div>
                          <div class="ov-score-row">
                              <span class="label">最终得分</span>
                              <span class="num" :style="{color: lv.color}">{{ Math.floor((lv.min + lv.max)/2) }}</span>
                              <span class="unit">分</span>
                          </div>
                        </div>
                    </div>
                    <div class="ov-content">
                        <div class="ov-comment-box">
                          <div class="qt">“</div>
                          <div class="txt">{{ lv.comment || '这里是评语预览...' }}</div>
                          <div class="qt r">”</div>
                        </div>
                        <div class="ov-info-row">
                          <div class="ov-user-info">
                              <div class="u-name">考生姓名</div>
                              <div class="u-meta">考试日期 · 班级</div>
                          </div>
                          <div class="ov-qr">
                              <img :src="QR_PLACEHOLDER" />
                              <span>扫码挑战</span>
                          </div>
                        </div>
                    </div>
                    <div class="ov-footer">Generated by Haruyuki.cn</div>
                 </div>
               </div>
            </div>
          </div>
        </div>

        <!-- 添加按钮 -->
        <div class="add-level-card" @click="addLevel">
          <div class="add-icon">+</div>
          <div class="add-text">添加新等级配置</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel { padding: 16px; background: var(--sos-bg-surface); border-radius: 12px; box-shadow: 0 2px 12px rgba(0,0,0,0.04); position: relative; }

button.ui.small { padding: 6px 12px; font-size: 12px; background: var(--sos-accent); color: var(--sos-bg-surface); border: 0; border-radius: 6px; cursor: pointer; }
button.ui.outline { background: transparent; border: 1px solid var(--sos-border-strong); color: var(--sos-text-secondary); }
button.ui.outline:hover { background: var(--sos-bg-muted); color: var(--sos-text-secondary); }

/* 容器与遮罩 */
.list-container { position: relative; min-height: 300px; }

.lock-overlay {
  position: absolute; inset: -20px;
  z-index: 10;
  display: flex; 
  align-items: flex-start;
  justify-content: center;
  padding-top: 80px;
  background: rgba(255,255,255,0.25);
  backdrop-filter: blur(0.7px);
}

.lock-content {
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-default);
  padding: 40px;
  border-radius: 16px;
  text-align: center;
  box-shadow: 0 20px 40px rgba(0,0,0,0.1);
  max-width: 400px;
}
.lock-icon { font-size: 40px; margin-bottom: 16px; }
.lock-content h3 { margin: 0 0 12px 0; font-size: 18px; color: var(--sos-text-primary); }
.lock-content p { font-size: 13px; color: var(--sos-text-secondary); line-height: 1.6; margin-bottom: 24px; }

.big-btn {
  width: 100%; padding: 12px; font-size: 15px; font-weight: 700;
  background: var(--sos-accent); color: var(--sos-bg-surface); border: 0; border-radius: 8px;
  cursor: pointer; transition: transform 0.1s;
  box-shadow: 0 4px 12px color-mix(in srgb, var(--sos-accent) 25%, transparent);
}
.big-btn:hover { background: var(--sos-accent-hover); transform: translateY(-1px); }
.big-btn:active { transform: translateY(1px); }

.sub-action { margin-top: 16px; font-size: 12px; }
.sub-action span { color: var(--sos-text-tertiary); cursor: pointer; text-decoration: underline; }
.sub-action span:hover { color: var(--sos-text-secondary); }

/* 被锁定的列表样式 */
.level-list { display: flex; flex-direction: column; gap: 16px; transition: all 0.3s; }
.level-list.blurred { 
  filter: grayscale(0.2); 
  pointer-events: none; 
  user-select: none; 
  opacity: 0.9; 
}

/* Guide Section */
.guide-box { background: var(--sos-bg-subtle); border: 1px solid var(--sos-border-default); border-radius: 8px; padding: 12px; margin-bottom: 20px; overflow: hidden; }
.guide-title { font-weight: 700; font-size: 14px; color: var(--sos-text-secondary); margin-bottom: 8px; }
.guide-row { display: flex; gap: 16px; overflow-x: auto; padding-bottom: 12px; scroll-snap-type: x mandatory; -webkit-overflow-scrolling: touch; }
.guide-card { 
  flex: 1; min-width: 0; /* Flexbox safe */
  background: var(--sos-bg-surface); border: 1px solid var(--sos-border-default); border-radius: 8px; padding: 12px; 
  display: flex; flex-direction: column; align-items: center; position: relative; 
}
.guide-card.recommend { border-color: #f59e0b; background: #fffbeb; }
.guide-card.warning { border-color: var(--sos-danger); }
.guide-card .tag { position: absolute; top: 0; right: 0; background: #f59e0b; color: var(--sos-bg-surface); font-size: 10px; padding: 2px 6px; border-bottom-left-radius: 6px; font-weight: 700; }
.guide-preview-wrapper { width: 100%; display: flex; justify-content: center; margin-bottom: 0; }
.guide-desc { font-size: 11px; color: var(--sos-text-secondary); text-align: center; line-height: 1.4; }
.guide-desc strong { color: var(--sos-text-primary); }

/* Level List Styles (Card & Form) */
.level-card { border: 1px solid var(--sos-border-default); border-radius: 8px; overflow: hidden; background: var(--sos-bg-surface); }
.lv-header { background: var(--sos-bg-subtle); padding: 8px 12px; border-bottom: 1px solid var(--sos-border-default); display: flex; justify-content: space-between; border-left: 4px solid var(--sos-border-strong); }
.lv-idx { font-weight: 900; color: var(--sos-text-secondary); font-size: 13px; }

/* 新的头部按钮组样式 */
.header-actions { display: flex; gap: 8px; }
.action-btn { background: transparent; border: 1px solid transparent; cursor: pointer; font-size: 12px; padding: 2px 8px; border-radius: 4px; transition: all 0.2s; display: flex; align-items: center; gap: 2px; }
.action-btn.del { color: var(--sos-danger); }
.action-btn.del:hover { background: var(--sos-danger-soft); }
.action-btn.insert { color: var(--sos-accent-2); }
.action-btn.insert:hover { background: var(--sos-info-soft); }

.lv-body-flex { display: flex; }
.form-area { flex: 1; padding: 16px; border-right: 1px solid var(--sos-bg-muted); }
.preview-area { width: 260px; padding: 20px; background: var(--sos-bg-subtle); display: flex; flex-direction: column; align-items: center; overflow: hidden; }

/* Form Styles */
.row { display: flex; gap: 12px; margin-bottom: 12px; }
.form-group { flex: 1; min-width: 0; }
.form-group.small { flex: 0 0 70px; }
label { display: block; font-size: 12px; font-weight: 700; color: var(--sos-text-secondary); margin-bottom: 4px; }
input[type="text"], input[type="number"], textarea, select { width: 100%; padding: 8px; border: 1px solid var(--sos-border-default); border-radius: 6px; font-size: 13px; }
.color-picker { width: 100%; height: 36px; padding: 0; border: 1px solid var(--sos-border-default); border-radius: 6px; cursor: pointer; }

/* 错误提示样式 */
.form-group.error-input input {
  border-color: var(--sos-danger);
  background-color: var(--sos-danger-soft);
  color: var(--sos-danger);
}
.error-msg-banner {
  flex-basis: 100%;
  color: var(--sos-danger);
  font-size: 12px;
  font-weight: 600;
  margin-top: -8px;
  margin-bottom: 8px;
  background: var(--sos-danger-soft);
  padding: 4px 8px;
  border-radius: 4px;
}

/* -------------------------------------------------
   NEW Responsive Image Selector 
   -------------------------------------------------
*/

/* 基础容器：Desktop 默认样式 */
.responsive-picker {
  display: block; /* 默认块级，允许子元素自然流式排列 */
  margin-top: 8px;
}
.scroll-track {
  display: flex;
  flex-wrap: wrap; /* Desktop 默认换行 */
  gap: 10px;
}

/* 基础元素样式 */
.img-option { 
  width: 50px; height: 50px; border-radius: 6px; border: 2px solid transparent; 
  cursor: pointer; overflow: hidden; background: var(--sos-bg-muted); position: relative; transition: all 0.2s; 
  flex-shrink: 0; /* 防止压缩 */
}
.img-option img { width: 100%; height: 100%; object-fit: cover; }
.img-option:hover { transform: translateY(-2px); box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
.img-option.active { border-color: var(--sos-accent); box-shadow: 0 0 0 2px color-mix(in srgb, var(--sos-accent) 30%, transparent); }
.img-option.custom .custom-badge { position: absolute; bottom: 0; right: 0; background: var(--sos-accent-2); color: var(--sos-bg-surface); font-size: 8px; padding: 1px 3px; border-top-left-radius: 4px; }

/* Upload Button Styles */
.upload-option { 
  border-radius: 6px; border: 1px dashed var(--sos-text-tertiary); 
  display: flex; flex-direction: column; align-items: center; justify-content: center; 
  cursor: pointer; background: var(--sos-bg-subtle); color: var(--sos-text-secondary); position: relative; transition: all 0.2s; 
}
.upload-option:hover { border-color: var(--sos-accent); color: var(--sos-accent); background: var(--sos-success-soft); }
.upload-option input[type="file"] { position: absolute; inset: 0; opacity: 0; cursor: pointer; }
.upload-option .text { font-size: 12px; font-weight: 600; margin-top: 2px; }

/* Desktop Upload Button (Flow Layout) */
.upload-option.desktop-only {
  width: 100px; height: 50px; /* 原有宽按钮 */
}

/* Mobile Fixed Upload Button (Square) */
.upload-option.square {
  width: 50px; height: 50px; 
}
.upload-icon { font-size: 24px; font-weight: 300; line-height: 1; }

/* Scroll Arrow Hint (Bouncing) */
.scroll-hint-arrow {
  position: absolute; right: 0; top: 50%; transform: translateY(-50%);
  background: rgba(255,255,255,0.85); border-radius: 50%; width: 24px; height: 24px;
  display: flex; align-items: center; justify-content: center;
  box-shadow: -2px 0 8px rgba(0,0,0,0.15); pointer-events: none; /* 透传点击 */
  font-size: 12px; color: var(--sos-text-secondary); font-weight: bold;
  animation: bounce-right 1.5s infinite; z-index: 5;
  backdrop-filter: blur(2px);
}
@keyframes bounce-right {
  0%, 100% { transform: translate(0, -50%); opacity: 0.8; }
  50% { transform: translate(5px, -50%); opacity: 1; }
}

/* --- Media Queries for Layout Switching --- */

/* Default / Desktop (> 768px) */
.mobile-only {
  display: none !important;
}

/* Mobile (<= 768px) */
@media (max-width: 768px) {
  .desktop-only {
    display: none !important;
  }
  
  .mobile-only {
    display: flex !important; /* Force flex for button/arrow */
  }

  /* 切换为 Flex 行布局 */
  .responsive-picker {
    display: flex;
    align-items: center;
    gap: 10px;
    position: relative; /* For arrow positioning */
  }
  
  /* 列表改为横向滚动 */
  .scroll-track {
    flex: 1;
    flex-wrap: nowrap;
    overflow-x: auto;
    padding-bottom: 4px; /* Space for shadow */
    padding-right: 30px; /* Space for arrow hint */
    scrollbar-width: none; /* Firefox */
  }
  .scroll-track::-webkit-scrollbar {
    display: none; /* Chrome/Safari */
  }
}

.add-level-card {
  border: 2px dashed var(--sos-border-default);
  border-radius: 8px;
  padding: 32px;
  background: var(--sos-bg-subtle);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  transition: all 0.2s;
  color: var(--sos-text-tertiary);
}
.add-level-card:hover {
  border-color: var(--sos-border-strong);
  background: var(--sos-bg-subtle);
  color: var(--sos-text-secondary);
}
.add-icon {
  font-size: 32px;
  line-height: 1;
  font-weight: 300;
}
.add-text {
  font-size: 14px;
  font-weight: 600;
}

/* 1:1 Preview Styles (Reused from paper.scss) */
.preview-label { font-size: 11px; font-weight: 700; color: var(--sos-text-tertiary); margin-bottom: 8px; text-transform: uppercase; }
.preview-scale-wrapper { width: 220px; height: 260px; position: relative; }
.overview-card-export { width: 600px; background: var(--sos-bg-surface); border-radius: 24px; overflow: hidden; box-shadow: 0 20px 60px rgba(0,0,0,0.1); font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; position: absolute; top: 0; left: 0; transform: scale(0.36); transform-origin: top left; display: flex; flex-direction: column; }
.ov-bg-deco { height: 140px; background: transparent; position: absolute; top: 0; left: 0; right: 0; z-index: 0; }
.ov-bg-deco::after { content: ""; position: absolute; inset: 0; background-image: radial-gradient(var(--sos-border-default) 1px, transparent 1px); background-size: 16px 16px; opacity: 0.5; }
.ov-header-row { position: relative; z-index: 2; display: flex; flex-direction: row; align-items: center; justify-content: space-between; margin-top: 40px; padding: 0 40px; }
.ov-char-container { display: flex; flex-direction: column; align-items: center; } /* 关键：移除了 width: 100% 以支持左右布局 */
.ov-sketch-container { position: absolute; top: 0; right: 0; width: 70%; height: 475px; z-index: 0; pointer-events: none; display: block; -webkit-mask-image: radial-gradient(circle at 80% 20%, black 30%, transparent 85%); mask-image: radial-gradient(circle at 80% 20%, black 30%, transparent 85%); }
.ov-sketch-img { width: 100%; height: 100%; object-fit: cover; object-position: top center; mix-blend-mode: multiply; opacity: 0.9; }
.ov-char-img-box { width: 160px; height: 160px; border-radius: 50%; border: 6px solid var(--sos-border-strong); background: var(--sos-bg-surface); overflow: hidden; box-shadow: 0 8px 24px rgba(0,0,0,0.12); display: flex; align-items: center; justify-content: center; }
.ov-char-img { width: 100%; height: 100%; object-fit: cover; }
.ov-avatar-empty { color: var(--sos-border-strong); font-weight: 900; font-size: 48px; }
.ov-level-badge { margin-top: -20px; background: var(--sos-text-secondary); color: var(--sos-bg-surface); padding: 8px 24px; border-radius: 999px; font-size: 24px; font-weight: 900; box-shadow: 0 4px 12px rgba(0,0,0,0.2); letter-spacing: 1px; align-self: center; }
.ov-content { padding: 30px 40px 40px; display: flex; flex-direction: column; align-items: center; position: relative; z-index: 2; }
.ov-score-row { display: flex; align-items: baseline; gap: 8px; margin-top: 20px; }
.ov-score-row .label { font-size: 16px; color: var(--sos-text-tertiary); font-weight: 600; }
.ov-score-row .num { font-family: "Impact", sans-serif; font-size: 60px; line-height: 1; }
.ov-score-row .unit { font-size: 18px; color: var(--sos-text-secondary); font-weight: 800; }
.ov-comment-box { background: var(--sos-bg-subtle); border-radius: 16px; padding: 24px 30px; position: relative; width: 100%; box-sizing: border-box; margin-bottom: 30px; }
.ov-comment-box .qt { font-size: 40px; line-height: 1; color: var(--sos-border-default); font-family: serif; position: absolute; }
.ov-comment-box .qt:not(.r) { top: 10px; left: 10px; }
.ov-comment-box .qt.r { bottom: -10px; right: 10px; }
.ov-comment-box .txt { font-size: 18px; line-height: 1.6; color: var(--sos-text-secondary); font-weight: 600; text-align: justify; }
.ov-info-row { width: 100%; display: flex; justify-content: space-between; align-items: center; border-top: 2px dashed var(--sos-border-default); padding-top: 24px; }
.ov-user-info { display: flex; flex-direction: column; gap: 4px; }
.ov-user-info .u-name { font-size: 22px; font-weight: 900; color: var(--sos-text-primary); }
.ov-user-info .u-meta { font-size: 14px; color: var(--sos-text-tertiary); font-weight: 500; }
.ov-qr { display: flex; flex-direction: column; align-items: center; gap: 4px; }
.ov-qr-img-placeholder { width: 80px; height: 80px; background: var(--sos-border-default); display: flex; align-items: center; justify-content: center; color: var(--sos-text-tertiary); font-size: 12px; font-weight: bold; border-radius: 8px; }
.ov-qr span { font-size: 10px; color: var(--sos-text-tertiary); font-weight: 700; letter-spacing: 1px; }
.ov-footer { background: var(--sos-border-subtle); color: var(--sos-text-tertiary); text-align: center; padding: 12px; font-size: 12px; font-weight: 600; letter-spacing: 0.5px; }

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

/* Mobile Adaptation */
@media (max-width: 768px) {
  .guide-row {
    flex-wrap: nowrap; /* 强制不换行，支持横滑 */
    justify-content: flex-start;
  }
  .guide-card {
    flex: 0 0 88%; /* 稍微露出下一张卡片，暗示可以滑动 */
    scroll-snap-align: center;
  }
  
  .lv-body-flex {
    flex-direction: column;
  }
  .form-area {
    border-right: none;
    border-bottom: 1px solid var(--sos-bg-muted);
  }
  .preview-area {
    width: 100%;
    padding: 16px;
    background: var(--sos-bg-subtle);
  }
  
  /* Make preview centered on mobile */
  .preview-scale-wrapper {
    margin: 0 auto;
  }
  
  .form-group.small {
    flex: 1; /* Make min/max width equal on mobile */
  }
  .name-group {
    flex: 0 0 100%; /* Force name to new line */
  }
}
.loading-tag {
  font-size: 10px;
  color: #d97706;
  background: #fef3c7;
  padding: 2px 6px;
  border-radius: 4px;
  margin-left: 6px;
  font-weight: normal;
}
</style>