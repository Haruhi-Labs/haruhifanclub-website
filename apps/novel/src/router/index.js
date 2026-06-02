// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'
import Shelf from '../views/Shelf.vue'
import Reader from '../views/Reader.vue'
import Admin from '../views/Admin.vue'
import FeedbackView from '../views/FeedbackView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'Shelf', component: Shelf },
    { path: '/read/:id', name: 'Reader', component: Reader, props: true },
    { path: '/admin', name: 'Admin', component: Admin },
    { path: '/feedback', name: 'Feedback', component: FeedbackView } // 顺便大小写统一一下
  ]
})

export default router
