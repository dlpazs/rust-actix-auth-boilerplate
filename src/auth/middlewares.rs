use std::pin::Pin;
use std::task::{Context, Poll};

use actix_identity::RequestIdentity;
use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use futures::Future;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.

pub struct CookieAuth;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for CookieAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CookieAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CookieAuthMiddleware { service })
    }
}

pub struct CookieAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service for CookieAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());
        let identity = RequestIdentity::get_identity(&req);
        dbg!(&identity);
        if identity.is_none() {
            // Should never reach this branch
            println!("Error not identity found");
            return Box::pin(async { Err(actix_web::error::ErrorUnauthorized("Unauthorized")) });
        } else {
            // Here I should check the identity first
            Box::pin(self.service.call(req))
        }
    }
}

pub struct JwtAuth;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for JwtAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware { service })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service for JwtAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let token = req
            .headers()
            .get("AUTHORIZATION")
            .map(|value| value.to_str().ok());
        //need verify token function step
        if token.is_none() {
            // Should never reach this branch
            println!("Error not identity found");
            return Box::pin(async { Err(actix_web::error::ErrorUnauthorized("Unauthorized")) });
        } else {
            // Here I should check the identity first
            Box::pin(self.service.call(req))
        }
    }
}
