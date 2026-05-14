export type Difficulty = 'easy' | 'medium' | 'hard'

export interface SampleCase {
  input: string
  output: string
}

export interface Problem {
  id: string
  title: string
  difficulty: Difficulty
  tags: string[]
  timeLimitMs: number
  memoryLimitMb: number
  allowedLanguages: string[]
  statement: string
  samples: SampleCase[]
}

export interface ProblemStatement {
  problemId: string
  format: 'markdown'
  content: string
}

export interface SubmissionSummary {
  id: string
  problemId: string
  language: string
  status:
    | 'queued'
    | 'dispatching'
    | 'staging'
    | 'running'
    | 'accepted'
    | 'wrong_answer'
    | 'runtime_error'
    | 'compile_error'
    | 'time_limit_exceeded'
    | 'memory_limit_exceeded'
    | 'system_error'
  submittedAt: string
  runtimeMs?: number
  memoryMb?: number
}

export interface SubmissionDetail {
  submission: SubmissionSummary
  score?: number
  maxScore?: number
}
