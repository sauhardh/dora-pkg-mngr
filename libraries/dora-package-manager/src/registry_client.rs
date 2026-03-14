use reqwest::Client;
use reqwest::Response;
use reqwest::multipart;
use reqwest::multipart::Part;
use serde_json::to_vec;

use crate::package::ManifestInfo;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

pub async fn publish_artifacts(
    path: &Path,
    url: &str,
    manifest: ManifestInfo,
) -> eyre::Result<Response> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // ARTIFACTS
    let file_bytes = fs::read(&path).await?;
    let file_part = Part::bytes(file_bytes)
        .file_name(
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("artifacts.tar.gz")
                .to_string(),
        )
        .mime_str("application/gzip")?;

    // JSON
    let manifest_json = to_vec(&manifest)?;
    let manifest_part = Part::bytes(manifest_json)
        .file_name("manifest.json")
        .mime_str("application/json")?;

    let form = multipart::Form::new()
        .part("file", file_part)
        .part("manifest", manifest_part);
    let res = client.post(url).multipart(form).send().await?;

    if !res.status().is_success() {
        let status = res.status();

        let body = res
            .text()
            .await
            .unwrap_or_else(|_| "<Failed to read body>".to_string());

        return Err(eyre::eyre!(
            "Failed to publish: {}.\n   Reason: {:?}",
            status,
            body
        ));
    }

    Ok(res)
}

#[tokio::test]
async fn test_publish_artifacts() {
    let path = Path::new(
        "/home/sk/Desktop/contribute/working/dora/libraries/dora-package-manager/nodename-0.0.1.tar.gz",
    );

    let mut map = HashMap::new();
    map.insert("hello".to_string(), "world".to_string());

    let url: &str = "http://localhost:8000/publish";
    let manifest = ManifestInfo {
        name: "".to_string(),
        version: "0".to_string(),
        owner: Some("kafle".to_string()),
        dependencies: Some(map),
        checksum: "".to_string(),
    };

    let result = publish_artifacts(&path, url, manifest).await;

    println!("Res {:#?}", result);
}
