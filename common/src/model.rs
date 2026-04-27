use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommonResult<T> {
    pub code: String,
    pub data: Option<T>,
    pub message: String,
}

impl<T> CommonResult<T> {
    pub fn success(data: Option<T>) -> Self {
        CommonResult {
            code: "OK".to_owned(),
            data,
            message: "".to_owned(),
        }
    }
    pub fn failed(message: &str) -> Self {
        Self::failed_with_code("ERROR", message)
    }

    pub fn failed_with_code(code: &str, message: &str) -> Self {
        CommonResult {
            code: code.to_string(),
            data: None,
            message: message.to_string(),
        }
    }
}
