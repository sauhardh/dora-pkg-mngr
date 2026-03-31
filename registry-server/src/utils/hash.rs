use hex;
use sha2::Digest;
use sha2::Sha256;

pub fn calculate_checksum(archive_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data = std::fs::read(archive_path)?;
    let hash = Sha256::digest(data);
    Ok(hex::encode(hash))
}
