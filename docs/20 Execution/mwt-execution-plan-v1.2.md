---
aliases:
  - MWT 실행 계획
  - MWT Sprint Plan
tags:
  - mwt
  - execution
  - roadmap
  - obsidian
doc_type: execution-plan
status: active
version: v1.2
updated: 2026-05-14
hub: "[[MWT 마스터 인덱스]]"
related:
  - "[[mwt-architecture-v1.3]]"
  - "[[mwt-problem-storage-and-staging-v1.1]]"
  - "[[mwt-fargate-sandbox-runner-v1.1]]"
  - "[[📋 IOI 체크리스트 (번역본)]]"
---

# MWT 실행 계획서 v1.2

> [!info] 문서 역할
> 최신 설계 기준을 실제 구현/운영 태스크로 쪼갠 실행 계획서다.

## 빠른 링크

- 허브: [[MWT 마스터 인덱스]]
- 아키텍처: [[mwt-architecture-v1.3]]
- 저장/스테이징: [[mwt-problem-storage-and-staging-v1.1]]
- 샌드박스 러너: [[mwt-fargate-sandbox-runner-v1.1]]
- 참고 체크리스트: [[📋 IOI 체크리스트 (번역본)]]

기준 문서:

- [[mwt-architecture-v1.3]]
- [[mwt-cost-down-v1.1]]
- [[mwt-problem-storage-and-staging-v1.1]]
- [[mwt-fargate-sandbox-runner-v1.1]]

작성일: 2026-04-28

## 1. 목적

이 문서는 최신 설계 기준을 실제 작업 단위로 쪼갠 실행 계획서다.

특히 아래 내용을 반영한다.

- statement는 S3 저장
- hidden tests는 bundle 단위 관리
- manifest는 DynamoDB 저장
- worker는 bundle을 `/tmp`에 staging 후 실행
- 상태 전이는 엄격한 단계 규칙으로 관리
- 운영 로그는 JSON structured log 기준으로 수집
- `system_error` 제출의 재처리와 rejudge 경로를 운영 절차에 포함

## 2. MVP 완료 조건

- [x] 로그인 가능
- [x] 문제 목록 조회 가능
- [x] 문제 상세 조회 가능
- [~] 관리자 문제 등록 가능
- [x] 문제 설명 본문 등록/조회 가능
- [x] 문제별 제출 가능 언어 제한 등록/조회 가능
- [ ] statement 업로드 가능
- [~] hidden tests bundle 생성/등록 가능
- [ ] 제출 생성 가능
- [ ] 최소 1개 언어로 채점 가능
- [ ] polling으로 결과 확인 가능

## 3. 전체 Phase

### Phase 0

- [x] 저장소 및 환경 준비

### Phase 1

- [x] 프론트 기본 뼈대

### Phase 2

- [x] API 및 데이터 모델

### Phase 3

- [~] 문제 자산 저장과 manifest

### Phase 4

- [ ] 제출 및 채점 파이프라인

### Phase 5

- [~] 관리자 문제 등록 플로우

### Phase 6

- [ ] 운영 최소 구성

### Phase 7

- [ ] 리허설과 장애 주입 테스트

## 4. 상세 태스크

## Phase 상세 설계 링크

- [[Phase 0 - 초기 세팅 상세 설계]]
- [[Phase 1 - 프론트 기본 뼈대 상세 설계]]
- [[Phase 2 - API 및 데이터 모델 상세 설계]]
- [[Phase 3 - 문제 자산과 Manifest 상세 설계]]
- [[Phase 4 - 제출 및 채점 파이프라인 상세 설계]]
- [[Phase 5 - 관리자 문제 등록 상세 설계]]
- [[Phase 6 - 운영 최소 구성 상세 설계]]
- [[Phase 7 - 리허설과 장애 주입 상세 설계]]

## Phase 0. 초기 세팅

### P0-001. 리전 결정

- [x] `ap-northeast-2` 선택
- [x] 비용 우선 여부 기록

추천:

- `ap-northeast-2`로 시작

이유:

- 한국 사용자 기준 지연시간과 운영 체감이 좋다
- 서비스 대상 지역과 리전이 일치해 네트워크 경로가 단순하다
- 서울 리전을 기준으로 실제 사용자 체감 성능을 초반부터 검증할 수 있다

완료 기준:

- [x] 기본 리전 확정

### P0-002. 저장소 구조 생성

- [x] `frontend/`
- [x] `backend/`
- [x] `judge/`
- [x] `infra/`
- [x] `docs/`

완료 기준:

- [x] 기본 폴더 구조 생성 완료

### P0-003. 공통 네이밍 규칙 정의

- [x] 리소스 prefix
- [x] bucket 이름
- [x] table 이름
- [x] queue 이름

완료 기준:

- [x] 네이밍 규칙 문서화

### P0-004. AWS 예산 알림 설정

- [ ] 월 예산 한도
- [ ] 경고 알림

완료 기준:

- [ ] 예산 알람 활성화

## Phase 1. 프론트 기본 뼈대

### P1-001. Vue 앱 초기화

- [x] Vue 프로젝트 생성
- [x] 기본 lint/format 구성

완료 기준:

- [x] 로컬 실행 가능

### P1-002. 라우팅 구성

- [x] `/login`
- [x] `/problems`
- [x] `/problems/:problemId`
- [x] `/submissions/:submissionId`
- [x] `/admin/problems/new`

완료 기준:

- [x] 주요 라우트 진입 가능

### P1-003. Cognito 로그인 연결

- [x] User Pool 생성
- [x] 로그인/로그아웃 구현
- [x] 보호 라우트 처리

완료 기준:

- [x] 로그인 성공 후 세션 유지

### P1-004. 문제 목록/상세 mock UI

- [x] 목록 화면
- [x] 상세 화면
- [x] statement 영역
- [x] sample 영역
- [x] 제출 입력 UI 초안

완료 기준:

- [x] mock 기준 화면 사용 가능

## Phase 2. API 및 데이터 모델

### P2-001. Rust Lambda workspace 구성

- [x] `public-api`
- [x] `admin-api`
- [x] `submission-consumer`
- [x] 공통 domain/infra crate

완료 기준:

- [x] 로컬 빌드 성공

### P2-002. core_table 생성

- [x] problem meta
- [x] problem manifest
- [x] submission meta
- [x] submission result

완료 기준:

- [x] DynamoDB table 생성 완료

### P2-003. 최소 GSI 구성

- [x] 내 제출 목록
- [x] 상태 모니터링

완료 기준:

- [x] GSI 생성 완료

### P2-004. problem meta 모델 구현

- [x] title
- [x] difficulty
- [x] tags
- [x] time/memory limit
- [x] statement_location
- [x] bundle_key
- [x] bundle_hash
- [x] checker_key
- [x] problem_version

완료 기준:

- [x] problem meta put/get 가능

### P2-005. manifest 모델 구현

- [x] manifest_version
- [x] bundle_format
- [x] bundle_size_bytes
- [x] case_count
- [x] checker_hash

완료 기준:

- [x] manifest put/get 가능

### P2-009. submission 추적 필드 확정

- [x] `attempt`
- [x] `worker_task_id`
- [x] `problem_version`
- [x] `manifest_version`
- [x] `bundle_hash`
- [x] `finalized_at`

완료 기준:

- [x] 결과 추적과 재채점에 필요한 필드가 모델에 반영됨

### P2-006. 문제 조회 API 구현

- [x] `GET /problems`
- [x] `GET /problems/{problemId}`

완료 기준:

- [x] 문제 메타 조회 가능

### P2-007. statement 조회 API 구현

- [x] `GET /problems/{problemId}/statement`
- [x] S3 markdown fetch 또는 presigned 반환 방식 결정

추천:

- API가 S3 markdown를 읽어 반환하는 방식으로 시작

이유:

- 현재 문서 셋 전체가 `statement는 S3` 기준으로 정렬되어 있다
- 프론트가 직접 S3 처리 로직을 알 필요가 없다
- 관리자 업로드 흐름과 사용자 조회 흐름이 단순해진다

완료 기준:

- [x] statement 렌더링 가능

### P2-008. 제출 조회 API 구현

- [x] `GET /submissions/{submissionId}`
- [x] `GET /users/me/submissions`

완료 기준:

- [x] 제출 상태와 목록 조회 가능

## Phase 3. 문제 자산 저장과 manifest

### P3-001. S3 bucket 생성

- [x] assets bucket
- [x] logs bucket

완료 기준:

- [x] bucket 생성 완료

### P3-002. 문제 자산 prefix 설계

- [x] statement 경로
- [x] sample 경로
- [x] bundle 경로
- [x] checker 경로

완료 기준:

- [x] 경로 규칙 문서화

### P3-003. statement 업로드 presign API

- [x] 관리자용 업로드 presign 발급

완료 기준:

- [~] statement를 S3로 업로드 가능

진행 메모:

- Phase 3 MVP의 즉시 노출용 문제 설명은 문제 등록 시 `statement_markdown`으로 저장한다.
- `/problems/{problem_id}/statement`는 `statement_markdown`을 우선 반환하고, 기존 데이터처럼 비어 있으면 S3 `statement_location`을 fallback으로 읽는다.
- S3 statement 업로드 UI는 Phase 5 후속 자산 업로드 흐름으로 남긴다.

### P3-004. sample 업로드 presign API

- [x] sample input/output presign 발급 가능

완료 기준:

- [~] sample 업로드 가능

### P3-005. hidden tests bundle 규격 정의

- [x] zip 또는 tar.zst 선택
- [x] bundle 내부 디렉터리 규칙 정의

추천:

- MVP는 `zip` 선택

이유:

- 구현이 가장 단순하다
- 로컬과 CI에서 다루기 쉽다
- 관리자 업로드/검증 도구를 빨리 만들 수 있다

완료 기준:

- [x] bundle 포맷 확정

### P3-006. manifest 스키마 확정

- [x] bundle_key
- [x] bundle_hash
- [x] case_count
- [x] weight
- [x] checker 정보

추천:

- 초기에는 관리자 입력 + 서버 검증 조합으로 시작

이유:

- 완전 자동 생성기를 먼저 만드는 것보다 빠르다
- hash, 필수 필드, 파일 존재 여부만 검증해도 초반 운영 리스크를 크게 줄일 수 있다
- 이후 필요해질 때 자동 생성기로 확장하기 쉽다

완료 기준:

- [x] manifest JSON 예시 확정

### P3-007. bundle finalize API 구현

- [x] 업로드 완료 후 manifest 저장
- [x] problem_version 증가

완료 기준:

- [x] 관리자 자산 등록 완료 처리 가능

2026-05-13 진행 메모:

- `admin-api`에 `GET /admin/problems`, `POST /admin/problems`, `GET /admin/problems/{problem_id}`,
  `POST /admin/problems/{problem_id}/assets/presign`,
  `POST /admin/problems/{problem_id}/bundle/finalize` 구현
- 문제 메타 등록은 개발기 smoke test 성공
- bundle presign/finalize는 로컬 테스트와 프론트 구현 완료, 개발기 화면 리허설 대기
- API Gateway 신규 조회 라우트와 admin Lambda `dynamodb:Scan` 권한 추가 필요

2026-05-14 진행 메모:

- `ProblemMeta`에 `statement_markdown`, `allowed_languages` 추가
- `POST /admin/problems`에서 문제 설명과 제출 가능 언어를 필수 검증
- 관리자 신규 등록 화면에 문제 설명 입력과 제출 가능 언어 체크박스 추가
- 관리자 목록/상세와 참가자 상세에서 제출 가능 언어 노출
- 참가자 제출 언어 select는 문제별 `allowed_languages` 기준으로 제한

## Phase 4. 제출 및 채점 파이프라인

### P4-001. 제출 생성 API 구현

- [ ] source code 입력 검증
- [ ] 요청 언어가 문제의 `allowed_languages`에 포함되는지 검증
- [ ] source를 S3 저장
- [ ] submission meta를 `queued`로 저장
- [ ] SQS 메시지 발행

추천:

- MVP는 API가 source code를 직접 받아 S3에 저장

이유:

- presigned upload보다 프론트와 상태 관리가 단순하다
- 소형 텍스트 제출물에는 충분히 적합하다
- 초반에는 분산된 업로드 플로우보다 일관된 API 플로우가 더 중요하다

완료 기준:

- [ ] 제출 생성 성공

### P4-002. SQS + DLQ 생성

- [ ] main queue
- [ ] dead-letter queue

완료 기준:

- [ ] 메시지 발행/수신 가능

### P4-003. submission-consumer 구현

- [ ] 메시지 검증
- [ ] ECS RunTask 호출

완료 기준:

- [ ] 큐 메시지로 worker 실행 가능

### P4-004. ECS cluster/ECR 준비

- [ ] cluster 생성
- [ ] ECR repository 생성
- [ ] task role 구성

완료 기준:

- [ ] worker 이미지 배포 가능

### P4-005. Rust/Python worker 초안 구현

- [ ] problem meta 조회
- [ ] manifest 조회
- [ ] source download
- [ ] bundle download
- [ ] `/tmp` 압축 해제
- [ ] Rust compile
- [ ] Python 실행 경로 구성
- [ ] testcase loop

추천:

- 첫 worker는 Rust + Python 두 언어를 함께 지원하도록 구현

이유:

- 온라인 저지 초기 사용자 기대 언어 범위를 맞추기 쉽다
- Rust는 컴파일 언어, Python은 인터프리터 언어라 서로 다른 실행 경로를 초기에 검증하기 좋다
- 이 둘을 먼저 지원하면 이후 C++나 Java 추가 시 추상화 방향을 더 잘 잡을 수 있다

완료 기준:

- [ ] Rust와 Python 각각 최소 1개 문제 end-to-end 채점 가능

### P4-006. 결과 저장 구현

- [ ] summary 저장
- [ ] testcase detail 저장
- [ ] logs S3 업로드

완료 기준:

- [ ] 결과 조회 API와 연결 가능

### P4-007. timeout/limit 제어 구현

- [ ] compile timeout
- [ ] testcase timeout
- [ ] total timeout
- [ ] output size limit

완료 기준:

- [ ] 무한루프/과다출력 방어 가능

### P4-008. 상태 전이 및 재처리 규칙 구현

- [ ] 허용 상태 전이 정의
- [ ] stuck submission 감시 기준 정의
- [ ] `system_error` 재처리 규칙 정의
- [ ] final state 자동 재처리 금지

완료 기준:

- [ ] 영구 `running` 또는 `dispatching` 상태가 남지 않음

### P4-009. structured log 및 DLQ 운영 절차 구현

- [ ] JSON 로그 필드 스키마 고정
- [ ] DLQ 진입 조건 정의
- [ ] 운영자 재큐잉 절차 문서화

완료 기준:

- [ ] 실패 제출의 원인 추적과 복구 경로 확인 가능

## Phase 5. 관리자 문제 등록

### P5-001. 관리자 권한 처리

- [x] Cognito group 또는 claim 기반

완료 기준:

- [x] 일반 사용자는 관리자 API 접근 불가

### P5-002. 관리자 문제 메타 등록 UI

- [x] 제목
- [x] 난이도
- [x] 제한 시간
- [x] 메모리 제한
- [x] 문제 설명
- [x] 제출 가능 언어
- [ ] 공개 여부

완료 기준:

- [x] 문제 메타 등록 가능

### P5-003. statement/sample 업로드 UI

- [ ] sample presigned 업로드
- [ ] sample 업로드 성공 확인
- [ ] S3 statement 업로드/교체 UI

진행 메모:

- MVP 문제 설명은 `statement_markdown`으로 등록/조회한다.
- S3 `statement.md` 업로드는 기존 데이터 fallback과 후속 고급 편집 경로로 유지한다.

### P5-006. 관리자 rejudge 경로 설계

- [ ] problem 단위 rejudge
- [ ] submission 범위 지정 rejudge
- [ ] 변경 사유 기록

완료 기준:

- [ ] bundle/checker/limits 변경 시 운영자 재채점 경로 존재

완료 기준:

- [ ] 문제 자산 업로드 가능

### P5-004. hidden tests bundle 등록 UI

- [x] bundle 업로드
- [x] manifest 입력 또는 자동 생성

추천:

- 초기에는 bundle 업로드 + manifest 입력 보조 방식 선택

이유:

- 완전 자동 manifest 생성 UI를 먼저 만드는 것보다 빠르다
- 업로드/검증/버전 증가 흐름을 먼저 안정화할 수 있다
- 관리자 도구는 초반에 편의성보다 성공률과 검증 가능성이 더 중요하다

완료 기준:

- [~] hidden tests 등록 가능

진행 메모:

- 관리자 목록 `/admin/problems`와 상세 `/admin/problems/{problem_id}` 화면 추가
- bundle ZIP 선택, browser-side SHA-256 계산, presigned S3 PUT, finalize 호출까지 UI 구현
- 개발기 완료 판단은 S3 CORS와 API Gateway 라우트 반영 후 실제 화면 리허설 성공 시점

### P5-005. 관리자 검증 루틴

- [ ] statement 존재 확인
- [ ] bundle 존재 확인
- [ ] manifest 일관성 확인

완료 기준:

- [ ] 불완전한 문제 공개 차단

## Phase 6. 운영 최소 구성

### P6-001. CloudWatch Logs retention 설정

- [ ] Lambda 7일
- [ ] worker 7일

완료 기준:

- [ ] 로그 무기한 누적 방지

### P6-002. S3 lifecycle 설정

- [ ] 제출 로그 삭제 정책
- [ ] 임시 산출물 삭제 정책

완료 기준:

- [ ] 저장소 비용 자동 통제

### P6-003. 핵심 알람 구성

- [ ] queue age
- [ ] Lambda error
- [ ] ECS task launch failure
- [ ] bundle download failure
- [ ] system_error 증가

완료 기준:

- [ ] 최소 장애 탐지 가능

### P6-004. 운영 체크리스트 작성

- [ ] stuck submission 대응
- [ ] DLQ 확인
- [ ] 비용 점검
- [ ] bundle 교체 절차
- [ ] manifest version 증가 절차
- [ ] problem_version 증가 절차
- [ ] 재채점 범위 판단 절차

완료 기준:

- [ ] 운영 문서 존재

### P6-005. 백업/복구 정책 정리

- [ ] DynamoDB PITR 사용 여부 결정
- [ ] S3 versioning 또는 보존 정책 결정
- [ ] 문제 자산 복구 절차 문서화
- [ ] manifest 복구 절차 문서화

추천:

- 메타데이터 복구를 우선하고, S3는 lifecycle + 필요 시 versioning으로 시작

이유:

- problem metadata와 manifest가 깨지면 서비스 운영 기준점이 무너진다
- 문제 자산은 S3에 있어 상대적으로 추적과 보존이 쉽다
- 개인 프로젝트 단계에서는 과한 백업 자동화보다 복구 절차 문서화가 더 중요하다

완료 기준:

- [ ] 문제 자산과 메타데이터 복구 경로가 문서화됨

## Phase 7. 리허설과 장애 주입 테스트

### P7-001. 무한루프 제출 검증

- [ ] `while(1)` 제출 실행
- [ ] total timeout 동작 확인
- [ ] 최종 상태와 로그 확인

완료 기준:

- [ ] 무한루프 제출이 정해진 시간 안에 종료됨

### P7-002. sleep 무한 대기 제출 검증

- [ ] `sleep(inf)` 또는 유사 제출 실행
- [ ] per-test timeout과 total timeout 확인

완료 기준:

- [ ] 장시간 대기 제출이 시스템을 붙잡지 않음

### P7-003. worker 중간 종료 복구 테스트

- [ ] 실행 중 task 강제 종료
- [ ] submission 상태 전이 확인
- [ ] 재시도 또는 system_error 처리 확인

완료 기준:

- [ ] worker 장애 시 제출이 영구적으로 멈추지 않음

### P7-004. bundle 교체 리허설

- [ ] hidden tests bundle 교체
- [ ] manifest version 증가
- [ ] problem_version 증가
- [ ] 재채점 절차 확인

추천:

- 리허설 우선순위 상위에 둔다

이유:

- 현재 구조에서 bundle/manifest/version 흐름은 가장 실수하기 쉬운 운영 포인트다
- 테스트케이스 변경 절차가 정리되지 않으면 결과 기준점이 쉽게 어긋난다

완료 기준:

- [ ] 테스트케이스 변경 절차를 재현 가능

### P7-005. statement 동시 조회 부하 테스트

- [ ] 다수 동시 요청으로 statement 조회
- [ ] 응답 시간과 오류율 확인

완료 기준:

- [ ] statement 배포 경로의 병목을 파악함

### P7-006. 제출 burst 테스트

- [ ] 짧은 시간에 여러 제출 생성
- [ ] SQS 적체와 worker launch 지연 확인

완료 기준:

- [ ] burst 시 queue와 worker 동작 특성을 파악함

### P7-007. 결과 일관성 검증

- [ ] 동일 제출 반복 실행
- [ ] borderline timeout 여부 확인
- [ ] 결과 흔들림 기록

완료 기준:

- [ ] 결정성 리스크가 문서화됨

### P7-008. 배포 전 전체 플로우 리허설

- [ ] 문제 등록
- [ ] statement/sample/bundle 업로드
- [ ] 제출 생성
- [ ] 채점 완료
- [ ] 결과 조회

완료 기준:

- [ ] 운영자가 전체 흐름을 한 번 끊김 없이 재현 가능

## 5. 첫 2주 집중 계획

### Week 1

- [ ] Phase 0 전체
- [x] P1-001
- [x] P1-002
- [x] P1-003
- [x] P1-004

목표:

- [x] 로그인 가능한 프론트 뼈대 완성

### Week 2

- [x] P2-001
- [x] P2-002
- [x] P2-003
- [x] P2-004
- [x] P2-005
- [x] P2-006
- [x] P2-007
- [x] P3-001
- [x] P3-002

목표:

- [x] 문제 메타/statement 조회 기반 완성

### Week 3 이후 우선순위

- [ ] P4-001 ~ P4-007
- [ ] P5-001 ~ P5-005
- [ ] P6-001 ~ P6-005
- [ ] P7-001 ~ P7-008

목표:

- [ ] 기능 구현 후 운영 리허설까지 완료

## 6. 최종 정리

이 계획의 핵심은 아래 네 흐름을 끝까지 완성하는 것이다.

- [ ] 문제 메타 등록
- [ ] 자산 업로드와 manifest 등록
- [ ] 제출 생성과 채점 실행
- [ ] polling 기반 결과 조회

그리고 IOI 체크리스트 관점에서 아래까지 검증되어야 한다.

- [ ] 실패 제출이 시스템을 붙잡지 않음
- [ ] 테스트케이스 변경 절차가 문서화됨
- [ ] worker 장애 시 복구 경로가 있음
- [ ] 기본 백업/복구 절차가 있음

이 수준까지 갖춰지면 이후 확장은 구조를 버리지 않고 추가할 수 있다.
