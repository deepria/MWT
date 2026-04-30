# DynamoDB Core Table

Phase 2 uses a single-table design for MVP metadata.

## Table

- Name: `mwt-core-table-prod`
- Partition key: `pk`
- Sort key: `sk`
- Billing: on-demand
- PITR: enabled

## Assets Bucket

- Name: `mwt-assets-prod-123456789012-ap-northeast-2-example`
- Statement location example:
  `s3://mwt-assets-prod-123456789012-ap-northeast-2-example/problems/sum-path/statement.md`

## Primary Entities

| Entity | `pk` | `sk` |
| --- | --- | --- |
| Problem meta | `PROBLEM#{problem_id}` | `META` |
| Problem manifest | `PROBLEM#{problem_id}` | `MANIFEST#{manifest_version}` |
| Submission meta | `SUBMISSION#{submission_id}` | `META` |
| Submission result summary | `SUBMISSION#{submission_id}` | `RESULT#SUMMARY` |
| Submission result detail | `SUBMISSION#{submission_id}` | `RESULT#DETAIL#{case_id}` |

All items include an `entity_type` attribute matching the logical entity name, for example
`problem_meta`, `problem_manifest`, `submission_meta`, and `submission_result_summary`.

## GSI

### `gsi1-user-submissions`

- `gsi1_pk`: `USER#{user_id}`
- `gsi1_sk`: `SUBMITTED_AT#{submitted_at}#SUBMISSION#{submission_id}`

Used by `GET /users/me/submissions`.

### `gsi2-submissions-by-status`

- `gsi2_pk`: `STATUS#{status}`
- `gsi2_sk`: `SUBMITTED_AT#{submitted_at}#SUBMISSION#{submission_id}`

Used by operations and stuck-submission monitoring.
