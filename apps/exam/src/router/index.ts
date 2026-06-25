import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import ExamPaper from '@/views/ExamPaper.vue';
import EditorView from '@/views/EditorView.vue';
import AdminView from '@/views/AdminView.vue';
// 统一账号 UI
import {
  LoginView,
  ProfileView,
  SettingsView,
  VerifyEmailView,
  ResetPasswordView,
  UserConsoleLayout,
  OverviewView as AccountOverviewView,
  MyArtworksView,
  MyArticlesView,
  MyExamsView,
  MyCommentsView,
  PointsView as AccountPointsView,
} from '@haruhi/auth-ui';

// 个人控制台导航分区：全集（全站可用——从任一 app 进入都能管理全站内容）。
const ACCOUNT_SECTIONS = [
  'overview',
  'artworks',
  'articles',
  'exams',
  'comments',
  'points',
  'profile',
  'settings',
];

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
    {
      path: '/account',
      component: UserConsoleLayout,
      props: { site: 'exam', basePath: '/account', sections: ACCOUNT_SECTIONS },
      children: [
        { path: '', name: 'account', component: AccountOverviewView },
        { path: 'artworks', name: 'account-artworks', component: MyArtworksView },
        { path: 'articles', name: 'account-articles', component: MyArticlesView },
        { path: 'exams', name: 'account-exams', component: MyExamsView },
        { path: 'comments', name: 'account-comments', component: MyCommentsView },
        { path: 'points', name: 'account-points', component: AccountPointsView },
        {
          path: 'profile',
          name: 'account-profile',
          component: ProfileView,
          props: { site: 'exam', embedded: true }
        },
        {
          path: 'settings',
          name: 'account-settings',
          component: SettingsView,
          props: { site: 'exam', embedded: true }
        }
      ]
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