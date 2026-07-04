import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import ExamPaper from '@/views/ExamPaper.vue';
import EditorView from '@/views/EditorView.vue';
import AdminView from '@/views/AdminView.vue';

// 试卷站独立于统一用户系统：无登录 / 账号路由；创建凭 edit_token 编辑。
const router = createRouter({
  // [修改] 传入 base 参数 '/exam/'
  history: createWebHistory('/exam/'),
  routes: [
    // 首页与 /exam/:id 详情页不加静态 title：首页用 index.html 兜底，详情页由视图内 usePageMeta 在数据加载后设置
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/create',
      name: 'create',
      component: EditorView,
      meta: { title: '出卷', noindex: true }
    },
    {
      path: '/haruhi',
      name: 'haruhi',
      component: ExamPaper,
      props: { isHaruhi: true },
      // 内置卷（mock-exam 的 config.title 为「凉宫春日大考试！」），路由级静态标题
      meta: { title: '凉宫春日大考试' }
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
      component: AdminView,
      meta: { title: '管理', noindex: true }
    }
  ]
});

export default router;
