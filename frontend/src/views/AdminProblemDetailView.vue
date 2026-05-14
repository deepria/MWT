<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'

import DifficultyBadge from '@/components/DifficultyBadge.vue'
import {
  type AdminProblem,
  finalizeAdminProblemBundle,
  getAdminProblem,
  type ManifestCaseRequest,
  presignAdminProblemAsset,
  uploadPresignedObject,
} from '@/services/apiClient'

const props = defineProps<{
  problemId: string
}>()

const problem = ref<AdminProblem | null>(null)
const isLoading = ref(true)
const isUploadingBundle = ref(false)
const errorMessage = ref('')
const bundleErrorMessage = ref('')
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

const totalCaseWeight = computed(() =>
  bundleCases.reduce(
    (sum, bundleCase) => sum + Number(bundleCase.weight || 0),
    0,
  ),
)

const canFinalizeBundle = computed(
  () =>
    problem.value !== null &&
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

onMounted(loadProblem)

async function loadProblem() {
  isLoading.value = true
  errorMessage.value = ''

  try {
    problem.value = await getAdminProblem(props.problemId)
  } catch (error) {
    errorMessage.value =
      error instanceof Error
        ? error.message
        : '문제 정보를 불러오지 못했습니다.'
  } finally {
    isLoading.value = false
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
  if (!canFinalizeBundle.value || !selectedBundle.value || !problem.value)
    return

  isUploadingBundle.value = true
  bundleErrorMessage.value = ''
  uploadedBundleKey.value = ''
  uploadedBundleHash.value = ''
  finalizeResult.value = null

  const file = selectedBundle.value
  const contentType = file.type || 'application/zip'

  try {
    const [upload, bundleHash] = await Promise.all([
      presignAdminProblemAsset(problem.value.id, {
        asset_type: 'bundle',
        content_type: contentType,
      }),
      sha256Hex(file),
    ])

    await uploadPresignedObject(upload.upload_url, file, contentType)

    const finalized = await finalizeAdminProblemBundle(problem.value.id, {
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
    await loadProblem()
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
      <h1>문제 관리 상세</h1>
      <RouterLink
        class="ghost-button page-action"
        :to="{ name: 'admin-problems' }"
      >
        목록으로
      </RouterLink>
    </div>

    <div v-if="isLoading" class="empty-state compact">
      <p>문제 정보를 불러오는 중입니다.</p>
    </div>

    <div v-else-if="errorMessage" class="empty-state compact">
      <p>{{ errorMessage }}</p>
    </div>

    <template v-else-if="problem">
      <section class="admin-summary">
        <div>
          <span class="status-pill">{{ problem.visibility }}</span>
          <h2>{{ problem.title }}</h2>
          <p>{{ problem.id }}</p>
        </div>

        <div class="row-meta">
          <DifficultyBadge :value="problem.difficulty" />
          <span>{{ problem.timeLimitMs }}ms</span>
          <span>{{ problem.memoryLimitMb }}MB</span>
          <span>{{ problem.allowedLanguages.join(', ') }}</span>
          <span>problem v{{ problem.problemVersion }}</span>
          <span>
            manifest
            {{
              problem.manifestVersion ? `v${problem.manifestVersion}` : '없음'
            }}
          </span>
        </div>

        <dl class="debug-list compact-list">
          <div>
            <dt>Statement</dt>
            <dd>{{ problem.statementLocation }}</dd>
          </div>
          <div>
            <dt>등록 설명</dt>
            <dd>
              <pre class="statement-block inline-statement">{{
                problem.statementMarkdown
              }}</pre>
            </dd>
          </div>
          <div>
            <dt>예제</dt>
            <dd>
              <div
                v-for="(sampleCase, index) in problem.sampleCases"
                :key="index"
                class="sample-grid compact-sample-grid"
              >
                <div>
                  <h3>입력 {{ index + 1 }}</h3>
                  <pre>{{ sampleCase.input }}</pre>
                </div>
                <div>
                  <h3>출력 {{ index + 1 }}</h3>
                  <pre>{{ sampleCase.output }}</pre>
                </div>
              </div>
            </dd>
          </div>
          <div>
            <dt>Bundle</dt>
            <dd>{{ problem.bundleKey ?? '아직 없음' }}</dd>
          </div>
          <div>
            <dt>Bundle hash</dt>
            <dd>{{ problem.bundleHash ?? '아직 없음' }}</dd>
          </div>
        </dl>
      </section>

      <section class="admin-form">
        <div class="section-heading">
          <h2>테스트 번들 업로드</h2>
          <span>업로드 시점에 새 URL 발급</span>
        </div>

        <label>
          번들 ZIP
          <input
            type="file"
            accept=".zip,application/zip"
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
    </template>
  </section>
</template>
