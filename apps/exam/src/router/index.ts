import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import ExamPaper from '@/views/ExamPaper.vue';
import EditorView from '@/views/EditorView.vue';
import AdminView from '@/views/AdminView.vue';
// 统一账号 UI
import { LoginView, ProfileView, SettingsView, VerifyEmailView, ResetPasswordView } from '@haruhi/auth-ui';

const router = createRouter({
  // [修改] 传入 base 参数 '/exam/'
  history: createWebHistory('/exam/'),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/create',
      name: 'create',
      component: EditorView
    },
    {
      path: '/haruhi',
      name: 'haruhi',
      component: ExamPaper,
      props: { isHaruhi: true }
    },
    {
      path: '/exam/:id',
      name: 'exam',
      component: ExamPaper,
      props: true
    },
    {
      path: '/admin',
      name: 'admin',
      component: AdminView
    },
    { path: '/login', name: 'login', component: LoginView, props: { site: 'exam' } },
    { path: '/account', name: 'account', component: ProfileView, props: { site: 'exam' } },
    {
      path: '/account/settings',
      name: 'account-settings',
      component: SettingsView,
      props: { site: 'exam' }
    },
    {
      path: '/verify-email',
      name: 'verify-email',
      component: VerifyEmailView,
      props: { site: 'exam' }
    },
    {
      path: '/reset-password',
      name: 'reset-password',
      component: ResetPasswordView,
      props: { site: 'exam' }
    }
  ]
});

export default router;