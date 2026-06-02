import type { ExamDatabase } from '@/types/exam';
import { getToken } from '@haruhi/api-client';

// 统一后端约定：模块 API 统一前缀 /api/exam（旧 /exam/api/* → /api/exam/*）。
// 端点相对路径、请求体、响应字段与旧版保持一致，仅前缀变更。
const API_BASE = '/api/exam';

// 统一 JWT：若已登录则自动带上 Authorization: Bearer <jwt>（替代旧的 X-Admin-Key 头）。
function authHeaders(): Record<string, string> {
  const token = getToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export const api = {
  // --- 文件操作 ---
  
  /** 上传文件 (图片会自动压缩后上传，音频上传后后端压缩) */
  async uploadFile(file: File): Promise<string> {
    const formData = new FormData();
    formData.append('file', file);
    
    // 请求地址为 /api/exam/upload
    const res = await fetch(`${API_BASE}/upload`, {
      method: 'POST',
      body: formData
    });
    
    if (!res.ok) throw new Error('Upload failed');
    const data = await res.json();
    return data.url; 
  },

  /** 批量删除文件 */
  async cleanupFiles(filePaths: string[]): Promise<void> {
    if (filePaths.length === 0) return;
    
    await fetch(`${API_BASE}/cleanup`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ files: filePaths })
    });
  },

  // --- 原有接口 ---

  // 获取所有试卷 (已发布)，支持分页
async getExams(page: number = 1, limit: number = 9, search: string = ''): Promise<{ data: ExamDatabase[], pagination: { page: number, limit: number, total: number, totalPages: number } }> {
    try {
      // [修改] 使用 URLSearchParams 构建查询参数，自动处理特殊字符编码
      const params = new URLSearchParams({
        page: page.toString(),
        limit: limit.toString()
      });
      
      // 只有当 search 有值时才追加参数
      if (search) {
        params.append('search', search);
      }

      // [修改] fetch URL 改为使用 params
      const res = await fetch(`${API_BASE}/exams?${params.toString()}`);
      
      if (!res.ok) throw new Error('Fetch failed');
      const result = await res.json();
      
      // 兼容旧版本（如果没有 pagination 字段，说明是旧版本）
      if (!result.pagination) {
        return {
          data: result.data || result,
          pagination: {
            page: 1,
            limit: result.data?.length || result.length || 9,
            total: result.data?.length || result.length || 0,
            totalPages: 1
          }
        };
      }
      return result;
    } catch (e) {
      console.warn('API Error', e);
      return {
        data: [],
        pagination: {
          page: 1,
          limit: 9,
          total: 0,
          totalPages: 0
        }
      };
    }
  },

  // 获取单个试卷
  async getExam(id: string): Promise<ExamDatabase | null> {
    const res = await fetch(`${API_BASE}/exams/${id}`);
    
    if (!res.ok) {
      if (res.status === 404) return null;
      
      const errInfo = await res.json().catch(() => ({}));
      
      const error = new Error('API_ERROR');
      (error as any).response = {
        status: res.status,
        data: errInfo
      };
      throw error;
    }
    
    return await res.json();
  },

  // 验证权限并获取完整试卷 (用于编辑)
  async verifyExam(id: string, token: string): Promise<ExamDatabase> {
    const res = await fetch(`${API_BASE}/exams/${id}/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token })
    });
    if (!res.ok) throw new Error('Verify failed');
    return await res.json();
  },

  // 创建试卷
  async createExam(data: ExamDatabase): Promise<{ id: string, editToken: string }> {
    const res = await fetch(`${API_BASE}/exams`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    if (!res.ok) throw new Error('Create failed');
    return await res.json();
  },

  // 更新试卷
  async updateExam(id: string, token: string, data: ExamDatabase): Promise<void> {
    const res = await fetch(`${API_BASE}/exams/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token, ...data })
    });
    if (!res.ok) throw new Error('Update failed');
  },

  // 导入试卷（从 JSON）——需要管理员权限，统一 JWT 鉴权（自动带 Authorization 头）。
  async importExam(data: ExamDatabase): Promise<{ id: string, editToken: string, importedQuestions: number, importedLevels: number }> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...authHeaders()
    };

    const res = await fetch(`${API_BASE}/exams/import`, {
      method: 'POST',
      headers,
      body: JSON.stringify(data)
    });
    
    if (!res.ok) {
      const errorData = await res.json().catch(() => ({ error: '导入失败' }));
      const error = new Error(errorData.error || '导入失败');
      (error as any).details = errorData.details;
      throw error;
    }
    
    return await res.json();
  }
};