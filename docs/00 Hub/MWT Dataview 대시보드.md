---
aliases:
  - MWT Dashboard
  - MWT Dataview Dashboard
tags:
  - mwt
  - dashboard
  - dataview
  - obsidian
doc_type: dashboard
status: active
updated: 2026-04-28
hub: "[[MWT 마스터 인덱스]]"
---

# MWT Dataview 대시보드

> [!info] 사용 전제
> 이 노트는 Obsidian의 `Dataview` 플러그인을 기준으로 작성했다.

## 빠른 링크

- 허브: [[MWT 마스터 인덱스]]
- 아키텍처: [[mwt-architecture-v1.3]]
- 실행 계획: [[mwt-execution-plan-v1.2]]
- 저장/스테이징: [[mwt-problem-storage-and-staging-v1.1]]
- 샌드박스 러너: [[mwt-fargate-sandbox-runner-v1.1]]
- 비용 절감안: [[mwt-cost-down-v1.1]]

## 폴더별 문서 수

```dataview
TABLE
  length(rows) AS "문서 수"
FROM "mwt"
GROUP BY file.folder
SORT file.folder ASC
```

## 현재 활성 문서

```dataview
TABLE
  doc_type AS "유형",
  version AS "버전",
  status AS "상태",
  updated AS "업데이트"
FROM "mwt"
WHERE status = "active"
SORT file.name ASC
```

## 문서 맵

```dataview
TABLE
  hub AS "허브",
  related AS "관련 문서"
FROM "mwt"
WHERE doc_type != "dashboard"
SORT doc_type ASC
```

## 아키텍처 관련 문서

```dataview
TABLE
  version AS "버전",
  updated AS "업데이트",
  related AS "관련 문서"
FROM "mwt"
WHERE contains(tags, "architecture")
   OR doc_type = "architecture"
   OR doc_type = "storage-and-staging"
   OR doc_type = "sandbox-runner"
SORT updated DESC
```

## 실행/운영 관련 문서

```dataview
TABLE
  version AS "버전",
  updated AS "업데이트",
  related AS "관련 문서"
FROM "mwt"
WHERE doc_type = "execution-plan"
   OR doc_type = "dashboard"
SORT updated DESC
```

## 비용 관련 문서

```dataview
TABLE
  version AS "버전",
  updated AS "업데이트"
FROM "mwt"
WHERE doc_type = "cost"
SORT updated DESC
```

## 태그별 문서

```dataview
TABLE
  tags AS "태그",
  doc_type AS "유형",
  version AS "버전"
FROM "mwt"
SORT file.name ASC
```

## 최근 수정 문서

```dataview
TABLE
  updated AS "업데이트",
  doc_type AS "유형",
  version AS "버전"
FROM "mwt"
SORT updated DESC
LIMIT 10
```

## 읽기 순서

```dataview
LIST
FROM "mwt"
WHERE file.name = "mwt-architecture-v1.3"
   OR file.name = "mwt-problem-storage-and-staging-v1.1"
   OR file.name = "mwt-fargate-sandbox-runner-v1.1"
   OR file.name = "mwt-execution-plan-v1.2"
   OR file.name = "mwt-cost-down-v1.1"
SORT file.name ASC
```

## 수동 체크포인트

- [ ] statement가 S3 기준으로 문서 전반에 일관되게 반영되어 있는가
- [ ] hidden tests가 bundle + manifest 기준으로 관리되는가
- [ ] worker가 `/tmp` staging 기반으로 설명되고 있는가
- [ ] IOI 체크리스트에서 가져온 리허설 항목이 실행 계획에 반영되어 있는가
- [ ] 새 문서를 만들 때 frontmatter와 `hub`, `related` 속성을 넣었는가

## 문서 작성 규칙

> [!tip] 새 노트 추가 시 권장 규칙
> - `mwt/` 폴더 안에서 생성
> - 역할에 맞는 하위 폴더에 생성
> - frontmatter에 `doc_type`, `status`, `version`, `updated` 포함
> - 상단에 `[[MWT 마스터 인덱스]]` 링크 추가
> - 관련 문서는 `related` 속성에 연결

## 참고

- 참고 체크리스트: [[📋 IOI 체크리스트 (번역본)]]
- 허브 노트: [[MWT 마스터 인덱스]]
