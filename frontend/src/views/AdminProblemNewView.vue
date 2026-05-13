<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'

import {
  createAdminProblem,
  finalizeAdminProblemBundle,
  type ManifestCaseRequest,
  presignAdminProblemAsset,
  uploadPresignedObject,
} from '@/services/apiClient'
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
const isUploadingBundle = ref(false)
const errorMessage = ref('')
const bundleErrorMessage = ref('')
const createdProblemId = ref('')
const selectedBundle = ref<File | null>(null)
const uploadedBundleKey = ref('')
const uploadedBundleHash = ref('')
const finalizeResult = ref<{
  manifest_version: number
  problem_version: number
  bundle_key: string
} | null>(null)

const bundleCases = reactive([
  {
    id: 1,
    inputPath: 'cases/001.in',
    outputPath: 'cases/001.out',
    weight: 100,
  },
])

const canSubmit = computed(
  () =>
    form.problemId.trim().length > 0 &&
    form.title.trim().length > 0 &&
    form.timeLimitMs >= 100 &&
    form.memoryLimitMb >= 16 &&
    !isSubmitting.value,
)

const totalCaseWeight = computed(() =>
  bundleCases.reduce(
    (sum, bundleCase) => sum + Number(bundleCase.weight || 0),
    0,
  ),
)

const targetProblemId = computed(
  () => createdProblemId.value || form.problemId.trim(),
)

const canFinalizeBundle = computed(
  () =>
    targetProblemId.value.length > 0 &&
    selectedBundle.value !== null &&
    bundleCases.length > 0 &&
    totalCaseWeight.value === 100 &&
    bundleCases.every(
      (bundleCase) =>
        bundleCase.id > 0 &&
        bundleCase.inputPath.trim().length > 0 &&
        bundleCase.outputPath.trim().length > 0 &&
        bundleCase.weight > 0,
    ) &&
    !isUploadingBundle.value,
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

function onBundleSelected(event: Event) {
  const input = event.target as HTMLInputElement
  selectedBundle.value = input.files?.[0] ?? null
  uploadedBundleKey.value = ''
  uploadedBundleHash.value = ''
  finalizeResult.value = null
  bundleErrorMessage.value = ''
}

function addBundleCase() {
  const nextId =
    bundleCases.reduce(
      (maxId, bundleCase) => Math.max(maxId, bundleCase.id),
      0,
    ) + 1

  bundleCases.push({
    id: nextId,
    inputPath: `cases/${String(nextId).padStart(3, '0')}.in`,
    outputPath: `cases/${String(nextId).padStart(3, '0')}.out`,
    weight: 0,
  })
}

function removeBundleCase(index: number) {
  if (bundleCases.length <= 1) return

  bundleCases.splice(index, 1)
}

function toManifestCases(): ManifestCaseRequest[] {
  return bundleCases.map((bundleCase) => ({
    id: Number(bundleCase.id),
    input_path: bundleCase.inputPath.trim(),
    output_path: bundleCase.outputPath.trim(),
    weight: Number(bundleCase.weight),
  }))
}

async function sha256Hex(file: File) {
  const hash = await window.crypto.subtle.digest(
    'SHA-256',
    await file.arrayBuffer(),
  )

  return Array.from(new Uint8Array(hash))
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')
}

async function uploadAndFinalizeBundle() {
  if (!canFinalizeBundle.value || !selectedBundle.value) return

  isUploadingBundle.value = true
  bundleErrorMessage.value = ''
  uploadedBundleKey.value = ''
  uploadedBundleHash.value = ''
  finalizeResult.value = null

  const file = selectedBundle.value
  const contentType = file.type || 'application/zip'

  try {
    const [upload, bundleHash] = await Promise.all([
      presignAdminProblemAsset(targetProblemId.value, {
        asset_type: 'bundle',
        content_type: contentType,
      }),
      sha256Hex(file),
    ])

    await uploadPresignedObject(upload.upload_url, file, contentType)

    const finalized = await finalizeAdminProblemBundle(targetProblemId.value, {
      bundle_key: upload.key,
      bundle_hash: `sha256:${bundleHash}`,
      bundle_size_bytes: file.size,
      cases: toManifestCases(),
    })

    uploadedBundleKey.value = upload.key
    uploadedBundleHash.value = `sha256:${bundleHash}`
    finalizeResult.value = {
      manifest_version: finalized.manifest_version,
      problem_version: finalized.problem_version,
      bundle_key: finalized.bundle_key,
    }
  } catch (error) {
    bundleErrorMessage.value =
      error instanceof Error ? error.message : '번들 finalize에 실패했습니다.'
  } finally {
    isUploadingBundle.value = false
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

    <section class="admin-form">
      <div class="section-heading">
        <h2>테스트 번들 업로드</h2>
        <span v-if="targetProblemId">{{ targetProblemId }}</span>
      </div>

      <label>
        번들 ZIP
        <input
          type="file"
          accept=".zip,application/zip"
          :disabled="!targetProblemId"
          @change="onBundleSelected"
        />
      </label>

      <div class="case-editor">
        <div class="case-editor-heading">
          <strong>Manifest cases</strong>
          <span :class="{ 'weight-error': totalCaseWeight !== 100 }">
            weight {{ totalCaseWeight }}/100
          </span>
        </div>

        <div
          v-for="(bundleCase, index) in bundleCases"
          :key="index"
          class="case-row"
        >
          <label>
            ID
            <input v-model.number="bundleCase.id" type="number" min="1" />
          </label>
          <label>
            Input path
            <input v-model.trim="bundleCase.inputPath" type="text" />
          </label>
          <label>
            Output path
            <input v-model.trim="bundleCase.outputPath" type="text" />
          </label>
          <label>
            Weight
            <input v-model.number="bundleCase.weight" type="number" min="1" />
          </label>
          <button
            class="ghost-button compact-button"
            type="button"
            :disabled="bundleCases.length <= 1"
            @click="removeBundleCase(index)"
          >
            삭제
          </button>
        </div>

        <button
          class="ghost-button add-case-button"
          type="button"
          @click="addBundleCase"
        >
          케이스 추가
        </button>
      </div>

      <p v-if="bundleErrorMessage" class="form-error">
        {{ bundleErrorMessage }}
      </p>

      <div v-if="finalizeResult" class="success-panel">
        <strong>번들이 finalize 됐습니다.</strong>
        <span>manifest v{{ finalizeResult.manifest_version }}</span>
        <span>problem v{{ finalizeResult.problem_version }}</span>
        <span>{{ finalizeResult.bundle_key }}</span>
      </div>

      <div v-else-if="uploadedBundleKey" class="success-panel">
        <strong>번들 업로드가 완료됐습니다.</strong>
        <span>{{ uploadedBundleKey }}</span>
        <span>{{ uploadedBundleHash }}</span>
      </div>

      <button
        class="primary-button"
        type="button"
        :disabled="!canFinalizeBundle"
        @click="uploadAndFinalizeBundle"
      >
        {{ isUploadingBundle ? '처리 중' : '업로드 후 finalize' }}
      </button>
    </section>
  </section>
</template>
