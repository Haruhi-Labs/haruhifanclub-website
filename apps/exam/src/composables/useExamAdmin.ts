// 考试平台管理员门禁（统一 JWT 单点登录）。
// 把"导入 JSON"功能里重复的管理员登录弹窗逻辑收敛到一处：
// 会话恢复 / 弹窗登录 / 校验 exam 权限全部走共享的 createAdminAuth('exam')。
// HomeView 与 EditorView 复用本 composable；AdminView 因 UI 不同直接用返回的 admin 实例。
import { reactive } from 'vue';
import { createAdminAuth } from '@haruhi/api-client';
import type { AdminAuth } from '@haruhi/api-client';

// exam 站点共享同一个 admin 鉴权实例
const admin: AdminAuth = createAdminAuth('exam');

interface AdminModalState {
  show: boolean;
  user: string;
  pw: string;
  error: string;
  loading: boolean;
  resolve: ((v: boolean) => void) | null;
}

export function useExamAdmin() {
  // 管理员登录弹窗状态（统一 JWT：用户名+密码）
  const adminModal = reactive<AdminModalState>({
    show: false,
    user: '',
    pw: '',
    error: '',
    loading: false,
    resolve: null,
  });

  // 显示登录弹窗，返回是否登录成功
  const showAdminModal = (): Promise<boolean> => {
    return new Promise((resolve) => {
      adminModal.show = true;
      adminModal.user = '';
      adminModal.pw = '';
      adminModal.error = '';
      adminModal.loading = false;
      adminModal.resolve = resolve;
    });
  };

  // 确保已登录且具备考试平台管理权限：会话恢复成功则直接 true；否则弹出登录框。
  const ensureAdmin = async (): Promise<boolean> => {
    const user = await admin.restore();
    if (user) return true;
    return showAdminModal();
  };

  // 登录校验（弹窗内点击"登录"）
  const verifyAdmin = async () => {
    if (!adminModal.user.trim() || !adminModal.pw) {
      adminModal.error = '请输入用户名和密码';
      return;
    }
    adminModal.loading = true;
    adminModal.error = '';
    try {
      const r = await admin.login(adminModal.user.trim(), adminModal.pw);
      if (!r.ok) {
        adminModal.error = r.error || '登录失败';
        return;
      }
      adminModal.show = false;
      adminModal.resolve?.(true);
    } finally {
      adminModal.loading = false;
    }
  };

  // 取消登录
  const cancelAdmin = () => {
    adminModal.show = false;
    adminModal.resolve?.(false);
  };

  return {
    admin,
    adminModal,
    showAdminModal,
    ensureAdmin,
    verifyAdmin,
    cancelAdmin,
  };
}
