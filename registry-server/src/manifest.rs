use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Package,
    pub dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub author: Option<String>,
    pub version: String,
    pub language: String,
    pub entrypoint: String,
}

impl Manifest {
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&content)?;

        Ok(manifest)
    }
}
