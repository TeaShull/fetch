use anyhow::{Result, anyhow, Context};
use reqwest::Url;
use scraper::{Html, Selector};
use tokio::runtime::Runtime;
use tokio::fs::File;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::{Path, PathBuf};

// Orchestrate extracting of file URLs and downloading them
async fn fetch_webpage_and_download_files(url: &str, extensions: &[&str], target_dir: &Path) -> Result<()> {
    let file_urls = extract_file_urls(url, extensions).await?;
    println!("File URLs: {:?}", file_urls);
    download_files(&file_urls, target_dir).await?;
    Ok(())
}

// Extract file URLs from a webpage
async fn extract_file_urls(url: &str, extensions: &[&str]) -> Result<Vec<String>> {
    let html = fetch_html_content(url).await?;
    let base_url = Url::parse(url)?;
    let urls = extract_urls_from_html(&html, &base_url);
    let file_urls = filter_urls_by_extension(urls, extensions);
    
    if file_urls.is_empty() {
        Err(anyhow!("No files found with specified extensions"))
    } else {
        Ok(file_urls)
    }
}

// Fetch HTML content from a given URL
async fn fetch_html_content(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    Ok(response.text().await?)
}

// Parses HTML and extracts URLs
fn extract_urls_from_html(html: &str, base_url: &Url) -> Vec<Url> {
    let document = Html::parse_document(html);
    document.select(&Selector::parse("a[href], img[src]").expect("Error parsing selector"))
        .filter_map(|element| element.value().attr("href").or_else(|| element.value().attr("src")))
        .filter_map(|url| Url::options().base_url(Some(&base_url)).parse(url).ok())
        .collect()
}

// Filters URLs by file extension
fn filter_urls_by_extension(urls: Vec<Url>, extensions: &[&str]) -> Vec<String> {
    urls.into_iter()
        .filter(|url| {
            let path = url.path();
            let ext = Path::new(path).extension().and_then(|ext| ext.to_str()).unwrap_or_default();
            extensions.contains(&ext)
        })
        .map(|url| url.to_string())
        .collect()
}

// Downloads files from a list of URLs
async fn download_files(file_urls: &[String], target_dir: &Path) -> Result<()> {
    ensure_directory_exists(target_dir).await?;
    for file_url in file_urls.iter() {
        download_file(file_url, target_dir).await?;
    }
    Ok(())
}

// Adjusted download_file function for async operations
async fn download_file(file_url: &str, target_dir: &Path) -> Result<()> {
    let url = Url::parse(file_url).context(format!("Failed to parse URL: {}", file_url))?;
    let file_name = extract_filename_from_url(&url);
    let file_path = construct_file_path(target_dir, &file_name)?;
    let file_path_str = file_path.to_str().ok_or_else(|| anyhow!("Failed to convert file path to string"))?;
    
    let response = reqwest::get(file_url).await
        .context(format!("Failed to download file: {}", file_url))?;
    let mut dest = File::create(&file_path)
        .await
        .context(format!("Failed to create file: {}", file_path_str))?;
    
    let content = response.bytes().await?;
    dest.write_all(&content)
        .await
        .context("Failed to copy content to file")?;
    println!("File downloaded to: {}", file_path_str);
    Ok(())
}

// Ensures that the target directory exists
async fn ensure_directory_exists(target_dir: &Path) -> Result<()> {
    fs::create_dir_all(target_dir)
        .await
        .context("Failed to create target directory")?;
    Ok(())
}

// Construct the full file path for the downloaded file.
fn construct_file_path(target_dir: &Path, file_name: &str) -> Result<PathBuf, anyhow::Error> {
    let file_path = target_dir.join(file_name);
    match file_path.to_str() {
        Some(_) => Ok(file_path),
        None => Err(anyhow!("Failed to construct file path")),
    }
}

// Extracts the file name from a URL.
fn extract_filename_from_url(file_url: &Url) -> String {
    file_url.path_segments()
        .and_then(|segments| segments.last())
        .unwrap_or("default_filename")
        .to_string()
}

fn main() -> Result<()> {
    let url = "";
    let extensions = ["pdf", "mp4"];
    let target_dir = Path::new("downloads"); // Specify your target directory
    
    Runtime::new()?.block_on(fetch_webpage_and_download_files(url, &extensions, &target_dir))?;
    Ok(())
}

