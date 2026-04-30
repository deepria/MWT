<script setup lang="ts">
import type { Difficulty } from '@/types/problem'

defineProps<{
  value: Difficulty
}>()

const labels: Record<Difficulty, string> = {
  easy: 'Easy',
  medium: 'Medium',
  hard: 'Hard',
}
</script>

<template>
  <span class="difficulty-badge" :data-level="value">{{ labels[value] }}</span>
</template>
