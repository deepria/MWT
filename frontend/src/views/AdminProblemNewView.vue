<script setup lang="ts">
import { reactive } from 'vue'

const form = reactive({
  title: '',
  difficulty: 'Easy',
  timeLimitMs: 1000,
  memoryLimitMb: 128,
  tags: '',
  statement: '',
})
</script>

<template>
  <section class="content-stack">
    <div class="page-heading">
      <p class="eyebrow">Admin</p>
      <h1>새 문제 등록</h1>
    </div>

    <form class="admin-form">
      <label>
        제목
        <input v-model="form.title" type="text" placeholder="문제 제목" />
      </label>

      <div class="form-row">
        <label>
          난이도
          <select v-model="form.difficulty">
            <option>Easy</option>
            <option>Medium</option>
            <option>Hard</option>
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
        Statement
        <textarea
          v-model="form.statement"
          rows="12"
          placeholder="문제 설명 초안"
        />
      </label>

      <button class="primary-button" type="button">mock 저장</button>
    </form>
  </section>
</template>
