---
tags:
  - mwt
  - execution
  - phase-5
doc_type: phase-design
status: active
phase: 5
updated: 2026-04-28
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

- P5-001 관리자 권한 처리
- P5-002 관리자 문제 메타 등록 UI
- P5-003 statement/sample 업로드 UI
- P5-004 hidden tests bundle 등록 UI
- P5-005 관리자 검증 루틴
- P5-006 관리자 rejudge 경로 설계
- P5-007 admin-api Lambda CI/CD 구성

## 추천안

- 메타 등록 + 자산 업로드 + 검증 결과 표시까지만 구현

## 상세 설계

### Phase 3 선구현 항목

Phase 3 리허설을 위해 `POST /admin/problems` 최소 API는 먼저 구현했다.
Phase 5에서는 이 API를 사용하는 관리자 UI와 검증 루틴을 완성한다.

### 관리자 권한

- Cognito group 기반

### 등록 플로우

1. 메타 등록
2. statement/sample 업로드
3. bundle 업로드
4. manifest 입력 또는 검증
5. 공개 가능 여부 판정

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
