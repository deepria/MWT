---
tags:
  - mwt
  - execution
  - phase-1
doc_type: phase-design
status: active
phase: 1
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
---

# Phase 1 - 프론트 기본 뼈대 상세 설계

## 목적

로그인 가능한 Vue 프론트 뼈대를 만들고, 문제 목록/상세 흐름을 mock 기반으로 먼저 굴린다.

## 연결 문서

- [[mwt-execution-plan-v1.2]]
- [[mwt-architecture-v1.3]]

## 포함 태스크

- [x] P1-001 Vue 앱 초기화
- [x] P1-002 라우팅 구성
- [x] P1-003 Cognito 로그인 연결 준비
- [x] P1-004 문제 목록/상세 mock UI

## 추천안

- TypeScript 사용
- 페이지는 `/login`, `/problems`, `/problems/:problemId`, `/submissions/:submissionId`, `/admin/problems/new`
- 로그인 후 `/problems` 진입

## 상세 설계

### 라우팅

- Public: `/login`
- Protected: 나머지 전부

### 상태 관리

- 인증 상태 store 1개
- API 클라이언트는 나중 Phase에서 연결

### 화면 우선순위

1. 로그인
2. 문제 목록
3. 문제 상세
4. 제출 결과
5. 관리자 등록

## 산출물

- [x] `frontend/` Vue 3 + TypeScript + Vite 기본 앱
- [x] 기본 레이아웃
- [x] 보호 라우트
- [x] mock 데이터 기반 문제 UI
- [x] mock 인증 세션과 Cognito 전환용 환경 변수 자리

## 완료 기준

- [x] 로그인 후 보호 화면 이동 가능
- [x] 문제 목록/상세 UI 흐름 확인 가능

## 구현 메모

- Phase 1에서는 실제 Cognito User Pool 생성 전이므로 `VITE_AUTH_PROVIDER=mock` 기반 로컬 세션으로 로그인/로그아웃과 보호 라우트를 검증한다.
- 실제 Cognito 값은 `frontend/.env.example`의 `VITE_COGNITO_*` 환경 변수에 맞춰 주입한다.
- 주요 화면은 `/login`, `/problems`, `/problems/:problemId`, `/submissions/:submissionId`, `/admin/problems/new`로 구성했다.

## 작업 내용 요약

- `frontend/`에 Vue 3 + TypeScript + Vite 앱을 생성했다.
- Vue Router로 `/login`, `/problems`, `/problems/:problemId`, `/submissions/:submissionId`, `/admin/problems/new` 라우트를 구성했다.
- Pinia 기반 인증 store를 추가하고, Phase 1에서는 `localStorage` mock 세션으로 로그인/로그아웃과 보호 라우트를 검증하도록 했다.
- Cognito 실제 연결을 위한 `VITE_AUTH_PROVIDER`, `VITE_COGNITO_USER_POOL_ID`, `VITE_COGNITO_CLIENT_ID`, `VITE_COGNITO_DOMAIN` 환경 변수 자리를 마련했다.
- mock 문제 데이터 기반으로 문제 목록, 문제 상세, statement, sample, 제출 입력, 제출 결과 화면을 구현했다.
- 관리자 문제 등록 초안 화면을 추가했다.
- Node 기준은 로컬/협업 안정성을 위해 Node `22.13.1`로 정했다.

## 최종 확인

- 실제 AWS Cognito User Pool 생성 완료
- Cognito Hosted UI Domain `.env.local` 반영 완료
- 실제 Cognito 로그인 성공 후 세션 유지와 보호 라우트 동작 확인 완료
- 로그아웃 후 로그인 화면 복귀 확인 완료

## AWS Cognito 전달받은 값

- User Pool 이름: `User pool - MWT`
- User Pool ID: `<COGNITO_USER_POOL_ID>`
- App Client ID: `<COGNITO_APP_CLIENT_ID>`
- Cognito Domain: `<COGNITO_HOSTED_UI_DOMAIN>`
- Region: `ap-northeast-2`
- JWKS URL: `<COGNITO_JWKS_URL>`
- Callback URL: `http://localhost:5173/auth/callback`

## Hosted UI 구현 메모

- 로그인은 Cognito Hosted UI Authorization Code + PKCE 흐름을 사용한다.
- `/login`에서 Cognito `/oauth2/authorize`로 이동한다.
- `/auth/callback`에서 authorization code를 `/oauth2/token`으로 교환한다.
- App Client는 client secret 없는 public client여야 한다.
- App Client OAuth 설정에 `Authorization code grant`, `openid`, `email`, `profile` scope가 필요하다.
- App Client callback URL은 `http://localhost:5173/auth/callback`과 정확히 일치해야 한다.
- App Client sign-out URL은 `http://localhost:5173/login`과 정확히 일치해야 한다.

## Cognito scope와 claim 활용 계획

### Phase 1 프론트

- `openid`
  - ID token 발급을 위한 필수 scope로 사용한다.
  - 프론트 세션의 사용자 고유 식별자는 ID token의 `sub` claim을 기준으로 삼는다.
- `email`
  - 상단 사용자 표시와 사용자 식별 보조값으로 `email` claim을 사용한다.
  - 이메일은 표시용/연락용 보조 정보이며 내부 primary key로 쓰지 않는다.
- `profile`
  - MVP에서는 필수 UI 기능에 의존하지 않는다.
  - 이후 이름, 닉네임 등 사용자 표시 정보를 확장할 때 선택적으로 사용한다.
- `cognito:groups`
  - token에 포함되는 경우 프론트에서 관리자 메뉴 노출 같은 UX 보조 판단에만 사용한다.
  - 실제 관리자 권한 판단은 Phase 2 API에서 다시 검증한다.

### 프론트 저장 원칙

- access token과 id token은 API 연동 전까지 최소 범위로만 저장한다.
- 사용자 표시 상태는 `sub`, `email`, `cognito:groups`에서 파생한 값만 store에 둔다.
- 권한이 필요한 API 호출은 Phase 2에서 백엔드 JWT 검증을 통과해야 한다.

## 해결된 이슈

- Hosted UI 이동은 정상 동작한다.
- `redirect_mismatch`는 App Client의 Allowed callback URL 설정으로 해소한다.
- `invalid_scope`는 App Client 허용 scope와 `VITE_COGNITO_SCOPES`를 동일하게 맞춰 해소한다.

## Phase 1 완료 판단

- 로그인 가능한 Vue 프론트 뼈대가 완성되었다.
- 문제 목록/상세 mock UI 흐름이 확인되었다.
- Cognito Hosted UI 기반 실제 로그인/로그아웃 흐름이 확인되었다.
- Phase 2에서는 API/JWT 검증과 데이터 모델 구현으로 넘어간다.
