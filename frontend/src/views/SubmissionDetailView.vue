<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'

import { getSubmission } from '@/services/apiClient'
import type { SubmissionDetail } from '@/types/problem'

const props = defineProps<{
  submissionId: string
}>()

const submission = ref<SubmissionDetail | null>(null)
const isLoading = ref(true)
const errorMessage = ref('')

async function loadSubmission() {
  isLoading.value = true
  errorMessage.value = ''

  try {
    submission.value = await getSubmission(props.submissionId)
  } catch (error) {
    submission.value = null
    errorMessage.value =
      error instanceof Error ? error.message : '제출을 불러오지 못했습니다.'
  } finally {
    isLoading.value = false
  }
}

onMounted(loadSubmission)
watch(() => props.submissionId, loadSubmission)
</script>

<template>
  <section v-if="isLoading" class="empty-state">
    <p>제출을 불러오는 중입니다.</p>
  </section>

  <section v-else-if="submission" class="content-stack">
    <div class="page-heading">
      <p class="eyebrow">Submission</p>
      <h1>{{ submission.submission.id }}</h1>
    </div>

    <div class="result-summary">
      <div>
        <span>상태</span>
        <strong>{{ submission.submission.status }}</strong>
      </div>
      <div>
        <span>언어</span>
        <strong>{{ submission.submission.language }}</strong>
      </div>
      <div>
        <span>실행 시간</span>
        <strong>{{ submission.submission.runtimeMs ?? '-' }}ms</strong>
      </div>
      <div>
        <span>메모리</span>
        <strong>{{ submission.submission.memoryMb ?? '-' }}MB</strong>
      </div>
    </div>

    <RouterLink
      :to="{
        name: 'problem-detail',
        params: { problemId: submission.submission.problemId },
      }"
    >
      문제로 돌아가기
    </RouterLink>
  </section>

  <section v-else class="empty-state">
    <h1>{{ errorMessage || '제출을 찾을 수 없습니다.' }}</h1>
  </section>
</template>
