#[macro_use]
extern crate diesel;

use actix_identity::Identity;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_service::Service;
use actix_web::http::{header::CONTENT_TYPE, HeaderValue};
use actix_web::{get, middleware, post, web, App, Error, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use futures::future::FutureExt;
use std::time::Duration;
use uuid::Uuid;

mod auth;
mod errors;
mod schema;
mod user;

lazy_static::lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("SECRET_KEY")
        .unwrap_or_else(|_| "0123".repeat(8));
}

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    // set up database connection pool
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind = "127.0.0.1:8080";
    let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());
    println!("Starting server at: {}", &bind);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(SECRET_KEY.as_bytes())
                    .name("_ged")
                    .path("/")
                    .domain(domain.as_str())
                    .secure(false), // this can only be true if you have https
            ))
            .wrap(middleware::Logger::default())
            .configure(user::config)
    })
    .bind(&bind)?
    .run()
    .await
}
