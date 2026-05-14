import type { Problem, SubmissionSummary } from '@/types/problem'

export const mockProblems: Problem[] = [
  {
    id: 'sum-path',
    title: '합 경로',
    difficulty: 'easy',
    tags: ['prefix-sum', 'implementation'],
    timeLimitMs: 1000,
    memoryLimitMb: 128,
    allowedLanguages: ['Rust', 'Python'],
    statement:
      '정수 배열과 여러 구간이 주어진다. 각 질의마다 구간의 합을 빠르게 출력하는 프로그램을 작성하라.',
    samples: [
      {
        input: '5 2\n1 2 3 4 5\n1 3\n2 5',
        output: '6\n14',
      },
    ],
  },
  {
    id: 'safe-maze',
    title: '안전한 미로',
    difficulty: 'medium',
    tags: ['graph', 'bfs'],
    timeLimitMs: 2000,
    memoryLimitMb: 256,
    allowedLanguages: ['Rust', 'Python'],
    statement:
      '벽과 빈 칸으로 이루어진 격자에서 시작점부터 도착점까지 이동 가능한 최단 거리를 구하라.',
    samples: [
      {
        input: '3 4\nS..#\n.#..\n...E',
        output: '5',
      },
    ],
  },
  {
    id: 'bundle-checker',
    title: '번들 체커',
    difficulty: 'hard',
    tags: ['hashing', 'validation'],
    timeLimitMs: 3000,
    memoryLimitMb: 512,
    allowedLanguages: ['Rust'],
    statement:
      '여러 테스트 파일의 해시 목록이 주어진다. manifest의 순서와 무결성을 검증하고 누락된 파일을 찾아라.',
    samples: [
      {
        input: '3\ncase1 a1\ncase2 b2\ncase3 c3\n2\ncase1 a1\ncase3 c3',
        output: 'case2',
      },
    ],
  },
]

export const mockSubmissions: SubmissionSummary[] = [
  {
    id: 'sub-20260429-001',
    problemId: 'sum-path',
    language: 'Rust',
    status: 'accepted',
    submittedAt: '2026-04-29T09:20:00+09:00',
    runtimeMs: 42,
    memoryMb: 18,
  },
]
