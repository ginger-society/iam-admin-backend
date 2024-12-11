use crate::models::schema::App;
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

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AppResponse {
    pub client_id: String,
    pub name: String,
    pub logo_url: Option<String>,
    pub disabled: bool,
    pub group_id: Option<i64>,
    pub tnc_link: Option<String>,
    pub allow_registration: bool,
    pub id: i64,
}

impl From<App> for AppResponse {
    fn from(app: App) -> Self {
        AppResponse {
            client_id: app.client_id,
            name: app.name,
            logo_url: app.logo_url,
            disabled: app.disabled,
            group_id: app.group_id,
            tnc_link: app.tnc_link,
            allow_registration: app.allow_registration,
            id: app.id,
        }
    }
}
