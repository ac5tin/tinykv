use std::fs::File;
use std::path::Path;

pub fn create_if_not_exist(fpath: &str) -> Result<(), anyhow::Error> {
    // create file if not already exist
    if !Path::new(fpath).exists() {
        File::create(fpath)?;
    }
    Ok(())
}
