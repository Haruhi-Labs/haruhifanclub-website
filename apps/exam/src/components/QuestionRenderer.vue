<script setup lang="ts">
import { computed } from 'vue';
import type { Question } from '@/types/exam';
import { useExamStore } from '@/stores/exam';
import { useAudioStore } from '@/stores/audio';

const props = defineProps<{
  questions: Question[];
  readonly?: boolean;
}>();

const examStore = useExamStore();
const audioStore = useAudioStore();

const isMarked = computed(() => examStore.result.submitted);

// Helper: Format seconds to mm:ss
const formatTime = (seconds: number) => {
  if (!seconds || isNaN(seconds)) return '0:00';
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
};

// Calculate progress bar style
const getBarStyle = (uid: string) => {
  if (audioStore.currentId === uid) {
    return { '--p': `${audioStore.progress}%` };
  }
  return { '--p': '0%' };
};

// Get display text for time (Current / Total)
const getTimeText = (uid: string, defaultDuration?: string) => {
  if (audioStore.currentId === uid) {
    const cur = formatTime(audioStore.currentTime);
    const dur = audioStore.duration ? formatTime(audioStore.duration) : (defaultDuration || '0:00');
    return `${cur} / ${dur}`;
  }
  return `0:00 / ${defaultDuration || '?'}`;
};

const handleChoose = (qId: string, key: string) => {
  if (!props.readonly) examStore.setAnswer(qId, key);
};

// 多选题处理
const handleMultipleChoose = (qId: string, key: string) => {
  if (!props.readonly) {
    const currentAnswer = examStore.answers[qId] || '';
    const answers = currentAnswer ? currentAnswer.split(',').map(a => a.trim()).filter(a => a) : [];
    
    if (answers.includes(key)) {
      // 取消选择
      const idx = answers.indexOf(key);
      answers.splice(idx, 1);
    } else {
      // 添加选择
      answers.push(key);
    }
    
    examStore.setAnswer(qId, answers.sort().join(','));
  }
};

// 检查多选题选项是否被选中
const isMultipleSelected = (qId: string, key: string): boolean => {
  const answer = examStore.answers[qId] || '';
  if (!answer) return false;
  return answer.split(',').map(a => a.trim()).includes(key);
};

const handleFill = (qId: string, e: Event) => {
  if (!props.readonly) {
    const val = (e.target as HTMLInputElement).value;
    examStore.setAnswer(qId, val);
  }
};
</script>

<template>
  <div>
    <div v-for="(q, idx) in questions" :key="q.id" class="q">
      <!-- Question Number & Score -->
      <div class="meta">
        <div class="num">
          <span>{{ q.no }}.</span>
          <span class="type">【{{ q.typeLabel }}】</span>
        </div>
        <div class="pts">{{ q.score }}分</div>
      </div>

      <!-- Question Content Blocks -->
      <div class="blocks">
        <template v-for="(b, bi) in q.stemBlocks" :key="q.id + '-s-' + bi">
          <div v-if="b.type === 'text'" class="blk-text">{{ b.text }}</div>
          
          <div v-else-if="b.type === 'image'" class="blk-img" @click="$emit('openimg', b.image)">
            <img :src="b.image?.src" :alt="b.image?.alt || 'image'" />
          </div>
          
          <div v-else-if="b.type === 'audio'" class="blk-audio" @click="audioStore.toggle(b.audio?.src || '', q.id + '_s_' + bi)">
            <div class="sticker">
              <svg viewBox="0 0 24 24" width="22" height="22" fill="none">
                <path v-if="audioStore.currentId === q.id + '_s_' + bi && audioStore.playing" d="M7 6h3v12H7zM14 6h3v12h-3z" fill="rgba(30,30,30,.75)"/>
                <path v-else d="M9 7.5v9l8-4.5-8-4.5z" fill="rgba(30,30,30,.72)"/>
              </svg>
            </div>
            <div class="audio-info">
              <div class="audio-title">{{ b.audio?.title }}</div>
              <!-- Real progress bar -->
              <div class="audio-bar" :style="getBarStyle(q.id + '_s_' + bi)"><i></i></div>
              <!-- Dynamic time display -->
              <div class="audio-time">{{ getTimeText(q.id + '_s_' + bi, b.audio?.durationText) }}</div>
            </div>
          </div>
        </template>
      </div>

      <!-- Single Choice -->
      <div v-if="q.type === 'choice'" class="opts">
        <div v-for="(op, oi) in q.options" :key="q.id + '-o-' + oi"
             class="opt" :class="{ disabled: readonly }"
             @click="handleChoose(q.id, op.key)">
          <span class="k">{{ op.key }}.</span>
          <span class="v">{{ op.text }}</span>
          
          <!-- Checked Animation -->
          <div class="mark-wrap" :class="{ on: examStore.answers[q.id] === op.key }">
             <svg :key="q.id + '_' + op.key + '_' + (examStore.tickNonce[q.id] || 0)" viewBox="0 0 60 44" width="56" height="44">
               <path d="M10 24 C14 27, 16 31, 20 34 C25 28, 33 18, 48 10"
                     fill="none" stroke="rgba(18,35,55,.88)" stroke-width="5.2"
                     stroke-linecap="round" stroke-linejoin="round"
                     style="filter:url(#inkTurb);" stroke-dasharray="200 0" stroke-dashoffset="200">
                 <animate attributeName="stroke-dashoffset" dur="0.38s" from="200" to="0" fill="freeze" />
               </path>
             </svg>
          </div>
        </div>
      </div>

      <!-- Multiple Choice -->
      <div v-else-if="q.type === 'multiple'" class="opts">
        <div v-for="(op, oi) in q.options" :key="q.id + '-o-' + oi"
             class="opt" :class="{ disabled: readonly, 'multiple-selected': isMultipleSelected(q.id, op.key) }"
             @click="handleMultipleChoose(q.id, op.key)">
          <span class="k">{{ op.key }}.</span>
          <span class="v">{{ op.text }}</span>
          
          <!-- Checked Animation (Checkbox style) -->
          <div class="mark-wrap" :class="{ on: isMultipleSelected(q.id, op.key) }">
             <svg :key="q.id + '_' + op.key + '_' + (examStore.tickNonce[q.id] || 0)" viewBox="0 0 60 44" width="56" height="44">
               <path d="M10 24 C14 27, 16 31, 20 34 C25 28, 33 18, 48 10"
                     fill="none" stroke="rgba(18,35,55,.88)" stroke-width="5.2"
                     stroke-linecap="round" stroke-linejoin="round"
                     style="filter:url(#inkTurb);" stroke-dasharray="200 0" stroke-dashoffset="200">
                 <animate attributeName="stroke-dashoffset" dur="0.38s" from="200" to="0" fill="freeze" />
               </path>
             </svg>
          </div>
        </div>
      </div>

      <!-- Judgment (True/False) -->
      <div v-else-if="q.type === 'judgment'" class="judgment-opts">
        <div class="judgment-option" 
             :class="{ active: examStore.answers[q.id] === 'true', disabled: readonly }"
             @click="handleChoose(q.id, 'true')">
          <span class="judgment-icon">✓</span>
          <div class="mark-wrap" :class="{ on: examStore.answers[q.id] === 'true' }">
            <svg :key="q.id + '_true_' + (examStore.tickNonce[q.id] || 0)" viewBox="0 0 60 44" width="56" height="44">
              <path d="M10 24 C14 27, 16 31, 20 34 C25 28, 33 18, 48 10"
                    fill="none" stroke="rgba(18,35,55,.88)" stroke-width="5.2"
                    stroke-linecap="round" stroke-linejoin="round"
                    style="filter:url(#inkTurb);" stroke-dasharray="200 0" stroke-dashoffset="200">
                <animate attributeName="stroke-dashoffset" dur="0.38s" from="200" to="0" fill="freeze" />
              </path>
            </svg>
          </div>
        </div>
        <div class="judgment-option" 
             :class="{ active: examStore.answers[q.id] === 'false', disabled: readonly }"
             @click="handleChoose(q.id, 'false')">
          <span class="judgment-icon">✗</span> 
          <div class="mark-wrap" :class="{ on: examStore.answers[q.id] === 'false' }">
            <svg :key="q.id + '_false_' + (examStore.tickNonce[q.id] || 0)" viewBox="0 0 60 44" width="56" height="44">
              <path d="M10 24 C14 27, 16 31, 20 34 C25 28, 33 18, 48 10"
                    fill="none" stroke="rgba(18,35,55,.88)" stroke-width="5.2"
                    stroke-linecap="round" stroke-linejoin="round"
                    style="filter:url(#inkTurb);" stroke-dasharray="200 0" stroke-dashoffset="200">
                <animate attributeName="stroke-dashoffset" dur="0.38s" from="200" to="0" fill="freeze" />
              </path>
            </svg>
          </div>
        </div>
      </div>

      <!-- Fill in blanks -->
      <div v-else-if="q.type === 'fill'" class="blank">
        <div class="hand">
          <span v-for="(ch, ci) in (examStore.answers[q.id] || '').split('')" 
                :key="q.id + '-ch-' + ci" class="ch" 
                :style="{ '--d': (ci * 38) + 'ms', '--r': (((ci * 17) % 5) - 2) + 'deg' }">
            {{ ch }}
          </span>
        </div>
        <div class="uline"></div>
        <input v-if="!readonly" type="text" :value="examStore.answers[q.id] || ''" 
               @input="handleFill(q.id, $event)" />
      </div>

      <!-- Marking & Analysis -->
      <div v-if="isMarked">
        <!-- Teacher's marks -->
        <svg v-if="examStore.result.judges[q.id]" class="teacher" viewBox="0 0 64 64">
           <path d="M12 36 C18 40, 22 46, 28 50 C34 42, 44 28, 56 16" fill="none" stroke="rgba(200,23,30,.92)" stroke-width="6.2" stroke-linecap="round" stroke-linejoin="round" style="filter:url(#inkTurb);" stroke-dasharray="240 0" stroke-dashoffset="240">
             <animate attributeName="stroke-dashoffset" dur="0.48s" from="240" to="0" fill="freeze" />
           </path>
        </svg>
        <svg v-else class="teacher" viewBox="0 0 64 64">
           <path d="M16 16 L48 48" fill="none" stroke="rgba(200,23,30,.92)" stroke-width="6.2" stroke-linecap="round" style="filter:url(#inkTurb);" stroke-dasharray="120 0" stroke-dashoffset="120">
             <animate attributeName="stroke-dashoffset" dur="0.32s" from="120" to="0" fill="freeze" />
           </path>
           <path d="M48 16 L16 48" fill="none" stroke="rgba(200,23,30,.92)" stroke-width="6.2" stroke-linecap="round" style="filter:url(#inkTurb);" stroke-dasharray="120 0" stroke-dashoffset="120">
             <animate attributeName="stroke-dashoffset" dur="0.38s" from="120" to="0" fill="freeze" />
           </path>
        </svg>

        <div class="analysis">
          <div class="tag">解析：</div>
          <div v-for="(b, bi) in q.analysisBlocks" :key="q.id + '-a-' + bi">
             <div v-if="b.type === 'text'" class="blk-text">{{ b.text }}</div>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>