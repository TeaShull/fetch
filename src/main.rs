use anyhow::Result;
use scraper::{Html, Selector};
use std::{fs::File, io::copy};
use tokio::runtime::Runtime;

async fn extract_image_urls(url: &str) -> Result<Vec<String>> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);
    let image_selector = Selector::parse("img").expect("Error parsing image selector");

    let base_url = "https://";

    let image_urls: Vec<String> = document
        .select(&image_selector)
        .filter_map(|element| element.value().attr("src"))
        .map(|src| if src.starts_with("//") { format!("{}{}", base_url, &src[2..]) } else { src.to_string() })
        .collect();

    Ok(image_urls)
}

async fn download_images(image_urls: &[String]) -> Result<()> {
    for (index, image_url) in image_urls.iter().enumerate() {
        let response = reqwest::get(image_url).await?;
        let mut dest = File::create(format!("image_{}.jpg", index))?;
        copy(&mut response.bytes().await?.as_ref(), &mut dest)?;
    }
    Ok(())
}

async fn fetch_webpage(url: &str) -> Result<()> {
    let image_urls = extract_image_urls(url).await?;
    println!("Image URLs: {:?}", image_urls);
    download_images(&image_urls).await?;
    Ok(())
}

fn main() -> Result<()> {
    let url = ""; 
    Runtime::new()?.block_on(fetch_webpage(url))?;
    Ok(())
}

