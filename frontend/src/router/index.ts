import { createRouter, createWebHistory } from 'vue-router'

import { useAuthStore } from '@/stores/auth'
import AdminProblemNewView from '@/views/AdminProblemNewView.vue'
import AuthCallbackView from '@/views/AuthCallbackView.vue'
import LoginView from '@/views/LoginView.vue'
import ProblemDetailView from '@/views/ProblemDetailView.vue'
import ProblemListView from '@/views/ProblemListView.vue'
import SubmissionDetailView from '@/views/SubmissionDetailView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/problems',
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView,
      meta: { public: true },
    },
    {
      path: '/auth/callback',
      name: 'auth-callback',
      component: AuthCallbackView,
      meta: { public: true },
    },
    {
      path: '/problems',
      name: 'problems',
      component: ProblemListView,
    },
    {
      path: '/problems/:problemId',
      name: 'problem-detail',
      component: ProblemDetailView,
      props: true,
    },
    {
      path: '/submissions/:submissionId',
      name: 'submission-detail',
      component: SubmissionDetailView,
      props: true,
    },
    {
      path: '/admin/problems/new',
      name: 'admin-problem-new',
      component: AdminProblemNewView,
    },
  ],
})

router.beforeEach((to) => {
  const auth = useAuthStore()

  if (!to.meta.public && !auth.isAuthenticated) {
    return {
      name: 'login',
      query: { redirect: to.fullPath },
    }
  }

  if (to.name === 'login' && auth.isAuthenticated) {
    return { name: 'problems' }
  }
})

export default router
