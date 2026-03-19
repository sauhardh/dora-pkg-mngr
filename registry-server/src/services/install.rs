//! When user want to get package from registry server
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use actix_web::{HttpResponse, get};
use serde_json::json;
use sqlx::Pool;
use sqlx::{PgPool, Postgres};

use crate::services::PackageVersion;
use crate::services::get_all_version;

#[get("/packages/{name}")]
pub async fn get_packages(
    db: web::Data<PgPool>,
    path: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let name = path.into_inner();

    let values: Vec<PackageVersion> = get_all_version(name, &db).await.map_err(|e| {
        ErrorInternalServerError(format!(
            "Could not retrive versions from the given name of package. \n Error: {}",
            e,
        ))
    })?;

    Ok(HttpResponse::Ok().json(json!({"status":"success", "message":values})))
}

#[derive(Debug)]
struct PackageDownload {
    pub storage_path: String,
    pub checksum: String,
    pub version: String,
}

async fn get_package_for_download(
    name: &String,
    version: Option<String>,
    db: &Pool<Postgres>,
) -> Result<Option<PackageDownload>, sqlx::Error> {
    match version {
        Some(v) => {
            sqlx::query_as!(
                PackageDownload,
                r#"
                SELECT v.storage_path, v.checksum, v.version
                FROM packages p
                JOIN versions v ON p.id = v.package_id
                WHERE p.name = $1 AND v.version = $2
                "#,
                name,
                v
            )
            .fetch_optional(db)
            .await
        }
        None => {
            sqlx::query_as!(
                PackageDownload,
                r#"
                SELECT v.storage_path, v.checksum, v.version
                FROM packages p
                JOIN versions v ON p.id = v.package_id
                WHERE p.name = $1
                ORDER BY v.created_at DESC
                LIMIT 1
                "#,
                name
            )
            .fetch_optional(db)
            .await
        }
    }
}

#[get("/packages/{name}/download")]
pub async fn download_package(
    db: web::Data<PgPool>,
    path: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let name = path.into_inner();

    let package = get_package_for_download(&name, None, &db)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    match package {
        None => {
            Ok(HttpResponse::NotFound()
                .json(json!({"status":"error", "message":"Package Not found"})))
        }

        Some(pkg) => {
            let file = tokio::fs::read(&pkg.storage_path)
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;

            Ok(HttpResponse::Ok()
                .content_type("application/octet-stream")
                .insert_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}-{}.tar.gz\"", name, pkg.version),
                ))
                .insert_header(("X-checksum", pkg.checksum))
                .body(file))
        }
    }
}

#[get("/packages/{name}/{version}/download")]
pub async fn download_specific_package(
    db: web::Data<PgPool>,
    path: web::Path<(String, String)>,
) -> actix_web::Result<HttpResponse> {
    let (name, version) = path.into_inner();

    let package = get_package_for_download(&name, Some(version), &db)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    match package {
        None => Ok(HttpResponse::NotFound()
            .json(json!({"status":"error", "message":"Package Not Found with given name"}))),

        Some(pkg) => {
            let file = tokio::fs::read(&pkg.storage_path)
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;

            Ok(HttpResponse::Ok()
                .content_type("application/octet-stream")
                .insert_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}-{}.tar.gz\"", name, pkg.version),
                ))
                .insert_header(("X-checksum", pkg.checksum))
                .body(file))
        }
    }
}
