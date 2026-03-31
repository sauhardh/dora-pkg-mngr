use std::path::Path;

use clap::Args;
use colored::Colorize;
use dora_package_manager::registry::download::RegistryDownload;
use dora_package_manager::registry::download::extract_package;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use super::{Executable, default_tracing};

#[derive(Debug, Args)]
pub struct Install {
    pub name: String,

    #[clap(short, long)]
    pub version: Option<String>,
}

impl Executable for Install {
    async fn execute(self) -> eyre::Result<()> {
        default_tracing()?;
        println!("\n {} ", "Working on it...".cyan().bold());
        let registry_url = "http://127.0.0.1:7878/api/packages";

        let download_url = match &self.version {
            Some(v) => format!("{}/{}/{}/download", registry_url, self.name, v),
            None => format!("{}/{}/download", registry_url, self.name),
        };

        let file_name = match &self.version {
            Some(v) => format!("{}-{}.tar.gz", self.name, v),
            None => format!("{}_latest.tar.gz", self.name),
        };

        let archive_path = std::env::current_dir()?.join(file_name);
        // println!("Archive path {}", archive_path.to_string_lossy());
        println!("🔍 Looking for a Package {}...", self.name.yellow());

        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{bar:40.cyan/blue}] {pos}% ({eta})")
                .unwrap()
                .progress_chars("█▓░"),
        );

        let pb_clone = pb.clone();
        let downloader = RegistryDownload::new();
        downloader
            .download(
                &download_url,
                &archive_path,
                Some(Box::new(move |downloaded, total_size| {
                    if total_size > 0 {
                        let percent = (downloaded * 100) / total_size;
                        pb_clone.set_position(percent);
                    }
                })),
            )
            .await?;

        pb.finish_and_clear();

        let dest = archive_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();

        println!(" {} ", "Extracting package...".cyan());
        extract_package(&archive_path, &dest)?;
        println!(
            "Successfully extracted to {}",
            dest.to_string_lossy().cyan()
        );
        println!("📦 {} ", "Download Completed!".green().bold());

        Ok(())
    }
}
