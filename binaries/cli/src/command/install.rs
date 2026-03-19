use std::path::Path;

use clap::Args;
use dora_package_manager::registry::download::RegistryDownload;
use dora_package_manager::registry::download::extract_package;
use eyre::ContextCompat;

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
        println!("Looking for a Package {}...", self.name);

        let downloader = RegistryDownload::new();
        downloader.download(&download_url, &archive_path).await?;

        let name = archive_path
            .file_name()
            .and_then(|f| f.to_str())
            .wrap_err("Invalid archive path")?
            .trim_end_matches(".tar.gz");
        let dest = archive_path.parent().unwrap_or(Path::new(".")).join(name);

        extract_package(&archive_path, &dest)?;

        println!("Successfully downloaded to {:?}", dest);
        Ok(())
    }
}
