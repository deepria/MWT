<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'

import { createAdminProblem } from '@/services/apiClient'
import type { Difficulty } from '@/types/problem'

const router = useRouter()
const languageOptions = ['Rust', 'Python']

const form = reactive({
  problemId: '',
  title: '',
  difficulty: 'easy' as Difficulty,
  timeLimitMs: 1000,
  memoryLimitMb: 128,
  tags: '',
  statementMarkdown: '',
  allowedLanguages: ['Rust'],
})

const isSubmitting = ref(false)
const errorMessage = ref('')
const createdProblemId = ref('')

const canSubmit = computed(
  () =>
    form.problemId.trim().length > 0 &&
    form.title.trim().length > 0 &&
    form.statementMarkdown.trim().length > 0 &&
    form.allowedLanguages.length > 0 &&
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
      statement_markdown: form.statementMarkdown.trim(),
      allowed_languages: form.allowedLanguages,
    })

    createdProblemId.value = problem.id
  } catch (error) {
    errorMessage.value =
      error instanceof Error ? error.message : '문제 등록에 실패했습니다.'
  } finally {
    isSubmitting.value = false
  }
}

async function goToCreatedProblem() {
  if (!createdProblemId.value) return

  await router.push({
    name: 'admin-problem-detail',
    params: { problemId: createdProblemId.value },
  })
}
</script>

<template>
  <section class="content-stack">
    <div class="page-heading">
      <p class="eyebrow">Admin</p>
      <h1>새 문제 등록</h1>
      <RouterLink
        class="ghost-button page-action"
        :to="{ name: 'admin-problems' }"
      >
        관리자 문제 목록
      </RouterLink>
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

      <label>
        문제 설명
        <textarea
          v-model="form.statementMarkdown"
          rows="12"
          placeholder="# 문제 설명&#10;&#10;입력과 출력 조건을 Markdown으로 작성"
        />
      </label>

      <fieldset class="checkbox-group">
        <legend>제출 가능 언어</legend>
        <label v-for="language in languageOptions" :key="language">
          <input
            v-model="form.allowedLanguages"
            type="checkbox"
            :value="language"
          />
          {{ language }}
        </label>
      </fieldset>

      <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>

      <div v-if="createdProblemId" class="success-panel">
        <strong>문제 메타가 등록됐습니다.</strong>
        <span>{{ createdProblemId }}</span>
        <RouterLink
          :to="{
            name: 'admin-problem-detail',
            params: { problemId: createdProblemId },
          }"
        >
          번들 업로드로 이동
        </RouterLink>
      </div>

      <button class="primary-button" type="submit" :disabled="!canSubmit">
        {{ isSubmitting ? '저장 중' : '문제 메타 등록' }}
      </button>
      <button
        v-if="createdProblemId"
        class="ghost-button"
        type="button"
        @click="goToCreatedProblem"
      >
        상세 화면 열기
      </button>
    </form>
  </section>
</template>
