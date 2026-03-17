use crate::manifest::Package;
use sqlx::PgPool;

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
