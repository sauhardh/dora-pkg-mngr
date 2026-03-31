use std::time::Duration;

use clap::Args;
use colored::Colorize;
use dora_package_manager::package::Package;
use dora_package_manager::registry::publish_artifacts;
use serde::{Deserialize, Serialize};

use super::{Executable, default_tracing};

#[derive(Debug, Args)]
pub struct Publish {}

impl Executable for Publish {
    async fn execute(self) -> eyre::Result<()> {
        default_tracing()?;
        println!("\n {} ", "Initializing...".cyan().bold());

        let pkg = Package::new();
        // Handles
        // 1) Finding root folder
        // 2) Traversing to manifest file i.e. dora.toml
        // 3) Filtering usable file only (follows .gitignore)
        // 4) Compress and archive it in .tar.gz
        // 5) Return path to archived_folder
        let artifacts_path = pkg.build()?;

        let pb = indicatif::ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Publishing artifacts...");

        let url: &str = "http://127.0.0.1:7878/api/publish";
        let res = publish_artifacts(&artifacts_path, url).await?;
        pb.finish();

        println!(
            "⏳ Info: Your Artifacts has been {}",
            res.json::<Response>().await?.message.cyan()
        );
        println!(" 📦 {}", "Successfully Published!".green().bold());

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    message: String,
    status: String,
}
