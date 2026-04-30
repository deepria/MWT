---
tags:
  - mwt
  - execution
  - phase-2
doc_type: phase-design
status: dev-api-verified
phase: 2
updated: 2026-04-30
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
---

# Phase 2 - API 및 데이터 모델 상세 설계

## 목적

DynamoDB core_table과 Rust Lambda API의 최소 뼈대를 구성해 문제 조회와 제출 조회의 기반을 만든다.

## 연결 문서

- [[mwt-architecture-v1.3]]
- [[mwt-problem-storage-and-staging-v1.1]]

## 포함 태스크

- [x] P2-001 Rust Lambda workspace 구성
- [x] P2-002 core_table 정의
- [x] P2-003 최소 GSI 정의
- [x] P2-004 problem meta 모델 구현
- [x] P2-005 manifest 모델 구현
- [x] P2-006 문제 조회 API 구현
- [x] P2-007 statement 조회 API 구현
- [x] P2-008 제출 조회 API 구현
- [x] P2-009 submission 추적 필드 확정

## 추천안

- `public-api`, `admin-api`, `submission-consumer` 3개로 시작
- `statement`는 S3 markdown를 API가 읽어서 반환

## 상세 설계

### 인증과 사용자 식별

- API Gateway/Lambda는 Cognito JWT를 검증한 요청만 보호 API로 통과시킨다.
- 사용자 내부 식별자는 Cognito ID token 또는 access token의 `sub` claim을 기준으로 한다.
- `email` claim은 화면 표시, 운영 조회, 감사 로그 보조 필드로만 사용한다.
- `profile` scope에서 오는 이름/닉네임 계열 claim은 MVP 데이터 모델의 필수값으로 두지 않는다.
- 관리자 권한은 `profile`이나 `email`이 아니라 `cognito:groups` claim 또는 별도 user role 엔티티로 판단한다.

### Cognito claim 사용 원칙

- `sub`: user_id 기준값
- `email`: 표시/검색/운영 보조값
- `cognito:groups`: 관리자 API 접근 여부 판단의 입력값
- `name`, `nickname`, `preferred_username`: 선택 표시값. 없다고 API가 실패하면 안 된다

### 보호 API 기준

- 공개 API는 문제 목록/상세 조회처럼 인증 없이 공개 가능한 범위로 제한한다.
- 제출 생성, 내 제출 목록, 관리자 API는 인증 필수로 둔다.
- 관리자 API는 `cognito:groups`에 `admin`이 있거나, 백엔드 user role이 admin인 경우에만 허용한다.
- 프론트의 관리자 메뉴 노출 여부는 UX 보조일 뿐이며, 백엔드 권한 검증을 대체하지 않는다.

### core_table 엔티티

- problem meta
- problem manifest
- submission meta
- submission result summary
- submission result detail

### 최소 GSI

- user submissions
- submissions by status

### API 범위

- `GET /problems`
- `GET /problems/{problemId}`
- `GET /problems/{problemId}/statement`
- `GET /submissions/{submissionId}`
- `GET /users/me/submissions`

### submission meta 추적 필드

- `status`
- `attempt`
- `worker_task_id`
- `problem_version`
- `manifest_version`
- `bundle_hash`
- `finalized_at`

원칙:

- 조회 API와 운영 도구가 같은 식별자와 버전 필드를 본다
- 재처리와 rejudge를 위해 최종 판정 당시의 bundle/version 정보를 남긴다

## 산출물

- [x] Rust workspace
- [x] DynamoDB 테이블/GSI 정의
- [x] 문제/제출 조회 API
- [x] submission 추적 필드 스키마

## 완료 기준

- [x] 문제 메타와 statement 조회 가능
- [x] 제출 상태와 목록 조회 가능
- [x] 재처리와 운영 추적에 필요한 submission 필드가 확정됨

## 작업 내용 요약

### 초기 구현

- `backend/`에 Rust Cargo workspace를 생성했다.
- 공통 crate로 `mwt-domain`, `mwt-infra`를 추가했다.
- Lambda entrypoint로 `public-api`, `admin-api`, `submission-consumer`를 추가했다.
- `ProblemMeta`, `ProblemManifest`, `SubmissionMeta`, `SubmissionResultSummary`, `SubmissionDetail` 모델을 구현했다.
- Cognito claim 기반 `AuthContext` 모델을 추가했다.
- repository trait로 problem, statement, submission 조회 경계를 정의했다.
- local 검증용 `MemoryRepository`를 추가했다.
- DynamoDB item 변환 레이어를 추가해 domain model과 single-table item shape 간 roundtrip을 검증했다.
- S3 statement location parser를 추가했다.
- `public-api`에 아래 조회 API를 구현했다.
  - `GET /problems`
  - `GET /problems/{problemId}`
  - `GET /problems/{problemId}/statement`
  - `GET /submissions/{submissionId}`
  - `GET /users/me/submissions`
- `infra/dynamodb/core-table.yaml`에 `<CORE_TABLE_NAME>`와 최소 GSI 2개를 CloudFormation으로 정의했다.
- `infra/dynamodb/README.md`에 single-table key 설계와 GSI 사용 목적을 문서화했다.

### 2026-04-30 추가 구현

- `mwt-infra`에 AWS SDK 기반 `AwsRepository`를 추가했다.
- DynamoDB SDK `AttributeValue`와 기존 JSON 기반 single-table item 변환 레이어를 연결했다.
- `ProblemRepository` 구현:
  - `GET /problems`용 공개 문제 목록 조회는 MVP 기준 `problem_meta`와 `visibility = public` scan으로 시작한다.
  - `GET /problems/{problemId}`는 `pk = PROBLEM#{problemId}`, `sk = META`로 조회한다.
  - manifest 조회는 `sk = MANIFEST#{manifestVersion}` 형식으로 조회한다.
- `SubmissionRepository` 구현:
  - `GET /submissions/{submissionId}`는 submission meta와 result summary를 조합해 반환한다.
  - `GET /users/me/submissions`는 `gsi1-user-submissions`를 사용한다.
- `StatementRepository` 구현:
  - `statement_location`의 `s3://bucket/key` 값을 파싱한다.
  - S3 `GetObject`로 markdown statement 본문을 읽어 반환한다.
- `public-api` Lambda entrypoint를 `MemoryRepository`에서 `AwsRepository`로 교체했다.
- `MWT_CORE_TABLE_NAME` 환경변수로 core table 이름을 주입할 수 있게 했다.
- API Gateway HTTP API JWT authorizer의 `authorizer.jwt.claims`에서 Cognito claim을 읽도록 연결했다.
  - `sub`
  - `email`
  - `cognito:groups`
- 프론트엔드에 `apiClient`를 추가하고 실제 API Gateway URL을 `VITE_API_BASE_URL`로 연결했다.
- 프론트 문제 목록, 문제 상세, statement, 제출 상세 화면을 mock data 대신 API 호출 기반으로 전환했다.
- Cognito Hosted UI token을 보호 API 요청의 `Authorization: Bearer ...` 헤더로 붙이도록 연결했다.

### AWS 콘솔에서 준비한 리소스

- Lambda 환경변수 `MWT_CORE_TABLE_NAME=<CORE_TABLE_NAME>` 추가 완료.
- Lambda 실행 역할에 DynamoDB read 권한과 S3 statement read 권한 추가 완료.
- API Gateway HTTP API 생성 완료.
  - URL: `<API_GATEWAY_BASE_URL>`
- Cognito JWT Authorizer 추가 완료.
- 보호 라우트에 authorizer 연결 완료.
  - `GET /users/me/submissions`
  - `GET /submissions/{submissionId}`
- API Gateway 배포 완료.
- CORS 설정 완료.
  - 로컬 개발 origin: `http://127.0.0.1:5173`
  - 허용 method/header는 프론트 호출 기준으로 설정했다.

### 2026-04-30 개발기 배포 및 디버깅

- Lambda가 ARM64 아키텍처임을 확인하고 `backend/README.md`에 ARM64 Lambda 빌드 절차를 정리했다.
- `cargo-zigbuild` 기반으로 Linux ARM64용 `public-api` 바이너리를 빌드했다.
  - target: `aarch64-unknown-linux-gnu`
  - runtime: `provided.al2023`
  - handler binary name: `bootstrap`
- Lambda 업로드용 zip을 생성했다.
  - `backend/target/lambda/public-api-arm64.zip`
- 개발기 Lambda에 ARM64 zip을 수동 업로드했다.
- 배포 직후 공개 API를 확인했다.
  - `GET /problems`가 `200 []`를 반환했다.
  - `GET /problems/sum-path`는 DynamoDB item 추가 전 `404 problem not found`를 반환했다.
- DynamoDB `<CORE_TABLE_NAME>`에 `sum-path` 문제 meta item을 추가했다.
  - `pk = PROBLEM#sum-path`
  - `sk = META`
  - `entity_type = problem_meta`
  - `visibility = public`
- S3에 statement markdown을 추가했다.
  - bucket: `<ASSETS_BUCKET_NAME>`
  - key: `problems/sum-path/statement.md`
- statement 조회가 처음에는 `500 Internal Server Error`를 반환했다.
- CloudWatch 로그 개선을 위해 AWS SDK error를 `Debug` 형식으로 남기도록 수정하고 Lambda zip을 재빌드했다.
- CloudWatch에서 실제 원인을 확인했다.
  - S3 error: `NoSuchBucket`
  - 기존 `statement_location` bucket: `<ASSETS_BUCKET_NAME>`
  - 실제 bucket: `<ASSETS_BUCKET_NAME>`
- DynamoDB `statement_location`과 Lambda IAM policy의 S3 resource를 실제 bucket 이름으로 수정했다.
  - `<STATEMENT_S3_URI>`
  - `<ASSETS_BUCKET_OBJECTS_ARN>`
- 수정 후 공개 문제 조회와 statement 조회가 모두 정상 동작함을 확인했다.

### 2026-04-30 Lambda CI/CD 구성

- 수동 Lambda zip 배포를 GitHub Actions 기반 CI/CD로 옮기기 위한 파일을 추가했다.
- `.github/workflows/deploy-public-api-lambda.yml`을 추가했다.
  - trigger: manual `workflow_dispatch`
  - backend formatting check
  - backend test
  - ARM64 Lambda package build
  - Lambda zip artifact upload
  - AWS OIDC role assume
  - `aws lambda update-function-code` 배포
- `backend/scripts/package-public-api.sh`를 추가해 로컬과 CI가 같은 패키징 절차를 쓰도록 했다.
- `infra/iam/github-actions-public-api-deploy-policy.json`에 GitHub Actions 배포 role 권한 정책을 추가했다.
- `infra/iam/README.md`에 GitHub Actions OIDC role 설정 절차와 trust policy template을 정리했다.
- GitHub Actions 설정값:
  - required secret: `AWS_GITHUB_ACTIONS_ROLE_ARN`
  - optional variable: `PUBLIC_API_LAMBDA_FUNCTION_NAME`
- 첫 repository push 전에 의도치 않은 배포 실패를 막기 위해 workflow는 우선 수동 실행 전용으로 둔다.
- OIDC role과 GitHub secret 등록 후 수동 배포가 성공하면 `backend/**` path 기반 `main` push trigger를 다시 켠다.

## 검증

### 로컬 검증

- `cargo fmt --all --check` 통과
- `cargo test` 통과
- `cargo build` 통과
- `npm run build` 통과
- `npm run lint -- --fix` 통과
- 로컬 Vite dev server 실행 확인.
  - `http://127.0.0.1:5173/`
- `backend/scripts/package-public-api.sh` 로컬 실행 통과.
  - `backend/target/lambda/public-api-arm64.zip` 생성 확인.
- GitHub Actions workflow YAML parse 확인.

### 개발기 AWS 검증

- API Gateway CORS preflight 확인.
  - `OPTIONS /problems`가 `204`를 반환했다.
- Lambda ARM64 zip 배포 후 API Gateway 실제 라우트 확인.
  - `GET /problems`가 `200 OK`를 반환한다.
  - `GET /problems/sum-path`가 `200 OK`를 반환한다.
  - `GET /problems/sum-path/statement`가 `200 OK`를 반환한다.
- 확인된 statement 응답:
  - `problem_id = sum-path`
  - `format = markdown`
  - `content = "# 합 경로\n\n정수 배열의 구간 합을 빠르게 구하라.\n"`

## AWS 준비 완료

- DynamoDB table: `<CORE_TABLE_NAME>`
- S3 assets bucket: `<ASSETS_BUCKET_NAME>`
- Lambda assume role trust policy 구성 완료
- Lambda IAM policy의 S3 resource를 실제 bucket인 `<ASSETS_BUCKET_OBJECTS_ARN>`로 수정 완료
- API Gateway HTTP API: `<API_GATEWAY_BASE_URL>`
- 개발기 `public-api` Lambda ARM64 수동 배포 완료

## 개발기 배포 전제

- 현재 Phase 2 코드는 로컬 구현, AWS 콘솔 리소스 준비, 개발기 `public-api` Lambda 수동 배포까지 완료된 상태다.
- 아직 Amplify 개발기 프론트도 배포하지 않았다.
- API Gateway를 통한 공개 문제 조회 API는 개발기에서 정상 검증됐다.
- 보호 API는 Cognito 로그인과 submission sample data가 필요하므로 Amplify 개발기 배포 후 검증한다.

## 남은 작업

- [x] 개발기 Lambda에 최신 `public-api` 바이너리 배포
- [x] Lambda CI/CD workflow 구성
- [ ] GitHub Actions용 AWS OIDC IAM role 생성
- [ ] GitHub secret `AWS_GITHUB_ACTIONS_ROLE_ARN` 등록
- [ ] GitHub Actions에서 `Deploy public-api Lambda` workflow 1회 수동 실행
- [ ] Amplify 개발기 프론트 배포
- [ ] Amplify 환경변수 구성
  - `VITE_API_BASE_URL`
  - `VITE_AUTH_PROVIDER=cognito`
  - `VITE_COGNITO_CLIENT_ID`
  - `VITE_COGNITO_DOMAIN`
  - `VITE_COGNITO_REDIRECT_SIGN_IN`
  - `VITE_COGNITO_REDIRECT_SIGN_OUT`
- [ ] Cognito App client callback/logout URL에 Amplify 개발기 URL 추가
- [ ] API Gateway CORS origin에 Amplify 개발기 URL 추가
- [ ] 개발기 end-to-end 검증
  - [x] `GET /problems`
  - [x] `GET /problems/{problemId}`
  - [x] `GET /problems/{problemId}/statement`
  - [ ] `GET /users/me/submissions`
  - [ ] `GET /submissions/{submissionId}`
