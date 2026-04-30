use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemVisibility {
    Draft,
    Public,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemMeta {
    pub problem_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub tags: Vec<String>,
    pub time_limit_ms: u32,
    pub memory_limit_mb: u32,
    pub visibility: ProblemVisibility,
    pub statement_location: String,
    pub bundle_key: Option<String>,
    pub bundle_hash: Option<String>,
    pub checker_key: Option<String>,
    pub checker_hash: Option<String>,
    pub problem_version: u32,
    pub manifest_version: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestCase {
    pub id: u32,
    pub input_path: String,
    pub output_path: String,
    pub weight: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemManifest {
    pub problem_id: String,
    pub manifest_version: u32,
    pub bundle_key: String,
    pub bundle_hash: String,
    pub bundle_format: String,
    pub bundle_size_bytes: u64,
    pub case_count: u32,
    pub cases: Vec<ManifestCase>,
    pub checker_key: Option<String>,
    pub checker_hash: Option<String>,
    pub problem_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemSummary {
    pub problem_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub tags: Vec<String>,
    pub time_limit_ms: u32,
    pub memory_limit_mb: u32,
}

impl From<ProblemMeta> for ProblemSummary {
    fn from(problem: ProblemMeta) -> Self {
        Self {
            problem_id: problem.problem_id,
            title: problem.title,
            difficulty: problem.difficulty,
            tags: problem.tags,
            time_limit_ms: problem.time_limit_ms,
            memory_limit_mb: problem.memory_limit_mb,
        }
    }
}
