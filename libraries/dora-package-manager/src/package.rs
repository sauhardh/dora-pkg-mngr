use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use eyre;
use eyre::Context;
use eyre::bail;
use flate2::Compression;
use flate2::write::GzEncoder;
use ignore::Walk;
use tar::Builder;

use crate::manifest::Manifest;

#[derive(Default)]
pub struct Package {
    pub path_traversal_limit: u8,
    pub manifest_file_name: String,
}

impl Package {
    pub fn new() -> Self {
        Self {
            path_traversal_limit: 3,
            manifest_file_name: "dora.toml".to_string(),
        }
    }

    pub fn find_project_root(&self) -> eyre::Result<PathBuf> {
        let mut cwd = env::current_dir()?;
        let mut travel: u8 = 0;

        while travel < self.path_traversal_limit {
            if cwd.join(&self.manifest_file_name).exists() {
                return Ok(cwd);
            }
            if !cwd.pop() {
                break;
            }
            travel += 1
        }

        Err(eyre::eyre!(
            "Could not find `{}` within range of {:?}",
            self.manifest_file_name,
            self.path_traversal_limit
        ))
    }

    pub fn read_manifest(&self, path: &Path) -> eyre::Result<(String, String)> {
        let manifest_path = path.join(&self.manifest_file_name);
        let manifest = Manifest::from_path(&manifest_path)?;
        let package = manifest.package;

        Ok((package.name, package.version))
    }

    pub fn collect_files(&self, path: &Path) -> eyre::Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for result in Walk::new(path) {
            let entry = result?;

            if entry.file_type().map(|f| f.is_file()).unwrap_or(false) {
                let path = entry.path().to_path_buf();
                if path.extension().map(|e| e == "gz").unwrap_or(false) {
                    continue;
                }
                files.push(path);
            }
        }

        Ok(files)
    }

    pub fn archive(
        &self,
        root: &Path,
        name: &str,
        version: &str,
        files_collection: Vec<PathBuf>,
    ) -> eyre::Result<PathBuf> {
        // TODO: Should we replace space?
        // let name = name.replace(" ", "");
        let name = name.trim();

        if name.contains(" ") {
            bail!("Cannot have space in between name");
        }

        let archive_name = format!("{}-{}.tar.gz", name, version);
        let archive_path = root.join(&archive_name);

        let tar_gz = File::create(&archive_path)?;
        let encoder = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(encoder);

        for file in files_collection {
            let relative = file.strip_prefix(&root)?;
            let archive_path = Path::new(&format!("{}-{}", name, version)).join(relative);
            tar.append_path_with_name(&file, archive_path)?;
        }

        tar.finish().wrap_err("Failed to finalize archive")?;

        Ok(archive_path)
    }

    pub fn build(&self) -> eyre::Result<PathBuf> {
        let root = self.find_project_root()?;
        let (manifest_name, manifest_version) = self.read_manifest(&root)?;
        let files_collection = self.collect_files(&root)?;

        let archived_path =
            self.archive(&root, &manifest_name, &manifest_version, files_collection)?;

        Ok(archived_path)
    }
}

#[cfg(test)]
mod package_test {
    use super::*;

    #[test]
    fn _collect_files() {
        let pkg = Package::new();
        let path = pkg.find_project_root();
        println!("path {:?}", path);

        let result = pkg.collect_files(&path.unwrap());
        println!("{:?}", result);
    }

    #[test]
    fn _build() {
        let pkg = Package::new();
        let result = pkg.build();
        print!("path {:#?}", result);
    }
}
