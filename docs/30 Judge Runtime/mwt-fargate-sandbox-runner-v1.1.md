---
aliases:
  - MWT Sandbox Runner
  - MWT Judge Supervisor
tags:
  - mwt
  - sandbox
  - fargate
  - judge
  - obsidian
doc_type: sandbox-runner
status: active
version: v1.1
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
related:
  - "[[mwt-architecture-v1.3]]"
  - "[[mwt-execution-plan-v1.2]]"
  - "[[mwt-problem-storage-and-staging-v1.1]]"
---

# MWT Fargate Sandbox Runner 설계서 v1.1

> [!warning] 문서 역할
> `isolate` 직접 도입 대신, Fargate 제약 안에서 구현하는 OJ용 실행 제어 계층을 정의한다.

## 빠른 링크

- 허브: [[MWT 마스터 인덱스]]
- 기준 아키텍처: [[mwt-architecture-v1.3]]
- 실행 계획: [[mwt-execution-plan-v1.2]]
- 저장/스테이징: [[mwt-problem-storage-and-staging-v1.1]]

작성일: 2026-04-28

## 1. 문서 목적

이 문서는 MWT 온라인 저지 플랫폼에서 `ioi/isolate`를 그대로 도입하지 않고, 그 핵심 운영 원칙을 Fargate 제약 안에서 재구성한 `sandbox runner` 설계를 정의한다.

목표는 아래와 같다.

- Fargate 기반 judge worker 구조를 유지한다.
- isolate 수준의 저수준 커널 기능에 직접 의존하지 않는다.
- 온라인 저지에 필요한 실행 제어, 자원 제한, 결과 수집, cleanup 흐름을 일관되게 구현한다.

## 2. 핵심 결론

현재 MWT의 방향은 아래처럼 정리된다.

- `Fargate + isolate 직접 도입`: 비권장
- `Fargate + isolate 철학을 반영한 sandbox runner`: 권장

즉, 이 문서의 목적은 `kernel-level sandbox clone`이 아니라 아래를 만드는 것이다.

`OJ용 execution supervisor`

## 3. 왜 isolate를 그대로 도입하지 않는가

`isolate`는 Linux namespaces, cgroups, setuid root 기반 제어를 활용하는 전통적인 OJ용 샌드박스다.
반면 Fargate는 privileged container, 대부분의 추가 capability, tmpfs 같은 저수준 런타임 옵션을 자유롭게 사용할 수 없다.

따라서 Fargate 위에서 isolate와 동일한 방식의 샌드박스를 구현하는 것은 현재 구조와 잘 맞지 않는다.

대신 아래 핵심만 가져온다.

- 실행 디렉터리 분리
- 입력/출력 경계 분리
- 시간 제한
- 메모리/자원 제한
- 네트워크 억제
- 실행 메타 수집
- cleanup 보장

## 4. 최종 목표

Sandbox runner의 목표는 아래다.

- 제출 단위로 깨끗한 실행 환경 제공
- 채점 중 예측 가능한 상태 전이 제공
- timeout, output limit, failure handling을 일관되게 보장
- 결과를 기계적으로 집계 가능하게 메타화
- task 종료 시 실행 흔적이 정리되도록 구성

이 목표는 `완전한 OS 샌드박스`보다 `안정적인 OJ 실행 제어 계층`에 가깝다.

## 5. 전체 구조

```text
[submission-consumer Lambda]
          |
          v
   [ECS Fargate judge worker]
          |
          v
   [sandbox runner supervisor]
      |       |       |       |       |
      v       v       v       v       v
   [stager] [compiler] [executor] [collector] [cleanup]
```

## 6. runner의 책임 범위

### runner가 담당하는 것

- submission 단위 workspace 생성
- source/bundle/checker staging
- compile/run lifecycle 제어
- timeout 처리
- output/log 제한
- 결과 요약 및 상세 기록
- cleanup

### runner가 직접 담당하지 않는 것

- queue polling
- task 실행 오케스트레이션
- 사용자 인증
- 장기 저장소 관리

## 7. 실행 모델

기본 원칙은 아래와 같다.

- 한 Fargate task는 한 submission만 처리한다.
- 한 submission은 하나의 workspace를 가진다.
- hidden tests는 bundle 단위로 받아 `/tmp`에 해제한다.
- 실행 결과는 summary와 detail로 분리해 저장한다.

## 8. 제출 workspace 구조

권장 구조:

```text
/tmp/judge/
  submissions/
    s_900001/
      source/
      build/
      tests/
      checker/
      logs/
      meta/
```

디렉터리 설명:

- `source/`: 제출 코드
- `build/`: 컴파일 산출물
- `tests/`: bundle 해제 결과
- `checker/`: checker 바이너리 또는 스크립트
- `logs/`: compile.log, run.log, result.json
- `meta/`: 내부 상태 메타, 실행 중간 산출물

## 9. 핵심 모듈 설계

## 9-1. stager

역할:

- problem metadata 조회
- manifest 조회
- source download
- bundle download
- checker download
- `/tmp` workspace 구성
- bundle 해제

입력:

- `submission_id`
- `problem_id`
- `language`

출력:

- 실행 준비가 끝난 로컬 workspace

검증 항목:

- source 존재 여부
- bundle 존재 여부
- checker 존재 여부
- bundle hash 검증
- 파일 개수/크기 상한 검증

## 9-2. compiler

역할:

- 언어별 컴파일 명령 실행
- 컴파일 로그 수집
- compile timeout 적용
- compile artifact 경로 고정

출력:

- 성공 시 실행 가능한 artifact
- 실패 시 `compile_error`

지원 언어 원칙:

- MVP는 Rust + Python 지원
- 이후 C++ 추가

## 9-3. executor

역할:

- testcase loop 수행
- stdin/stdout/stderr 리디렉션
- per-test timeout 적용
- 비정상 종료 감지

출력:

- testcase별 실행 결과

판정 대상:

- accepted
- wrong_answer
- time_limit
- runtime_error
- memory_limit
- system_error

## 9-4. limiter

역할:

- 전체 제출 wall-clock timeout
- per-test timeout
- output size limit
- compile artifact size limit
- testcase count safety check

주의:

- Fargate에서 isolate처럼 세밀한 프로세스/cgroup 제어는 어렵다.
- 따라서 runner 레벨 watchdog과 작업 단계별 hard cutoff가 중요하다.

## 9-5. collector

역할:

- summary 생성
- testcase detail 생성
- compile.log, run.log, result.json 작성
- 최종 상태 판정

예시 summary:

```json
{
  "status": "accepted",
  "passed": 20,
  "failed": 0,
  "max_time_ms": 18,
  "max_memory_kb": 1380
}
```

## 9-6. cleanup

역할:

- workspace 정리
- 중간 산출물 삭제
- 로컬 로그 정리

원칙:

- S3 업로드와 DynamoDB 반영이 끝난 뒤 cleanup
- task 종료 시 잔여 파일이 남아도 결국 ephemeral storage와 함께 폐기되지만, 논리적으로는 cleanup 단계를 분리한다.

## 10. 상태 머신

권장 상태 전이:

```text
queued
 -> dispatching
 -> staging
 -> compiling
 -> running
 -> accepted
 -> wrong_answer
 -> time_limit
 -> memory_limit
 -> runtime_error
 -> compile_error
 -> system_error
```

상태 전이 원칙:

- staging 실패는 `system_error`
- bundle/hash 불일치는 `system_error`
- 컴파일 실패는 `compile_error`
- 런타임 비정상 종료는 `runtime_error`

### 상태 전이 제약

- `queued` 이후에는 정의된 다음 단계로만 이동한다
- `dispatching`은 worker 할당 성공 시에만 `staging`으로 이동한다
- `staging`, `compiling`, `running`은 동일 `attempt` 안에서 역행하지 않는다
- final state에 도달한 submission은 자동 재처리하지 않는다
- `system_error`만 운영 정책에 따라 새 `attempt`로 재처리할 수 있다

## 11. timeout 정책

### compile timeout

- 언어별 상한 적용

### per-test timeout

- 문제 time limit 기준 적용
- 필요 시 multiplier 적용

### total timeout

- 전체 submission에 대한 최종 상한

권장 원칙:

- compile timeout
- testcase timeout
- total timeout

이 세 가지를 항상 동시에 둔다.

## 12. 리허설과 장애 주입 원칙

Sandbox runner는 구현만으로 끝나면 안 되고, 실제 실패 상황을 반복 검증해야 한다.

필수 리허설 항목:

- `while(1)` 제출
- `sleep(inf)` 제출
- 과다 stdout/stderr 출력 제출
- compile이 오래 걸리는 제출
- worker task 중간 종료
- bundle hash mismatch 상황

검증 목표:

- timeout이 기대대로 동작하는지
- system_error와 runtime_error가 혼동되지 않는지
- worker 장애가 제출 영구 정체로 이어지지 않는지

## 13. 결정성에 대한 현실 인식

IOI 같은 대회 시스템은 실행 시간 결정성을 강하게 요구하지만, Fargate 기반 환경은 bare metal 또는 isolate 기반 환경만큼 강한 결정성을 보장하지 않는다.

따라서 runner는 아래 원칙을 따른다.

- borderline timeout 문제는 운영상 별도 관찰
- 언어별 timeout multiplier를 보수적으로 적용
- 동일 제출 반복 실행 결과를 점검하는 리허설 포함
- 장기적으로 더 강한 결정성이 필요하면 EC2 기반 judge 전환 검토

## 14. output/log 제한

### output 제한

- stdout/stderr 최대 크기 제한
- 초과 시 강제 중단 또는 잘라내기 정책 적용

### log 제한

- compile.log 최대 크기
- run.log 최대 크기
- result.json은 구조화된 소형 결과만 저장

이유:

- 로그 폭주 방지
- S3 저장 비용 억제
- 디버깅 품질 유지

## 15. structured log 원칙

runner는 문자열 위주의 로그 대신 JSON structured log를 기본으로 한다.

필수 필드:

- `event`
- `submission_id`
- `problem_id`
- `language`
- `attempt`
- `worker_task_id`
- `problem_version`
- `manifest_version`
- `bundle_hash`
- `status`
- `duration_ms`
- `timestamp`

이유:

- 실패 원인 분류를 빠르게 한다
- DLQ와 재처리 대상을 묶어 추적하기 쉽다
- verdict, timeout, infra failure를 분리 집계하기 쉽다

## 16. bundle 실행 원칙

이 문서에서 가장 중요한 실무 포인트다.

### 저장

- hidden tests는 원본 파일로 관리 가능
- 채점 배포물은 bundle로 생성

### 실행

- worker는 bundle 하나만 가져온다
- `/tmp`에 압축 해제 후 로컬 파일 접근만 사용한다

### 금지

- S3의 작은 테스트 파일을 제출마다 개별 fetch

## 17. bundle 변경 관리 원칙

hidden tests와 checker는 단순 파일이 아니라 운영 대상이다.

변경 시 원칙:

- bundle 교체 시 `manifest_version` 증가
- 테스트셋 의미가 바뀌면 `problem_version` 증가
- checker 교체 시 checker hash 갱신
- 변경 후 재채점 대상 범위 결정

이 절차가 없으면 테스트셋은 바뀌었는데 제출 결과는 옛 기준으로 남는 문제가 생길 수 있다.

## 18. network / privilege 원칙

### network

- worker task의 egress 최소화
- 필요 없는 외부 통신 차단

### privilege

- privileged container 사용 안 함
- 최소 권한 task role
- 필요한 S3 prefix만 허용

### filesystem

- 제출별 staging 분리
- source/build/tests/logs 분리

## 19. 실패 처리 정책

### staging 실패

- source 누락
- bundle 누락
- checker 누락
- bundle hash mismatch

처리:

- `system_error`

### compile 실패

처리:

- `compile_error`

### testcase 실행 실패

처리:

- `wrong_answer`
- `time_limit`
- `runtime_error`
- `memory_limit`

### 내부 예외

처리:

- `system_error`

### 재처리 정책

- `compile_error`, `wrong_answer`, `time_limit`, `runtime_error`, `memory_limit`은 자동 재처리하지 않는다
- `system_error`는 원인 분류 후 재큐잉 또는 최종 실패 확정 중 하나를 선택한다
- worker 중간 종료나 RunTask launch 실패는 재처리 후보로 분류한다

## 20. 복구와 정합성 원칙

runner는 실패 자체보다, 실패 후 상태를 일관되게 남기는 것이 중요하다.

원칙:

- 결과 저장 전 실패하면 submission은 `system_error` 또는 재처리 가능 상태로 남겨야 한다
- worker 중간 종료 시 영구 `running` 상태가 남지 않도록 상위 계층에서 감시한다
- result.json, compile.log, run.log는 가능한 한 같은 제출 버전 기준으로 묶어 저장한다
- bundle/checker/source 버전 정보는 결과와 함께 추적 가능해야 한다

추가 원칙:

- `dispatching`, `staging`, `running` 상태는 운영 기준 시간을 넘기면 stuck submission으로 분류한다
- stuck submission은 자동 감시 또는 운영자 점검 후 `system_error` 또는 재처리 상태로 정리한다

## 21. 결과 저장 구조

### DynamoDB

- submission meta
- submission summary
- testcase detail

### S3

- compile.log
- run.log
- result.json

원칙:

- 목록/상태 조회는 DynamoDB
- 큰 로그와 부가 산출물은 S3

## 22. 구현 우선순위

### v1

- stager
- compiler
- executor
- collector
- cleanup
- compile/per-test/total timeout
- output/log size limit

### v1.1

- bundle hash 검증 강화
- checker 실행 통합
- 관리자 검증 루틴 강화
- structured log 스키마 고정
- 상태 고착 감시와 재처리 기준 추가

### v2

- 더 강한 프로세스 감시
- 언어별 세부 제한
- EC2 + isolate 전환 검토 가능성 유지

## 23. 한계와 현실 평가

이 runner는 아래를 제공한다.

- 실용적인 제출 격리
- 일관된 실행 제어
- 예측 가능한 결과 수집

하지만 아래는 제공하지 않는다.

- isolate 수준의 저수준 namespace/cgroup 직접 제어
- 완전한 커널 레벨 샌드박스
- 고강도 exploit 방어

따라서 이 설계의 공식 목표는 아래처럼 정의해야 한다.

- MVP 목표: `실용적이고 안정적인 OJ 실행 제어`
- 장기 목표: `더 강한 샌드박스 필요 시 judge 계층 재설계`

## 23. 권장 용어

문서와 코드에서는 아래 용어를 권장한다.

- `sandbox runner`
- `execution supervisor`
- `judge supervisor`

권장하지 않는 용어:

- `isolate-compatible`
- `isolate clone`

## 24. 최종 결론

현재 MWT 구조에서는 `isolate를 그대로 넣는 것`보다 `isolate의 핵심 운영 원칙을 구현한 sandbox runner`를 만드는 것이 맞다.

정리하면 아래와 같다.

- Fargate 유지
- bundle 기반 staging 유지
- runner가 compile/run/collect/cleanup을 통제
- timeout과 output limit을 강하게 적용

이 방식이 비용, 구현 난이도, 운영 현실성 사이에서 가장 균형이 좋다.
