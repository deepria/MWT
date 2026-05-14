use async_trait::async_trait;
use mwt_domain::problem::{ProblemManifest, ProblemMeta, ProblemVisibility};
use mwt_domain::submission::{SubmissionDetail, SubmissionMeta};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("storage error: {0}")]
    Storage(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PresignedUpload {
    pub bucket: String,
    pub key: String,
    pub upload_url: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMetadata {
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizedProblemBundle {
    pub problem: ProblemMeta,
    pub manifest: ProblemManifest,
}

#[async_trait]
pub trait ProblemRepository: Send + Sync {
    async fn list_public_problems(&self) -> RepositoryResult<Vec<ProblemMeta>>;
    async fn list_all_problems(&self) -> RepositoryResult<Vec<ProblemMeta>>;
    async fn get_problem(&self, problem_id: &str) -> RepositoryResult<ProblemMeta>;
    async fn get_manifest(
        &self,
        problem_id: &str,
        manifest_version: u32,
    ) -> RepositoryResult<ProblemManifest>;
}

#[async_trait]
pub trait StatementRepository: Send + Sync {
    async fn get_statement_markdown(&self, statement_location: &str) -> RepositoryResult<String>;
}

#[async_trait]
pub trait AssetUploadRepository: Send + Sync {
    async fn presign_put_object(
        &self,
        bucket: &str,
        key: &str,
        content_type: &str,
        expires_in_seconds: u64,
    ) -> RepositoryResult<PresignedUpload>;

    async fn head_object(&self, bucket: &str, key: &str) -> RepositoryResult<ObjectMetadata>;
}

#[async_trait]
pub trait ProblemAssetRepository: Send + Sync {
    async fn create_problem(&self, problem: ProblemMeta) -> RepositoryResult<ProblemMeta>;

    async fn update_problem_visibility(
        &self,
        problem: ProblemMeta,
        visibility: ProblemVisibility,
    ) -> RepositoryResult<ProblemMeta>;

    async fn finalize_problem_bundle(
        &self,
        problem: ProblemMeta,
        manifest: ProblemManifest,
    ) -> RepositoryResult<FinalizedProblemBundle>;
}

#[async_trait]
pub trait SubmissionRepository: Send + Sync {
    async fn get_submission(&self, submission_id: &str) -> RepositoryResult<SubmissionDetail>;
    async fn list_user_submissions(&self, user_id: &str) -> RepositoryResult<Vec<SubmissionMeta>>;
}
