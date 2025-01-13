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
    pub code: i32,
    pub err_code: i32,
    #[serde(alias = "studentExist")]
    pub student_exist: bool,
    pub message: String,
}