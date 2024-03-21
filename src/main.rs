use reqwest::Error;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::copy;

async fn extract_image_urls(url: &str) -> Result<Vec<String>, Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let html = Html::parse_document(&body);

    let image_selector = Selector::parse("img").unwrap();
    let image_urls: Vec<String> = html
        .select(&image_selector)
        .filter_map(|element| element.value().attr("src"))
        .filter_map(|src| Some(src.to_owned()))
        .collect();

    Ok(image_urls)
}

async fn fetch_webpage(url: &str) -> Result<(), Error> {
    let image_urls = extract_image_urls(url).await?;
    println!("Image URLs: {:?}", image_urls);
    download_images(&image_urls).await?;
    Ok(())
}

async fn download_images(image_urls: &[String]) -> Result<(), Error> {
    for (index, image_url) in image_urls.iter().enumerate() {
        let response = reqwest::get(image_url).await?;
        let mut dest = File::create(format!("image_{}.jpg", index))
            .map_err(|err| err.into())?;
        copy(&mut response.bytes().await?.as_ref(), &mut dest)
            .map_err(|err| err.into())?;
    }
    Ok(())
}

fn main() {
    let url = "https://boards.4chan.org/fit/thread/73870683";
    tokio::runtime::Runtime::new().unwrap().block_on(fetch_webpage(url)).unwrap();
}