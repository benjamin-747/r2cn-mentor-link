use serde::{Deserialize, Serialize};


#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommonResult<T> {
    pub data: Option<T>,
    pub message: String,
}

impl<T> CommonResult<T> {
    pub fn success(data: Option<T>) -> Self {
        CommonResult {
            data,
            message: "".to_owned(),
        }
    }
    pub fn failed(message: &str) -> Self {
        CommonResult {
            data: None,
            message: message.to_string(),
        }
    }
}