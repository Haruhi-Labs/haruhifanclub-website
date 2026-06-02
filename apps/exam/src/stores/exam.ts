import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { UserResult, LevelConfig, Question, ExamDatabase } from '@/types/exam';
import { EXAM_DB } from '@/data/mock-exam';
import { getOrCreateExamNo, getTodayISO, formatDateCNFromISO, normalizeText } from '@/utils';
import { api } from '@/services/api';

export const useExamStore = defineStore('exam', () => {
  // --- State ---
  
  // 系统状态
  const currentExamId = ref('');
  const loading = ref(false);
  const error = ref('');

  // 🚫 访问控制状态
  const accessStatus = ref<'allow' | 'pending' | 'locked'>('allow'); 
  const accessReason = ref('');

  // 试卷数据
  const paper = ref<ExamDatabase>(JSON.parse(JSON.stringify(EXAM_DB)));
  
  // 🗑️ 上传文件垃圾回收追踪
  // 记录当前编辑器会话中上传过的所有文件路径
  const sessionUploads = ref<Set<string>>(new Set());

  // 考生信息
  const userName = ref('');
  const examNo = ref('');
  const className = ref('北高一年五班');
  const examDateISO = ref(getTodayISO());
  const examDateText = ref(formatDateCNFromISO(examDateISO.value));
  
  // 答题状态
  const answers = ref<Record<string, string>>({});
  const result = ref<UserResult>({
    submitted: false,
    score: 0,
    judges: {}
  });
  
  const tickNonce = ref<Record<string, number>>({});

  // --- Getters ---

  const totalScore = computed(() => {
    return paper.value.questions.reduce((sum: number, q: Question) => sum + (q.score || 0), 0);
  });

  const currentLevel = computed<LevelConfig | undefined>(() => {
    if (!result.value.submitted) return undefined;
    const score = result.value.score;
    
    if (!paper.value.levels || paper.value.levels.length === 0) {
      return {
        id: 'default',
        min: 0,
        max: 1000,
        name: '及格',
        color: '#333333',
        img: '',
        sketch: '',
        comment: '恭喜完成考试！'
      };
    }
    
    return paper.value.levels.find((l: LevelConfig) => score >= l.min && score <= l.max) || paper.value.levels[0];
  });

  // --- Actions ---

  // 记录上传文件路径
  const recordUpload = (url: string) => {
    // 统一约定：后端上传返回路径形如 /uploads/exam/<file>，直接以 /uploads/ 前缀即可捕获，
    // 用于编辑器会话内未使用文件的垃圾回收。
    if (url.startsWith('/uploads/')) {
      sessionUploads.value.add(url);
    }
  };

  // 清理当前会话上传但最终未使用的文件
  const cleanupUnusedUploads = async (finalData: ExamDatabase) => {
    if (sessionUploads.value.size === 0) return;

    // 1. 收集所有还在使用的 URL
    const usedUrls = new Set<string>();
    
    // 检查 Levels
    finalData.levels.forEach(lv => {
      if (lv.img) usedUrls.add(lv.img);
      if (lv.sketch) usedUrls.add(lv.sketch);
    });

    // 检查 Questions
    finalData.questions.forEach(q => {
      if (q.stemBlocks) {
        q.stemBlocks.forEach(b => {
          if (b.image?.src) usedUrls.add(b.image.src);
          if (b.audio?.src) usedUrls.add(b.audio.src);
        });
      }
    });

    // 2. 计算差集 (已上传 - 最终使用)
    const unusedFiles: string[] = [];
    sessionUploads.value.forEach(url => {
      if (!usedUrls.has(url)) {
        unusedFiles.push(url);
      }
    });

    // 3. 调用后端清理
    if (unusedFiles.length > 0) {
      console.log('Cleaning up unused files:', unusedFiles);
      await api.cleanupFiles(unusedFiles);
      
      // 从 session set 中移除已清理的
      unusedFiles.forEach(f => sessionUploads.value.delete(f));
    }
  };

  const init = () => {
    const raw = localStorage.getItem('haruhi_user_global');
    if (raw) {
      try {
        const data = JSON.parse(raw);
        if (data.userName) userName.value = data.userName;
      } catch(e) {}
    }
  };

  const loadExam = async (id?: string, isHaruhi = false) => {
    loading.value = true;
    error.value = '';
    accessStatus.value = 'allow';
    accessReason.value = '';
    
    if (isHaruhi || !id || id === 'haruhi') {
      currentExamId.value = 'haruhi';
      paper.value = JSON.parse(JSON.stringify(EXAM_DB));
      className.value = '北高一年五班';
      loading.value = false;
    } else {
      currentExamId.value = id;
      try {
        const data = await api.getExam(id);
        if (data) {
          paper.value = data;
          className.value = '北高一年五班';
        } else {
          error.value = '试卷不存在';
        }
      } catch (e: any) {
        console.error("Load Exam Error:", e);
        
        if (e.response && e.response.status === 403 && e.response.data?.error === 'EXAM_UNAVAILABLE') {
            accessStatus.value = e.response.data.status;
            accessReason.value = e.response.data.reason || '该试卷暂无法访问';
            error.value = ''; 
        } else {
            error.value = '试卷加载失败或网络错误';
        }
      } finally {
        loading.value = false;
      }
    }
    
    if (accessStatus.value !== 'allow' || error.value) return;

    const lsKey = `haruhi_exam_${currentExamId.value}`;
    const raw = localStorage.getItem(lsKey);
    
    answers.value = {};
    result.value = { submitted: false, score: 0, judges: {} };
    tickNonce.value = {};
    
    if (raw) {
      try {
        const data = JSON.parse(raw);
        if (data.userName) userName.value = data.userName;
        if (data.answers) answers.value = data.answers;
        if (data.result) result.value = data.result;
        if (data.examNo) examNo.value = data.examNo;
      } catch(e) {}
    }

    if (!examNo.value) {
       examNo.value = getOrCreateExamNo(examDateISO.value, lsKey);
    }
    
    const today = getTodayISO();
    if (examDateISO.value !== today && !result.value.submitted) {
      examDateISO.value = today;
      examDateText.value = formatDateCNFromISO(today);
    }
  };

  const verifyAndLoadExam = async (id: string, token: string) => {
    loading.value = true;
    accessStatus.value = 'allow';
    sessionUploads.value.clear(); // 编辑模式重置 Session Uploads
    try {
      const data = await api.verifyExam(id, token);
      paper.value = data;
      currentExamId.value = id;
      return true;
    } catch (e) {
      error.value = '验证失败：链接无效或试卷不存在';
      return false;
    } finally {
      loading.value = false;
    }
  };

  const saveNewExam = async (newPaper: ExamDatabase) => {
    // 保存前执行清理
    await cleanupUnusedUploads(newPaper);
    
    const cleanPaper = JSON.parse(JSON.stringify(newPaper));
    const res = await api.createExam(cleanPaper);
    const tokenKey = `exam_token_${res.id}`;
    localStorage.setItem(tokenKey, res.editToken);
    return res;
  };

  const updateExam = async (id: string, token: string, newData: ExamDatabase) => {
    // 保存前执行清理
    await cleanupUnusedUploads(newData);

    const cleanPaper = JSON.parse(JSON.stringify(newData));
    await api.updateExam(id, token, cleanPaper);
  };

  const listExams = async (page: number = 1, limit: number = 9, search: string = '') => {
    return await api.getExams(page, limit, search);
  };

  const setAnswer = (qId: string, val: string) => {
    if (result.value.submitted) return;
    answers.value[qId] = val;
    tickNonce.value[qId] = (tickNonce.value[qId] || 0) + 1;
  };

  const submitPaper = () => {
    if (result.value.submitted) return;
    let score = 0;
    const judges: Record<string, boolean> = {};
    paper.value.questions.forEach((q: Question) => {
      const userAns = normalizeText(answers.value[q.id] || '');
      const correctAns = normalizeText(q.answer);
      
      let isCorrect = false;
      
      if (q.type === 'multiple') {
        // 多选题：比较答案集合（顺序无关）
        const userAnswers = userAns ? userAns.split(',').map(a => a.trim()).filter(a => a).sort() : [];
        const correctAnswers = correctAns ? correctAns.split(',').map(a => a.trim()).filter(a => a).sort() : [];
        isCorrect = userAnswers.length === correctAnswers.length && 
                   userAnswers.every((ans, idx) => ans === correctAnswers[idx]);
      } else {
        // 单选题、判断题、填空题：直接比较
        isCorrect = userAns === correctAns;
      }
      
      judges[q.id] = isCorrect;
      if (isCorrect) score += q.score;
    });
    result.value = { submitted: true, score, judges };
  };

  const resetExam = () => {
    answers.value = {};
    result.value = { submitted: false, score: 0, judges: {} };
    tickNonce.value = {};
    const today = getTodayISO();
    examDateISO.value = today;
    examDateText.value = formatDateCNFromISO(today);
  };

  watch([answers, result, userName, examNo], () => {
    if (!currentExamId.value || accessStatus.value !== 'allow' || error.value) return;
    const lsKey = `haruhi_exam_${currentExamId.value}`;
    localStorage.setItem(lsKey, JSON.stringify({
      userName: userName.value,
      answers: answers.value,
      result: result.value,
      examNo: examNo.value
    }));
  }, { deep: true });

  watch(userName, (val) => {
    if (val) localStorage.setItem('haruhi_user_global', JSON.stringify({ userName: val }));
  });

  return {
    paper,
    loading,
    error,
    accessStatus,
    accessReason,
    userName,
    examNo,
    className,
    examDateText,
    answers,
    result,
    tickNonce,
    totalScore,
    currentLevel,
    sessionUploads, // export state
    init,
    loadExam,
    verifyAndLoadExam,
    saveNewExam,
    updateExam,
    listExams,
    setAnswer,
    submitPaper,
    resetExam,
    recordUpload // export action
  };
});