// 与 GPT-SoVITS WebUI 完全一致的选项集（中文标签 → TTS_infer_pack 代号）。

/** 合成/参考语种（dict_language_v2 全量 11 项） */
export const LANGS = [
  { label: '日文', value: 'all_ja' },
  { label: '中文', value: 'all_zh' },
  { label: '英文', value: 'en' },
  { label: '粤语', value: 'all_yue' },
  { label: '韩文', value: 'all_ko' },
  { label: '中英混合', value: 'zh' },
  { label: '日英混合', value: 'ja' },
  { label: '粤英混合', value: 'yue' },
  { label: '韩英混合', value: 'ko' },
  { label: '多语种混合', value: 'auto' },
  { label: '多语种混合(粤语)', value: 'auto_yue' },
]

/** 切句方式（cut_method_map 全量 6 项） */
export const CUTS = [
  { label: '不切', value: 'cut0' },
  { label: '凑四句一切', value: 'cut1' },
  { label: '凑50字一切', value: 'cut2' },
  { label: '按中文句号。切', value: 'cut3' },
  { label: '按英文句号.切', value: 'cut4' },
  { label: '按标点符号切', value: 'cut5' },
]

/** RVC 导出格式（与 WebUI 批量推理 format1 一致） */
export const RVC_FORMATS = ['wav', 'flac', 'mp3', 'm4a']
