use std::env;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use flate2::Compression;
use flate2::write::GzEncoder;
use ignore::Walk;
use tar::Builder;

use crate::manifest::Manifest;

struct Packager {
    pub path_traversal_limit: u8,
    pub manifest_file_name: String,
}

impl Packager {
    pub fn new() -> Self {
        Self {
            path_traversal_limit: 3,
            manifest_file_name: "dora.toml".to_string(),
        }
    }

    pub fn find_project_root(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

        Err(format!(
            "Could not find `{}` within range of {:?}",
            self.manifest_file_name, self.path_traversal_limit
        )
        .into())
    }

    pub fn read_manifest(
        &self,
        path: &Path,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let manifest_path = path.join(&self.manifest_file_name);
        let manifest = Manifest::from_path(&manifest_path)?;

        let package = manifest.package;
        Ok((package.name, package.version))
    }

    pub fn collect_files(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();

        for result in Walk::new(path) {
            let entry = result?;

            if entry.file_type().is_some() {
                files.push(entry.path().to_path_buf());
            }
        }

        Ok(files)
    }

    pub fn archive(
        &self,
        root: &Path,
        name: String,
        version: String,
        files_collection: Vec<PathBuf>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let name = name.replace(" ", "");
        let archive_name = format!("{}-{}.tar.gz", name, version);

        let tar_gz = File::create(&archive_name)?;
        let encoder = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(encoder);

        for file in files_collection {
            let relative = file.strip_prefix(&root)?;
            let archive_path = Path::new(&format!("{}-{}", name, version)).join(relative);
            tar.append_path_with_name(&file, archive_path)?;
        }

        let output_path = root.join(&archive_name);
        Ok(output_path)
    }

    pub fn build(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let root = self.find_project_root()?;
        let (name, version) = self.read_manifest(&root)?;
        let files_collection = self.collect_files(&root)?;

        self.archive(&root, name, version, files_collection)
    }
}

#[cfg(test)]
mod PackagerTest {
    use super::*;

    #[test]
    fn test_collect_files() {
        let pkg = Packager::new();
        let path = pkg.find_project_root();
        println!("path {:?}", path);

        let result = pkg.collect_files(&path.unwrap());
        println!("{:?}", result);
    }

    #[test]
    fn test_build() {
        let pkg = Packager::new();
        let result = pkg.build();
        print!("path {:#?}", result);
    }
}
