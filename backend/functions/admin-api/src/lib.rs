use lambda_http::http::StatusCode;
use lambda_http::RequestExt;
use lambda_http::{Body, Error, Request, Response};
use mwt_domain::auth::AuthContext;
use mwt_domain::problem::{
    Difficulty, ManifestCase, ProblemManifest, ProblemMeta, ProblemVisibility,
};
use mwt_infra::repository::{
    AssetUploadRepository, ProblemAssetRepository, ProblemRepository, RepositoryError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const PRESIGN_EXPIRES_IN_SECONDS: u64 = 900;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AssetType {
    Statement,
    SampleInput,
    SampleOutput,
    Bundle,
    Checker,
}

#[derive(Debug, Deserialize)]
struct PresignRequest {
    asset_type: AssetType,
    content_type: Option<String>,
    sample_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FinalizeBundleRequest {
    bundle_key: String,
    bundle_hash: String,
    bundle_size_bytes: u64,
    cases: Vec<ManifestCase>,
    checker_key: Option<String>,
    checker_hash: Option<String>,
}

#[derive(Debug, Serialize)]
struct FinalizeBundleResponse {
    problem_id: String,
    problem_version: u32,
    manifest_version: u32,
    bundle_key: String,
    bundle_hash: String,
}

#[derive(Debug, Deserialize)]
struct CreateProblemRequest {
    problem_id: String,
    title: String,
    difficulty: Difficulty,
    tags: Vec<String>,
    time_limit_ms: u32,
    memory_limit_mb: u32,
    visibility: Option<ProblemVisibility>,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
}

pub async fn handle_request<R>(
    request: Request,
    repository: R,
    assets_bucket: String,
) -> Result<Response<Body>, Error>
where
    R: ProblemRepository + AssetUploadRepository + ProblemAssetRepository,
{
    let method = request.method().as_str();
    let path = request.uri().path();
    let segments = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match (method, segments.as_slice()) {
        ("GET", ["admin", "problems"]) => {
            if let Some(response) = require_admin(&request) {
                return response;
            }
            list_admin_problems(&repository).await
        }
        ("POST", ["admin", "problems"]) => {
            if let Some(response) = require_admin(&request) {
                return response;
            }
            create_problem(&request, &repository, &assets_bucket).await
        }
        ("GET", ["admin", "problems", problem_id]) => {
            if let Some(response) = require_admin(&request) {
                return response;
            }
            get_admin_problem(&repository, problem_id).await
        }
        ("POST", ["admin", "problems", problem_id, "assets", "presign"]) => {
            if let Some(response) = require_admin(&request) {
                return response;
            }
            presign_problem_asset(&request, &repository, &assets_bucket, problem_id).await
        }
        ("POST", ["admin", "problems", problem_id, "bundle", "finalize"]) => {
            if let Some(response) = require_admin(&request) {
                return response;
            }
            finalize_problem_bundle(&request, &repository, &assets_bucket, problem_id).await
        }
        _ => json_response(
            StatusCode::NOT_FOUND,
            &ErrorBody {
                message: "route not found".to_string(),
            },
        ),
    }
}

async fn list_admin_problems<R>(repository: &R) -> Result<Response<Body>, Error>
where
    R: ProblemRepository,
{
    let mut problems = repository.list_all_problems().await?;
    problems.sort_by(|left, right| left.problem_id.cmp(&right.problem_id));

    json_response(StatusCode::OK, &problems)
}

async fn get_admin_problem<R>(repository: &R, problem_id: &str) -> Result<Response<Body>, Error>
where
    R: ProblemRepository,
{
    if !is_safe_segment(problem_id) {
        return bad_request("invalid problem id");
    }

    let problem = match repository.get_problem(problem_id).await {
        Ok(problem) => problem,
        Err(RepositoryError::NotFound(_)) => return not_found("problem not found"),
        Err(error) => return Err(error.into()),
    };

    json_response(StatusCode::OK, &problem)
}

async fn create_problem<R>(
    request: &Request,
    repository: &R,
    assets_bucket: &str,
) -> Result<Response<Body>, Error>
where
    R: ProblemAssetRepository,
{
    let body = match body_text(request) {
        Ok(body) => body,
        Err(response) => return Ok(response),
    };
    let payload = match serde_json::from_str::<CreateProblemRequest>(&body) {
        Ok(payload) => payload,
        Err(_) => return bad_request("invalid JSON body"),
    };

    if let Err(message) = validate_create_problem(&payload) {
        return bad_request(message);
    }

    let problem = ProblemMeta {
        statement_location: format!(
            "s3://{assets_bucket}/problems/{}/statement.md",
            payload.problem_id
        ),
        problem_id: payload.problem_id,
        title: payload.title.trim().to_string(),
        difficulty: payload.difficulty,
        tags: payload
            .tags
            .into_iter()
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect(),
        time_limit_ms: payload.time_limit_ms,
        memory_limit_mb: payload.memory_limit_mb,
        visibility: payload.visibility.unwrap_or(ProblemVisibility::Draft),
        bundle_key: None,
        bundle_hash: None,
        checker_key: None,
        checker_hash: None,
        problem_version: 1,
        manifest_version: None,
    };
    let created = repository.create_problem(problem).await?;

    json_response(StatusCode::CREATED, &created)
}

async fn presign_problem_asset<R>(
    request: &Request,
    repository: &R,
    assets_bucket: &str,
    problem_id: &str,
) -> Result<Response<Body>, Error>
where
    R: ProblemRepository + AssetUploadRepository,
{
    if !is_safe_segment(problem_id) {
        return bad_request("invalid problem id");
    }

    let problem = match repository.get_problem(problem_id).await {
        Ok(problem) => problem,
        Err(RepositoryError::NotFound(_)) => return not_found("problem not found"),
        Err(error) => return Err(error.into()),
    };
    let body = match body_text(request) {
        Ok(body) => body,
        Err(response) => return Ok(response),
    };
    let payload = match serde_json::from_str::<PresignRequest>(&body) {
        Ok(payload) => payload,
        Err(_) => return bad_request("invalid JSON body"),
    };
    let next_manifest_version = problem.manifest_version.unwrap_or(0) + 1;
    let key = match asset_key(problem_id, next_manifest_version, &payload) {
        Ok(key) => key,
        Err(response) => return Ok(response),
    };
    let content_type = payload
        .content_type
        .as_deref()
        .unwrap_or_else(|| default_content_type(&payload.asset_type));
    let upload = repository
        .presign_put_object(
            assets_bucket,
            &key,
            content_type,
            PRESIGN_EXPIRES_IN_SECONDS,
        )
        .await?;

    json_response(StatusCode::OK, &upload)
}

async fn finalize_problem_bundle<R>(
    request: &Request,
    repository: &R,
    assets_bucket: &str,
    problem_id: &str,
) -> Result<Response<Body>, Error>
where
    R: ProblemRepository + AssetUploadRepository + ProblemAssetRepository,
{
    if !is_safe_segment(problem_id) {
        return bad_request("invalid problem id");
    }

    let problem = match repository.get_problem(problem_id).await {
        Ok(problem) => problem,
        Err(RepositoryError::NotFound(_)) => return not_found("problem not found"),
        Err(error) => return Err(error.into()),
    };
    let body = match body_text(request) {
        Ok(body) => body,
        Err(response) => return Ok(response),
    };
    let payload = match serde_json::from_str::<FinalizeBundleRequest>(&body) {
        Ok(payload) => payload,
        Err(_) => return bad_request("invalid JSON body"),
    };

    let next_manifest_version = problem.manifest_version.unwrap_or(0) + 1;
    let next_problem_version = problem.problem_version + 1;
    let expected_bundle_key =
        format!("problems/{problem_id}/bundles/tests-v{next_manifest_version}.zip");

    if payload.bundle_key != expected_bundle_key {
        return bad_request("bundle_key does not match next manifest version");
    }
    if let Err(message) = validate_hash("bundle_hash", &payload.bundle_hash) {
        return bad_request(message);
    }
    if let Err(message) = validate_cases(&payload.cases) {
        return bad_request(message);
    }
    if let Err(message) = validate_checker(problem_id, next_manifest_version, &payload) {
        return bad_request(message);
    }

    let object = match repository
        .head_object(assets_bucket, &payload.bundle_key)
        .await
    {
        Ok(object) => object,
        Err(RepositoryError::NotFound(_)) => return bad_request("bundle object not found"),
        Err(error) => return Err(error.into()),
    };

    if object.size_bytes != payload.bundle_size_bytes {
        return bad_request("bundle_size_bytes does not match S3 object size");
    }

    let manifest = ProblemManifest {
        problem_id: problem.problem_id.clone(),
        manifest_version: next_manifest_version,
        bundle_key: payload.bundle_key,
        bundle_hash: payload.bundle_hash,
        bundle_format: "zip".to_string(),
        bundle_size_bytes: payload.bundle_size_bytes,
        case_count: payload.cases.len() as u32,
        cases: payload.cases,
        checker_key: payload.checker_key,
        checker_hash: payload.checker_hash,
        problem_version: next_problem_version,
    };
    let finalized = repository
        .finalize_problem_bundle(problem, manifest)
        .await?;

    json_response(
        StatusCode::OK,
        &FinalizeBundleResponse {
            problem_id: finalized.problem.problem_id,
            problem_version: finalized.problem.problem_version,
            manifest_version: finalized.manifest.manifest_version,
            bundle_key: finalized.manifest.bundle_key,
            bundle_hash: finalized.manifest.bundle_hash,
        },
    )
}

fn asset_key(
    problem_id: &str,
    next_manifest_version: u32,
    payload: &PresignRequest,
) -> Result<String, Response<Body>> {
    match payload.asset_type {
        AssetType::Statement => Ok(format!("problems/{problem_id}/statement.md")),
        AssetType::SampleInput => {
            let sample_id = required_safe_sample_id(payload)?;
            Ok(format!("problems/{problem_id}/samples/{sample_id}.in"))
        }
        AssetType::SampleOutput => {
            let sample_id = required_safe_sample_id(payload)?;
            Ok(format!("problems/{problem_id}/samples/{sample_id}.out"))
        }
        AssetType::Bundle => Ok(format!(
            "problems/{problem_id}/bundles/tests-v{next_manifest_version}.zip"
        )),
        AssetType::Checker => Ok(format!(
            "problems/{problem_id}/checker/checker-v{next_manifest_version}"
        )),
    }
}

fn required_safe_sample_id(payload: &PresignRequest) -> Result<&str, Response<Body>> {
    let sample_id = payload
        .sample_id
        .as_deref()
        .ok_or_else(|| error_response(StatusCode::BAD_REQUEST, "sample_id is required"))?;

    if !is_safe_segment(sample_id) {
        return Err(error_response(StatusCode::BAD_REQUEST, "invalid sample_id"));
    }

    Ok(sample_id)
}

fn default_content_type(asset_type: &AssetType) -> &'static str {
    match asset_type {
        AssetType::Statement => "text/markdown",
        AssetType::SampleInput | AssetType::SampleOutput => "text/plain",
        AssetType::Bundle => "application/zip",
        AssetType::Checker => "application/octet-stream",
    }
}

fn validate_hash(attribute: &'static str, value: &str) -> Result<(), &'static str> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(match attribute {
            "checker_hash" => "checker_hash must start with sha256:",
            _ => "bundle_hash must start with sha256:",
        });
    };

    if hex.len() != 64 || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(match attribute {
            "checker_hash" => "checker_hash must contain 64 hex characters",
            _ => "bundle_hash must contain 64 hex characters",
        });
    }

    Ok(())
}

fn validate_create_problem(payload: &CreateProblemRequest) -> Result<(), &'static str> {
    if !is_safe_segment(&payload.problem_id) {
        return Err("invalid problem_id");
    }
    if payload.title.trim().is_empty() {
        return Err("title is required");
    }
    if payload.time_limit_ms < 100 {
        return Err("time_limit_ms must be at least 100");
    }
    if payload.memory_limit_mb < 16 {
        return Err("memory_limit_mb must be at least 16");
    }
    if payload
        .tags
        .iter()
        .any(|tag| !tag.trim().is_empty() && !is_safe_tag(tag.trim()))
    {
        return Err("tags must contain only letters, numbers, dash, underscore, or plus");
    }

    Ok(())
}

fn is_safe_tag(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '+'))
}

fn validate_cases(cases: &[ManifestCase]) -> Result<(), &'static str> {
    if cases.is_empty() {
        return Err("cases must not be empty");
    }

    let mut ids = HashSet::new();
    let mut weight_sum = 0u32;
    for case in cases {
        if case.id == 0 || !ids.insert(case.id) {
            return Err("cases[].id must be unique and greater than 0");
        }
        if case.weight == 0 {
            return Err("cases[].weight must be greater than 0");
        }
        weight_sum = weight_sum.saturating_add(case.weight);
        if !is_safe_relative_path(&case.input_path) || !is_safe_relative_path(&case.output_path) {
            return Err("case paths must be safe relative paths");
        }
    }

    if weight_sum != 100 {
        return Err("case weights must sum to 100");
    }

    Ok(())
}

fn validate_checker(
    problem_id: &str,
    next_manifest_version: u32,
    payload: &FinalizeBundleRequest,
) -> Result<(), &'static str> {
    match (&payload.checker_key, &payload.checker_hash) {
        (None, None) => Ok(()),
        (Some(checker_key), Some(checker_hash)) => {
            let expected_checker_key =
                format!("problems/{problem_id}/checker/checker-v{next_manifest_version}");
            if checker_key != &expected_checker_key {
                return Err("checker_key does not match next manifest version");
            }
            validate_hash("checker_hash", checker_hash)
        }
        _ => Err("checker_key and checker_hash must be provided together"),
    }
}

fn is_safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && value
            .split('/')
            .all(|segment| is_safe_segment(segment) || safe_file_name(segment))
}

fn safe_file_name(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

fn is_safe_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn auth_context(request: &Request) -> Option<AuthContext> {
    let claims = request
        .request_context_ref()
        .and_then(|context| context.authorizer())
        .and_then(|authorizer| authorizer.jwt.as_ref())
        .map(|jwt| &jwt.claims);

    let user_id = claims
        .and_then(|claims| claims.get("sub"))
        .cloned()
        .or_else(|| header(request, "x-mwt-user-id"))?;
    let email = claims
        .and_then(|claims| claims.get("email"))
        .cloned()
        .or_else(|| header(request, "x-mwt-email"));
    let groups = claims
        .and_then(|claims| claims.get("cognito:groups"))
        .cloned()
        .or_else(|| header(request, "x-mwt-groups"))
        .map(|value| parse_groups(&value))
        .unwrap_or_default();

    Some(AuthContext {
        user_id,
        email,
        groups,
    })
}

fn require_admin(request: &Request) -> Option<Result<Response<Body>, Error>> {
    match auth_context(request) {
        Some(auth) if auth.is_admin() => None,
        Some(_) => Some(forbidden("admin access required")),
        None => Some(unauthorized("authentication required")),
    }
}

fn header(request: &Request, name: &str) -> Option<String> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn parse_groups(value: &str) -> Vec<String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Vec::new();
    }

    if let Ok(groups) = serde_json::from_str::<Vec<String>>(trimmed) {
        return groups
            .into_iter()
            .map(|group| group.trim().to_string())
            .filter(|group| !group.is_empty())
            .collect();
    }

    trimmed
        .trim_matches(['[', ']'])
        .split(',')
        .map(|group| group.trim().trim_matches('"').to_string())
        .filter(|group| !group.is_empty())
        .collect()
}

fn body_text(request: &Request) -> Result<String, Response<Body>> {
    match request.body() {
        Body::Text(text) => Ok(text.clone()),
        Body::Binary(bytes) => String::from_utf8(bytes.clone()).map_err(|_| {
            error_response(StatusCode::BAD_REQUEST, "request body must be UTF-8 JSON")
        }),
        Body::Empty => Err(error_response(
            StatusCode::BAD_REQUEST,
            "request body is required",
        )),
    }
}

fn not_found(message: &str) -> Result<Response<Body>, Error> {
    json_response(
        StatusCode::NOT_FOUND,
        &ErrorBody {
            message: message.to_string(),
        },
    )
}

fn unauthorized(message: &str) -> Result<Response<Body>, Error> {
    json_response(
        StatusCode::UNAUTHORIZED,
        &ErrorBody {
            message: message.to_string(),
        },
    )
}

fn forbidden(message: &str) -> Result<Response<Body>, Error> {
    json_response(
        StatusCode::FORBIDDEN,
        &ErrorBody {
            message: message.to_string(),
        },
    )
}

fn bad_request(message: &str) -> Result<Response<Body>, Error> {
    json_response(
        StatusCode::BAD_REQUEST,
        &ErrorBody {
            message: message.to_string(),
        },
    )
}

fn error_response(status: StatusCode, message: &str) -> Response<Body> {
    json_response(
        status,
        &ErrorBody {
            message: message.to_string(),
        },
    )
    .expect("error response should serialize")
}

fn json_response<T>(status: StatusCode, body: &T) -> Result<Response<Body>, Error>
where
    T: Serialize,
{
    let body = serde_json::to_string(body)?;

    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::Text(body))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::http::{HeaderName, HeaderValue, Method};
    use mwt_infra::memory::MemoryRepository;

    #[tokio::test]
    async fn lists_admin_problems_for_admin() {
        let request = request(
            Method::GET,
            "/admin/problems",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            "",
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text_from_response(response);
        assert!(body.contains(r#""problem_id":"sum-path""#));
        assert!(body.contains(r#""visibility":"public""#));
    }

    #[tokio::test]
    async fn gets_admin_problem_for_admin() {
        let request = request(
            Method::GET,
            "/admin/problems/sum-path",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            "",
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text_from_response(response);
        assert!(body.contains(r#""problem_id":"sum-path""#));
        assert!(body.contains(r#""manifest_version":1"#));
    }

    #[tokio::test]
    async fn presigns_statement_upload_for_admin() {
        let request = request(
            Method::POST,
            "/admin/problems/sum-path/assets/presign",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            r#"{"asset_type":"statement"}"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text_from_response(response);
        assert!(body.contains("problems/sum-path/statement.md"));
        assert!(body.contains("mwt-assets-test"));
    }

    #[tokio::test]
    async fn presigns_sample_upload_with_safe_sample_id() {
        let request = request(
            Method::POST,
            "/admin/problems/sum-path/assets/presign",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            r#"{"asset_type":"sample_input","sample_id":"001"}"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_text_from_response(response).contains("problems/sum-path/samples/001.in"));
    }

    #[tokio::test]
    async fn rejects_presign_without_admin_group() {
        let request = request(
            Method::POST,
            "/admin/problems/sum-path/assets/presign",
            &[("x-mwt-user-id", "normal-user")],
            r#"{"asset_type":"statement"}"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_unsafe_sample_id() {
        let request = request(
            Method::POST,
            "/admin/problems/sum-path/assets/presign",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            r#"{"asset_type":"sample_output","sample_id":"../001"}"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn creates_problem_for_admin() {
        let request = request(
            Method::POST,
            "/admin/problems",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            r#"{
              "problem_id":"two-sum",
              "title":"Two Sum",
              "difficulty":"easy",
              "tags":["array","hash-map"],
              "time_limit_ms":1000,
              "memory_limit_mb":128
            }"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = body_text_from_response(response);
        assert!(body.contains(r#""problem_id":"two-sum""#));
        assert!(body.contains("s3://mwt-assets-test/problems/two-sum/statement.md"));
        assert!(body.contains(r#""visibility":"draft""#));
        assert!(body.contains(r#""problem_version":1"#));
    }

    #[tokio::test]
    async fn rejects_problem_create_with_unsafe_id() {
        let request = request(
            Method::POST,
            "/admin/problems",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            r#"{
              "problem_id":"../two-sum",
              "title":"Two Sum",
              "difficulty":"easy",
              "tags":["array"],
              "time_limit_ms":1000,
              "memory_limit_mb":128
            }"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn parses_group_claim_variants() {
        assert_eq!(parse_groups("admin"), vec!["admin"]);
        assert_eq!(
            parse_groups("admin,participant"),
            vec!["admin", "participant"]
        );
        assert_eq!(parse_groups(r#"["admin"]"#), vec!["admin"]);
        assert_eq!(
            parse_groups(r#"["admin","participant"]"#),
            vec!["admin", "participant"]
        );
    }

    #[tokio::test]
    async fn finalizes_bundle_for_admin() {
        let request = request(
            Method::POST,
            "/admin/problems/sum-path/bundle/finalize",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            r#"{
              "bundle_key":"problems/sum-path/bundles/tests-v2.zip",
              "bundle_hash":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
              "bundle_size_bytes":1024,
              "cases":[
                {"id":1,"input_path":"tests/001.in","output_path":"tests/001.out","weight":100}
              ],
              "checker_key":null,
              "checker_hash":null
            }"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text_from_response(response);
        assert!(body.contains(r#""manifest_version":2"#));
        assert!(body.contains(r#""problem_version":2"#));
        assert!(body.contains("tests-v2.zip"));
    }

    #[tokio::test]
    async fn rejects_finalize_when_weights_do_not_sum_to_100() {
        let request = request(
            Method::POST,
            "/admin/problems/sum-path/bundle/finalize",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            r#"{
              "bundle_key":"problems/sum-path/bundles/tests-v2.zip",
              "bundle_hash":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
              "bundle_size_bytes":1024,
              "cases":[
                {"id":1,"input_path":"tests/001.in","output_path":"tests/001.out","weight":50}
              ],
              "checker_key":null,
              "checker_hash":null
            }"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_finalize_for_wrong_bundle_version() {
        let request = request(
            Method::POST,
            "/admin/problems/sum-path/bundle/finalize",
            &[("x-mwt-user-id", "admin-user"), ("x-mwt-groups", "admin")],
            r#"{
              "bundle_key":"problems/sum-path/bundles/tests-v1.zip",
              "bundle_hash":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
              "bundle_size_bytes":1024,
              "cases":[
                {"id":1,"input_path":"tests/001.in","output_path":"tests/001.out","weight":100}
              ],
              "checker_key":null,
              "checker_hash":null
            }"#,
        );

        let response = handle_request(
            request,
            MemoryRepository::default(),
            "mwt-assets-test".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    fn request(method: Method, uri: &str, headers: &[(&str, &str)], body: &str) -> Request {
        let mut request = Request::new(Body::Text(body.to_string()));
        *request.method_mut() = method;
        *request.uri_mut() = uri.parse().unwrap();

        for (name, value) in headers {
            request.headers_mut().insert(
                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }

        request
    }

    fn body_text_from_response(response: Response<Body>) -> String {
        match response.into_body() {
            Body::Text(text) => text,
            other => panic!("unexpected body: {other:?}"),
        }
    }
}
