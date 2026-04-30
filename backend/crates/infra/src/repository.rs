use async_trait::async_trait;
use mwt_domain::problem::{ProblemManifest, ProblemMeta};
use mwt_domain::submission::{SubmissionDetail, SubmissionMeta};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("storage error: {0}")]
    Storage(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[async_trait]
pub trait ProblemRepository: Send + Sync {
    async fn list_public_problems(&self) -> RepositoryResult<Vec<ProblemMeta>>;
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
pub trait SubmissionRepository: Send + Sync {
    async fn get_submission(&self, submission_id: &str) -> RepositoryResult<SubmissionDetail>;
    async fn list_user_submissions(&self, user_id: &str) -> RepositoryResult<Vec<SubmissionMeta>>;
}
