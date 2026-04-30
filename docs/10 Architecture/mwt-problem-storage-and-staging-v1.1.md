---
aliases:
  - MWT 저장 전략
  - MWT Problem Storage
  - MWT Staging Design
tags:
  - mwt
  - storage
  - s3
  - dynamodb
  - obsidian
doc_type: storage-and-staging
status: active
version: v1.1
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
related:
  - "[[mwt-architecture-v1.3]]"
  - "[[mwt-execution-plan-v1.2]]"
  - "[[mwt-fargate-sandbox-runner-v1.1]]"
---

# MWT 문제 데이터 저장/배포/채점 Staging 설계서 v1.1

> [!example] 문서 역할
> 문제 메타, statement, bundle, manifest, `/tmp` staging을 어떤 계층에 둘지 정의한다.

## 빠른 링크

- 허브: [[MWT 마스터 인덱스]]
- 기준 아키텍처: [[mwt-architecture-v1.3]]
- 실행 계획: [[mwt-execution-plan-v1.2]]
- 샌드박스 러너: [[mwt-fargate-sandbox-runner-v1.1]]

작성일: 2026-04-23

## 1. 문서 목적

이 문서는 MWT에서 문제 데이터와 테스트케이스를 어떻게 저장하고, 채점 시점에 어떤 단위로 가져와 실행할지 정의한다.

이번 버전은 옵션 나열을 줄이고, 현재 프로젝트의 최종 판단을 명확히 고정한다.

## 2. 최종 결론

- `problem metadata`: DynamoDB
- `statement`: S3
- `samples`: S3
- `hidden tests`: S3 bundle
- `checker`: S3
- `manifest`: DynamoDB
- `execution staging`: Fargate `/tmp`

핵심 원칙:

`저장은 영구 저장소에 하고, 실행은 로컬 staging에서 한다.`

## 3. 왜 이렇게 가는가

온라인 저지에서 중요한 것은 저장 가능 여부보다 채점 시점의 실행 경로다.

즉, 중요한 질문은 아래다.

- metadata 조회가 빠른가
- 자산을 안정적으로 배포할 수 있는가
- worker가 작은 파일을 반복 fetch하지 않는가
- 채점 실행이 재현 가능한가

이 기준에서 가장 현실적인 답이 `DB + S3 + local staging` 구조다.

## 4. 계층별 역할

### DynamoDB

- problem metadata
- statement location
- bundle 정보
- manifest 정보
- checker 정보
- problem_version

### S3

- statement markdown
- sample input/output
- hidden tests bundle
- checker/generator
- 문제 관련 정적 자산

### Fargate `/tmp`

- source staging
- bundle download
- bundle extract
- compile output
- run logs

## 5. 최종 저장 구조

### 5-1. problem metadata

예시 필드:

- `problem_id`
- `title`
- `difficulty`
- `tags`
- `time_limit_ms`
- `memory_limit_mb`
- `visibility`
- `statement_location`
- `bundle_key`
- `bundle_hash`
- `checker_key`
- `checker_hash`
- `problem_version`
- `manifest_version`

### 5-2. statement

- S3에 markdown 파일 저장
- DB에는 location만 저장

이유:

- 자산 관리 흐름을 단순화
- sample과 동일 저장 계층 사용
- 향후 CDN/cache 확장 쉬움

### 5-3. hidden tests

- S3에 bundle 단위 저장
- 원본 파일이 있더라도 worker는 bundle만 사용

이유:

- 작은 파일 다건 fetch 방지
- worker 시작 시간 안정화
- 요청 수와 메타데이터 비용 감소

## 6. manifest 설계

manifest는 DB에 저장되는 실행 참조 정보다.

권장 필드:

- `manifest_version`
- `problem_id`
- `bundle_key`
- `bundle_hash`
- `bundle_format`
- `bundle_size_bytes`
- `case_count`
- `cases`
- `checker_key`
- `checker_hash`
- `problem_version`

예시:

```json
{
  "manifest_version": 1,
  "problem_id": "p_1001",
  "bundle_key": "problems/p_1001/bundles/tests-v1.zip",
  "bundle_hash": "sha256:...",
  "bundle_format": "zip",
  "bundle_size_bytes": 184239,
  "case_count": 20,
  "cases": [
    { "id": 1, "in": "001.in", "out": "001.out", "weight": 5 },
    { "id": 2, "in": "002.in", "out": "002.out", "weight": 5 }
  ],
  "checker_key": "problems/p_1001/checker/checker-v1",
  "checker_hash": "sha256:...",
  "problem_version": 1
}
```

## 7. bundle 원칙

### 저장

- 원본 테스트 파일은 개별 파일이어도 됨
- 최종 채점용 배포물은 bundle로 생성

### 실행

- worker는 bundle 하나를 받아서 `/tmp`에 푼 뒤 사용
- 개별 테스트 파일을 S3에서 하나씩 읽지 않음

### 권장 형식

- MVP: `zip`
- 이후 최적화: `tar.zst`

## 8. 채점 staging 흐름

1. worker가 submission 수신
2. DynamoDB에서 problem metadata 조회
3. manifest에서 bundle/checker 정보 확인
4. S3에서 source 다운로드
5. S3에서 bundle 다운로드
6. `/tmp/judge/submissions/{submission_id}`에 압축 해제
7. checker 필요 시 함께 staging
8. compile
9. testcase loop 실행
10. 결과 저장

## 9. staging 디렉터리 예시

```text
/tmp/judge/
  submissions/
    s_900001/
      source/
      build/
      tests/
      checker/
      logs/
```

원칙:

- 제출 단위 완전 분리
- source/build/tests/logs 분리
- task 종료 시 자동 폐기 전제

## 10. 관리자 등록 흐름

1. 문제 메타 등록
2. statement 업로드
3. sample 업로드
4. hidden tests bundle 업로드
5. checker 업로드
6. manifest 저장
7. problem_version 증가

## 11. 변경 관리 원칙

문제 자산은 저장만 하면 끝나는 데이터가 아니라, 버전이 있는 운영 대상이다.

### bundle 변경 시

- `manifest_version` 증가
- 필요 시 `problem_version` 증가
- bundle hash 갱신
- 재채점 대상 범위 결정

### checker 변경 시

- checker hash 갱신
- 판정 로직 변경 여부 검토
- 필요 시 기존 제출 재채점

### limits 변경 시

- time limit / memory limit 변경 기록
- 사용자 노출 시점 결정
- 재채점 여부 판단

이 절차가 없으면 문제 데이터와 제출 결과의 기준 시점이 어긋날 수 있다.

## 12. 복구와 백업 관점

이 문서의 구조는 저장과 실행을 분리하기 때문에, 복구 포인트도 분리해서 봐야 한다.

### 메타데이터

- problem metadata
- manifest
- version 정보

복구 원칙:

- DynamoDB 복구 경로 문서화
- 마지막 정상 manifest 기준 복구 가능해야 함

### 자산

- statement
- samples
- bundle
- checker

복구 원칙:

- S3 보존 정책 또는 versioning 검토
- 배포 중인 bundle/checker를 다시 특정 가능해야 함

## 13. 운영 검증 항목

이 저장 구조는 아래 시나리오에서 반드시 검증한다.

- statement 업로드 후 즉시 조회 가능 여부
- bundle 교체 후 새 manifest 반영 여부
- 잘못된 bundle hash 입력 시 차단 여부
- worker가 오래된 manifest를 참조하지 않는지 확인
- bundle 변경 후 재채점 절차 재현 가능 여부

## 14. 비권장 안티패턴

- hidden tests 전체를 DB BLOB에 직접 저장
- S3의 작은 파일을 제출마다 개별 fetch
- worker가 실행 중 원격 저장소를 계속 조회
- 파일시스템 디렉터리를 조회 DB처럼 사용

## 15. 최종 결론

MWT의 문제 데이터 구조는 아래로 고정하는 것이 가장 현실적이다.

- 메타는 DynamoDB
- 자산은 S3
- hidden tests는 bundle
- 실행은 `/tmp` staging

이 구조가 조회, 업로드, 배포, 채점 실행을 가장 깔끔하게 분리한다.
