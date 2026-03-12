use reqwest::{
    blocking::{Client, Response},
    header::CONTENT_TYPE,
};
use std::path::Path;

const URL: &str = "http://localhost:3000/";

pub fn publish_artifacts(path: &Path) -> Result<Response, Box<dyn std::error::Error>> {
    let artifacts = std::fs::read(path)?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let res = client
        .post(URL)
        .header(CONTENT_TYPE, "application/gzip")
        .body(artifacts)
        .send()?;

    if !res.status().is_success() {
        let status = res.status();

        let body = res
            .text()
            .unwrap_or_else(|_| "<Failed to read body>".to_string());

        return Err(format!("Failed to publish: {}.\n   Reason: {:?}", status, body).into());
    }

    Ok(res)
}

#[test]
fn test_publish_artifacts() {
    let path = Path::new(
        "/home/sk/Desktop/contribute/working/dora/libraries/dora-package-manager/nodename-0.0.1.tar.gz",
    );
    let result = publish_artifacts(&path);

    println!("Res {:?}", result);
}
