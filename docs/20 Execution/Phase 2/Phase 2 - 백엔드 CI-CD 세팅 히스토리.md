---
tags:
  - mwt
  - execution
  - phase-2
  - cicd
  - backend
doc_type: setup-history
status: completed
phase: 2
updated: 2026-05-13
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
related:
  - "[[Phase 2 - API 및 데이터 모델 상세 설계]]"
---

# Phase 2 - 백엔드 CI/CD 세팅 히스토리

## 목적

`public-api` Rust Lambda를 수동 zip 업로드에서 GitHub Actions 기반 반복 배포로 옮긴 과정을 기록한다.

## 최종 상태

- GitHub Actions workflow: `.github/workflows/deploy-public-api-lambda.yml`
- 실행 방식: 수동 `workflow_dispatch`, `main` backend 변경 push 자동 배포
- 대상 브랜치: `main`
- 대상 Lambda: `mwt-public-api`
- AWS region: `ap-northeast-2`
- 인증 방식: GitHub Actions OIDC role assume
- 필수 GitHub secret: `AWS_GITHUB_ACTIONS_ROLE_ARN`
- 선택 GitHub variable: `PUBLIC_API_LAMBDA_FUNCTION_NAME`
- 패키지 산출물: `backend/target/lambda/public-api-arm64.zip`

## 배경

Phase 2 초반에는 로컬에서 ARM64 Lambda zip을 빌드한 뒤 AWS 콘솔에서 직접 업로드했다. 이 방식은 개발기 확인에는 충분했지만, 아래 문제가 있었다.

- 배포 절차가 로컬 환경에 묶인다.
- zip 생성 절차가 사람마다 달라질 수 있다.
- Lambda 코드 업데이트 이력이 GitHub workflow에 남지 않는다.
- Phase 3 이후 API 변경이 잦아지면 수동 업로드가 병목이 된다.

그래서 Phase 2 마무리 시점에 `public-api`만 먼저 CI/CD로 옮겼다. `admin-api`, `submission-consumer`는 아직 실질 구현 전이므로 별도 workflow로 분리하지 않았다.

## 구현 순서

### 1. 로컬 패키징 스크립트 정리

`backend/scripts/package-public-api.sh`를 추가해 로컬과 CI가 같은 명령으로 Lambda zip을 만들도록 했다.

주요 동작:

- Rust target: `aarch64-unknown-linux-gnu`
- build command: `cargo zigbuild --release --target aarch64-unknown-linux-gnu -p mwt-public-api`
- Lambda custom runtime executable name: `bootstrap`
- zip path: `backend/target/lambda/public-api-arm64.zip`

이 스크립트는 Lambda 런타임 설정과 맞춘다.

- Runtime: `provided.al2023`
- Architecture: `arm64`
- Handler: `bootstrap`

### 2. GitHub Actions workflow 추가

`.github/workflows/deploy-public-api-lambda.yml`을 추가했다.

Workflow 단계:

1. repository checkout
2. Rust stable toolchain 설치
3. `aarch64-unknown-linux-gnu` target 추가
4. Zig 설치
5. Cargo cache 설정
6. `cargo-zigbuild` 설치
7. `cargo fmt --all --check`
8. `cargo test`
9. `backend/scripts/package-public-api.sh`
10. zip artifact upload
11. AWS OIDC credential 설정
12. `aws lambda update-function-code`
13. `aws lambda wait function-updated`
14. Lambda configuration summary 출력

처음에는 자동 배포를 켜지 않고 수동 실행만 허용했다.

```yaml
on:
  workflow_dispatch:
```

이유:

- 첫 OIDC role 설정 실패가 `main` push 배포 실패로 보이지 않게 한다.
- Lambda 함수명, IAM 권한, ARM64 패키징이 한 번에 맞는지 먼저 검증한다.
- Phase 2 종료 전에는 의도한 시점에만 개발기 Lambda를 바꾸는 편이 안전하다.

첫 수동 배포가 성공한 뒤에는 `main` push 자동 배포 trigger를 추가했다.

```yaml
on:
  workflow_dispatch:
  push:
    branches:
      - main
    paths:
      - "backend/**"
      - ".github/workflows/deploy-public-api-lambda.yml"
```

자동 배포는 backend 코드나 workflow가 바뀔 때만 실행된다. 문서, 프론트, 인프라 문서만 바뀐 push는 Lambda 배포를 일으키지 않는다.

### 3. GitHub Actions OIDC IAM role 준비

AWS IAM에 GitHub Actions용 OIDC provider와 role을 만들었다.

OIDC provider:

- Provider URL: `https://token.actions.githubusercontent.com`
- Audience: `sts.amazonaws.com`

Role trust policy 조건:

- repository: `deepria/MWT`
- branch: `main`
- action: `sts:AssumeRoleWithWebIdentity`

실제 role ARN은 GitHub secret에만 저장한다.

```text
AWS_GITHUB_ACTIONS_ROLE_ARN
```

### 4. Lambda deploy permission policy 추가

`infra/iam/github-actions-public-api-deploy-policy.json`을 추가했다.

허용 action:

- `lambda:GetFunctionConfiguration`
- `lambda:UpdateFunctionCode`

Resource는 `ap-northeast-2`의 `mwt-public-api` Lambda로 제한한다. 실제 account id는 정책 파일에 반영했다.

### 5. GitHub secret 등록

GitHub repository settings에서 Actions secret을 추가했다.

```text
AWS_GITHUB_ACTIONS_ROLE_ARN
```

선택 variable은 기본값이 있으므로 이번 단계에서는 필수가 아니다.

```text
PUBLIC_API_LAMBDA_FUNCTION_NAME=mwt-public-api
```

### 6. 첫 수동 배포 실행

GitHub Actions에서 `Deploy public-api Lambda` workflow를 `main` 기준으로 수동 실행했다.

확인된 첫 실행 결과:

- status: `completed`
- conclusion: `success`
- total duration: 약 7분 11초
- ARM64 package build step: 약 4분
- AWS credential configure step: success
- Lambda code deploy step: success

첫 실행이 길었던 주된 이유는 Rust/Zig 기반 cross compile과 Cargo cache warm-up이다.

## 검증 결과

배포 후 확인한 항목:

- `Deploy public-api Lambda` workflow 수동 실행 성공
- `aws lambda update-function-code` 성공
- `aws lambda wait function-updated` 성공
- 개발기 API Gateway 라우트 정상 응답
- Amplify origin 기준 API Gateway CORS 정상 응답

확인된 API:

- `GET /problems`
- `GET /problems/{problemId}`
- `GET /problems/{problemId}/statement`
- `GET /users/me/submissions`
- `GET /submissions/{submissionId}`

## 관찰된 경고

GitHub Actions 실행 후 아래 경고가 출력됐다.

- Node.js 20 actions deprecation warning
- 대상 action:
  - `actions/checkout@v4`
  - `actions/upload-artifact@v4`
  - `goto-bus-stop/setup-zig@v2`

이 경고는 배포 실패가 아니며 workflow는 성공했다. 추후 대응 선택지는 아래와 같다.

- 각 action의 Node.js 24 대응 버전으로 업데이트한다.
- workflow env에 `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true`를 추가해 미리 Node.js 24로 실행해 본다.

## 현재 보류한 일

- Lambda alias 또는 stage별 배포 전략

아래 항목은 구현 대상 Phase의 태스크로 이관했다.

- `submission-consumer` Lambda 배포 workflow 추가: [[Phase 4 - 제출 및 채점 파이프라인 상세 설계]]의 P4-010
- `admin-api` Lambda 배포 workflow 추가: [[Phase 5 - 관리자 문제 등록 상세 설계]]의 P5-007

## Phase 2 결론

Phase 2 기준 백엔드 CI/CD 목표는 완료됐다. 현재 `public-api`는 GitHub Actions에서 테스트, ARM64 패키징, Lambda 코드 업데이트까지 한 번에 수행할 수 있다.
