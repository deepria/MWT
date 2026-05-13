# S3 Problem Assets

Phase 3 uses two private S3 buckets. The production problem assets bucket already
exists:

```text
mwt-assets-prod-123456789012-ap-northeast-2-example
```

## Buckets

- Problem assets bucket: statement, samples, hidden test bundles, and checkers.
- Judge logs bucket: compile, run, and result logs from judge workers.

New buckets should block public access, require TLS, and use server-side encryption.
The existing assets bucket should keep versioning enabled so older bundle/checker
objects can remain reproducible for previous submissions.

## Deploy

```bash
aws cloudformation deploy \
  --stack-name mwt-problem-assets-prod \
  --template-file infra/s3/problem-assets.yaml \
  --parameter-overrides \
    Environment=prod \
    CreateAssetsBucket=false \
    LogsBucketName=mwt-logs-prod-123456789012-ap-northeast-2-example \
    AllowedCorsOrigins=https://<frontend-origin> \
    LogsRetentionDays=180
```

Do not try to create the existing production assets bucket with this stack unless
it is first imported into CloudFormation. For a brand-new environment, set
`CreateAssetsBucket=true` and pass an unused `AssetsBucketName`.

The production logs bucket was created from the AWS console with a 180-day
lifecycle retention policy.

## Prefixes

```text
problems/{problem_id}/statement.md
problems/{problem_id}/samples/{sample_id}.in
problems/{problem_id}/samples/{sample_id}.out
problems/{problem_id}/bundles/tests-v{manifest_version}.zip
problems/{problem_id}/checker/checker-v{manifest_version}
```
