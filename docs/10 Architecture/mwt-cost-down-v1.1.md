---
aliases:
  - MWT 비용 절감안
  - MWT Cost Down
tags:
  - mwt
  - cost
  - aws
  - obsidian
doc_type: cost
status: active
version: v1.1
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
related:
  - "[[mwt-architecture-v1.3]]"
  - "[[mwt-execution-plan-v1.2]]"
---

# MWT 비용 극한 절약형 설계서 v1.1

> [!tip] 문서 역할
> MWT를 개인 프로젝트 수준에서 가장 싸게 운영하기 위한 기준안이다.

## 빠른 링크

- 허브: [[MWT 마스터 인덱스]]
- 기준 아키텍처: [[mwt-architecture-v1.3]]
- 실행 계획: [[mwt-execution-plan-v1.2]]

작성일: 2026-04-23

## 1. 문서 목적

이 문서는 MWT를 개인 프로젝트 수준에서 가장 싸게 운영하기 위한 기준안을 정의한다.

이번 버전은 아래 최신 결론을 반영한다.

- statement와 hidden tests는 S3 중심
- hidden tests는 bundle 단위로 관리
- worker는 상시 실행하지 않고 RunTask만 사용
- 채점은 로컬 staging 기반으로 수행

## 2. 핵심 원칙

`항상 켜져 있는 것과 고정비를 없애고, 제출이 들어올 때만 짧게 실행한다.`

## 3. 절감 우선순위

### 1순위

- NAT Gateway 제거
- 상시 ECS Service 제거
- WAF 미도입
- websocket 미도입

### 2순위

- 지원 언어 최소화
- testcase bundle 단위 관리
- worker 이미지 최소화
- 로그 보존 기간 단축

### 3순위

- 운영 대시보드 최소화
- 고급 분석 기능 미도입

## 4. 최종 절약형 구조

```text
[Amplify Hosting + Vue]
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
             [Consumer Lambda]
                         |
                         v
            [ECS Fargate RunTask only]
                    |            |
                    v            v
                  [S3]     [CloudWatch Logs]
```

## 5. 가장 싼 MVP 구성

- Amplify Hosting
- API Gateway HTTP API
- Rust Lambda
- Cognito User Pool
- DynamoDB on-demand
- S3
- SQS Standard
- ECS Fargate RunTask only

## 6. 비용 관점의 저장 전략

### 메타와 자산 분리

- `problem metadata + manifest`: DynamoDB
- `statement + samples + hidden tests bundle + checker`: S3
- `execution staging`: Fargate `/tmp`

### 이유

- DB에는 작은 메타만 둔다
- 큰 파일은 S3에 둔다
- 실행은 원격 fetch 반복이 아니라 한 번의 bundle preload로 끝낸다

## 7. 비용을 많이 잡아먹는 실수

### 안티패턴

- hidden tests를 작은 파일로 개별 fetch
- 상시 worker 서비스 운영
- 로그 무기한 보관
- 큰 런타임 이미지 여러 개 유지
- NAT Gateway 상시 사용

## 8. 채점 비용 절감 전략

### 언어

- 1차: Rust + Python 지원
- 2차: C++ 추가
- Java는 후순위

### worker

- 최소 사양부터 시작
- RunTask 기반만 사용
- 큰 base image 피하기
- 제출량 증가 전까지 warm worker pool은 도입하지 않기

### testcase

- hidden tests는 bundle로 묶기
- sample과 hidden을 분리
- 필요 이상으로 케이스 수 늘리지 않기

## 9. 로그와 저장소 절감 전략

### CloudWatch Logs

- retention 7일
- verbose 로그 금지

### S3

- logs bucket lifecycle 180일
- 문제 자산만 장기 보관
- 제출 산출물은 짧게 보관

## 10. 네트워크 절감 전략

### 기본 원칙

- 개인 실험용에서는 가장 비싼 네트워크 구성을 피한다
- 보안 이상형보다 비용 절감형 구조를 우선한다

### 확정

- 기본 리전은 `ap-northeast-2`
- 비용만 보면 다른 리전이 유리할 수 있지만, MWT는 서울 리전에서 실제 사용자 체감 성능을 먼저 검증한다

## 11. 운영 절감 전략

- alarm은 꼭 필요한 것만
- DLQ는 두되, 대형 관제 구성은 하지 않음
- polling 사용
- bundle 검증 실패와 queue 적체만 우선 모니터링

## 12. 월비용 감각

### 매우 가벼운 사용

- 월 `5~10달러` 가능

### 주말마다 테스트

- 월 `10~20달러` 가능

### Fargate 자주 실행, 로그 많음

- 월 `20~50달러` 가능

### 주의

NAT Gateway가 붙는 순간 체감 비용이 급격히 올라갈 수 있다.

## 13. 최종 결론

가장 싼 실전 조합은 아래다.

`Amplify + HTTP API + Lambda + DynamoDB + S3 + SQS + 필요할 때만 Fargate`

그리고 채점은 아래 원칙으로 고정한다.

- hidden tests는 S3 bundle
- manifest는 DynamoDB
- 실행은 `/tmp` staging

이렇게 해야 개인 프로젝트 수준에서도 비용을 통제하면서 OJ 구조를 유지할 수 있다.
