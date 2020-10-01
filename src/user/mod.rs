use actix_identity::Identity;

use actix_web::{
    delete, error::BlockingError, get, guard, post, web, App, Error, HttpResponse, HttpServer,
};
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DbPool;
use crate::auth;
use crate::errors::ServiceError;
pub mod model;
mod service;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/auth").service(register).service(login));
    cfg.service(
        web::scope("/api/user")
            .wrap(auth::middlewares::CookieAuth)
            .service(get_user),
    );
}

/// Finds user by UID.
#[get("/{user_id}")]
pub async fn get_user(
    pool: web::Data<DbPool>,
    user_uid: web::Path<Uuid>,
    id: Identity,
) -> Result<HttpResponse, Error> {
    let user_uid = user_uid.into_inner();
    let conn = pool.get().expect("couldn't get db connection from DbPool");
    dbg!(&id.identity());
    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || service::find_user_by_uid(user_uid, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let res = HttpResponse::NotFound().body(format!("No user found with uid: {}", user_uid));
        Ok(res)
    }
}

#[post("/register")]
pub async fn register(
    pool: web::Data<DbPool>,
    data: web::Json<model::NewUser>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from DbPool");

    let newUser = model::NewUser {
        name: data.name.to_string(),
        email: data.email.to_string(),
        password: data.password.to_string(),
    };

    let user = web::block(move || service::insert_new_user(newUser, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(user))
}

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    data: web::Json<AuthData>,
    id: Identity,
) -> Result<HttpResponse, Error> {
    let res = web::block(move || verify_user(data.into_inner(), pool))
        .await
        .map_err(|e| {
            eprintln!("Login Error: {}", e);
            HttpResponse::Unauthorized().finish()
        })?;
    let user_string = serde_json::to_string(&res).unwrap();
    id.remember(user_string);
    Ok(HttpResponse::Ok().json(res))
}

fn verify_user(auth_data: AuthData, pool: web::Data<DbPool>) -> Result<model::User, ServiceError> {
    use crate::schema::users::dsl::{email, users};
    let conn: &PgConnection = &pool.get().unwrap();
    let mut items = users
        .filter(email.eq(&auth_data.email))
        .first::<model::User>(conn)
        .optional()?;
    if let Some(user) = items {
        let verified = auth::verify(&user.password, &auth_data.password.as_bytes());
        if verified {
            return Ok(user.into());
        }
    }
    Err(ServiceError::Unauthorized)
}

#[delete("/logout")]
pub async fn logout(id: Identity) -> Result<HttpResponse, Error> {
    id.forget();
    Ok(HttpResponse::Ok().json("Logged out"))
}
