use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionStatus {
    Queued,
    Dispatching,
    Staging,
    Running,
    Accepted,
    WrongAnswer,
    RuntimeError,
    CompileError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    SystemError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionMeta {
    pub submission_id: String,
    pub problem_id: String,
    pub user_id: String,
    pub language: String,
    pub status: SubmissionStatus,
    pub attempt: u32,
    pub worker_task_id: Option<String>,
    pub problem_version: u32,
    pub manifest_version: u32,
    pub bundle_hash: String,
    pub submitted_at: String,
    pub finalized_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionResultSummary {
    pub submission_id: String,
    pub status: SubmissionStatus,
    pub score: Option<u32>,
    pub max_score: Option<u32>,
    pub runtime_ms: Option<u32>,
    pub memory_mb: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionDetail {
    pub submission: SubmissionMeta,
    pub result: Option<SubmissionResultSummary>,
}
