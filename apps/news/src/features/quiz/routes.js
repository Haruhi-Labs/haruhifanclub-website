// quiz 特性路由：答题游戏（隐藏 NavBar）
const QuizView = () => import('@/features/quiz/views/QuizView.vue');

export const quizRoutes = [
    {
        path: '/quiz-game',
        name: 'quiz',
        component: QuizView,
        meta: { hideNavbar: true }
    }
];

export default quizRoutes;
