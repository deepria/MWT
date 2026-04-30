---
tags:
  - mwt
  - execution
  - phase-7
doc_type: phase-design
status: active
phase: 7
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
---

# Phase 7 - 리허설과 장애 주입 상세 설계

## 목적

구현이 끝난 시스템이 실제 실패 상황에서도 버티는지 검증한다.

## 연결 문서

- [[📋 IOI 체크리스트 (번역본)]]
- [[mwt-fargate-sandbox-runner-v1.1]]
- [[mwt-architecture-v1.3]]

## 포함 태스크

- P7-001 무한루프 제출 검증
- P7-002 sleep 무한 대기 제출 검증
- P7-003 worker 중간 종료 복구 테스트
- P7-004 bundle 교체 리허설
- P7-005 statement 동시 조회 부하 테스트
- P7-006 제출 burst 테스트
- P7-007 결과 일관성 검증
- P7-008 배포 전 전체 플로우 리허설
- P7-009 stuck submission 감시 검증

## 추천안

- 우선순위는 `while(1)` -> `worker kill` -> `bundle 교체` -> `statement 동시 조회`

## 상세 설계

### 핵심 검증 질문

- timeout이 실제로 작동하는가
- worker 장애 후 submission이 영구 정체되지 않는가
- bundle/manifest/version 변경이 안전하게 반영되는가
- Fargate 환경에서 결과 흔들림이 어느 정도인가

### 성공 기준

- 무한루프 제출이 total timeout 안에서 종료된다
- worker 중간 종료 후 submission이 영구 `running`으로 남지 않는다
- bundle 교체 후 새 `manifest_version`과 `bundle_hash`가 결과에 반영된다
- 동일 제출 반복 실행 시 verdict 흔들림이 운영 허용 범위 안에 있다

### 산출물

- 리허설 로그
- 실패 케이스 기록
- 운영 개선 메모
- stuck submission 대응 기록

## 완료 기준

- 배포 전 전체 플로우를 한 번 끊김 없이 재현 가능
- 대표 실패 시나리오에 대한 대응 경로가 확인됨
- 상태 고착 감시와 복구 절차가 실제로 검증됨
