<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'

import DifficultyBadge from '@/components/DifficultyBadge.vue'
import { type AdminProblem, listAdminProblems } from '@/services/apiClient'

const problems = ref<AdminProblem[]>([])
const isLoading = ref(true)
const errorMessage = ref('')

onMounted(async () => {
  try {
    problems.value = await listAdminProblems()
  } catch (error) {
    errorMessage.value =
      error instanceof Error
        ? error.message
        : '관리자 문제 목록을 불러오지 못했습니다.'
  } finally {
    isLoading.value = false
  }
})
</script>

<template>
  <section class="content-stack">
    <div class="page-heading">
      <p class="eyebrow">Admin</p>
      <h1>관리자 문제 목록</h1>
      <RouterLink
        class="primary-button page-action"
        :to="{ name: 'admin-problem-new' }"
      >
        새 문제 등록
      </RouterLink>
    </div>

    <div v-if="isLoading" class="empty-state compact">
      <p>관리자 문제 목록을 불러오는 중입니다.</p>
    </div>

    <div v-else-if="errorMessage" class="empty-state compact">
      <p>{{ errorMessage }}</p>
    </div>

    <div v-else-if="problems.length === 0" class="empty-state compact">
      <p>등록된 문제가 없습니다.</p>
    </div>

    <div v-else class="problem-list">
      <RouterLink
        v-for="problem in problems"
        :key="problem.id"
        class="problem-row"
        :to="{
          name: 'admin-problem-detail',
          params: { problemId: problem.id },
        }"
      >
        <div>
          <div class="row-title">{{ problem.title }}</div>
          <div class="tag-list">
            <span class="status-pill">{{ problem.visibility }}</span>
            <span v-for="tag in problem.tags" :key="tag">{{ tag }}</span>
          </div>
        </div>

        <div class="row-meta">
          <DifficultyBadge :value="problem.difficulty" />
          <span>{{ problem.timeLimitMs }}ms</span>
          <span>{{ problem.memoryLimitMb }}MB</span>
          <span>{{ problem.allowedLanguages.join(', ') }}</span>
          <span>
            manifest
            {{
              problem.manifestVersion ? `v${problem.manifestVersion}` : '없음'
            }}
          </span>
          <span>{{ problem.bundleKey ? 'bundle 있음' : 'bundle 없음' }}</span>
        </div>
      </RouterLink>
    </div>
  </section>
</template>
