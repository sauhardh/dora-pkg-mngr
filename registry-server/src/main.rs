use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::get;
use actix_web::middleware::Logger;
use actix_web::web;
use dotenv::dotenv;
use log::info;
use serde_json::json;
use sqlx::PgPool;
use sqlx::Pool;
use sqlx::Postgres;

use crate::services::download_package;
use crate::services::download_specific_package;
use crate::services::get_packages;
use crate::services::serve_publish;

pub mod manifest;
pub mod services;

#[get("")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Dora registry running sound and good"
    }))
}

async fn initialize_db() -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL").expect("Failed to get DATABASE_URL from env");
    let db = PgPool::connect(&url).await?;
    Ok(db)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let db = initialize_db()
        .await
        .expect("Failed to connect to db with provided url on env");
    info!("\t DB initialized successfully");

    const PORT: u16 = 7878;
    info!("\t Starting server localhost at port {PORT}");

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .service(health)
                    .service(serve_publish)
                    .service(download_package)
                    .service(download_specific_package)
                    .service(get_packages),
            )
    })
    .workers(3)
    .bind(("127.0.0.1", PORT))?
    .run()
    .await
}
