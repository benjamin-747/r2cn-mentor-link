use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchStuTask {
    pub login: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidateStudent {
    pub login: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidateStudentRes {
    pub success: bool,
    pub student_name: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsppValidateStudentRes {
    pub code: i32,
    pub err_code: i32,
    #[serde(alias = "studentExist")]
    pub student_exist: bool,
    pub message: String,
    #[serde(alias = "suStudentName")]
    pub su_student_name: Option<String>,
}
