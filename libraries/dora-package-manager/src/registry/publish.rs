use reqwest::Client;
use reqwest::Response;
use reqwest::multipart;
use reqwest::multipart::Part;

use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

pub async fn publish_artifacts(path: &Path, url: &str) -> eyre::Result<Response> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // ARTIFACTS //
    let file_bytes = fs::read(&path).await?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("artifacts.tar.gz")
        .to_string();

    let file_part = Part::bytes(file_bytes)
        .file_name(file_name.clone())
        .mime_str("application/gzip")?;

    let form = multipart::Form::new().part(file_name, file_part);
    let res = client.post(url.trim()).multipart(form).send().await?;

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

#[cfg(test)]
mod registry_client_test {
    use super::*;
    use crate::package::Package;

    #[tokio::test]
    #[ignore]
    async fn _publish_artifacts() {
        let pkg = Package::new();
        let artifacts_path = pkg.build().unwrap();

        let url: &str = "http://127.0.0.1:7878/api/publish";
        let result = publish_artifacts(&artifacts_path, url).await;

        println!("Res {:#?}", result);
    }
}
