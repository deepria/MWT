use async_trait::async_trait;
use mwt_domain::problem::{
    Difficulty, ManifestCase, ProblemManifest, ProblemMeta, ProblemVisibility,
};
use mwt_domain::submission::{
    SubmissionDetail, SubmissionMeta, SubmissionResultSummary, SubmissionStatus,
};

use crate::repository::{
    AssetUploadRepository, FinalizedProblemBundle, ObjectMetadata, PresignedUpload,
    ProblemAssetRepository, ProblemRepository, RepositoryError, RepositoryResult,
    StatementRepository, SubmissionRepository,
};

#[derive(Debug, Clone)]
pub struct MemoryRepository {
    problems: Vec<ProblemMeta>,
    manifests: Vec<ProblemManifest>,
    submissions: Vec<SubmissionDetail>,
}

impl Default for MemoryRepository {
    fn default() -> Self {
        let problem = ProblemMeta {
            problem_id: "sum-path".to_string(),
            title: "합 경로".to_string(),
            difficulty: Difficulty::Easy,
            tags: vec!["prefix-sum".to_string(), "implementation".to_string()],
            time_limit_ms: 1000,
            memory_limit_mb: 128,
            visibility: ProblemVisibility::Public,
            statement_location:
                "s3://mwt-assets-prod-123456789012-ap-northeast-2-example/problems/sum-path/statement.md"
                    .to_string(),
            bundle_key: Some("problems/sum-path/bundles/tests-v1.zip".to_string()),
            bundle_hash: Some("sha256:mock-bundle".to_string()),
            checker_key: None,
            checker_hash: None,
            problem_version: 1,
            manifest_version: Some(1),
        };

        let manifest = ProblemManifest {
            problem_id: "sum-path".to_string(),
            manifest_version: 1,
            bundle_key: "problems/sum-path/bundles/tests-v1.zip".to_string(),
            bundle_hash: "sha256:mock-bundle".to_string(),
            bundle_format: "zip".to_string(),
            bundle_size_bytes: 1024,
            case_count: 1,
            cases: vec![ManifestCase {
                id: 1,
                input_path: "001.in".to_string(),
                output_path: "001.out".to_string(),
                weight: 100,
            }],
            checker_key: None,
            checker_hash: None,
            problem_version: 1,
        };

        let submission = SubmissionMeta {
            submission_id: "sub-20260429-001".to_string(),
            problem_id: "sum-path".to_string(),
            user_id: "mock-user-001".to_string(),
            language: "Rust".to_string(),
            status: SubmissionStatus::Accepted,
            attempt: 1,
            worker_task_id: Some("local-worker-001".to_string()),
            problem_version: 1,
            manifest_version: 1,
            bundle_hash: "sha256:mock-bundle".to_string(),
            submitted_at: "2026-04-29T09:20:00+09:00".to_string(),
            finalized_at: Some("2026-04-29T09:20:02+09:00".to_string()),
        };

        Self {
            problems: vec![problem],
            manifests: vec![manifest],
            submissions: vec![SubmissionDetail {
                submission: submission.clone(),
                result: Some(SubmissionResultSummary {
                    submission_id: submission.submission_id,
                    status: SubmissionStatus::Accepted,
                    score: Some(100),
                    max_score: Some(100),
                    runtime_ms: Some(42),
                    memory_mb: Some(18),
                }),
            }],
        }
    }
}

#[async_trait]
impl ProblemRepository for MemoryRepository {
    async fn list_public_problems(&self) -> RepositoryResult<Vec<ProblemMeta>> {
        Ok(self
            .problems
            .iter()
            .filter(|problem| problem.visibility == ProblemVisibility::Public)
            .cloned()
            .collect())
    }

    async fn list_all_problems(&self) -> RepositoryResult<Vec<ProblemMeta>> {
        Ok(self.problems.clone())
    }

    async fn get_problem(&self, problem_id: &str) -> RepositoryResult<ProblemMeta> {
        self.problems
            .iter()
            .find(|problem| problem.problem_id == problem_id)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(problem_id.to_string()))
    }

    async fn get_manifest(
        &self,
        problem_id: &str,
        manifest_version: u32,
    ) -> RepositoryResult<ProblemManifest> {
        self.manifests
            .iter()
            .find(|manifest| {
                manifest.problem_id == problem_id && manifest.manifest_version == manifest_version
            })
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(format!("{problem_id}:{manifest_version}")))
    }
}

#[async_trait]
impl StatementRepository for MemoryRepository {
    async fn get_statement_markdown(&self, statement_location: &str) -> RepositoryResult<String> {
        if statement_location.contains("sum-path") {
            Ok("# 합 경로\n\n정수 배열의 구간 합을 빠르게 구하라.".to_string())
        } else {
            Err(RepositoryError::NotFound(statement_location.to_string()))
        }
    }
}

#[async_trait]
impl AssetUploadRepository for MemoryRepository {
    async fn presign_put_object(
        &self,
        bucket: &str,
        key: &str,
        _content_type: &str,
        expires_in_seconds: u64,
    ) -> RepositoryResult<PresignedUpload> {
        Ok(PresignedUpload {
            bucket: bucket.to_string(),
            key: key.to_string(),
            upload_url: format!("https://example.com/{bucket}/{key}"),
            expires_in_seconds,
        })
    }

    async fn head_object(&self, _bucket: &str, key: &str) -> RepositoryResult<ObjectMetadata> {
        if key.contains("missing") {
            return Err(RepositoryError::NotFound(key.to_string()));
        }

        Ok(ObjectMetadata { size_bytes: 1024 })
    }
}

#[async_trait]
impl ProblemAssetRepository for MemoryRepository {
    async fn create_problem(&self, problem: ProblemMeta) -> RepositoryResult<ProblemMeta> {
        if self
            .problems
            .iter()
            .any(|existing| existing.problem_id == problem.problem_id)
        {
            return Err(RepositoryError::Storage(format!(
                "problem already exists: {}",
                problem.problem_id
            )));
        }

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

        Ok(FinalizedProblemBundle { problem, manifest })
    }
}

#[async_trait]
impl SubmissionRepository for MemoryRepository {
    async fn get_submission(&self, submission_id: &str) -> RepositoryResult<SubmissionDetail> {
        self.submissions
            .iter()
            .find(|detail| detail.submission.submission_id == submission_id)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(submission_id.to_string()))
    }

    async fn list_user_submissions(&self, user_id: &str) -> RepositoryResult<Vec<SubmissionMeta>> {
        Ok(self
            .submissions
            .iter()
            .filter(|detail| detail.submission.user_id == user_id)
            .map(|detail| detail.submission.clone())
            .collect())
    }
}
