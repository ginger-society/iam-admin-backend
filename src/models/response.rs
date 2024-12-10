use crate::models::schema::User;
use chrono::{DateTime, NaiveDate, Utc};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema)]
pub struct UserResponse {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub email_id: String,
    pub is_root: bool,
    pub is_active: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            first_name: user.first_name,
            last_name: user.last_name,
            middle_name: user.middle_name,
            email_id: user.email_id,
            is_root: user.is_root,
            is_active: user.is_active,
        }
    }
}
