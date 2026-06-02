<template>
  <div class="quiz-page-container">
    <!-- 独立的页头 -->
    <header class="quiz-header">
      <!-- Logo：已经改为 webp -->
      <img src="/haruhi-logo-192.webp" alt="Logo" class="header-logo-img">

      <span class="header-title">超~简单的凉宫入坑小测试</span>
    </header>

    <div id="quiz-app">
      <!-- 图片预览模态框 -->
      <transition name="fade">
        <div v-if="showModal" class="image-modal" @click="closeModal">
          <img :src="modalImage" alt="预览图片" @click.stop>
          <div class="modal-close-hint">点击任意空白处关闭</div>
        </div>
      </transition>

      <transition name="fade" mode="out-in">
        <!-- 1. 欢迎页 -->
        <div v-if="currentStage === 'welcome'" key="welcome" class="card">
          <div style="font-size: 4rem; margin-bottom: 20px;">🌟</div>
          <h1>超~简单的凉宫入坑小测试！</h1>
          <p>来测测你的凉宫浓度吧~🤤<br>答题后会立即显示解析哦😋</p>
          <button class="btn" @click="startQuiz">开始挑战</button>
        </div>

        <!-- 2. 答题页 -->
        <div v-else-if="currentStage === 'quiz'" key="quiz" class="card">
          <!-- 摇摆的贴纸（改为 webp） -->
          <img src="/quiz/konata.webp" class="konata-sticker" alt="Konata">

          <div class="progress-container">
            <div class="progress-bar" :style="{ width: progressPercentage + '%' }"></div>
          </div>

          <div class="question-content">
            <h2>{{ currentQuestion.text }}</h2>

            <!-- 题目图片 -->
            <img
              v-if="currentQuestion.questionImage"
              :src="formatImgPath(currentQuestion.questionImage)"
              class="question-img"
              alt="题目配图"
              @click="previewImage(formatImgPath(currentQuestion.questionImage))"
            >

            <div class="options-list">
              <div
                v-for="(option, index) in currentQuestion.options"
                :key="index"
                class="option-item"
                :class="getOptionClass(index)"
                @click="handleOptionClick(index)"
              >
                <div class="option-circle"></div>
                <span>{{ option }}</span>
              </div>
            </div>

            <!-- 答案解析区域 -->
            <div v-if="hasAnswered" class="explanation-section">
              <div class="feedback-title" :class="isCurrentCorrect ? 'is-correct' : 'is-wrong'">
                <span>{{ isCurrentCorrect ? '🎉 回答正确' : '❌ 回答错误' }}</span>
              </div>

              <div class="explanation-text">
                <strong>解析：</strong>{{ currentQuestion.explanation }}
              </div>

              <!-- 解析图片 -->
              <div
                v-if="currentQuestion.images && currentQuestion.images.length > 0"
                class="explanation-images"
              >
                <img
                  v-for="(img, idx) in currentQuestion.images"
                  :key="idx"
                  :src="formatImgPath(img)"
                  class="explanation-img"
                  @click="previewImage(formatImgPath(img))"
                >
              </div>

              <div style="text-align: right; margin-top: 15px;">
                <button
                  class="btn"
                  style="padding: 10px 30px; font-size: 1rem;"
                  @click="nextQuestion"
                >
                  {{ isLastQuestion ? '查看结果' : '下一题' }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- 3. 结果页 -->
        <div v-else-if="currentStage === 'result'" key="result" class="card">
          <div style="font-size: 4rem; margin-bottom: 20px;">🏆</div>
          <div style="margin-bottom: 20px; font-weight: bold; color: var(--accent-color);">
            挑战完成
          </div>

          <h1 style="font-size: 3.5rem; margin: 10px 0;">{{ score }} / {{ questions.length }}</h1>
          <p>{{ resultMessage }}</p>

          <button class="btn" @click="restartQuiz">再来一次</button>
          <div style="margin-top: 20px;">
            <!-- 注意：如果您的项目中没有配置路由，这里可能会报错，可以改为普通的 a 标签 -->
            <router-link to="/" class="back-link">返回主页</router-link>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';

// --- 状态定义 ---
const currentStage = ref('welcome');
const currentQuestionIndex = ref(0);
const score = ref(0);
const hasAnswered = ref(false);
const selectedOptionIndex = ref(-1);
const isCurrentCorrect = ref(false);
const showModal = ref(false);
const modalImage = ref('');

// --- 资源路径处理：自动把 png/jpg 转成 .webp ---
const formatImgPath = (filename) => {
  if (!filename) return '';

  // 外链直接返回
  if (filename.startsWith('http')) return filename;

  // 以 / 开头：认为是 public 根路径
  if (filename.startsWith('/')) {
    return filename.replace(/\.(png|jpe?g)$/i, '.webp');
  }

  // 默认认为在 /quiz 下
  return `/quiz/${filename}`.replace(/\.(png|jpe?g)$/i, '.webp');
};

// --- 题目数据（保持原来的 .png 命名不动） ---
const questions = [
  {
    id: 1,
    text: "在《凉宫春日的叹息》中，凉宫春日拍电影时希望有什么动物出现？",
    questionImage: "",
    options: ["A.鸽子，黑猫", "B.白猫，野兔", "C.黑狗，萤火虫", "D.白猫，赤兔"],
    correctIndex: 0,
    explanation: "就是A哦！",
    images: []
  },
  {
    id: 2,
    text: "下列哪个选项的动作没有在《凉宫春日的忧郁》动画版中阿虚和春日回到现实世界前做过？",
    questionImage: "",
    options: ["A.接吻", "B.搭肩", "C.摸头", "D.对视"],
    correctIndex: 2,
    explanation: "看看搭搭亲亲都干过了哦！😚",
    images: ["2222222.png", "222222222222.png"]
  },
  {
    id: 3,
    text: "在《凉宫春日的忧郁》中，谁提醒了阿虚睡美人的故事？",
    questionImage: "",
    options: ["A.长门有希", "B.古泉一树", "C.凉宫春日", "D.朝比奈实玖瑠"],
    correctIndex: 0,
    explanation: "是长门提醒了阿虚sleeping beauty🧐",
    images: []
  },
  {
    id: 4,
    text: "在《孤岛症候群》中，谁的“尸体”被发现了？",
    questionImage: "",
    options: ["A.主人", "B.女仆", "C.管家", "D.主人弟弟"],
    correctIndex: 0,
    explanation: "是宅邸的主人，也就是多丸圭一的尸体被发现了😮",
    images: []
  },
  {
    id: 5,
    text: "古泉一树在动画《凉宫春日》系列中说过几次“真羡慕你啊”？",
    questionImage: "",
    options: ["A.1", "B.2", "C.3", "D.4"],
    correctIndex: 2,
    explanation: "是三次哦，请看图片🥴",
    images: ["55555.png", "5555555.png", "55555555.png"]
  },
  {
    id: 6,
    text: "平野绫最喜欢的片段的所在集数是？",
    questionImage: "",
    options: ["A.1", "B.6", "C.19", "D.26"],
    correctIndex: 3,
    explanation: "是26集春日与阿虚在树下的谈话🥰",
    images: ["666666.png"]
  },
  {
    id: 7,
    text: "凉宫春日的眼瞳是什么颜色的？",
    questionImage: "",
    options: ["A.红白色", "B.棕橘色", "C.赤黑色", "D.紫红色"],
    correctIndex: 1,
    explanation: "是棕橘色哦，听说只有最最最喜欢春日的家伙才知道呢😎",
    images: ["77777.png"]
  },
  {
    id: 8,
    text: "这是哪个字幕组制作的字幕？",
    questionImage: "88888.png",
    options: ["A.诸神", "B.华盟", "C.SOSG", "D.以上都不是"],
    correctIndex: 2,
    explanation: "是SOSG字幕组制作的，这题是不是有点难呀？🤯",
    images: []
  },
  {
    id: 9,
    text: "《漫无止境的八月》的哪一集中阿虚在春日的活动计划表上画了该图标？",
    questionImage: "99999.png",
    options: ["A.14", "B.15", "C.16", "D.17"],
    correctIndex: 2,
    explanation: "是16集啦！哎呦这题也好难，错了的话也不要气馁，接下来就是最后一题啦🤕",
    images: ["99999999.png"]
  },
  {
    id: 10,
    text: "下面哪首不是长门有希的角色歌？",
    questionImage: "",
    options: ["A.窓辺の予感", "B.under Mebius", "C.探していた風景", "D.ただの秘密"],
    correctIndex: 3,
    explanation: "D选项是古泉唱的哦🤪",
    images: ["10000.png"]
  },
];

// --- 计算属性 ---
const currentQuestion = computed(() => questions[currentQuestionIndex.value]);

const progressPercentage = computed(() => {
  return (
    ((currentQuestionIndex.value + (hasAnswered.value ? 1 : 0)) /
      questions.length) *
    100
  );
});

const isLastQuestion = computed(
  () => currentQuestionIndex.value === questions.length - 1
);

const resultMessage = computed(() => {
  const ratio = score.value / questions.length;
  if (ratio === 1) return "不是哥们你开挂了吧，怎么会有人全对的，这么厉害😡";
  if (ratio >= 0.8) return "哇，你一定是资深凉粉，长门大萌神会保佑你😤";
  if (ratio >= 0.6) return "还是不错的嘛，这么难的题都能及格，可以去申请加入SOS团了🤗";
  if (ratio >= 0.4) return "哎呀，被难倒了吧~八嘎八嘎~要不要再试一次？🤣";
  if (ratio >= 0.2) return "杂鱼中的杂鱼，你这家伙是假粉吧！👊";
  return "你是人类吗😨";
});

// --- 方法 ---
const startQuiz = () => {
  currentStage.value = 'quiz';
  currentQuestionIndex.value = 0;
  score.value = 0;
  resetQuestionState();
};

const resetQuestionState = () => {
  hasAnswered.value = false;
  selectedOptionIndex.value = -1;
  isCurrentCorrect.value = false;
};

const handleOptionClick = (index) => {
  if (hasAnswered.value) return;

  hasAnswered.value = true;
  selectedOptionIndex.value = index;

  const correctIdx = currentQuestion.value.correctIndex;
  if (index === correctIdx) {
    isCurrentCorrect.value = true;
    score.value++;
  } else {
    isCurrentCorrect.value = false;
  }
};

const getOptionClass = (index) => {
  if (!hasAnswered.value) return '';

  const correctIdx = currentQuestion.value.correctIndex;
  if (index === correctIdx) {
    return 'correct disabled';
  }
  if (index === selectedOptionIndex.value && index !== correctIdx) {
    return 'wrong disabled';
  }
  return 'disabled';
};

const nextQuestion = () => {
  if (isLastQuestion.value) {
    currentStage.value = 'result';
  } else {
    currentQuestionIndex.value++;
    resetQuestionState();

    const appElement = document.getElementById('quiz-app');
    if (appElement) {
      appElement.scrollIntoView({ behavior: 'smooth' });
    }
  }
};

const restartQuiz = () => {
  startQuiz();
};

const previewImage = (url) => {
  modalImage.value = url;
  showModal.value = true;
};

const closeModal = () => {
  showModal.value = false;
};
</script>

<style scoped>
/* Scoped 样式，只影响当前页面 */

/* 定义变量 */
.quiz-page-container {
  /* 背景图改为 webp */
  --bg-image: url('/quiz/R-C.webp');
  --glass-bg: rgba(255, 255, 255, 0.85);
  --glass-border: rgba(255, 255, 255, 0.6);
  --header-bg: rgba(255, 255, 255, 0.6);
  --text-primary: #2d3748;
  --text-secondary: #4a5568;
  --accent-color: #e53e3e;
  --primary-color: #3182ce;
  --success-color: #48bb78;
  --error-color: #f56565;
  --btn-bg: rgba(255, 255, 255, 0.6);
  --btn-hover: rgba(255, 255, 255, 0.9);
  --option-bg: rgba(255, 255, 255, 0.5);
  --option-hover: rgba(255, 255, 255, 0.8);
  --shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.15);
  --blur-strength: 15px;

  font-family: 'Noto Sans SC', sans-serif;
  position: fixed;
  inset: 0;
  height: 100vh;
  width: 100vw;
  z-index: 9999;
  display: block;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  color: var(--text-primary);
}

/* 背景图伪元素 */
.quiz-page-container::before {
  content: '';
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image: var(--bg-image);
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  filter: blur(3px) brightness(0.95);
  z-index: -1;
  transform: scale(1.02);
}

/* 页头 */
.quiz-header {
  position: fixed;
  top: 25px;
  left: 25px;
  z-index: 100;
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 8px 20px 8px 10px;
  background: var(--header-bg);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1px solid var(--glass-border);
  border-radius: 50px;
  color: var(--text-primary);
  letter-spacing: 0.5px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.header-logo-img {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid rgba(255, 255, 255, 0.2);
}

/* 内容包裹层 */
#quiz-app {
  width: 100%;
  max-width: 650px;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: stretch;
  margin: 0 auto;
  padding: 100px 20px 40px;
  position: relative;
  z-index: 1;
}

.card {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--blur-strength));
  -webkit-backdrop-filter: blur(var(--blur-strength));
  border: 1px solid var(--glass-border);
  border-radius: 24px;
  padding: 40px 30px;
  box-shadow: var(--shadow);
  text-align: center;
  position: relative;
  overflow: hidden;
  color: var(--text-primary);
  transition: all 0.4s ease;
  height: auto;
}

h1 {
  font-size: 2rem;
  margin-bottom: 1rem;
  font-weight: 700;
}

h2 {
  font-size: 1.5rem;
  margin-bottom: 1.5rem;
  font-weight: 600;
  line-height: 1.4;
}

p {
  font-size: 1.1rem;
  line-height: 1.6;
  margin-bottom: 2rem;
  color: var(--text-secondary);
}

.btn {
  display: inline-block;
  padding: 15px 45px;
  border-radius: 50px;
  border: 1px solid var(--glass-border);
  background: var(--btn-bg);
  color: var(--text-primary);
  font-size: 1.1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  backdrop-filter: blur(5px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
  margin-top: 20px;
}

.btn:hover {
  background: var(--btn-hover);
  transform: translateY(-3px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
  border-color: var(--accent-color);
}

.question-img {
  width: 100%;
  max-height: 250px;
  object-fit: contain;
  border-radius: 12px;
  margin-bottom: 20px;
  border: 1px solid var(--glass-border);
  cursor: zoom-in;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transition: transform 0.3s ease;
  background: rgba(0, 0, 0, 0.03);
}

.question-img:hover {
  transform: scale(1.02);
}

.options-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
  margin-top: 10px;
}

.option-item {
  background: var(--option-bg);
  padding: 18px 20px;
  border-radius: 16px;
  border: 1px solid var(--glass-border);
  cursor: pointer;
  transition: all 0.3s ease;
  text-align: left;
  display: flex;
  align-items: center;
  color: var(--text-primary);
  position: relative;
}

.option-circle {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid var(--text-secondary);
  margin-right: 15px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
}

.option-item:not(.disabled):hover {
  background: var(--option-hover);
  transform: translateX(5px);
  border-color: var(--primary-color);
}

.option-item.disabled {
  cursor: default;
  opacity: 0.9;
}

.option-item.correct {
  background: rgba(72, 187, 120, 0.25);
  border-color: var(--success-color);
}

.option-item.correct .option-circle {
  border-color: var(--success-color);
  background: var(--success-color);
}

.option-item.wrong {
  background: rgba(245, 101, 101, 0.25);
  border-color: var(--error-color);
  animation: shake 0.5s;
}

.option-item.wrong .option-circle {
  border-color: var(--error-color);
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  20%, 60% { transform: translateX(-5px); }
  40%, 80% { transform: translateX(5px); }
}

.explanation-section {
  margin-top: 25px;
  padding: 20px;
  background: rgba(255, 255, 255, 0.6);
  border-radius: 16px;
  border: 1px solid var(--glass-border);
  text-align: left;
  animation: slideDown 0.4s ease-out;
}

.feedback-title {
  font-weight: bold;
  font-size: 1.1rem;
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.feedback-title.is-correct {
  color: var(--success-color);
}
.feedback-title.is-wrong {
  color: var(--error-color);
}

.explanation-text {
  font-size: 0.95rem;
  color: var(--text-secondary);
  line-height: 1.6;
  margin-bottom: 15px;
}

.explanation-images {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 10px;
  margin-top: 15px;
}

.explanation-img {
  width: 100%;
  height: 120px;
  object-fit: cover;
  border-radius: 8px;
  border: 1px solid var(--glass-border);
  cursor: zoom-in;
  transition: transform 0.2s;
  background: rgba(0, 0, 0, 0.1);
}

.progress-container {
  width: 100%;
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  margin-bottom: 25px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), var(--accent-color));
  transition: width 0.5s ease;
}

.konata-sticker {
  width: 80px;
  height: auto;
  display: block;
  margin: 0 auto 5px;
  animation: swing 2s ease-in-out infinite alternate;
  transform-origin: bottom center;
  filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.2));
}

@keyframes swing {
  0% { transform: rotate(-6deg); }
  100% { transform: rotate(6deg); }
}

.image-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.85);
  z-index: 10000;
  display: flex;
  justify-content: center;
  align-items: center;
  backdrop-filter: blur(8px);
  animation: fadeIn 0.3s ease;
  flex-direction: column;
}

.image-modal img {
  max-width: 90%;
  max-height: 80vh;
  border-radius: 8px;
  box-shadow: 0 5px 30px rgba(0, 0, 0, 0.5);
  animation: zoomIn 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.modal-close-hint {
  margin-top: 20px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
  background: rgba(255, 255, 255, 0.1);
  padding: 6px 16px;
  border-radius: 20px;
}

.back-link {
  display: inline-block;
  color: var(--text-secondary);
  text-decoration: none;
  font-size: 0.9rem;
}
.back-link:hover {
  text-decoration: underline;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
@keyframes zoomIn {
  from { transform: scale(0.9); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}
@keyframes slideDown {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* 移动端适配 */
@media (max-width: 480px) {
  .card { padding: 30px 20px; }
  h1 { font-size: 1.6rem; }
  h2 { font-size: 1.2rem; }
  .option-item { padding: 15px; font-size: 0.95rem; }
  .question-img { max-height: 200px; }
  .quiz-page-container {
    /* 移动端背景图：已经改为 webp */
    --bg-image: url('/quiz/20200805165343_kjmwd.webp');
  }
}
</style>
