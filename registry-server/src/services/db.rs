use crate::manifest::Package;
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use sqlx::Pool;
use sqlx::Postgres;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PackageVersion {
    pub version: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_all_version(
    name: String,
    db: &Pool<Postgres>,
) -> Result<Vec<PackageVersion>, sqlx::Error> {
    let versions: Vec<PackageVersion> = sqlx::query_as!(
        PackageVersion,
        r#"
        SELECT v.version, v.created_at
        FROM packages p
        JOIN versions v ON p.id = v.package_id
        WHERE p.name = $1
        ORDER BY v.created_at DESC
        "#,
        name
    )
    .fetch_all(db)
    .await?;

    Ok(versions)
}

pub async fn store_manifest_in_db(
    db: &PgPool,
    pkg: Package,
    storage_path: String,
) -> Result<(), sqlx::Error> {
    let package_id: i32 = sqlx::query_scalar!(
        r#"
        INSERT INTO packages (name, author)
        VALUES ($1, $2)
        ON CONFLICT (name)
        DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
        pkg.name,
        pkg.author
    )
    .fetch_one(db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO versions (package_id, version, checksum, language, storage_path)
        VALUES ($1,$2,$3,$4, $5)
        "#,
        package_id,
        pkg.version,
        pkg.checksum,
        pkg.language,
        storage_path
    )
    .execute(db)
    .await?;

    Ok(())
}
