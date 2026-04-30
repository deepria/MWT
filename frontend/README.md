# MWT Frontend

Vue 3 + TypeScript 기반 MWT 웹앱 뼈대입니다.

## Node

이 프로젝트는 Node `22.13.1` 기준으로 맞춥니다.

```sh
nvm use
```

## 실행

```sh
npm install
npm run dev
```

## 인증

Phase 1은 `VITE_AUTH_PROVIDER=mock`을 기본으로 사용합니다. 로그인 성공 시 세션은
`localStorage`에 저장되며, 보호 라우트 진입과 로그아웃 흐름을 검증할 수 있습니다.

실제 Cognito 연결 시 `.env`에 아래 값을 채운 뒤 인증 store의 `login` 구현을 Cognito SDK 또는
Hosted UI 흐름으로 교체합니다.

```sh
VITE_AUTH_PROVIDER=cognito
VITE_AWS_REGION=ap-northeast-2
VITE_COGNITO_USER_POOL_ID=ap-northeast-2_EXAMPLE
VITE_COGNITO_CLIENT_ID=examplepublicclientid
VITE_COGNITO_DOMAIN=https://mwt-example.auth.ap-northeast-2.amazoncognito.com
VITE_COGNITO_SCOPES=openid email profile
VITE_COGNITO_REDIRECT_SIGN_IN=http://localhost:5173/auth/callback
VITE_COGNITO_REDIRECT_SIGN_OUT=http://localhost:5173/login
```
