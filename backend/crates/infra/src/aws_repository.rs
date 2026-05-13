use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client as S3Client;
use mwt_domain::problem::{ProblemManifest, ProblemMeta};
use mwt_domain::submission::{SubmissionDetail, SubmissionMeta};
use serde_json::{Map, Number, Value};

use crate::dynamodb_item::{
    problem_manifest_from_item, problem_manifest_to_item, problem_meta_from_item,
    problem_meta_to_item, submission_meta_from_item, submission_result_summary_from_item,
    DynamoItem,
};
use crate::dynamodb_keys::{
    problem_manifest_sk, problem_meta_sk, problem_pk, submission_meta_sk, submission_pk,
    user_submissions_pk, CORE_TABLE_NAME, USER_SUBMISSIONS_INDEX,
};
use crate::repository::{
    AssetUploadRepository, FinalizedProblemBundle, ObjectMetadata, PresignedUpload,
    ProblemAssetRepository, ProblemRepository, RepositoryError, RepositoryResult,
    StatementRepository, SubmissionRepository,
};
use crate::s3_location::S3Location;

#[derive(Debug, Clone)]
pub struct AwsRepository {
    dynamodb: DynamoDbClient,
    s3: S3Client,
    table_name: String,
}

impl AwsRepository {
    pub fn new(dynamodb: DynamoDbClient, s3: S3Client, table_name: impl Into<String>) -> Self {
        Self {
            dynamodb,
            s3,
            table_name: table_name.into(),
        }
    }

    pub fn with_default_table(dynamodb: DynamoDbClient, s3: S3Client) -> Self {
        Self::new(dynamodb, s3, CORE_TABLE_NAME)
    }

    async fn get_item(&self, pk: String, sk: String) -> RepositoryResult<Option<DynamoItem>> {
        let output = self
            .dynamodb
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S(pk))
            .key("sk", AttributeValue::S(sk))
            .send()
            .await
            .map_err(storage_error)?;

        output.item.map(attribute_map_to_item).transpose()
    }

    async fn get_required_item(&self, pk: String, sk: String) -> RepositoryResult<DynamoItem> {
        self.get_item(pk.clone(), sk.clone())
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("{pk}:{sk}")))
    }
}

#[async_trait]
impl ProblemRepository for AwsRepository {
    async fn list_public_problems(&self) -> RepositoryResult<Vec<ProblemMeta>> {
        let output = self
            .dynamodb
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type AND visibility = :visibility")
            .expression_attribute_values(
                ":entity_type",
                AttributeValue::S("problem_meta".to_string()),
            )
            .expression_attribute_values(":visibility", AttributeValue::S("public".to_string()))
            .send()
            .await
            .map_err(storage_error)?;

        output
            .items
            .unwrap_or_default()
            .into_iter()
            .map(attribute_map_to_item)
            .map(|result| result.and_then(|item| map_item(item, problem_meta_from_item)))
            .collect()
    }

    async fn list_all_problems(&self) -> RepositoryResult<Vec<ProblemMeta>> {
        let output = self
            .dynamodb
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(
                ":entity_type",
                AttributeValue::S("problem_meta".to_string()),
            )
            .send()
            .await
            .map_err(storage_error)?;

        output
            .items
            .unwrap_or_default()
            .into_iter()
            .map(attribute_map_to_item)
            .map(|result| result.and_then(|item| map_item(item, problem_meta_from_item)))
            .collect()
    }

    async fn get_problem(&self, problem_id: &str) -> RepositoryResult<ProblemMeta> {
        let item = self
            .get_required_item(problem_pk(problem_id), problem_meta_sk().to_string())
            .await?;

        map_item(item, problem_meta_from_item)
    }

    async fn get_manifest(
        &self,
        problem_id: &str,
        manifest_version: u32,
    ) -> RepositoryResult<ProblemManifest> {
        let item = self
            .get_required_item(
                problem_pk(problem_id),
                problem_manifest_sk(manifest_version),
            )
            .await?;

        map_item(item, problem_manifest_from_item)
    }
}

#[async_trait]
impl StatementRepository for AwsRepository {
    async fn get_statement_markdown(&self, statement_location: &str) -> RepositoryResult<String> {
        let location = S3Location::parse(statement_location)
            .map_err(|error| RepositoryError::Storage(error.to_string()))?;
        let output = self
            .s3
            .get_object()
            .bucket(location.bucket)
            .key(location.key)
            .send()
            .await
            .map_err(storage_error)?;
        let bytes = output
            .body
            .collect()
            .await
            .map_err(storage_error)?
            .into_bytes();

        String::from_utf8(bytes.to_vec())
            .map_err(|error| RepositoryError::Storage(error.to_string()))
    }
}

#[async_trait]
impl AssetUploadRepository for AwsRepository {
    async fn presign_put_object(
        &self,
        bucket: &str,
        key: &str,
        content_type: &str,
        expires_in_seconds: u64,
    ) -> RepositoryResult<PresignedUpload> {
        let presigning_config =
            PresigningConfig::expires_in(Duration::from_secs(expires_in_seconds))
                .map_err(storage_error)?;
        let request = self
            .s3
            .put_object()
            .bucket(bucket)
            .key(key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(storage_error)?;

        Ok(PresignedUpload {
            bucket: bucket.to_string(),
            key: key.to_string(),
            upload_url: request.uri().to_string(),
            expires_in_seconds,
        })
    }

    async fn head_object(&self, bucket: &str, key: &str) -> RepositoryResult<ObjectMetadata> {
        let output = self
            .s3
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(storage_error)?;
        let content_length = output.content_length().ok_or_else(|| {
            RepositoryError::Storage("S3 object has no content length".to_string())
        })?;

        u64::try_from(content_length)
            .map(|size_bytes| ObjectMetadata { size_bytes })
            .map_err(|error| RepositoryError::Storage(error.to_string()))
    }
}

#[async_trait]
impl ProblemAssetRepository for AwsRepository {
    async fn create_problem(&self, problem: ProblemMeta) -> RepositoryResult<ProblemMeta> {
        let item = item_to_attribute_map(
            problem_meta_to_item(&problem)
                .map_err(|error| RepositoryError::Storage(error.to_string()))?,
        )?;

        self.dynamodb
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(pk)")
            .send()
            .await
            .map_err(storage_error)?;

        Ok(problem)
    }

    async fn finalize_problem_bundle(
        &self,
        mut problem: ProblemMeta,
        manifest: ProblemManifest,
    ) -> RepositoryResult<FinalizedProblemBundle> {
        problem.bundle_key = Some(manifest.bundle_key.clone());
        problem.bundle_hash = Some(manifest.bundle_hash.clone());
        problem.checker_key = manifest.checker_key.clone();
        problem.checker_hash = manifest.checker_hash.clone();
        problem.problem_version = manifest.problem_version;
        problem.manifest_version = Some(manifest.manifest_version);

        let manifest_attributes = item_to_attribute_map(
            problem_manifest_to_item(&manifest)
                .map_err(|error| RepositoryError::Storage(error.to_string()))?,
        )?;
        let problem_attributes = item_to_attribute_map(
            problem_meta_to_item(&problem)
                .map_err(|error| RepositoryError::Storage(error.to_string()))?,
        )?;

        self.dynamodb
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(manifest_attributes))
            .send()
            .await
            .map_err(storage_error)?;

        self.dynamodb
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(problem_attributes))
            .send()
            .await
            .map_err(storage_error)?;

        Ok(FinalizedProblemBundle { problem, manifest })
    }
}

#[async_trait]
impl SubmissionRepository for AwsRepository {
    async fn get_submission(&self, submission_id: &str) -> RepositoryResult<SubmissionDetail> {
        let submission_item = self
            .get_required_item(
                submission_pk(submission_id),
                submission_meta_sk().to_string(),
            )
            .await?;
        let submission = map_item(submission_item, submission_meta_from_item)?;

        let result = self
            .get_item(submission_pk(submission_id), "RESULT#SUMMARY".to_string())
            .await?
            .map(|item| map_item(item, submission_result_summary_from_item))
            .transpose()?;

        Ok(SubmissionDetail { submission, result })
    }

    async fn list_user_submissions(&self, user_id: &str) -> RepositoryResult<Vec<SubmissionMeta>> {
        let output = self
            .dynamodb
            .query()
            .table_name(&self.table_name)
            .index_name(USER_SUBMISSIONS_INDEX)
            .key_condition_expression("gsi1_pk = :user_pk")
            .expression_attribute_values(
                ":user_pk",
                AttributeValue::S(user_submissions_pk(user_id)),
            )
            .scan_index_forward(false)
            .send()
            .await
            .map_err(storage_error)?;

        output
            .items
            .unwrap_or_default()
            .into_iter()
            .map(attribute_map_to_item)
            .map(|result| result.and_then(|item| map_item(item, submission_meta_from_item)))
            .collect()
    }
}

fn attribute_map_to_item(
    attributes: HashMap<String, AttributeValue>,
) -> RepositoryResult<DynamoItem> {
    attributes
        .into_iter()
        .map(|(key, value)| Ok((key, attribute_value_to_json(value)?)))
        .collect()
}

fn attribute_value_to_json(value: AttributeValue) -> RepositoryResult<Value> {
    match value {
        AttributeValue::S(value) => Ok(Value::String(value)),
        AttributeValue::N(value) => number_from_string(&value),
        AttributeValue::Bool(value) => Ok(Value::Bool(value)),
        AttributeValue::Null(_) => Ok(Value::Null),
        AttributeValue::L(values) => values
            .into_iter()
            .map(attribute_value_to_json)
            .collect::<RepositoryResult<Vec<_>>>()
            .map(Value::Array),
        AttributeValue::M(values) => values
            .into_iter()
            .map(|(key, value)| Ok((key, attribute_value_to_json(value)?)))
            .collect::<RepositoryResult<Map<String, Value>>>()
            .map(Value::Object),
        other => Err(RepositoryError::Storage(format!(
            "unsupported DynamoDB attribute value: {other:?}"
        ))),
    }
}

pub fn item_to_attribute_map(
    item: DynamoItem,
) -> RepositoryResult<HashMap<String, AttributeValue>> {
    item.into_iter()
        .map(|(key, value)| Ok((key, json_to_attribute_value(value)?)))
        .collect()
}

fn json_to_attribute_value(value: Value) -> RepositoryResult<AttributeValue> {
    match value {
        Value::Null => Ok(AttributeValue::Null(true)),
        Value::Bool(value) => Ok(AttributeValue::Bool(value)),
        Value::Number(value) => Ok(AttributeValue::N(value.to_string())),
        Value::String(value) => Ok(AttributeValue::S(value)),
        Value::Array(values) => values
            .into_iter()
            .map(json_to_attribute_value)
            .collect::<RepositoryResult<Vec<_>>>()
            .map(AttributeValue::L),
        Value::Object(values) => values
            .into_iter()
            .map(|(key, value)| Ok((key, json_to_attribute_value(value)?)))
            .collect::<RepositoryResult<HashMap<_, _>>>()
            .map(AttributeValue::M),
    }
}

fn number_from_string(value: &str) -> RepositoryResult<Value> {
    if let Ok(number) = value.parse::<i64>() {
        return Ok(Value::Number(Number::from(number)));
    }
    if let Ok(number) = value.parse::<u64>() {
        return Ok(Value::Number(Number::from(number)));
    }
    value
        .parse::<f64>()
        .ok()
        .and_then(Number::from_f64)
        .map(Value::Number)
        .ok_or_else(|| RepositoryError::Storage(format!("invalid DynamoDB number: {value}")))
}

fn map_item<T>(
    item: DynamoItem,
    mapper: fn(&DynamoItem) -> Result<T, crate::dynamodb_item::DynamoItemError>,
) -> RepositoryResult<T> {
    mapper(&item).map_err(|error| RepositoryError::Storage(error.to_string()))
}

fn storage_error(error: impl std::fmt::Debug) -> RepositoryError {
    RepositoryError::Storage(format!("{error:?}"))
}

#[cfg(test)]
mod tests {
    use mwt_domain::problem::{Difficulty, ProblemMeta, ProblemVisibility};

    use crate::dynamodb_item::{problem_meta_from_item, problem_meta_to_item};

    use super::*;

    #[test]
    fn converts_item_to_attribute_values_and_back() {
        let problem = ProblemMeta {
            problem_id: "sum-path".to_string(),
            title: "합 경로".to_string(),
            difficulty: Difficulty::Easy,
            tags: vec!["prefix-sum".to_string()],
            time_limit_ms: 1000,
            memory_limit_mb: 128,
            visibility: ProblemVisibility::Public,
            statement_location:
                "s3://mwt-assets-prod-123456789012-ap-northeast-2-example/problems/sum-path/statement.md"
                    .to_string(),
            bundle_key: Some("problems/sum-path/bundles/tests-v1.zip".to_string()),
            bundle_hash: Some("sha256:bundle".to_string()),
            checker_key: None,
            checker_hash: None,
            problem_version: 1,
            manifest_version: Some(1),
        };

        let item = problem_meta_to_item(&problem).unwrap();
        let attributes = item_to_attribute_map(item).unwrap();
        let roundtrip_item = attribute_map_to_item(attributes).unwrap();

        assert_eq!(problem_meta_from_item(&roundtrip_item).unwrap(), problem);
    }
}
