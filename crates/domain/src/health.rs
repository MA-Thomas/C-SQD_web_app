use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiHealth {
    pub service: String,
    pub status: String,
}
