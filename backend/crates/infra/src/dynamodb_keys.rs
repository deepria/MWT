pub const CORE_TABLE_NAME: &str = "mwt-core-table-prod";
pub const USER_SUBMISSIONS_INDEX: &str = "gsi1-user-submissions";
pub const SUBMISSIONS_BY_STATUS_INDEX: &str = "gsi2-submissions-by-status";

pub fn problem_pk(problem_id: &str) -> String {
    format!("PROBLEM#{problem_id}")
}

pub fn problem_meta_sk() -> &'static str {
    "META"
}

pub fn problem_manifest_sk(manifest_version: u32) -> String {
    format!("MANIFEST#{manifest_version:010}")
}

pub fn submission_pk(submission_id: &str) -> String {
    format!("SUBMISSION#{submission_id}")
}

pub fn submission_meta_sk() -> &'static str {
    "META"
}

pub fn user_submissions_pk(user_id: &str) -> String {
    format!("USER#{user_id}")
}

pub fn user_submissions_sk(submitted_at: &str, submission_id: &str) -> String {
    format!("SUBMITTED_AT#{submitted_at}#SUBMISSION#{submission_id}")
}

pub fn submissions_by_status_pk(status: &str) -> String {
    format!("STATUS#{status}")
}

pub fn submissions_by_status_sk(submitted_at: &str, submission_id: &str) -> String {
    format!("SUBMITTED_AT#{submitted_at}#SUBMISSION#{submission_id}")
}
