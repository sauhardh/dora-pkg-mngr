use std::fs;
use std::path::Path;
use std::path::PathBuf;

use eyre::Context;
use eyre::bail;
use flate2::read::GzDecoder;
use reqwest::Client;
use reqwest::Response;
use tar::Archive;

pub async fn get_versions(url: &str) -> eyre::Result<Response> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let res = client.get(url.trim()).send().await?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res
            .text()
            .await
            .unwrap_or_else(|_| "<Failed to read body>".to_string());

        return Err(eyre::eyre!(
            "Failed to get versions for requested package \n status: {}. \n  Reason: {:?}",
            status,
            body
        ));
    }

    Ok(res)
}

#[derive(Debug, Default)]
pub struct RegistryDownload {
    client: Client,
}

impl RegistryDownload {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_versions(self, url: &str) -> eyre::Result<Vec<String>> {
        let res = self.client.get(url.trim()).send().await?;

        if !res.status().is_success() {
            bail!("Requested Package not found");
        }

        let body: serde_json::Value = res
            .json()
            .await
            .wrap_err("Failed to parse version list reponse. Expected in JSON format")?;

        let versions = body["message"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|x| x.to_string())
            .collect();

        Ok(versions)
    }

    pub async fn download(&self, url: &str, dest: &PathBuf) -> eyre::Result<()> {
        self.fetch_and_save(url, dest).await
    }

    pub async fn fetch_and_save(&self, url: &str, dest: &PathBuf) -> eyre::Result<()> {
        let res = self
            .client
            .get(url.trim())
            .send()
            .await
            .wrap_err_with(|| "Failed to connect with registry for request package")?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_else(|_| "Unknown error".into());
            bail!("Failed to download package ({}): {}", status, body);
        }

        let expected_checksum = res
            .headers()
            .get("X-Checksum")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        let bytes = res
            .bytes()
            .await
            .wrap_err("Failed to read response bytes")?;

        if let Some(expected) = expected_checksum {
            let actual = sha256_hex(&bytes);
            // if actual != expected {
            //     bail!(
            //         "Checksum mismatch! expected: {} but got {}",
            //         expected,
            //         actual
            //     );
            // }
        }

        tokio::fs::write(dest, &bytes)
            .await
            .wrap_err_with(|| format!("Failed to write package to {:?}", dest))?;

        Ok(())
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

pub fn extract_package(archive_path: &PathBuf, dest: &PathBuf) -> eyre::Result<PathBuf> {
    let tar_gz = std::fs::File::open(archive_path)
        .wrap_err_with(|| format!("Failed to open archive at {:?}", archive_path))?;

    let decoder = GzDecoder::new(tar_gz);
    let mut tar = Archive::new(decoder);

    std::fs::create_dir_all(dest)
        .wrap_err_with(|| format!("Failed to create destination directory {:?}", dest))?;

    tar.unpack(dest)
        .wrap_err_with(|| format!("Failed to unpack archive into {:?}", dest))?;

    Ok(dest.to_path_buf())
}
