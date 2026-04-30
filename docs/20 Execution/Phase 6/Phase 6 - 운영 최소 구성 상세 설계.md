---
tags:
  - mwt
  - execution
  - phase-6
doc_type: phase-design
status: active
phase: 6
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
---

# Phase 6 - 운영 최소 구성 상세 설계

## 목적

최소한의 로그 보존, 알람, 백업/복구 문서를 갖춰 개인 프로젝트 수준의 운영 가능 상태를 만든다.

## 연결 문서

- [[mwt-architecture-v1.3]]
- [[📋 IOI 체크리스트 (번역본)]]

## 포함 태스크

- P6-001 CloudWatch Logs retention
- P6-002 S3 lifecycle
- P6-003 핵심 알람 구성
- P6-004 운영 체크리스트 작성
- P6-005 백업/복구 정책 정리
- P6-006 재처리/rejudge 운영 절차 정리

## 추천안

- 로그 7일
- S3 lifecycle 우선
- 메타데이터 복구 우선

## 상세 설계

### 알람

- queue age
- DLQ messages
- Lambda error
- ECS task launch failure
- bundle download failure
- system_error 증가
- judge failure rate
- p95 judge duration
- timeout rate
- submission stuck count

### 복구

- DynamoDB PITR 여부 결정
- S3 versioning 또는 보존 정책
- manifest 복구 절차 문서화

### 운영자 액션

- DLQ 메시지를 확인하고 원인을 분류한다
- `system_error` 제출은 재큐잉 또는 최종 실패 확정 중 하나로 정리한다
- bundle, checker, limits 변경 후에는 필요 범위만 rejudge한다

## 산출물

- retention/lifecycle 설정
- 알람
- 운영 문서
- 복구 문서
- 재처리/rejudge runbook

## 완료 기준

- 장애 탐지와 복구 경로가 최소한 문서화되어 있다
- 실패 제출을 운영자가 재처리하거나 rejudge할 절차가 존재한다
