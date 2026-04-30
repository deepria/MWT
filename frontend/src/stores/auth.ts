import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

import {
  clearHostedUiTokens,
  completeHostedUiLogin,
  redirectToHostedUi,
  redirectToHostedUiLogout,
} from '@/services/hostedUiAuth'

export interface AuthUser {
  id: string
  email: string
  groups: string[]
}

const STORAGE_KEY = 'mwt.auth.session'
const authProvider = import.meta.env.VITE_AUTH_PROVIDER ?? 'mock'

function readStoredUser(): AuthUser | null {
  const raw = window.localStorage.getItem(STORAGE_KEY)

  if (!raw) return null

  try {
    return JSON.parse(raw) as AuthUser
  } catch {
    window.localStorage.removeItem(STORAGE_KEY)
    return null
  }
}

export const useAuthStore = defineStore('auth', () => {
  const user = ref<AuthUser | null>(readStoredUser())
  const isAuthenticated = computed(() => user.value !== null)

  async function login(
    email: string,
    _password: string,
    redirectPath = '/problems',
  ) {
    if (authProvider === 'cognito') {
      await redirectToHostedUi(redirectPath)
      return
    }

    const nextUser = {
      id: 'mock-user-001',
      email,
      groups: email.includes('admin') ? ['admin'] : ['participant'],
    }

    user.value = nextUser
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(nextUser))
  }

  async function completeLogin(code: string, state: string) {
    const result = await completeHostedUiLogin(code, state)

    user.value = result.user
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(result.user))

    return result.redirectPath
  }

  async function logout() {
    if (authProvider === 'cognito') {
      clearHostedUiTokens()
      user.value = null
      window.localStorage.removeItem(STORAGE_KEY)
      redirectToHostedUiLogout()
      return
    }

    user.value = null
    window.localStorage.removeItem(STORAGE_KEY)
  }

  return {
    user,
    isAuthenticated,
    login,
    completeLogin,
    logout,
  }
})
