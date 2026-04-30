<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'

import DifficultyBadge from '@/components/DifficultyBadge.vue'
import { listProblems } from '@/services/apiClient'
import type { Problem } from '@/types/problem'

const problems = ref<Problem[]>([])
const isLoading = ref(true)
const errorMessage = ref('')

onMounted(async () => {
  try {
    problems.value = await listProblems()
  } catch (error) {
    errorMessage.value =
      error instanceof Error
        ? error.message
        : '문제 목록을 불러오지 못했습니다.'
  } finally {
    isLoading.value = false
  }
})
</script>

<template>
  <section class="content-stack">
    <div class="page-heading">
      <p class="eyebrow">Problems</p>
      <h1>문제 목록</h1>
    </div>

    <div v-if="isLoading" class="empty-state compact">
      <p>문제 목록을 불러오는 중입니다.</p>
    </div>

    <div v-else-if="errorMessage" class="empty-state compact">
      <p>{{ errorMessage }}</p>
    </div>

    <div v-else class="problem-list">
      <RouterLink
        v-for="problem in problems"
        :key="problem.id"
        class="problem-row"
        :to="{ name: 'problem-detail', params: { problemId: problem.id } }"
      >
        <div>
          <div class="row-title">{{ problem.title }}</div>
          <div class="tag-list">
            <span v-for="tag in problem.tags" :key="tag">{{ tag }}</span>
          </div>
        </div>

        <div class="row-meta">
          <DifficultyBadge :value="problem.difficulty" />
          <span>{{ problem.timeLimitMs }}ms</span>
          <span>{{ problem.memoryLimitMb }}MB</span>
        </div>
      </RouterLink>
    </div>
  </section>
</template>
