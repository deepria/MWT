use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: String,
    pub email: Option<String>,
    pub groups: Vec<String>,
}

impl AuthContext {
    pub fn is_admin(&self) -> bool {
        self.groups.iter().any(|group| group == "admin")
    }
}
