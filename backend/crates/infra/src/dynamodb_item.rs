use std::collections::BTreeMap;

use mwt_domain::problem::{ProblemManifest, ProblemMeta};
use mwt_domain::submission::{SubmissionMeta, SubmissionResultSummary};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Map, Value};
use thiserror::Error;

use crate::dynamodb_keys::{
    problem_manifest_sk, problem_meta_sk, problem_pk, submission_meta_sk, submission_pk,
    submissions_by_status_pk, submissions_by_status_sk, user_submissions_pk, user_submissions_sk,
};

pub type DynamoItem = BTreeMap<String, Value>;

#[derive(Debug, Error)]
pub enum DynamoItemError {
    #[error("missing attribute: {0}")]
    MissingAttribute(&'static str),
    #[error("invalid attribute {attribute}: {message}")]
    InvalidAttribute {
        attribute: &'static str,
        message: String,
    },
}

pub type DynamoItemResult<T> = Result<T, DynamoItemError>;

pub fn problem_meta_to_item(problem: &ProblemMeta) -> DynamoItemResult<DynamoItem> {
    let mut item = model_to_item(problem)?;
    item.insert("pk".to_string(), json!(problem_pk(&problem.problem_id)));
    item.insert("sk".to_string(), json!(problem_meta_sk()));
    item.insert("entity_type".to_string(), json!("problem_meta"));

    Ok(item)
}

pub fn problem_meta_from_item(item: &DynamoItem) -> DynamoItemResult<ProblemMeta> {
    item_to_model(item)
}

pub fn problem_manifest_to_item(manifest: &ProblemManifest) -> DynamoItemResult<DynamoItem> {
    let mut item = model_to_item(manifest)?;
    item.insert("pk".to_string(), json!(problem_pk(&manifest.problem_id)));
    item.insert(
        "sk".to_string(),
        json!(problem_manifest_sk(manifest.manifest_version)),
    );
    item.insert("entity_type".to_string(), json!("problem_manifest"));

    Ok(item)
}

pub fn problem_manifest_from_item(item: &DynamoItem) -> DynamoItemResult<ProblemManifest> {
    item_to_model(item)
}

pub fn submission_meta_to_item(submission: &SubmissionMeta) -> DynamoItemResult<DynamoItem> {
    let status = enum_to_snake_string(&submission.status)?;
    let mut item = model_to_item(submission)?;

    item.insert(
        "pk".to_string(),
        json!(submission_pk(&submission.submission_id)),
    );
    item.insert("sk".to_string(), json!(submission_meta_sk()));
    item.insert("entity_type".to_string(), json!("submission_meta"));
    item.insert(
        "gsi1_pk".to_string(),
        json!(user_submissions_pk(&submission.user_id)),
    );
    item.insert(
        "gsi1_sk".to_string(),
        json!(user_submissions_sk(
            &submission.submitted_at,
            &submission.submission_id
        )),
    );
    item.insert(
        "gsi2_pk".to_string(),
        json!(submissions_by_status_pk(&status)),
    );
    item.insert(
        "gsi2_sk".to_string(),
        json!(submissions_by_status_sk(
            &submission.submitted_at,
            &submission.submission_id
        )),
    );

    Ok(item)
}

pub fn submission_meta_from_item(item: &DynamoItem) -> DynamoItemResult<SubmissionMeta> {
    item_to_model(item)
}

pub fn submission_result_summary_to_item(
    result: &SubmissionResultSummary,
) -> DynamoItemResult<DynamoItem> {
    let mut item = model_to_item(result)?;
    item.insert(
        "pk".to_string(),
        json!(submission_pk(&result.submission_id)),
    );
    item.insert("sk".to_string(), json!("RESULT#SUMMARY"));
    item.insert(
        "entity_type".to_string(),
        json!("submission_result_summary"),
    );

    Ok(item)
}

pub fn submission_result_summary_from_item(
    item: &DynamoItem,
) -> DynamoItemResult<SubmissionResultSummary> {
    item_to_model(item)
}

fn model_to_item<T>(model: &T) -> DynamoItemResult<DynamoItem>
where
    T: Serialize,
{
    let value = serde_json::to_value(model).map_err(|error| DynamoItemError::InvalidAttribute {
        attribute: "model",
        message: error.to_string(),
    })?;
    let object = value.as_object().ok_or(DynamoItemError::InvalidAttribute {
        attribute: "model",
        message: "model did not serialize to an object".to_string(),
    })?;

    Ok(object
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect())
}

fn item_to_model<T>(item: &DynamoItem) -> DynamoItemResult<T>
where
    T: DeserializeOwned,
{
    let object = item
        .iter()
        .filter(|(key, _)| !is_storage_key(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Map<String, Value>>();

    serde_json::from_value(Value::Object(object)).map_err(|error| {
        DynamoItemError::InvalidAttribute {
            attribute: "item",
            message: error.to_string(),
        }
    })
}

fn enum_to_snake_string<T>(value: &T) -> DynamoItemResult<String>
where
    T: Serialize,
{
    serde_json::to_value(value)
        .map_err(|error| DynamoItemError::InvalidAttribute {
            attribute: "enum",
            message: error.to_string(),
        })?
        .as_str()
        .map(str::to_string)
        .ok_or(DynamoItemError::InvalidAttribute {
            attribute: "enum",
            message: "enum did not serialize to a string".to_string(),
        })
}

fn is_storage_key(key: &str) -> bool {
    matches!(
        key,
        "pk" | "sk" | "entity_type" | "gsi1_pk" | "gsi1_sk" | "gsi2_pk" | "gsi2_sk"
    )
}

#[cfg(test)]
mod tests {
    use mwt_domain::problem::{
        Difficulty, ManifestCase, ProblemManifest, ProblemMeta, ProblemVisibility,
    };
    use mwt_domain::submission::{SubmissionMeta, SubmissionResultSummary, SubmissionStatus};

    use super::*;

    #[test]
    fn maps_problem_meta_keys_and_roundtrips() {
        let problem = sample_problem();
        let item = problem_meta_to_item(&problem).unwrap();

        assert_eq!(item.get("pk"), Some(&json!("PROBLEM#sum-path")));
        assert_eq!(item.get("sk"), Some(&json!("META")));
        assert_eq!(item.get("entity_type"), Some(&json!("problem_meta")));
        assert_eq!(problem_meta_from_item(&item).unwrap(), problem);
    }

    #[test]
    fn maps_problem_manifest_keys_and_roundtrips() {
        let manifest = ProblemManifest {
            problem_id: "sum-path".to_string(),
            manifest_version: 7,
            bundle_key: "problems/sum-path/bundles/tests-v7.zip".to_string(),
            bundle_hash: "sha256:bundle".to_string(),
            bundle_format: "zip".to_string(),
            bundle_size_bytes: 2048,
            case_count: 1,
            cases: vec![ManifestCase {
                id: 1,
                input_path: "001.in".to_string(),
                output_path: "001.out".to_string(),
                weight: 100,
            }],
            checker_key: None,
            checker_hash: None,
            problem_version: 3,
        };

        let item = problem_manifest_to_item(&manifest).unwrap();

        assert_eq!(item.get("pk"), Some(&json!("PROBLEM#sum-path")));
        assert_eq!(item.get("sk"), Some(&json!("MANIFEST#0000000007")));
        assert_eq!(item.get("entity_type"), Some(&json!("problem_manifest")));
        assert_eq!(problem_manifest_from_item(&item).unwrap(), manifest);
    }

    #[test]
    fn maps_submission_gsi_keys_and_roundtrips() {
        let submission = sample_submission();
        let item = submission_meta_to_item(&submission).unwrap();

        assert_eq!(item.get("pk"), Some(&json!("SUBMISSION#sub-20260429-001")));
        assert_eq!(item.get("sk"), Some(&json!("META")));
        assert_eq!(item.get("entity_type"), Some(&json!("submission_meta")));
        assert_eq!(item.get("gsi1_pk"), Some(&json!("USER#user-001")));
        assert_eq!(
            item.get("gsi1_sk"),
            Some(&json!(
                "SUBMITTED_AT#2026-04-29T09:20:00+09:00#SUBMISSION#sub-20260429-001"
            ))
        );
        assert_eq!(item.get("gsi2_pk"), Some(&json!("STATUS#accepted")));
        assert_eq!(
            item.get("gsi2_sk"),
            Some(&json!(
                "SUBMITTED_AT#2026-04-29T09:20:00+09:00#SUBMISSION#sub-20260429-001"
            ))
        );
        assert_eq!(submission_meta_from_item(&item).unwrap(), submission);
    }

    #[test]
    fn maps_submission_result_summary_and_roundtrips() {
        let result = SubmissionResultSummary {
            submission_id: "sub-20260429-001".to_string(),
            status: SubmissionStatus::Accepted,
            score: Some(100),
            max_score: Some(100),
            runtime_ms: Some(42),
            memory_mb: Some(18),
        };

        let item = submission_result_summary_to_item(&result).unwrap();

        assert_eq!(item.get("pk"), Some(&json!("SUBMISSION#sub-20260429-001")));
        assert_eq!(item.get("sk"), Some(&json!("RESULT#SUMMARY")));
        assert_eq!(
            item.get("entity_type"),
            Some(&json!("submission_result_summary"))
        );
        assert_eq!(submission_result_summary_from_item(&item).unwrap(), result);
    }

    fn sample_problem() -> ProblemMeta {
        ProblemMeta {
            problem_id: "sum-path".to_string(),
            title: "합 경로".to_string(),
            difficulty: Difficulty::Easy,
            tags: vec!["prefix-sum".to_string()],
            time_limit_ms: 1000,
            memory_limit_mb: 128,
            visibility: ProblemVisibility::Public,
            statement_markdown: "# 합 경로\n\n정수 배열의 구간 합을 빠르게 구하라.".to_string(),
            statement_location:
                "s3://mwt-assets-prod-123456789012-ap-northeast-2-example/problems/sum-path/statement.md"
                    .to_string(),
            allowed_languages: vec!["Rust".to_string(), "Python".to_string()],
            bundle_key: Some("problems/sum-path/bundles/tests-v1.zip".to_string()),
            bundle_hash: Some("sha256:bundle".to_string()),
            checker_key: None,
            checker_hash: None,
            problem_version: 1,
            manifest_version: Some(1),
        }
    }

    fn sample_submission() -> SubmissionMeta {
        SubmissionMeta {
            submission_id: "sub-20260429-001".to_string(),
            problem_id: "sum-path".to_string(),
            user_id: "user-001".to_string(),
            language: "Rust".to_string(),
            status: SubmissionStatus::Accepted,
            attempt: 1,
            worker_task_id: Some("task-001".to_string()),
            problem_version: 1,
            manifest_version: 1,
            bundle_hash: "sha256:bundle".to_string(),
            submitted_at: "2026-04-29T09:20:00+09:00".to_string(),
            finalized_at: Some("2026-04-29T09:20:02+09:00".to_string()),
        }
    }
}
