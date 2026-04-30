---
tags:
  - mwt
  - execution
  - phase-4
doc_type: phase-design
status: active
phase: 4
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
---

# Phase 4 - 제출 및 채점 파이프라인 상세 설계

## 목적

제출 생성부터 Fargate 채점 실행, 결과 저장까지 end-to-end 파이프라인을 완성한다.

## 연결 문서

- [[mwt-fargate-sandbox-runner-v1.1]]
- [[mwt-problem-storage-and-staging-v1.1]]
- [[mwt-architecture-v1.3]]

## 포함 태스크

- P4-001 제출 생성 API 구현
- P4-002 SQS + DLQ 생성
- P4-003 submission-consumer 구현
- P4-004 ECS cluster/ECR 준비
- P4-005 Rust/Python worker 초안 구현
- P4-006 결과 저장 구현
- P4-007 timeout/limit 제어 구현
- P4-008 상태 전이 및 재처리 규칙 구현
- P4-009 structured log 및 DLQ 운영 절차 구현

## 추천안

- source는 API가 받아 S3 저장
- 큐는 SQS Standard + DLQ
- worker는 Rust + Python 2개 언어 지원으로 시작

## 상세 설계

### 흐름

1. `POST /submissions`
2. source S3 저장
3. submission meta `queued`
4. SQS 발행
5. consumer가 RunTask 호출
6. worker가 source/bundle/checker staging
7. compile/run
8. summary/detail/log 저장

### 상태

- queued
- dispatching
- staging
- compiling
- running
- accepted
- wrong_answer
- time_limit
- memory_limit
- runtime_error
- compile_error
- system_error

### 상태 전이 규칙

- `queued -> dispatching -> staging -> compiling -> running -> final state`
- final state 도달 후에는 자동 재처리하지 않음
- `system_error`만 새 `attempt`로 재처리 가능
- `dispatching`, `staging`, `running` 고착은 운영 감시 대상

### DLQ 및 재처리 규칙

- RunTask launch 실패와 일시적 infra 오류는 재시도 후 DLQ 대상
- bundle/checker/source 누락과 hash mismatch는 `system_error`로 기록
- `compile_error`, `wrong_answer`, `time_limit`, `runtime_error`는 재큐잉하지 않음
- DLQ 확인 후 운영자는 원인 분류, 재큐잉, 최종 실패 확정 중 하나를 수행

### structured log 필드

- `event`
- `submission_id`
- `problem_id`
- `language`
- `attempt`
- `worker_task_id`
- `status`
- `verdict`
- `duration_ms`
- `bundle_hash`

## 산출물

- SQS/DLQ
- ECS/ECR
- worker v1
- end-to-end 채점
- 상태 전이 및 재처리 규칙
- structured log 스키마

## 완료 기준

- 최소 한 문제 기준으로 Rust와 Python 채점이 각각 끝까지 동작한다
- 실패 제출이 DLQ 또는 최종 상태로 정리되고 영구 고착되지 않는다
