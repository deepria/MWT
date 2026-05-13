<script setup lang="ts">
import { RouterLink, RouterView, useRouter } from 'vue-router'

import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

async function logout() {
  await auth.logout()
  await router.push({ name: 'login' })
}
</script>

<template>
  <div class="app-shell">
    <header class="topbar">
      <RouterLink
        class="brand"
        :to="{ name: auth.isAuthenticated ? 'problems' : 'login' }"
      >
        <span class="brand-mark">M</span>
        <span>MWT Judge</span>
      </RouterLink>

      <nav v-if="auth.isAuthenticated" class="topnav" aria-label="주요 메뉴">
        <RouterLink :to="{ name: 'problems' }">문제</RouterLink>
        <RouterLink :to="{ name: 'admin-problems' }">관리자</RouterLink>
      </nav>

      <div class="session-area">
        <span v-if="auth.user" class="user-label">{{ auth.user.email }}</span>
        <button
          v-if="auth.isAuthenticated"
          class="ghost-button"
          type="button"
          @click="logout"
        >
          로그아웃
        </button>
      </div>
    </header>

    <main class="page-shell">
      <RouterView />
    </main>
  </div>
</template>
