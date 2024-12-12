use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_root: bool,
}

#[derive(Deserialize, JsonSchema, Debug, Serialize)]
pub struct InviteRequest {
    pub email_id: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub is_root: bool,
}
