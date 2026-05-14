---
tags:
  - mwt
  - execution
  - phase-3
doc_type: phase-design
status: active
phase: 3
updated: 2026-05-14
hub: "[[MWT 마스터 인덱스]]"
parent_plan: "[[mwt-execution-plan-v1.2]]"
---

# Phase 3 - 문제 자산과 Manifest 상세 설계

## 목적

문제 자산 저장 구조와 manifest 등록 흐름을 고정한다.

Phase 2에서 problem/manifest 도메인 모델과 DynamoDB 매핑은 이미 일부 선반영됐다.
Phase 3에서는 남은 S3 인프라, 관리자 업로드 presign, bundle finalize 흐름을 구현하고
문서와 코드의 자산 규칙을 최종 정렬한다.
개발기 리허설을 위해 최소 문제 메타 생성 API도 Phase 3에서 선구현한다.

## 연결 문서

- [[mwt-problem-storage-and-staging-v1.1]]
- [[mwt-architecture-v1.3]]

## 포함 태스크

- [x] P3-001 S3 bucket 생성
- [x] P3-002 문제 자산 prefix 설계
- [x] P3-003 statement 업로드 presign API
- [x] P3-004 sample 업로드 presign API
- [x] P3-005 hidden tests bundle 규격 정의
- [x] P3-006 manifest 스키마 확정
- [x] P3-007 bundle finalize API 구현

상태 기준:

- `[x]`: Phase 2 또는 설계 문서에서 완료로 인정 가능
- `[~]`: 핵심 방향 또는 IaC 초안은 있으나 실제 배포/구현 완료 전
- `[ ]`: 구현 산출물이 아직 없음

## 추천안

- assets/logs bucket 분리
- hidden tests는 `zip bundle`
- manifest는 관리자 입력 + 서버 검증

## 상세 설계

### 히스토리 점검 결과

2026-05-13 기준 git 히스토리에는 Phase 3 완료 커밋이 따로 없다.
다만 `0c6ad8e`의 Phase 2 기본 구축 커밋에서 아래 항목이 선반영됐다.

- `ProblemMeta`, `ProblemManifest`, `ManifestCase` 도메인 모델
- DynamoDB `problem_manifest` item 매핑
- `ProblemRepository::get_manifest`
- `statement_location`, `bundle_key`, `bundle_hash`, `checker_key`, `checker_hash`, `problem_version`, `manifest_version` 필드
- `mwt-problem-storage-and-staging-v1.1`의 저장 계층 결정

따라서 Phase 3는 모델을 새로 만드는 단계가 아니라,
자산 업로드와 finalize를 실제 운영 가능한 흐름으로 연결하는 단계다.

2026-05-13 진행 결과:

- `admin-api` Lambda handler에 관리자 문제 메타 생성/조회, presign, bundle finalize 경로를 구현했다.
- `ProblemRepository::list_all_problems`를 추가해 관리자 목록에서 draft 포함 전체 문제를 볼 수 있게 했다.
- `ProblemAssetRepository::create_problem`, `finalize_problem_bundle`로 DynamoDB 저장 흐름을 연결했다.
- `AssetUploadRepository::presign_put_object`, `head_object`로 S3 presigned PUT과 object 존재/size 확인을 연결했다.
- Cognito `cognito:groups` claim 파싱 문제를 수정해 JSON array/CSV/header mock 모두 `admin` group을 인식한다.
- 프론트에서 관리자 목록, 메타 등록, 상세 번들 업로드/finalize 화면을 구현했다.
- 로컬 검증: `cargo fmt --all --check && cargo test`, `npm run lint`, `npm run build`, `bash scripts/package-admin-api.sh` 통과.
- 개발기 확인: `POST /admin/problems`는 실제 DynamoDB `PutItem`까지 성공했다.
- 남은 운영 확인: 신규 `GET /admin/problems`, `GET /admin/problems/{problem_id}` API Gateway 라우트 추가와 `dynamodb:Scan` 권한 반영.

2026-05-14 보강 결과:

- 문제 등록과 관리자/참가자 조회에서 문제 설명 본문이 빠져 있던 문제를 보완했다.
- `ProblemMeta`에 `statement_markdown`과 `allowed_languages`를 추가했다.
- `ProblemMeta`에 참가자 노출 예제인 `sample_cases`를 추가했다.
- `POST /admin/problems`는 `statement_markdown`, `allowed_languages`, `sample_cases`를 필수로 받는다.
- `/problems/{problem_id}/statement`는 `statement_markdown`이 있으면 이를 우선 반환하고,
  기존 데이터처럼 값이 비어 있으면 S3 `statement_location`을 fallback으로 읽는다.
- 프론트 관리자 등록 화면에 문제 설명 입력과 제출 가능 언어 체크박스를 추가했다.
- 프론트 관리자 등록 화면에 예제 입력/출력 editor를 추가했다.
- 참가자 문제 상세의 제출 언어 선택은 문제별 `allowed_languages`를 따른다.

### 완료로 볼 수 있는 항목

#### P3-005 hidden tests bundle 규격 정의

MVP bundle 포맷은 `zip`으로 확정한다.

이유:

- 로컬과 CI에서 기본 도구로 다루기 쉽다.
- 관리자 업로드 검증 도구를 빠르게 만들 수 있다.
- 이후 최적화가 필요하면 `tar.zst`로 확장 가능하다.

Bundle 내부 권장 구조:

```text
tests/
  001.in
  001.out
  002.in
  002.out
```

MVP에서는 manifest의 `cases[].input_path`, `cases[].output_path`가 bundle root 기준
상대 경로를 가진다. 위 구조를 사용할 경우 값은 `tests/001.in`, `tests/001.out`처럼 저장한다.

#### P3-006 manifest 스키마 확정

Rust 도메인 모델 기준 manifest 필드는 아래로 정렬한다.

```json
{
  "problem_id": "sum-path",
  "manifest_version": 1,
  "bundle_key": "problems/sum-path/bundles/tests-v1.zip",
  "bundle_hash": "sha256:...",
  "bundle_format": "zip",
  "bundle_size_bytes": 184239,
  "case_count": 2,
  "cases": [
    {
      "id": 1,
      "input_path": "tests/001.in",
      "output_path": "tests/001.out",
      "weight": 50
    },
    {
      "id": 2,
      "input_path": "tests/002.in",
      "output_path": "tests/002.out",
      "weight": 50
    }
  ],
  "checker_key": null,
  "checker_hash": null,
  "problem_version": 1
}
```

검증 규칙:

- `bundle_format`은 Phase 3 MVP에서 `zip`만 허용한다.
- `bundle_hash`와 `checker_hash`는 `sha256:{hex}` 형식을 사용한다.
- `case_count`는 `cases.length`와 일치해야 한다.
- `cases[].id`는 1 이상이고 중복될 수 없다.
- `cases[].weight`는 1 이상이어야 한다.
- 모든 `cases[].weight`의 합은 100이어야 한다.
- `input_path`, `output_path`는 절대 경로와 `..` 경로를 허용하지 않는다.
- `checker_key`가 있으면 `checker_hash`도 있어야 한다.

확정 결정:

- weight 합계는 100으로 강제한다.
- 참가자에게 공개되는 sample case는 manifest에 포함하지 않는다.
- manifest는 hidden tests bundle만 관리한다.
- Phase 3 MVP의 사용자 노출 sample은 `ProblemMeta.sample_cases`에 저장한다.
- S3 `problems/{problem_id}/samples/...` prefix는 후속 파일 기반 sample 업로드/교체 UI를 위한 자산 경로로 유지한다.

### S3 bucket

P3-001에서 CloudFormation 또는 동등한 IaC 파일을 추가한다.
2026-05-13 기준 assets bucket은 기존 `mwt-assets-prod-123456789012-ap-northeast-2-example`을 사용하고,
logs bucket은 웹 콘솔에서 생성했다. logs bucket lifecycle은 180일 보관으로 설정했다.

권장 파일:

```text
infra/s3/problem-assets.yaml
```

Bucket 구성:

- assets bucket: 기존 `mwt-assets-prod-123456789012-ap-northeast-2-example` 사용
- logs bucket: 채점 compile/run/result 로그 저장

기본 정책:

- public access block 활성화
- server-side encryption 활성화
- 기존 assets bucket versioning 활성화 여부 확인
- logs bucket lifecycle 180일 보관
- presigned PUT에서 필요한 CORS만 허용

### S3 prefix

- `problems/{problem_id}/statement.md`
- `problems/{problem_id}/samples/{sample_id}.in`
- `problems/{problem_id}/samples/{sample_id}.out`
- `problems/{problem_id}/bundles/tests-v{n}.zip`
- `problems/{problem_id}/checker/checker-v{n}`

규칙:

- `problem_id`는 기존 문제 메타의 ID를 그대로 사용한다.
- `{n}`은 finalize 시점에 확정되는 다음 `manifest_version`과 맞춘다.
- statement는 문제당 현재본 1개를 유지한다.
- Phase 3 MVP의 즉시 노출용 문제 설명은 `ProblemMeta.statement_markdown`에 저장한다.
  S3 `statement.md`는 기존 데이터 fallback과 후속 statement 업로드 UI를 위한 자산 경로로 유지한다.
- Phase 3 MVP의 즉시 노출용 예제 입출력은 `ProblemMeta.sample_cases`에 저장한다.
  S3 sample prefix는 후속 sample 파일 업로드 UI를 위한 자산 경로로 유지한다.
- bundle/checker는 버전 prefix를 유지해 이전 제출의 재현성을 보장한다.

### presign API

#### 문제 메타 생성

Phase 3 리허설용으로 문제 메타 생성과 관리자 조회 endpoint를 먼저 구현한다.
참가자용 `/problems` 목록은 `visibility=public`만 반환하므로, `draft` 문제 확인과
나중 업로드를 위해 관리자 전용 목록/상세 API를 별도로 둔다.

Endpoint:

```text
GET /admin/problems
POST /admin/problems
GET /admin/problems/{problem_id}
POST /admin/problems/{problem_id}/assets/presign
POST /admin/problems/{problem_id}/bundle/finalize
```

요청 예시:

```json
{
  "problem_id": "two-sum",
  "title": "Two Sum",
  "difficulty": "easy",
  "tags": ["array", "hash-map"],
  "time_limit_ms": 1000,
  "memory_limit_mb": 128,
  "statement_markdown": "# Two Sum\n\n정수 배열에서 합이 target이 되는 두 원소를 찾는다.",
  "allowed_languages": ["Rust", "Python"],
  "sample_cases": [
    {
      "input": "4 9\n2 7 11 15",
      "output": "1 2"
    }
  ],
  "visibility": "draft"
}
```

서버 기본값:

- `statement_location`: `s3://{assets_bucket}/problems/{problem_id}/statement.md`
- `problem_version`: `1`
- `manifest_version`: `null`
- `bundle_key`, `bundle_hash`, `checker_key`, `checker_hash`: `null`
- `visibility`: 요청에 없으면 `draft`

필수 검증:

- `statement_markdown`은 비어 있을 수 없다.
- `allowed_languages`는 1개 이상이어야 한다.
- 언어명은 영문/숫자/공백/`-`, `_`, `+`, `#`만 허용한다.
- 중복 언어명은 저장 전 제거한다.
- `sample_cases`는 1개 이상이어야 한다.
- `sample_cases[].input`, `sample_cases[].output`은 비어 있을 수 없다.

#### 자산 presign

P3-003/P3-004에서 `admin-api`에 관리자 전용 presign endpoint를 추가한다.
presigned URL은 짧게 살아야 하므로 문제 메타 등록 시점이 아니라 실제 업로드 버튼을
누르는 시점에 새로 발급한다.

브라우저 테스트 흐름은 `/admin/problems/{problem_id}` 상세 화면에서 실행한다.

Endpoint:

```text
POST /admin/problems/{problem_id}/assets/presign
```

요청 예시:

```json
{
  "asset_type": "bundle",
  "content_type": "application/zip"
}
```

`asset_type` 후보:

- `statement`
- `sample_input`
- `sample_output`
- `bundle`
- `checker`

응답 예시:

```json
{
  "bucket": "mwt-assets-prod-123456789012-ap-northeast-2-example",
  "key": "problems/sum-path/bundles/tests-v1.zip",
  "upload_url": "https://...",
  "expires_in_seconds": 900
}
```

서버 책임:

- 관리자 권한 확인
- 문제 존재 여부 확인
- `asset_type`별 허용 key 생성
- presigned PUT URL 발급
- 클라이언트가 임의 S3 key를 지정하지 못하게 차단

### finalize 흐름

1. 업로드 완료
2. `bundle_key`의 S3 object 존재 확인
3. size 검증과 hash 형식 검증
4. manifest request 검증
5. `MANIFEST#{manifest_version}` item 저장
6. `ProblemMeta`의 `bundle_key`, `bundle_hash`, `checker_key`, `checker_hash`,
   `manifest_version`, `problem_version` 갱신

Endpoint:

```text
POST /admin/problems/{problem_id}/bundle/finalize
```

요청 예시:

```json
{
  "bundle_key": "problems/sum-path/bundles/tests-v1.zip",
  "bundle_hash": "sha256:...",
  "bundle_size_bytes": 184239,
  "cases": [
    {
      "id": 1,
      "input_path": "tests/001.in",
      "output_path": "tests/001.out",
      "weight": 50
    }
  ],
  "checker_key": null,
  "checker_hash": null
}
```

응답 예시:

```json
{
  "problem_id": "sum-path",
  "problem_version": 2,
  "manifest_version": 2,
  "bundle_key": "problems/sum-path/bundles/tests-v2.zip",
  "bundle_hash": "sha256:..."
}
```

주의:

- finalize는 같은 `problem_id`에 대해 순차 처리되어야 한다.
- 현재 로컬 구현은 manifest 저장 후 problem meta를 갱신한다.
  개발기 배포 전 DynamoDB conditional write 또는 transaction으로 같은 `problem_id`의 동시 finalize 경쟁 상태를 막는 보강이 필요하다.
- S3 object 누락, size mismatch, 잘못된 hash 형식은 400 계열 오류로 반환한다.
- Phase 3 MVP 구현은 S3 `head_object`로 존재/size를 확인하고, `sha256:{hex}` 형식의 hash를 manifest 기준값으로 저장한다.
  서버가 bundle 내용을 다시 다운로드해 sha256을 재계산하는 방식은 bundle 크기와 Lambda 비용을 고려해 MVP 범위에서 제외한다.

## 산출물

- assets/logs bucket IaC
- 자산 경로 규칙 문서
- bundle/manifest 규격
- statement/sample/bundle/checker presign API
- bundle finalize API
- manifest 저장 및 problem meta 갱신 repository 함수

## 다음 실행 순서

1. 문제 설명/제출 언어/예제 보강분을 `admin-api`, `public-api`, frontend에 배포
2. 관리자 화면에서 신규 문제 등록 시 설명, 제출 가능 언어, 예제 입출력이 저장되는지 확인
3. 참가자 문제 상세에서 설명 본문, 예제, 언어 제한이 노출되는지 확인
4. 관리자 화면에서 기존 `test` 문제가 목록에 보이는지 확인
5. 상세 화면에서 bundle ZIP 업로드 후 finalize 리허설
6. finalize 동시성 보강을 DynamoDB transaction 또는 conditional write로 후속 처리

## 완료 기준

- 문제 자산이 S3와 DynamoDB 기준으로 일관되게 등록된다
- 관리자 API로 statement/sample/bundle/checker 업로드 URL을 발급할 수 있다.
- bundle finalize 후 `ProblemMeta`와 `ProblemManifest`가 같은 version/hash를 가리킨다.
- 잘못된 hash, 누락된 S3 object, 잘못된 manifest는 저장 전에 차단된다.

현재 완료 판단:

- 로컬 코드와 테스트 기준 완료.
- 개발기 전체 완료 판단은 API Gateway 신규 라우트, Lambda 실행 역할 권한, S3 CORS 반영 후 화면 리허설 성공 시점으로 둔다.
