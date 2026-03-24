use std::fs;
use std::fs::File;
use std::io::Bytes;
use std::io::Write;
use std::os::unix::process;
use std::path::Path;
use std::path::PathBuf;

use eyre::Context;
use eyre::bail;
use flate2::read::GzDecoder;
use reqwest::Client;
use reqwest::Response;
use tar::Archive;
use tokio::io::AsyncWriteExt;

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

    pub async fn download(
        &self,
        url: &str,
        dest: &PathBuf,
        progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> eyre::Result<()> {
        self.fetch_and_save(url, dest, progress).await
    }

    pub async fn fetch_and_save(
        &self,
        url: &str,
        dest: &PathBuf,
        progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> eyre::Result<()> {
        let mut res = self
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

        // let bytes = res
        //     .bytes()
        //     .await
        //     .wrap_err("Failed to read response bytes")?;

        let total_size = res.content_length().unwrap_or(0);
        let mut downloaded = 0;

        let mut file = tokio::fs::File::create(dest).await?;
        while let Some(chunk) = res.chunk().await? {
            file.write_all(&chunk).await?;

            if let Some(ref p) = progress {
                downloaded += chunk.len() as u64;
                p(downloaded, total_size);
            }
        }

        // TODO: CHECKSUM
        if let Some(expected) = expected_checksum {
            // let actual = sha256_hex(&bytes);
            // if actual != expected {
            //     bail!(
            //         "Checksum mismatch! expected: {} but got {}",
            //         expected,
            //         actual
            //     );
            // }
        }

        // tokio::fs::write(dest, &bytes)
        //     .await
        //     .wrap_err_with(|| format!("Failed to write package to {:?}", dest))?;

        file.flush().await?;

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
