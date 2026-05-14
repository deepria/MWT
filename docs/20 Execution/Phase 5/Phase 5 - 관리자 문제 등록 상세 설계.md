---
tags:
  - mwt
  - execution
  - phase-5
doc_type: phase-design
status: active
phase: 5
updated: 2026-05-14
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
---

# Phase 5 - 관리자 문제 등록 상세 설계

## 목적

운영자가 문제 메타와 자산을 등록하고 검증할 수 있는 최소 관리자 플로우를 구현한다.

## 연결 문서

- [[mwt-problem-storage-and-staging-v1.1]]
- [[mwt-architecture-v1.3]]

## 포함 태스크

- [x] P5-001 관리자 권한 처리
- [x] P5-002 관리자 문제 메타 등록 UI
- [ ] P5-003 statement/sample 업로드 UI
- [~] P5-004 hidden tests bundle 등록 UI
- [ ] P5-005 관리자 검증 루틴
- [ ] P5-006 관리자 rejudge 경로 설계
- [x] P5-007 admin-api Lambda CI/CD 구성

## 추천안

- 메타 등록 + 자산 업로드 + 검증 결과 표시까지만 구현

## 상세 설계

### Phase 3 선구현 항목

Phase 3 리허설을 위해 관리자 문제 메타 등록, 관리자 목록/상세 조회,
bundle presign/finalize UI를 먼저 구현했다.
Phase 5에서는 statement/sample 업로드, 검증 루틴, 공개 전환 정책을 완성한다.

2026-05-13 진행 결과:

- `/admin/problems`: draft 포함 관리자 문제 목록 화면 추가
- `/admin/problems/new`: 문제 메타만 등록하는 화면으로 정리
- `/admin/problems/{problem_id}`: 문제 상태 확인과 bundle ZIP 업로드/finalize 화면 추가
- `POST /admin/problems`: 메타 등록 API 구현 및 개발기 smoke test 성공
- `GET /admin/problems`, `GET /admin/problems/{problem_id}`: 관리자 조회 API 구현
- `POST /admin/problems/{problem_id}/assets/presign`: 업로드 시점 presigned URL 발급
- `POST /admin/problems/{problem_id}/bundle/finalize`: S3 object size 확인, manifest 검증, DynamoDB 저장
- `deploy-admin-api-lambda.yml`과 `package-admin-api.sh`로 admin-api 배포 경로 분리
- 로컬 검증: backend tests, frontend lint/build, admin-api package 통과

2026-05-14 보강 결과:

- 문제 등록 시 문제 설명 본문을 `statement_markdown`으로 함께 저장한다.
- 문제 등록 시 `allowed_languages`로 제출 가능 언어를 제한한다.
- 문제 등록 시 참가자 노출 예제 입출력을 `sample_cases`로 함께 저장한다.
- 관리자 신규 등록 화면에 문제 설명 textarea와 제출 가능 언어 체크박스를 추가했다.
- 관리자 신규 등록 화면에 예제 입력/출력 editor를 추가했다.
- 관리자 목록/상세와 참가자 문제 상세에서 제출 가능 언어를 노출한다.
- 참가자 제출 패널의 언어 선택지는 문제별 `allowed_languages`만 사용한다.
- 참가자 문제 상세의 예제 영역은 `sample_cases`를 사용한다.
- S3 `statement.md` 업로드는 후속 statement 업로드 UI 범위로 남기고,
  MVP의 즉시 조회 가능한 설명 본문은 DynamoDB 문제 메타에 저장한다.
- S3 sample 업로드는 후속 파일 기반 sample 교체 UI 범위로 남기고,
  MVP의 즉시 조회 가능한 예제 입출력은 DynamoDB 문제 메타에 저장한다.
- finalized bundle, 문제 설명, 예제, 제출 가능 언어가 모두 준비된 문제를 `public`으로 전환하는 관리자 API/UI를 추가했다.

남은 개발기 작업:

- 문제 설명/제출 언어/예제 보강분을 `admin-api`, `public-api`, frontend에 배포
- 실제 화면에서 신규 문제 등록, 공개 전환, 목록/상세 조회, 참가자 상세 조회를 리허설
- 실제 화면에서 bundle finalize 리허설

### 관리자 권한

- Cognito group 기반
- `cognito:groups` claim에 `admin`이 포함된 사용자만 관리자 API 접근 가능
- mock 개발 모드에서는 `x-mwt-groups: admin` header로 동일한 흐름 검증

### 등록 플로우

1. `/admin/problems/new`에서 메타, 문제 설명, 제출 가능 언어, 예제 입출력을 먼저 등록
2. `/admin/problems`에서 draft 포함 관리자 문제 목록 확인
3. `/admin/problems/{problem_id}` 상세 화면에서 업로드 시점에 presigned URL 발급
4. bundle 업로드 후 manifest cases와 hash/size로 finalize
5. 공개 가능 조건을 만족하면 `/admin/problems/{problem_id}/visibility`로 `public` 전환
6. sample 업로드와 공개 가능 여부 판정 고도화
7. S3 statement 업로드 UI는 후속 고급 편집/자산 업로드 흐름에서 제공

현재 구현된 관리자 화면:

- 관리자 문제 목록: 제목, 난이도, 제한, 제출 가능 언어, visibility, manifest/bundle 상태 표시
- 새 문제 등록: `problem_id`, 제목, 난이도, 시간 제한, 메모리 제한, 태그, 문제 설명, 제출 가능 언어, 예제 입력/출력 입력
- 관리자 상세: 문제 설명, 제출 가능 언어, 예제 입출력, statement 위치, bundle/hash 상태 표시, bundle ZIP 선택, case path/weight 입력, 업로드 후 finalize, 공개 전환

아직 미구현:

- S3 sample 업로드/교체 UI
- S3 statement 업로드/교체 UI
- checker 업로드 UI
- rejudge 요청 UI

### rejudge 운영 플로우

1. bundle, checker, limits 변경 사유 기록
2. 영향받는 submission 범위 선택
3. rejudge 요청 생성
4. 재채점 진행 상황 확인
5. 결과 반영 및 운영 메모 기록

## 산출물

- 관리자 화면
- 업로드 플로우
- 검증 결과 표시
- rejudge 운영 절차
- admin-api Lambda 배포 workflow

### P5-007 admin-api Lambda CI/CD 구성

- `.github/workflows/deploy-admin-api-lambda.yml` 또는 공통 reusable Lambda deploy workflow를 추가한다.
- `admin-api`를 Lambda 런타임/아키텍처에 맞게 패키징한다.
- GitHub Actions OIDC role 권한에 `admin-api` Lambda code update 권한을 추가한다.
- 첫 배포는 수동 실행으로 검증하고, 이후 `backend/**` 변경 기준 자동 배포 여부를 결정한다.

완료 기준:

- GitHub Actions에서 `admin-api` Lambda를 빌드, 테스트, 패키징, 배포할 수 있다.
- 배포 후 관리자 권한이 없는 사용자는 접근이 거부되고, admin group 사용자는 관리자 API를 호출할 수 있다.

## 완료 기준

- 불완전한 문제는 공개할 수 없고, 완전한 문제는 등록 가능하다
- bundle/checker/limits 변경 후 운영자가 재채점 경로를 사용할 수 있다
- `admin-api` 변경을 재현 가능한 CI/CD 경로로 개발기 Lambda에 배포할 수 있다
