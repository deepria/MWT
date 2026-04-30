---
aliases:
  - MWT 아키텍처
  - MWT Architecture
tags:
  - mwt
  - architecture
  - aws
  - obsidian
doc_type: architecture
status: active
version: v1.3
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
related:
  - "[[mwt-execution-plan-v1.2]]"
  - "[[mwt-problem-storage-and-staging-v1.1]]"
  - "[[mwt-fargate-sandbox-runner-v1.1]]"
  - "[[mwt-cost-down-v1.1]]"
---

# MWT 아키텍처 설계서 v1.3

> [!abstract] 문서 역할
> MWT의 현재 기준 아키텍처를 정의하는 기준 문서다.

## 빠른 링크

- 허브: [[MWT 마스터 인덱스]]
- 실행 계획: [[mwt-execution-plan-v1.2]]
- 저장/스테이징: [[mwt-problem-storage-and-staging-v1.1]]
- 샌드박스 러너: [[mwt-fargate-sandbox-runner-v1.1]]
- 비용 절감안: [[mwt-cost-down-v1.1]]

작성일: 2026-04-28

## 1. 문서 목적

이 문서는 MWT 온라인 저지 플랫폼의 현재 기준 아키텍처를 정의한다.

이번 버전은 아래 결론을 기준으로 기존 문서를 정리한 통합판이다.

- 프론트와 일반 API는 서버리스
- 제출은 큐 기반 비동기 처리
- 채점은 Fargate 워커에서 수행
- 문제 메타는 DynamoDB
- 문제 자산은 S3
- hidden tests는 bundle 단위로 관리
- 채점 실행은 worker 로컬 staging(`/tmp`)에서 수행

## 2. 최종 한 줄 요약

`serverless web + async submission queue + container judge + S3 bundle staging`

## 3. MVP 범위

### 포함

- 로그인
- 문제 목록 조회
- 문제 상세 조회
- 제출 생성
- 제출 상태 조회
- 채점 결과 조회
- 관리자 문제 등록

### 제외

- contest
- leaderboard
- websocket
- rejudge
- 다국어 대량 지원
- 고급 운영 분석

## 4. 전체 구조

```text
[Amplify Hosting + Vue]
          |
          v
     [CloudFront CDN]
          |
          v
     [API Gateway]
          |
          v
    [Lambda API - Rust]
      |       |       \
      |       |        \
      v       v         v
 [Cognito] [DynamoDB]  [SQS]
                         |
                         v
             [submission-consumer]
                  Lambda
                         |
                         v
               [ECS Fargate RunTask]
                    |           |
                    v           v
                  [S3]    [CloudWatch Logs]
                    |
                    v
               [DynamoDB update]
```

## 5. 컴포넌트별 책임

### Amplify Hosting + Vue

- 사용자 웹앱 제공
- 로그인, 문제 조회, 제출, 결과 확인 UI 담당

### API Gateway + Lambda(Rust)

- 문제 조회 API
- 제출 생성 API
- 제출 상태 조회 API
- 관리자 문제 등록 API

### DynamoDB

- 사용자 메타
- 문제 메타
- statement 위치 정보
- testcase manifest 정보
- 제출 메타
- 제출 결과 요약/상세

### S3

- statement.md
- samples
- hidden tests 원본
- hidden tests bundle
- checker/generator
- 제출 소스
- compile/run/result 로그

### SQS

- 제출 비동기화
- 채점 워커 실행과 API 응답 분리

### ECS Fargate

- 사용자 코드 컴파일/실행/채점
- S3 bundle을 로컬 staging에 내려받아 사용

## 6. 문제 데이터 저장 전략

### 원칙

- 메타데이터는 DB
- 자산은 object storage
- 실행은 local staging

### 최종 구조

- `problem metadata`: DynamoDB
- `statement`: S3
- `samples`: S3
- `hidden tests`: S3 bundle
- `checker`: S3
- `manifest`: DynamoDB
- `judge staging`: Fargate `/tmp`

### 이유

- 조회와 검색은 DB가 담당
- 대형 자산과 파일 배포는 S3가 담당
- 실제 실행은 로컬 파일시스템이 담당

## 7. 문제 등록과 자산 관리

### 관리자 업로드 흐름

1. 관리자 API로 문제 메타 등록
2. statement, sample, hidden tests, checker를 S3에 업로드
3. hidden tests를 bundle로 생성
4. bundle hash와 manifest를 DynamoDB에 기록
5. problem_version 증가

### 관리자 API 최소 범위

- `POST /admin/problems`
- `PUT /admin/problems/{problem_id}`
- `POST /admin/problems/{problem_id}/assets/presign`
- `POST /admin/problems/{problem_id}/bundle/finalize`

## 8. 제출 처리 흐름

1. 프론트가 `POST /submissions` 호출
2. API가 제출 소스를 S3에 저장
3. API가 submission meta를 DynamoDB에 `queued`로 저장
4. API가 SQS에 메시지 발행
5. consumer Lambda가 ECS RunTask 호출
6. worker가 problem metadata와 manifest를 조회
7. worker가 S3에서 testcase bundle과 checker를 다운로드
8. worker가 `/tmp`에 압축 해제 후 compile/run 수행
9. worker가 결과 요약/상세를 DynamoDB에 저장
10. worker가 로그를 S3에 저장
11. 프론트는 polling으로 결과 확인

### 추적 필드 원칙

- submission, result, log는 같은 추적 키를 공유한다
- 최소 필드는 `submission_id`, `problem_id`, `language`, `problem_version`, `manifest_version`, `bundle_hash`다
- worker 실행 추적을 위해 `worker_task_id`, `attempt`, `started_at`, `finished_at`를 남긴다
- 운영 분석과 재채점을 위해 결과 요약과 로그가 같은 버전 정보를 참조해야 한다

## 9. 채점 워커 원칙

### 실행 원칙

- 제출 코드는 항상 비신뢰 코드
- 제출 단위 staging 디렉터리 분리
- compile timeout, testcase timeout, total timeout 적용
- 출력 크기 제한
- 로그 크기 제한

### bundle 원칙

- 저장은 파일 단위 가능
- 실행은 bundle 단위
- 작은 파일 수백 개를 개별 fetch하지 않음

### 언어 지원 원칙

- MVP는 Rust + Python 지원
- 이후 C++ 추가
- Java는 후순위

## 10. 데이터 모델 방향

### core_table 엔티티

- user
- problem
- problem_manifest
- submission
- submission_result_summary
- submission_result_detail

### 최소 GSI

- 내 제출 목록
- 상태 모니터링

초기에는 이 두 개만 유지한다.

## 11. 운영 원칙

### 필수 모니터링

- SQS queue depth
- SQS oldest message age
- Lambda error rate
- ECS task launch failure
- system_error 증가율
- DynamoDB throttling
- bundle download failure
- API request latency
- statement fetch latency

### 리허설 원칙

MVP라도 아래 항목은 실제 배포 전 반드시 반복 검증한다.

- `while(1)` 제출 처리
- `sleep(inf)` 제출 처리
- worker task 중간 종료
- queue 적체 상황
- statement 동시 조회 부하
- testcase bundle 교체 후 재채점

### 최소 운영 도구

- DLQ 확인 및 재큐잉 절차를 운영 문서에 둔다
- `system_error` 제출은 재처리 후보로 분류한다
- testcase, checker, limits 변경 시 관리자 rejudge 경로를 제공한다

### 변경 관리 원칙

testcase, checker, limits 변경은 운영 절차로 문서화한다.

- bundle 교체
- manifest version 증가
- problem_version 증가
- 재채점 대상 선정
- 사용자 노출 시점 결정

운영 중 테스트케이스를 바꿀 수는 있지만, 어떤 순서로 반영하고 어떤 범위를 재채점할지 미리 정해두지 않으면 실수 가능성이 크다.

### 백업 및 복구 원칙

- DynamoDB는 point-in-time recovery 사용 여부를 명시
- 문제 자산 S3는 versioning 또는 별도 보존 정책 검토
- manifest와 problem metadata 복구 절차 문서화
- 배포 전후 DB/S3 상태 확인 절차 마련

### 필수 보존 정책

- CloudWatch Logs retention 7일
- logs bucket lifecycle 7일 또는 30일
- 오래된 제출 산출물 자동 삭제

## 12. 비용 원칙

### 기본 전략

- 상시 ECS Service 금지
- 제출 시에만 Fargate RunTask
- NAT Gateway 미도입
- WAF 미도입
- HTTP API 사용

### 절약 포인트

- 초기 지원 언어는 Rust + Python으로 제한
- testcase bundle 크기 최소화
- 큰 로그 장기 보관 금지
- DynamoDB on-demand 사용

### 확장 경로

- 기본 구조는 `RunTask only`를 유지한다
- 제출량이 꾸준히 증가하거나 이벤트성 burst가 생기면 warm worker pool을 검토한다
- worker pool 도입 시 bundle cache와 preload를 함께 도입한다
- contest, leaderboard, snapshot 계산은 별도 도메인으로 후속 분리한다

## 13. 보안 원칙

### MVP 보안 기준

- task role 최소 권한
- S3 prefix 최소 권한
- CPU/memory hard limit
- timeout 강제 종료
- 제출 간 staging 분리

### 주의

Fargate는 운영 편의성이 높지만, 전통적인 OJ의 고강도 샌드박스를 완전히 대체하지는 않는다.
따라서 MVP의 목표는 완전한 연구용 격리가 아니라, 실용적이고 안정적인 제출 격리다.

### 결정성에 대한 현실 인식

IOI 수준의 대회 시스템은 실행 시간 결정성을 매우 중시하지만, 현재 Fargate 기반 MVP에서는 bare metal 또는 isolate 기반 환경만큼 강한 결정성을 기대하기 어렵다.

따라서 아래 원칙을 둔다.

- borderline timeout 문제는 운영상 별도 관찰
- 언어별 timeout multiplier를 보수적으로 설정
- 더 강한 결정성이 필요해질 경우 EC2 기반 judge 계층 전환 검토

## 14. 최종 결론

현재 MWT의 정답은 아래 구조다.

- 웹과 일반 API는 서버리스
- 제출은 큐 기반 비동기
- 채점은 Fargate
- 문제 메타는 DynamoDB
- 문제 자산은 S3
- hidden tests는 bundle
- 실행은 worker 로컬 staging

이 구조는 1인 개발로 시작 가능하고, 비용을 낮게 유지하면서도 나중에 버리지 않고 확장할 수 있다.
