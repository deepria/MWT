---
tags:
  - mwt
  - execution
  - phase-3
doc_type: phase-design
status: active
phase: 3
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
---

# Phase 3 - 문제 자산과 Manifest 상세 설계

## 목적

문제 자산 저장 구조와 manifest 등록 흐름을 고정한다.

## 연결 문서

- [[mwt-problem-storage-and-staging-v1.1]]
- [[mwt-architecture-v1.3]]

## 포함 태스크

- P3-001 S3 bucket 생성
- P3-002 문제 자산 prefix 설계
- P3-003 statement 업로드 presign API
- P3-004 sample 업로드 presign API
- P3-005 hidden tests bundle 규격 정의
- P3-006 manifest 스키마 확정
- P3-007 bundle finalize API 구현

## 추천안

- assets/logs bucket 분리
- hidden tests는 `zip bundle`
- manifest는 관리자 입력 + 서버 검증

## 상세 설계

### S3 경로

- `problems/{problem_id}/statement.md`
- `problems/{problem_id}/samples/...`
- `problems/{problem_id}/bundles/tests-v{n}.zip`
- `problems/{problem_id}/checker/...`

### finalize 흐름

1. 업로드 완료
2. hash 검증
3. manifest 저장
4. `problem_version` 증가

## 산출물

- assets/logs bucket
- 자산 경로 규칙
- bundle/manifest 규격
- finalize API

## 완료 기준

- 문제 자산이 S3와 DynamoDB 기준으로 일관되게 등록된다
