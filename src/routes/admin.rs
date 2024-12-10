use crate::models::response::UserResponse;
use crate::models::schema::User;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{insert_into, PgConnection, RunQueryDsl};
use diesel::{prelude::*, update};
use ginger_shared_rs::rocket_models::MessageResponse;
use ginger_shared_rs::rocket_utils::{APIClaims, Claims};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;
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

#[openapi]
#[get("/users?<page>&<page_size>")]
pub fn get_paginated_users(
    rdb: &State<Pool<ConnectionManager<PgConnection>>>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<Json<Vec<UserResponse>>, rocket::http::Status> {
    use crate::models::schema::schema::user::dsl::*;

    let mut conn = rdb
        .get()
        .map_err(|_| rocket::http::Status::InternalServerError)?;

    // Default values for pagination
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);

    if page_size == 0 {
        return Err(rocket::http::Status::BadRequest);
    }

    // Calculate offset
    let offset = (page - 1) * page_size;

    // Query the database
    match user
        .order_by(created_at.desc())
        .limit(page_size as i64)
        .offset(offset as i64)
        .load::<User>(&mut conn)
    {
        Ok(users) => {
            let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            Ok(Json(response))
        }
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}
