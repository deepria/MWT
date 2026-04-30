<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { getHostedUiDebugConfig } from '@/services/hostedUiAuth'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()
const errorMessage = ref('')
const detailMessage = ref('')
const debugConfig = getHostedUiDebugConfig()

onMounted(async () => {
  const cognitoError = route.query.error
  const cognitoErrorDescription = route.query.error_description
  const code = route.query.code
  const state = route.query.state

  if (typeof cognitoError === 'string') {
    errorMessage.value = `Cognito 오류: ${cognitoError}`
    detailMessage.value =
      typeof cognitoErrorDescription === 'string'
        ? cognitoErrorDescription
        : 'Hosted UI 또는 App Client 설정을 확인하세요.'
    return
  }

  if (typeof code !== 'string' || typeof state !== 'string') {
    errorMessage.value = 'Cognito callback 값이 올바르지 않습니다.'
    detailMessage.value =
      '정상 callback은 code와 state query parameter를 포함해야 합니다.'
    return
  }

  try {
    const redirectPath = await auth.completeLogin(code, state)
    await router.replace(redirectPath || '/problems')
  } catch (error) {
    errorMessage.value =
      error instanceof Error
        ? error.message
        : 'Cognito 로그인 완료 처리에 실패했습니다.'
  }
})
</script>

<template>
  <section class="empty-state">
    <div>
      <p class="eyebrow">Cognito</p>
      <h1>{{ errorMessage || '로그인 처리 중입니다.' }}</h1>
      <p v-if="detailMessage" class="callback-detail">{{ detailMessage }}</p>
      <dl v-if="errorMessage" class="debug-list">
        <div>
          <dt>Callback URL</dt>
          <dd>{{ debugConfig.redirectSignIn }}</dd>
        </div>
        <div>
          <dt>Sign-out URL</dt>
          <dd>{{ debugConfig.redirectSignOut }}</dd>
        </div>
        <div>
          <dt>Client ID</dt>
          <dd>{{ debugConfig.clientId }}</dd>
        </div>
        <div>
          <dt>Scopes</dt>
          <dd>{{ debugConfig.scopes }}</dd>
        </div>
      </dl>
      <RouterLink v-if="errorMessage" :to="{ name: 'login' }">
        로그인으로 돌아가기
      </RouterLink>
    </div>
  </section>
</template>
