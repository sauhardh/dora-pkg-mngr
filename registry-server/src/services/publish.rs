//! When user publish their's package to registry server
use std::fs::File;
use std::io::Read;
use std::io::Write;

use actix_multipart::Field;
use actix_multipart::Multipart;
use actix_web::HttpResponse;
use actix_web::error::ErrorBadRequest;
use actix_web::error::ErrorInternalServerError;
use actix_web::post;
use actix_web::web;
use flate2::read::GzDecoder;
use futures_util::TryStreamExt;
use serde_json::json;
use sqlx::PgPool;
use tar::Archive;

use crate::manifest::Manifest;
use crate::manifest::Package;
use crate::services::store_manifest_in_db;

#[inline]
fn create_dir_if_not_exist() -> Result<String, Box<dyn std::error::Error>> {
    // TODO: Should be absolute path
    // However, in production, we may use cloud storage
    let storage_dir = "./storage";
    std::fs::create_dir_all(storage_dir)?;

    Ok(storage_dir.to_string())
}

#[inline]
async fn save_to_storage(
    mut field: Field,
    path: &str,
    file_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: dir_version look it up
    let (dir_name, dir_version) = file_name.rsplit_once('-').ok_or("Invalid artifact name")?;

    let package_dir = format!("{}/{}", path, dir_name);
    std::fs::create_dir_all(&package_dir)?;

    let file_path = format!("{}/{}", package_dir, dir_version);
    let mut f = std::fs::File::create(&file_path)?;

    while let Some(chunk) = field.try_next().await? {
        f.write_all(&chunk)?;
    }

    Ok(file_path)
}

#[inline]
fn get_info(field: &Field) -> Result<(String, String), Box<dyn std::error::Error>> {
    let file_name = field
        .name()
        .ok_or_else(|| ErrorBadRequest("Failed to get file_name from field"))?
        .to_string();

    let content_type = field
        .content_type()
        .ok_or_else(|| ErrorBadRequest("Failed to get content-type from field"))?
        .to_string();

    Ok((file_name, content_type))
}

fn extract_manifest_from_tar(archive_path: &String) -> Result<String, Box<dyn std::error::Error>> {
    let tar_gz = File::open(archive_path)?;
    let decoder = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        // NOTE: SECURITY (prevent directory traversal attack)
        // It filter outs the archive having path "../.." to prevent from accessing server's path
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            continue;
        }

        if path.file_name().map(|f| f == "dora.toml").unwrap_or(false) {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            println!("content {:#?}", content);
            return Ok(content);
        }
    }

    Err("dora.toml -> manifest file not found in the archive".into())
}

#[inline]
fn parse_manifest_str(content: &str) -> Result<Package, Box<dyn std::error::Error>> {
    let manifest: Manifest = toml::from_str(content)?;
    let pkg = manifest.package;

    Ok(pkg)
}

/// API /publish
/// Handles user publishing their packages
/// Stores the manifest on db
#[post("/publish")]
pub async fn serve_publish(
    db: web::Data<PgPool>,
    mut payload: Multipart,
) -> actix_web::Result<HttpResponse> {
    let path = create_dir_if_not_exist()?;
    let mut archive_path: Option<String> = None;

    while let Some(field) = payload.try_next().await? {
        let (file_name, content_type) = get_info(&field)?;
        match content_type.to_lowercase().as_str() {
            "application/gzip" => {
                archive_path = Some(save_to_storage(field, &path, &file_name).await?);
            }
            _ => {
                // TODO:  use logging
                println!("Unknown field type: {}", content_type);
                return Ok(HttpResponse::BadRequest().json(json!({
                    "status":"error",
                    "message":"Unknown field type of the file. Only supports `application/json` and `application/gzip`"
                })));
            }
        }
    }

    // For storing manifest in db
    let archive_path = archive_path.ok_or(ErrorInternalServerError(
        "Could not found the archived folder on given path",
    ))?;

    let content = extract_manifest_from_tar(&archive_path)?;
    let manifest = parse_manifest_str(&content)?;

    store_manifest_in_db(&db, manifest, archive_path)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {}", e)))?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message":"Completed"
    })))
}

#[cfg(test)]
mod test_publish {
    use super::*;

    #[test]
    fn _extract_manifest_from_tar() {
        let archive_path = "./storage/nodename/0.0.1.tar.gz".to_string();
        let x = extract_manifest_from_tar(&archive_path);
        println!("Result {:#?}", x);
    }
}
