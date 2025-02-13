use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchStuTask {
    pub login: String,
}



