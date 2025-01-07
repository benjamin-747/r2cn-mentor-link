use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidateTask {
    pub github_repo_id: String,
    pub point: i32,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidateStudent {
    pub git_email: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidateStudentRes {
    pub code: i32,
    pub err_code: i32,
    #[serde(alias = "studentExist")]
    pub student_exist: bool,
    pub message: String,
}
