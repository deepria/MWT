# MWT - Serverless Online Judge

AWS 서버리스 아키텍처를 기반으로 설계하는 경량 온라인 저지(OJ) 프로젝트입니다.

MWT는 문제 조회, 제출 접수, 비동기 채점, 결과 조회까지의 흐름을 작게 나누어 구현하는 것을 목표로 합니다. 현재 저장소는 Vue 기반 프론트엔드, Rust Lambda 백엔드, AWS 인프라 설계 문서를 함께 관리합니다.

## 현재 상태

- 프론트엔드: Vue 3 + TypeScript + Vite 기반 웹앱 뼈대
- 백엔드: Rust Lambda workspace와 Phase 2 조회 API 범위
- 인프라: DynamoDB single-table 설계, Lambda 배포용 IAM/OIDC 문서
- 채점 파이프라인: SQS, Fargate, S3 기반의 비동기 채점 구조를 목표로 설계 중

## 아키텍처 목표

```text
Frontend
  -> API Gateway
  -> Lambda
  -> SQS
  -> Fargate Judge Runner
  -> S3 / DynamoDB
```

## 디렉터리 구조

```text
.
├── frontend/           # Vue 3 + TypeScript 웹앱
├── backend/            # Rust Lambda workspace
├── infra/              # DynamoDB, IAM 등 AWS 인프라 문서/정의
├── docs/               # 아키텍처, 실행 계획, 런타임 설계 문서
└── judge/              # 채점 런타임 작업 영역
```

## 기술 스택

- Frontend: Vue 3, TypeScript, Vite, Pinia, Vue Router
- Backend: Rust, Lambda custom runtime(`provided.al2023`)
- AWS: API Gateway, Lambda, DynamoDB, S3, SQS, Fargate
- CI/CD: GitHub Actions OIDC 기반 Lambda 배포

## 빠른 시작

### Frontend

Node.js `22.13.x` 기준입니다.

```sh
cd frontend
nvm use
npm install
npm run dev
```

검증 명령:

```sh
npm run typecheck
npm run lint
npm run build
```

### Backend

Rust `1.82` 이상 기준입니다.

```sh
cd backend
cargo fmt --check
cargo test
cargo build
```

Lambda ARM64 패키징:

```sh
cd backend
bash scripts/package-public-api.sh
```

## 주요 문서

- [Frontend README](frontend/README.md)
- [Backend README](backend/README.md)
- [DynamoDB 설계](infra/dynamodb/README.md)
- [IAM/OIDC 설정](infra/iam/README.md)
- [아키텍처 문서](docs/10%20Architecture/mwt-architecture-v1.3.md)
- [실행 계획](docs/20%20Execution/mwt-execution-plan-v1.2.md)

## 방향성

서버리스 환경에서 온라인 저지 시스템을 얼마나 단순하고 비용 효율적으로 구성할 수 있는지 검증하는 것이 이 프로젝트의 핵심입니다. MVP 단계에서는 복잡한 운영 기능보다 제출-채점-결과 조회 흐름을 안정적으로 분리하고, 이후 문제 관리와 채점 런타임을 점진적으로 확장합니다.
