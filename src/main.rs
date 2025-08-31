use core::error;
use std::{fmt::Display, path::Path};
use tokio::{fs::{self, File}, io::{AsyncReadExt, AsyncWriteExt}};
use reqwest::get;

#[tokio::main]
async fn main() {
    let _ = check_release("HDR-Releases").await;
    let _ = check_release("HDR-PreReleases").await;
}

async fn check_release(repo: impl Into<String> + Display + Copy + AsRef<Path>) -> Result<(), Box<dyn error::Error>> {
    let github = octocrab::instance();

    // Get the latest release's name
    let release = github.repos("HDR-Development", repo).releases().get_latest().await?;
    let release_name = release.name.ok_or("Error finding release_name").unwrap();

    // Release path to save the release name = <repo_name>/release.txt
    let release_path = format!("{}/release.txt", repo);
    fs::create_dir_all(repo).await?;
    let mut release_file = File::options().read(true).write(true).create(true).open(&release_path).await?;
    let mut contents = String::new();
    let _ = release_file.read_to_string(&mut contents).await?;
    
    // If latest release matches, do nothing
    if contents.eq_ignore_ascii_case(&release_name) {
        return Ok(());
    }

    // Otherwise, download the release and save the new release name
    println!("New release found for {}", repo);
    let asset = release.assets.iter().find(|&x| x.name.eq("switch-package.zip")).ok_or("Error finding switch-package.zip").unwrap();
    let response = get(asset.browser_download_url.clone()).await?;
    let content = response.bytes().await?;
    let download_path: String = format!("{}/switch-package.zip", repo);
    let mut switch_package = File::create(download_path).await?;
    switch_package.write_all(&content).await?;
    fs::write(&release_path, release_name).await?;

    return Ok(());
}
