export type BlockType = 'text' | 'image' | 'audio';

export interface ContentBlock {
  type: BlockType;
  text?: string;
  image?: {
    src: string;
    alt?: string;
    title?: string;
  };
  audio?: {
    src: string;
    title?: string;
    durationText?: string;
  };
}

export interface Option {
  key: string;
  text: string;
}

export interface Question {
  id: string;
  no: number;
  column: 'C1' | 'C2' | 'C3' | 'C4';
  type: 'choice' | 'fill' | 'judgment' | 'multiple';
  typeLabel: string;
  score: number;
  stemBlocks: ContentBlock[];
  options?: Option[];
  answer: string; // 单选题/判断题/填空题：单个答案；多选题：逗号分隔的答案，如 "A,B,C"
  analysisBlocks?: ContentBlock[];
}

export interface LevelConfig {
  id: string;
  min: number;
  max: number;
  name: string;
  color: string;
  img: string;
  sketch: string;
  comment: string;
}

export interface ExamConfig {
  title: string;
  paperTitle: string;
  paperSubtitle: string;
  paperMeta: string;
  // [新增] 必填字段
  author?: string;
  contact?: string;
  className?: string;

  exportHeaderTitle: string;
  exportHeaderSub: string;
  exportFooterText: string;
  exportOrgName: string;
  qrCodeText: string;
  qrCodeImg: string;
}

export interface ExamDatabase {
  id: string;
  config: ExamConfig;
  questions: Question[];
  levels: LevelConfig[];
  status?: 'pending' | 'published' | 'locked';
  edit_token?: string;
  visit_count?: number;
  ai_reason?: string;
  created_at?: string;
}

export interface UserResult {
  submitted: boolean;
  score: number;
  judges: Record<string, boolean>;
}