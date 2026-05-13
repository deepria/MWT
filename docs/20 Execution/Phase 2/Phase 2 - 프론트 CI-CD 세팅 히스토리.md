---
tags:
  - mwt
  - execution
  - phase-2
  - cicd
  - frontend
doc_type: setup-history
status: completed
phase: 2
updated: 2026-05-13
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
related:
  - "[[Phase 2 - API 및 데이터 모델 상세 설계]]"
---

# Phase 2 - 프론트 CI/CD 세팅 히스토리

## 목적

`frontend/` Vue 앱을 AWS Amplify Hosting 개발기 환경에 연결하고, Cognito Hosted UI와 API Gateway CORS까지 개발기 end-to-end로 동작하게 만든 과정을 기록한다.

## 최종 상태

- Hosting: AWS Amplify Hosting
- Repository: `deepria/MWT`
- Branch: `main`
- Monorepo app root: `frontend`
- Amplify app name: `MWT`
- 개발기 URL: `https://main.dt3yir3oksp8j.amplifyapp.com`
- Framework detection: 없음
- SSR: disabled
- Build command: `npm run build`
- Frontend env provider: Amplify environment variables

## 배경

Phase 1에서는 로컬 Vite dev server와 mock auth로 프론트 흐름을 검증했다. Phase 2에서 실제 API Gateway, Cognito Hosted UI, 보호 API 검증이 필요해지면서 개발기 정적 호스팅이 필요해졌다.

Amplify Hosting을 선택한 이유:

- GitHub branch와 바로 연결할 수 있다.
- Vite static build 산출물 배포에 충분하다.
- Cognito callback URL과 API Gateway CORS origin을 안정적인 HTTPS origin으로 고정할 수 있다.
- Phase 2 기준으로 별도 CDN/S3 수동 배포보다 빠르다.

## 구현 순서

### 1. Amplify app 생성

AWS Amplify에서 GitHub repository를 연결했다.

설정:

- Repository service: GitHub
- Repository: `deepria/MWT`
- Branch: `main`
- Monorepo app root: `frontend`
- App name: `MWT`
- Build instance type: standard
- SSR deployment: disabled

Monorepo이므로 `frontend`를 app root로 지정했다. 이 설정이 빠지면 repository root에서 build가 실행되어 Vite app을 찾지 못할 수 있다.

### 2. Build 설정

Amplify가 사용하는 프론트 build command는 아래다.

```text
npm run build
```

`frontend/package.json`의 build script:

```json
{
  "build": "vue-tsc -b && vite build"
}
```

Vite build 산출물은 `frontend/dist`다. Amplify app root가 `frontend`이므로 artifact base directory는 `dist` 기준으로 동작한다.

### 3. Amplify 환경변수 설정

Amplify 환경변수에 프론트 빌드 타임 값을 추가했다.

```text
AMPLIFY_MONOREPO_APP_ROOT=frontend
AMPLIFY_DIFF_DEPLOY=false
VITE_API_BASE_URL=https://abcdef1234.execute-api.ap-northeast-2.amazonaws.com
VITE_AUTH_PROVIDER=cognito
VITE_COGNITO_CLIENT_ID=<COGNITO_APP_CLIENT_ID>
VITE_COGNITO_DOMAIN=https://mwt-example.auth.ap-northeast-2.amazoncognito.com
VITE_COGNITO_SCOPES=openid email profile
VITE_COGNITO_REDIRECT_SIGN_IN=https://main.dt3yir3oksp8j.amplifyapp.com/auth/callback
VITE_COGNITO_REDIRECT_SIGN_OUT=https://main.dt3yir3oksp8j.amplifyapp.com/login
```

주의:

- `VITE_COGNITO_CLIENT_ID`는 실제 값이므로 문서에는 placeholder로 남긴다.
- `VITE_COGNITO_DOMAIN`은 Cognito issuer나 JWKS URL이 아니라 Hosted UI domain이다.
- Vite 환경변수는 build time에 번들에 포함되므로 값을 바꾼 뒤 Amplify redeploy가 필요하다.

### 4. Cognito App Client URL 추가

Amplify 개발기 URL이 나온 뒤 Cognito App Client 설정에 callback/logout URL을 추가했다.

Allowed callback URL:

```text
https://main.dt3yir3oksp8j.amplifyapp.com/auth/callback
```

Allowed sign-out URL:

```text
https://main.dt3yir3oksp8j.amplifyapp.com/login
```

로컬 개발용 URL은 유지한다.

```text
http://127.0.0.1:5173/auth/callback
http://127.0.0.1:5173/login
```

### 5. API Gateway CORS origin 추가

API Gateway HTTP API CORS 설정에 Amplify origin을 추가했다.

Allowed origin:

```text
https://main.dt3yir3oksp8j.amplifyapp.com
```

로컬 개발 origin도 유지한다.

```text
http://127.0.0.1:5173
```

필요 header:

```text
authorization
content-type
```

필요 method:

```text
GET
OPTIONS
```

### 6. Redeploy

Amplify 환경변수와 Cognito redirect 설정을 맞춘 뒤 Amplify를 재배포했다. 재배포 후 Vite bundle에 Cognito와 API Gateway 값이 반영됐다.

## 검증 결과

### Amplify 정적 호스팅

Amplify 개발기 URL이 `200 OK`를 반환했다.

```text
https://main.dt3yir3oksp8j.amplifyapp.com
```

### 공개 API

Amplify origin에서 API Gateway 공개 API 호출이 성공했다.

```text
GET /problems
```

확인된 응답:

- status: `200`
- `access-control-allow-origin: https://main.dt3yir3oksp8j.amplifyapp.com`
- body에 `sum-path` 문제 포함

### CORS preflight

보호 API preflight가 성공했다.

```text
OPTIONS /users/me/submissions
```

확인된 응답:

- status: `204`

### 화면 확인

Amplify 개발기에서 아래 흐름을 확인했다.

- 문제 목록 화면 진입
- 문제 상세 화면 진입
- statement 조회
- Cognito 로그인 왕복
- 보호 API `200` 응답

### API 확인

Phase 2 end-to-end 기준으로 아래 API가 개발기에서 동작함을 확인했다.

- `GET /problems`
- `GET /problems/{problemId}`
- `GET /problems/{problemId}/statement`
- `GET /users/me/submissions`
- `GET /submissions/{submissionId}`

## 헷갈렸던 값

### JWKS URL

아래 값은 `VITE_COGNITO_DOMAIN`이 아니다.

```text
https://cognito-idp.ap-northeast-2.amazonaws.com/<USER_POOL_ID>/.well-known/jwks.json
```

이 값은 JWT 서명 검증용 공개키 URL이다.

### Issuer URL

아래 값도 `VITE_COGNITO_DOMAIN`이 아니다.

```text
https://cognito-idp.ap-northeast-2.amazonaws.com/<USER_POOL_ID>
```

이 값은 API Gateway JWT authorizer issuer에 해당한다.

### Hosted UI domain

프론트에서 사용하는 값은 Hosted UI domain이다.

```text
https://mwt-example.auth.ap-northeast-2.amazoncognito.com
```

코드는 이 domain에 `/oauth2/authorize`, `/oauth2/token`, `/logout`을 붙여 사용한다.

## 현재 보류한 일

- Amplify custom domain 연결
- branch preview 환경
- 자동 diff deploy 최적화
- 프론트 전용 GitHub Actions lint/build check
- Amplify 환경변수의 stage 분리

현재는 Phase 2 개발기 검증을 우선했으므로 `main` branch 기반 단일 개발기 호스팅으로 충분하다.

## Phase 2 결론

Phase 2 기준 프론트 CI/CD와 개발기 호스팅 목표는 완료됐다. Amplify 개발기 URL에서 Vue 앱이 배포되고, Cognito Hosted UI와 API Gateway가 같은 개발기 origin 기준으로 연결된다.
