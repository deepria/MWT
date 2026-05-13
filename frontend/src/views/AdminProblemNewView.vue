<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'

import { createAdminProblem } from '@/services/apiClient'
import type { Difficulty } from '@/types/problem'

const form = reactive({
  problemId: '',
  title: '',
  difficulty: 'easy' as Difficulty,
  timeLimitMs: 1000,
  memoryLimitMb: 128,
  tags: '',
})

const isSubmitting = ref(false)
const errorMessage = ref('')
const createdProblemId = ref('')

const canSubmit = computed(
  () =>
    form.problemId.trim().length > 0 &&
    form.title.trim().length > 0 &&
    form.timeLimitMs >= 100 &&
    form.memoryLimitMb >= 16 &&
    !isSubmitting.value,
)

function tagsFromInput() {
  return form.tags
    .split(',')
    .map((tag) => tag.trim())
    .filter(Boolean)
}

async function submitProblem() {
  if (!canSubmit.value) return

  isSubmitting.value = true
  errorMessage.value = ''
  createdProblemId.value = ''

  try {
    const problem = await createAdminProblem({
      problem_id: form.problemId.trim(),
      title: form.title.trim(),
      difficulty: form.difficulty,
      tags: tagsFromInput(),
      time_limit_ms: form.timeLimitMs,
      memory_limit_mb: form.memoryLimitMb,
    })

    createdProblemId.value = problem.id
  } catch (error) {
    errorMessage.value =
      error instanceof Error ? error.message : '문제 등록에 실패했습니다.'
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <section class="content-stack">
    <div class="page-heading">
      <p class="eyebrow">Admin</p>
      <h1>새 문제 등록</h1>
    </div>

    <form class="admin-form" @submit.prevent="submitProblem">
      <label>
        문제 ID
        <input
          v-model.trim="form.problemId"
          type="text"
          placeholder="two-sum"
          autocomplete="off"
        />
      </label>

      <label>
        제목
        <input v-model="form.title" type="text" placeholder="문제 제목" />
      </label>

      <div class="form-row">
        <label>
          난이도
          <select v-model="form.difficulty">
            <option value="easy">Easy</option>
            <option value="medium">Medium</option>
            <option value="hard">Hard</option>
          </select>
        </label>
        <label>
          시간 제한(ms)
          <input v-model.number="form.timeLimitMs" type="number" min="100" />
        </label>
        <label>
          메모리 제한(MB)
          <input v-model.number="form.memoryLimitMb" type="number" min="16" />
        </label>
      </div>

      <label>
        태그
        <input v-model="form.tags" type="text" placeholder="graph, bfs" />
      </label>

      <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>

      <div v-if="createdProblemId" class="success-panel">
        <strong>문제 메타가 등록됐습니다.</strong>
        <span>{{ createdProblemId }}</span>
        <RouterLink :to="`/problems/${createdProblemId}`">
          문제 상세 확인
        </RouterLink>
      </div>

      <button class="primary-button" type="submit" :disabled="!canSubmit">
        {{ isSubmitting ? '저장 중' : '문제 메타 등록' }}
      </button>
    </form>
  </section>
</template>
