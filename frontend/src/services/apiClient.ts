import { getHostedUiAuthorizationToken } from '@/services/hostedUiAuth'
import type {
  Difficulty,
  Problem,
  ProblemStatement,
  SampleCase,
  SubmissionDetail,
  SubmissionSummary,
} from '@/types/problem'

interface ApiProblem {
  problem_id: string
  title: string
  difficulty: Difficulty
  tags: string[]
  time_limit_ms: number
  memory_limit_mb: number
  visibility?: 'draft' | 'public' | 'archived'
  statement_markdown?: string
  statement_location?: string
  allowed_languages?: string[]
  sample_cases?: SampleCase[]
  bundle_key?: string | null
  bundle_hash?: string | null
  checker_key?: string | null
  checker_hash?: string | null
  problem_version?: number
  manifest_version?: number | null
}

interface CreateProblemRequest {
  problem_id: string
  title: string
  difficulty: Difficulty
  tags: string[]
  time_limit_ms: number
  memory_limit_mb: number
  statement_markdown: string
  allowed_languages: string[]
  sample_cases: SampleCase[]
}

type AssetType =
  | 'statement'
  | 'sample_input'
  | 'sample_output'
  | 'bundle'
  | 'checker'

interface PresignAssetRequest {
  asset_type: AssetType
  content_type?: string
  sample_id?: string
}

interface PresignedUploadResponse {
  bucket: string
  key: string
  upload_url: string
  expires_in_seconds: number
}

export interface ManifestCaseRequest {
  id: number
  input_path: string
  output_path: string
  weight: number
}

interface FinalizeBundleRequest {
  bundle_key: string
  bundle_hash: string
  bundle_size_bytes: number
  cases: ManifestCaseRequest[]
  checker_key?: string
  checker_hash?: string
}

interface FinalizeBundleResponse {
  problem_id: string
  problem_version: number
  manifest_version: number
  bundle_key: string
  bundle_hash: string
}

interface UpdateProblemVisibilityRequest {
  visibility: 'draft' | 'public' | 'archived'
}

interface UpdateProblemContentRequest {
  statement_markdown: string
  sample_cases: SampleCase[]
}

export interface AdminProblem {
  id: string
  title: string
  difficulty: Difficulty
  tags: string[]
  timeLimitMs: number
  memoryLimitMb: number
  statementMarkdown: string
  visibility: 'draft' | 'public' | 'archived'
  statementLocation: string
  allowedLanguages: string[]
  sampleCases: SampleCase[]
  bundleKey: string | null
  bundleHash: string | null
  checkerKey: string | null
  checkerHash: string | null
  problemVersion: number
  manifestVersion: number | null
}

interface ApiStatement {
  problem_id: string
  format: 'markdown'
  content: string
}

interface ApiSubmissionMeta {
  submission_id: string
  problem_id: string
  language: string
  status: SubmissionSummary['status']
  submitted_at: string
}

interface ApiSubmissionResult {
  score?: number
  max_score?: number
  runtime_ms?: number
  memory_mb?: number
}

interface ApiSubmissionDetail {
  submission: ApiSubmissionMeta
  result?: ApiSubmissionResult
}

const apiBaseUrl = (import.meta.env.VITE_API_BASE_URL ?? '').replace(/\/$/, '')

function requireApiBaseUrl() {
  if (!apiBaseUrl) {
    throw new Error('VITE_API_BASE_URL이 설정되지 않았습니다.')
  }

  return apiBaseUrl
}

async function request<T>(
  path: string,
  options: {
    auth?: boolean
    method?: string
    body?: unknown
  } = {},
) {
  const headers = new Headers()

  if (options.auth) {
    const token = getHostedUiAuthorizationToken()

    if (token) {
      headers.set('authorization', `Bearer ${token}`)
    } else {
      setMockAuthHeaders(headers)
    }
  }

  if (options.body !== undefined) {
    headers.set('content-type', 'application/json')
  }

  const response = await window.fetch(`${requireApiBaseUrl()}${path}`, {
    method: options.method ?? 'GET',
    headers,
    body: options.body === undefined ? undefined : JSON.stringify(options.body),
  })

  if (!response.ok) {
    throw new Error(await responseErrorMessage(response))
  }

  return (await response.json()) as T
}

async function responseErrorMessage(response: Response) {
  try {
    const body = (await response.json()) as { message?: string }
    return body.message ?? `API 요청 실패: ${response.status}`
  } catch {
    return `API 요청 실패: ${response.status}`
  }
}

function setMockAuthHeaders(headers: Headers) {
  if ((import.meta.env.VITE_AUTH_PROVIDER ?? 'mock') !== 'mock') return

  const raw = window.localStorage.getItem('mwt.auth.session')
  if (!raw) return

  try {
    const user = JSON.parse(raw) as {
      id?: string
      email?: string
      groups?: string[]
    }

    if (user.id) headers.set('x-mwt-user-id', user.id)
    if (user.email) headers.set('x-mwt-email', user.email)
    if (user.groups?.length) headers.set('x-mwt-groups', user.groups.join(','))
  } catch {
    // Ignore malformed local mock sessions.
  }
}

function toProblem(problem: ApiProblem): Problem {
  return {
    id: problem.problem_id,
    title: problem.title,
    difficulty: problem.difficulty,
    tags: problem.tags,
    timeLimitMs: problem.time_limit_ms,
    memoryLimitMb: problem.memory_limit_mb,
    allowedLanguages: problem.allowed_languages ?? ['Rust', 'Python'],
    statement: problem.statement_markdown ?? '',
    samples: problem.sample_cases ?? [],
  }
}

function toAdminProblem(problem: ApiProblem): AdminProblem {
  return {
    id: problem.problem_id,
    title: problem.title,
    difficulty: problem.difficulty,
    tags: problem.tags,
    timeLimitMs: problem.time_limit_ms,
    memoryLimitMb: problem.memory_limit_mb,
    statementMarkdown: problem.statement_markdown ?? '',
    visibility: problem.visibility ?? 'draft',
    statementLocation: problem.statement_location ?? '',
    allowedLanguages: problem.allowed_languages ?? ['Rust', 'Python'],
    sampleCases: problem.sample_cases ?? [],
    bundleKey: problem.bundle_key ?? null,
    bundleHash: problem.bundle_hash ?? null,
    checkerKey: problem.checker_key ?? null,
    checkerHash: problem.checker_hash ?? null,
    problemVersion: problem.problem_version ?? 1,
    manifestVersion: problem.manifest_version ?? null,
  }
}

function toStatement(statement: ApiStatement): ProblemStatement {
  return {
    problemId: statement.problem_id,
    format: statement.format,
    content: statement.content,
  }
}

function toSubmissionSummary(
  submission: ApiSubmissionMeta,
  result?: ApiSubmissionResult,
): SubmissionSummary {
  return {
    id: submission.submission_id,
    problemId: submission.problem_id,
    language: submission.language,
    status: submission.status,
    submittedAt: submission.submitted_at,
    runtimeMs: result?.runtime_ms,
    memoryMb: result?.memory_mb,
  }
}

function toSubmissionDetail(detail: ApiSubmissionDetail): SubmissionDetail {
  return {
    submission: toSubmissionSummary(detail.submission, detail.result),
    score: detail.result?.score,
    maxScore: detail.result?.max_score,
  }
}

export async function listProblems() {
  const problems = await request<ApiProblem[]>('/problems')

  return problems.map(toProblem)
}

export async function getProblem(problemId: string) {
  return toProblem(await request<ApiProblem>(`/problems/${problemId}`))
}

export async function getProblemStatement(problemId: string) {
  return toStatement(
    await request<ApiStatement>(`/problems/${problemId}/statement`),
  )
}

export async function getSubmission(submissionId: string) {
  return toSubmissionDetail(
    await request<ApiSubmissionDetail>(`/submissions/${submissionId}`, {
      auth: true,
    }),
  )
}

export async function listMySubmissions() {
  const submissions = await request<ApiSubmissionMeta[]>(
    '/users/me/submissions',
    { auth: true },
  )

  return submissions.map((submission) => toSubmissionSummary(submission))
}

export async function createAdminProblem(problem: CreateProblemRequest) {
  return toAdminProblem(
    await request<ApiProblem>('/admin/problems', {
      auth: true,
      method: 'POST',
      body: problem,
    }),
  )
}

export async function listAdminProblems() {
  const problems = await request<ApiProblem[]>('/admin/problems', {
    auth: true,
  })

  return problems.map(toAdminProblem)
}

export async function getAdminProblem(problemId: string) {
  return toAdminProblem(
    await request<ApiProblem>(
      `/admin/problems/${encodeURIComponent(problemId)}`,
      {
        auth: true,
      },
    ),
  )
}

export async function presignAdminProblemAsset(
  problemId: string,
  payload: PresignAssetRequest,
) {
  return request<PresignedUploadResponse>(
    `/admin/problems/${encodeURIComponent(problemId)}/assets/presign`,
    {
      auth: true,
      method: 'POST',
      body: payload,
    },
  )
}

export async function finalizeAdminProblemBundle(
  problemId: string,
  payload: FinalizeBundleRequest,
) {
  return request<FinalizeBundleResponse>(
    `/admin/problems/${encodeURIComponent(problemId)}/bundle/finalize`,
    {
      auth: true,
      method: 'POST',
      body: payload,
    },
  )
}

export async function updateAdminProblemVisibility(
  problemId: string,
  payload: UpdateProblemVisibilityRequest,
) {
  return toAdminProblem(
    await request<ApiProblem>(
      `/admin/problems/${encodeURIComponent(problemId)}/visibility`,
      {
        auth: true,
        method: 'PATCH',
        body: payload,
      },
    ),
  )
}

export async function updateAdminProblemContent(
  problemId: string,
  payload: UpdateProblemContentRequest,
) {
  return toAdminProblem(
    await request<ApiProblem>(
      `/admin/problems/${encodeURIComponent(problemId)}/content`,
      {
        auth: true,
        method: 'PATCH',
        body: payload,
      },
    ),
  )
}

export async function uploadPresignedObject(
  uploadUrl: string,
  file: File,
  contentType: string,
) {
  const response = await window.fetch(uploadUrl, {
    method: 'PUT',
    headers: {
      'content-type': contentType,
    },
    body: file,
  })

  if (!response.ok) {
    throw new Error(await responseErrorMessage(response))
  }
}
