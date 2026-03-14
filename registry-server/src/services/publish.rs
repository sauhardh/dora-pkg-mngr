use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::post;
use actix_web::web;

use serde_json::json;

struct PublishBody {}

#[post("/publish")]
pub async fn publish() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "success",
        "message":""
    }))
}
