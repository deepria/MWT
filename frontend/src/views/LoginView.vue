<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const isSubmitting = ref(false)
const errorMessage = ref('')
const authProvider = import.meta.env.VITE_AUTH_PROVIDER ?? 'mock'
const isHostedUi = authProvider === 'cognito'

async function submit() {
  isSubmitting.value = true
  errorMessage.value = ''

  try {
    const redirectPath =
      (route.query.redirect as string | undefined) ?? '/problems'

    if (isHostedUi) {
      await auth.login('', '', redirectPath)
      return
    }

    await auth.login('admin@mwt.local', 'password', redirectPath)
    await router.push(redirectPath)
  } catch (error) {
    errorMessage.value =
      error instanceof Error
        ? error.message
        : '로그인에 실패했습니다. 입력값을 확인하세요.'
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <section class="login-layout">
    <div class="login-copy">
      <p class="eyebrow">MWT Online Judge</p>
      <h1>문제 풀이와 채점 흐름을 먼저 굴립니다.</h1>
      <p>
        Phase 1에서는 Cognito 연결 전환을 염두에 둔 mock 세션으로 보호 라우트와
        문제 UI를 검증합니다.
      </p>
    </div>

    <form class="form-panel" @submit.prevent="submit">
      <p class="auth-note">
        {{
          isHostedUi
            ? 'AWS Cognito Hosted UI로 이동합니다.'
            : 'Mock 세션으로 로그인합니다.'
        }}
      </p>

      <button class="primary-button" type="submit" :disabled="isSubmitting">
        {{
          isSubmitting
            ? '로그인 중'
            : isHostedUi
              ? 'Cognito로 로그인'
              : 'Mock 로그인'
        }}
      </button>

      <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>
    </form>
  </section>
</template>
