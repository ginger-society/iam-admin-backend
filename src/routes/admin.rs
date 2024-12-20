use crate::models::request::{InviteRequest, UpdateUserRequest};
use crate::models::response::{AppResponse, UserResponse};
use crate::models::schema::{App, User};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use diesel::dsl::exists;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{insert_into, PgConnection, RunQueryDsl};
use diesel::{prelude::*, update};
use ginger_shared_rs::rocket_models::MessageResponse;
use ginger_shared_rs::rocket_utils::{APIClaims, Claims};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use NotificationService::apis::configuration::ApiKey as NotificationApiKey;
use NotificationService::apis::default_api::{send_email, SendEmailParams};
use NotificationService::get_configuration as get_notification_service_configuration;
use NotificationService::models::EmailRequest;

#[derive(Serialize, JsonSchema)]
pub struct PaginatedResponse<T> {
    pub total_count: usize,
    pub data: Vec<T>,
}
#[openapi]
#[get("/users?<page>&<page_size>&<search>")]
pub fn get_paginated_users(
    _claims: Claims,
    rdb: &State<Pool<ConnectionManager<PgConnection>>>,
    page: Option<usize>,
    page_size: Option<usize>,
    search: Option<String>,
) -> Result<Json<PaginatedResponse<UserResponse>>, rocket::http::Status> {
    use crate::models::schema::schema::user::dsl::*;

    let mut conn = rdb
        .get()
        .map_err(|_| rocket::http::Status::InternalServerError)?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);

    if page_size == 0 {
        return Err(rocket::http::Status::BadRequest);
    }

    let offset = (page - 1) * page_size;
    let like_pattern = search.as_deref().map(|s| format!("%{}%", s));

    // Total count query
    let total_count: i64 = match like_pattern {
        Some(ref pattern) => user
            .filter(
                first_name
                    .ilike(pattern)
                    .or(last_name.ilike(pattern))
                    .or(middle_name.ilike(pattern))
                    .or(email_id.ilike(pattern)),
            )
            .count()
            .get_result(&mut conn),
        None => user.count().get_result(&mut conn),
    }
    .map_err(|_| rocket::http::Status::InternalServerError)?;

    // Paginated results query
    let results = match like_pattern {
        Some(ref pattern) => user
            .filter(
                first_name
                    .ilike(pattern)
                    .or(last_name.ilike(pattern))
                    .or(middle_name.ilike(pattern))
                    .or(email_id.ilike(pattern)),
            )
            .order_by(created_at.desc())
            .limit(page_size as i64)
            .offset(offset as i64)
            .load::<User>(&mut conn),
        None => user
            .order_by(created_at.desc())
            .limit(page_size as i64)
            .offset(offset as i64)
            .load::<User>(&mut conn),
    }
    .map_err(|_| rocket::http::Status::InternalServerError)?;

    let response: Vec<UserResponse> = results.into_iter().map(UserResponse::from).collect();

    Ok(Json(PaginatedResponse {
        total_count: total_count as usize,
        data: response,
    }))
}

#[openapi]
#[get("/user?<email>")]
pub fn get_user_by_email(
    _claims: Claims,
    rdb: &State<Pool<ConnectionManager<PgConnection>>>,
    email: String,
) -> Result<Json<UserResponse>, rocket::http::Status> {
    use crate::models::schema::schema::user::dsl::*;

    let mut conn = rdb
        .get()
        .map_err(|_| rocket::http::Status::InternalServerError)?;

    match user
        .filter(email_id.eq(email.clone()))
        .first::<User>(&mut conn)
    {
        Ok(user_record) => Ok(Json(UserResponse::from(user_record))),
        Err(diesel::result::Error::NotFound) => Err(rocket::http::Status::NotFound),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[openapi]
#[put("/user/<email>", format = "json", data = "<update_request>")]
pub fn update_user_by_email(
    _claims: Claims,
    rdb: &State<Pool<ConnectionManager<PgConnection>>>,
    email: String,
    update_request: Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, rocket::http::Status> {
    use crate::models::schema::schema::user::dsl::*;

    let mut conn = rdb
        .get()
        .map_err(|_| rocket::http::Status::InternalServerError)?;

    // Build the update query
    match diesel::update(user.filter(email_id.eq(email.clone())))
        .set((
            first_name.eq(&update_request.first_name),
            middle_name.eq(&update_request.middle_name),
            last_name.eq(&update_request.last_name),
            is_active.eq(&update_request.is_active),
            is_root.eq(&update_request.is_root),
        ))
        .get_result::<User>(&mut conn)
    {
        Ok(updated_user) => Ok(Json(UserResponse::from(updated_user))),
        Err(diesel::result::Error::NotFound) => Err(rocket::http::Status::NotFound),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}
#[openapi]
#[get("/applications?<page>&<page_size>&<search>")]
pub fn list_paginated_applications(
    _claims: Claims,
    rdb: &State<Pool<ConnectionManager<PgConnection>>>,
    page: Option<usize>,
    page_size: Option<usize>,
    search: Option<String>,
) -> Result<Json<PaginatedResponse<AppResponse>>, rocket::http::Status> {
    use crate::models::schema::schema::app::dsl::*;

    let mut conn = rdb
        .get()
        .map_err(|_| rocket::http::Status::InternalServerError)?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);

    if page_size == 0 {
        return Err(rocket::http::Status::BadRequest);
    }

    let offset = (page - 1) * page_size;
    let like_pattern = search.as_deref().map(|s| format!("%{}%", s));

    // Total count query
    let total_count: i64 = match like_pattern {
        Some(ref pattern) => app
            .filter(name.ilike(pattern).or(client_id.ilike(pattern)))
            .count()
            .get_result(&mut conn),
        None => app.count().get_result(&mut conn),
    }
    .map_err(|_| rocket::http::Status::InternalServerError)?;

    // Paginated results query
    let results = match like_pattern {
        Some(ref pattern) => app
            .filter(name.ilike(pattern).or(client_id.ilike(pattern)))
            .order_by(name.desc())
            .limit(page_size as i64)
            .offset(offset as i64)
            .load::<App>(&mut conn),
        None => app
            .order_by(name.desc())
            .limit(page_size as i64)
            .offset(offset as i64)
            .load::<App>(&mut conn),
    }
    .map_err(|_| rocket::http::Status::InternalServerError)?;

    let response: Vec<AppResponse> = results.into_iter().map(AppResponse::from).collect();

    Ok(Json(PaginatedResponse {
        total_count: total_count as usize,
        data: response,
    }))
}

#[openapi]
#[get("/group-exists/<uuid>")]
pub fn check_group_exists(
    uuid: String,
    rdb: &State<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Json<bool>, rocket::http::Status> {
    use crate::models::schema::schema::group::dsl::*;

    // Attempt to get a database connection
    let mut conn = rdb
        .get()
        .map_err(|_| rocket::http::Status::InternalServerError)?;

    // Check if the group exists
    match diesel::select(exists(group.filter(identifier.eq(&uuid)))).get_result::<bool>(&mut conn) {
        Ok(exists) => Ok(Json(exists)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[openapi]
#[get("/user-exists/<email>")]
pub fn check_user_exists(
    email: String,
    rdb: &State<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Json<bool>, rocket::http::Status> {
    use crate::models::schema::schema::user::dsl::*;

    // Attempt to get a database connection
    let mut conn = rdb
        .get()
        .map_err(|_| rocket::http::Status::InternalServerError)?;

    // Check if the user exists
    match diesel::select(exists(user.filter(email_id.eq(&email)))).get_result::<bool>(&mut conn) {
        Ok(exists) => Ok(Json(exists)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[openapi]
#[post("/create-invite", data = "<invite_request>")]
pub async fn create_invite(
    invite_request: Json<InviteRequest>,
    cache_pool: &State<Pool<RedisConnectionManager>>,
) -> Result<(), Status> {
    let mut cache_connection = cache_pool.get().map_err(|_| Status::ServiceUnavailable)?;

    // Generate a unique token
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    // Serialize the invite data as JSON
    let invite_data =
        serde_json::to_string(&*invite_request).map_err(|_| Status::InternalServerError)?;

    // Save the invite data in Redis with a token as the key
    let expiration = 3600; // 1 hour in seconds
    cache_connection
        .set_ex(&token, invite_data, expiration)
        .map_err(|_| Status::InternalServerError)?;

    // Configure the email sending service
    let mut configuration = get_notification_service_configuration();

    let token_str = env::var("ISC_SECRET").expect("ISC_SECRET must be set");

    configuration.api_key = Some(NotificationApiKey {
        key: token_str,
        prefix: None,
    });

    // Send the invite email
    let email_body = format!(
        "Hello {first_name},\n\nYou have been invited to join our platform. Use the following link to accept the invite: \
        https://iam-staging.gingersociety/#/accept-invite/{token}\n\nThe link expires in 1 hour.",
        first_name = invite_request.first_name,
        token = token
    );

    let email_request = EmailRequest {
        to: invite_request.email_id.clone(),
        subject: "You're Invited!".to_string(),
        message: email_body,
        reply_to: None,
    };

    send_email(&configuration, SendEmailParams { email_request })
        .await
        .map_err(|_| Status::ServiceUnavailable)?;

    Ok(())
}
