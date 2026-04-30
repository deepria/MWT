<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import DifficultyBadge from '@/components/DifficultyBadge.vue'
import { getProblem, getProblemStatement } from '@/services/apiClient'
import type { Problem } from '@/types/problem'

const props = defineProps<{
  problemId: string
}>()

const router = useRouter()
const language = ref('Rust')
const source = ref('fn main() {\n    println!("hello mwt");\n}\n')
const problem = ref<Problem | null>(null)
const statement = ref('')
const isLoading = ref(true)
const errorMessage = ref('')

async function loadProblem() {
  isLoading.value = true
  errorMessage.value = ''

  try {
    const [problemResponse, statementResponse] = await Promise.all([
      getProblem(props.problemId),
      getProblemStatement(props.problemId),
    ])

    problem.value = problemResponse
    statement.value = statementResponse.content
  } catch (error) {
    problem.value = null
    statement.value = ''
    errorMessage.value =
      error instanceof Error ? error.message : '문제를 불러오지 못했습니다.'
  } finally {
    isLoading.value = false
  }
}

onMounted(loadProblem)
watch(() => props.problemId, loadProblem)

async function submitSolution() {
  await router.push({
    name: 'submission-detail',
    params: { submissionId: 'sub-20260429-001' },
  })
}
</script>

<template>
  <section v-if="isLoading" class="empty-state">
    <p>문제를 불러오는 중입니다.</p>
  </section>

  <section v-else-if="problem" class="detail-grid">
    <article class="content-stack">
      <div class="page-heading">
        <p class="eyebrow">Problem</p>
        <h1>{{ problem.title }}</h1>
        <div class="inline-meta">
          <DifficultyBadge :value="problem.difficulty" />
          <span>{{ problem.timeLimitMs }}ms</span>
          <span>{{ problem.memoryLimitMb }}MB</span>
        </div>
      </div>

      <section class="plain-section">
        <h2>문제 설명</h2>
        <pre class="statement-block">{{ statement }}</pre>
      </section>

      <section v-if="problem.samples.length > 0" class="plain-section">
        <h2>예제</h2>
        <div
          v-for="(sample, index) in problem.samples"
          :key="index"
          class="sample-grid"
        >
          <div>
            <h3>입력</h3>
            <pre>{{ sample.input }}</pre>
          </div>
          <div>
            <h3>출력</h3>
            <pre>{{ sample.output }}</pre>
          </div>
        </div>
      </section>
    </article>

    <aside class="submit-panel">
      <h2>제출</h2>
      <label>
        언어
        <select v-model="language">
          <option>Rust</option>
          <option>Python</option>
        </select>
      </label>
      <label>
        소스 코드
        <textarea v-model="source" rows="14" spellcheck="false" />
      </label>
      <button class="primary-button" type="button" @click="submitSolution">
        제출
      </button>
    </aside>
  </section>

  <section v-else class="empty-state">
    <h1>{{ errorMessage || '문제를 찾을 수 없습니다.' }}</h1>
  </section>
</template>
