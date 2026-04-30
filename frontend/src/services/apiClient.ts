import { getHostedUiAuthorizationToken } from '@/services/hostedUiAuth'
import type {
  Difficulty,
  Problem,
  ProblemStatement,
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

async function request<T>(path: string, options: { auth?: boolean } = {}) {
  const headers = new Headers()

  if (options.auth) {
    const token = getHostedUiAuthorizationToken()

    if (token) {
      headers.set('authorization', `Bearer ${token}`)
    }
  }

  const response = await window.fetch(`${requireApiBaseUrl()}${path}`, {
    headers,
  })

  if (!response.ok) {
    throw new Error(`API 요청 실패: ${response.status}`)
  }

  return (await response.json()) as T
}

function toProblem(problem: ApiProblem): Problem {
  return {
    id: problem.problem_id,
    title: problem.title,
    difficulty: problem.difficulty,
    tags: problem.tags,
    timeLimitMs: problem.time_limit_ms,
    memoryLimitMb: problem.memory_limit_mb,
    statement: '',
    samples: [],
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
