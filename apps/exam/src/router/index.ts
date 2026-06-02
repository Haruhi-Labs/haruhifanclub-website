import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import ExamPaper from '@/views/ExamPaper.vue';
import EditorView from '@/views/EditorView.vue';
import AdminView from '@/views/AdminView.vue';

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
    }
  ]
});

export default router;