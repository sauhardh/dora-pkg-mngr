use clap::Args;
use dora_package_manager::registry_client::publish_artifacts;
use dora_package_manager::{manifest, package::Package};

use super::{Executable, default_tracing};

#[derive(Debug, Args)]
pub struct Publish {}

impl Executable for Publish {
    async fn execute(self) -> eyre::Result<()> {
        default_tracing()?;

        let pkg = Package::new();
        // Handles
        // 1) Finding root folder
        // 2) Traversing to manifest file i.e. dora.toml
        // 3) Filtering usable file only (follows .gitignore)
        // 4) Compress and archive it in .tar.gz
        // 5) Return path to archived_folder
        let (artifacts_path, manifest) = pkg.build()?;

        let url: &str = "http://localhost:8000/api/publish";
        let res = publish_artifacts(&artifacts_path, url, manifest).await?;

        println!(" 📦 Successfully Published {:?}", artifacts_path);
        println!(" <!> Info: {:?}", res.text().await?);

        Ok(())
    }
}
