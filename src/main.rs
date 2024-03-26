use anyhow::{Result, anyhow, Context};
use reqwest::Url;
use scraper::{Html, Selector};
use tokio::runtime::Runtime;
use tokio::fs;

use std::{fs::File, io::copy, path::Path};

async fn extract_file_urls(url: &str, extensions: &[&str]) -> Result<Vec<String>> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    // Parse the base URL to handle paths and schemes correctly.
    let base_url = Url::parse(url)?;

    let file_urls: Vec<String> = document
        .select(&Selector::parse("a[href], img[src]").expect("Error parsing selector"))
        .filter_map(|element| element.value().attr("href").or_else(|| element.value().attr("src")))
         .filter_map(|url| {
            // Use the Url crate to try and parse the URL found in the href or src attribute.
            // This handles absolute, root-relative, and relative URLs correctly.
            match Url::options().base_url(Some(&base_url)).parse(url) {
                Ok(parsed_url) => {
                    let path = parsed_url.path();
                    let ext = Path::new(path)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or_default();
                    
                    if extensions.contains(&ext) {
                        Some(parsed_url.to_string())
                    } else {
                        None
                    }
                },
                Err(_) => None,
            }
        })
        .collect();

    if file_urls.is_empty() {
        Err(anyhow!("No files found with specified extensions"))
    } else {
        Ok(file_urls)
    }
}

async fn download_files(file_urls: &[String]) -> Result<()> {
    // Ensure the target directory exists; create it if it doesn't.
    let target_dir = "downloaded_files";
    fs::create_dir_all(target_dir).await.context("Failed to create target directory")?;

    for file_url in file_urls.iter() {
        let url = Url::parse(file_url)
            .context(format!("Failed to parse URL: {}", file_url))?;
        let file_name = url
            .path_segments()
            .and_then(|segments| segments.last()) // Take the last segment of the path
            .unwrap_or("default_filename"); // Provide a default if unable to extract filename

        // Combine the target directory with the extracted file name
        let file_path = Path::new(target_dir).join(file_name);
        let file_path_str = file_path
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert file path to string"))?;

        let response = reqwest::get(file_url).await
            .context(format!("Failed to download file: {}", file_url))?;

        // Create the file in the specified path
        let mut dest = File::create(&file_path)
            .context(format!("Failed to create file: {}", file_path_str))?;
        
        copy(&mut response.bytes().await?.as_ref(), &mut dest)
            .context("Failed to copy content to file")?;
        
        println!("File downloaded to: {}", file_path_str);
    }

    Ok(())
}
async fn fetch_webpage_and_download_files(url: &str, extensions: &[&str]) -> Result<()> {
    let file_urls = extract_file_urls(url, extensions).await?;
    println!("File URLs: {:?}", file_urls);
    download_files(&file_urls).await?;
    Ok(())
}

fn main() -> Result<()> {
    let url = "https://www.rust-lang.org/";
    let extensions = ["jpg", "png", "pdf", "mp4"];
    Runtime::new()?.block_on(fetch_webpage_and_download_files(url, &extensions))?;
    Ok(())
}
