# MWT Backend

Rust Lambda workspace for Phase 2.

## Layout

- `crates/domain`: shared API and persistence models
- `crates/infra`: repository traits, in-memory adapter, DynamoDB key helpers
- `functions/public-api`: public/user-facing query API
- `functions/admin-api`: admin problem metadata, asset presign, and bundle finalize API
- `functions/submission-consumer`: submission worker dispatcher placeholder

## Local Checks

```sh
cargo fmt --check
cargo test
cargo build
```

## Build Lambda Zip For ARM64

The deployed Lambda functions use the ARM64 architecture. Build the Rust
binary for Linux ARM64, then package the executable as `bootstrap` for the
custom `provided.al2023` runtime.

Prerequisites on macOS:

```sh
rustup target add aarch64-unknown-linux-gnu
brew install zig
cargo install cargo-zigbuild
```

Build and package `public-api`:

```sh
bash scripts/package-public-api.sh
```

Upload `target/lambda/public-api-arm64.zip` to the Lambda function.

Lambda settings:

- Runtime: `provided.al2023`
- Architecture: `arm64`
- Handler: `bootstrap`
- Environment variable: `MWT_CORE_TABLE_NAME=mwt-core-table-prod`

## Phase 2 API Scope

- `GET /problems`
- `GET /problems/{problemId}`
- `GET /problems/{problemId}/statement`
- `GET /submissions/{submissionId}`
- `GET /users/me/submissions`

## AWS Resources

- DynamoDB table: `mwt-core-table-prod`
- S3 assets bucket: `mwt-assets-prod-123456789012-ap-northeast-2-example`

The Lambda execution role needs `s3:GetObject` on:

```text
arn:aws:s3:::mwt-assets-prod-123456789012-ap-northeast-2-example/*
```

## Lambda CI/CD

GitHub Actions workflow:

```text
.github/workflows/deploy-public-api-lambda.yml
```

The workflow builds the ARM64 package, uploads it as an artifact, then deploys
it to the `mwt-public-api` Lambda with `aws lambda update-function-code`.

Required GitHub secret:

```text
AWS_GITHUB_ACTIONS_ROLE_ARN
```

Optional GitHub variable:

```text
PUBLIC_API_LAMBDA_FUNCTION_NAME=mwt-public-api
```

IAM setup notes live in:

```text
infra/iam/README.md
```
