use actix_web::{web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(_: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok()
}
