use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::get;
use actix_web::web;

use log::info;
use serde_json::json;

pub mod services;

#[get("")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Dora registry running sound and good"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    const PORT: u16 = 7878;
    info!("Starting server localhost at port {PORT}");

    HttpServer::new(|| {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .service(web::scope("/api/").service(health))
    })
    .workers(3)
    .bind(("127.0.0.1", PORT))?
    .run()
    .await
}
