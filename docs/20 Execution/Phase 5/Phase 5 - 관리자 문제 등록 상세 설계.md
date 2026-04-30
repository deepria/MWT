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

## 추천안

- 메타 등록 + 자산 업로드 + 검증 결과 표시까지만 구현

## 상세 설계

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

## 완료 기준

- 불완전한 문제는 공개할 수 없고, 완전한 문제는 등록 가능하다
- bundle/checker/limits 변경 후 운영자가 재채점 경로를 사용할 수 있다
