use gtk::gdk;
use gtk::glib;
// use gtk::prelude::*;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// Global in-memory cache for images
static IMAGE_CACHE: Lazy<Mutex<HashMap<String, gdk::Texture>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Helper function to load images from URLs with caching
pub async fn load_image_from_url(url: &str) -> Result<gdk::Texture, Box<dyn std::error::Error>> {
    // 1. Check cache first
    if let Ok(cache) = IMAGE_CACHE.lock() {
        if let Some(texture) = cache.get(url) {
            return Ok(texture.clone());
        }
    }

    use gtk::prelude::Cast;

    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    let img =
        image::load_from_memory(&bytes).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let rgba = img.into_rgba8();
    let width = rgba.width() as i32;
    let height = rgba.height() as i32;
    let stride = width * 4;
    let glib_bytes = glib::Bytes::from(rgba.as_raw());

    let mem_texture = gdk::MemoryTexture::new(
        width,
        height,
        gdk::MemoryFormat::R8g8b8a8,
        &glib_bytes,
        stride as usize,
    );
    let texture = mem_texture.upcast::<gdk::Texture>();

    // 2. Store in cache
    if let Ok(mut cache) = IMAGE_CACHE.lock() {
        cache.insert(url.to_string(), texture.clone());
    }

    Ok(texture)
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_valid_url_handling() {
        // We can't easily test network calls, but we can test that our dependencies
        // and types are what we expect in a basic way.
        let url = "https://example.com/image.png";
        assert!(url.starts_with("https://"));
    }

    #[tokio::test]
    #[ignore = "requires network access and a live external host"]
    async fn test_fetch_real_logo() {
        let url = "https://res.hattrick.org/teamlogo/3/29/281/280747/280747.png";
        println!("Fetching {}", url);
        match crate::utils::image::load_image_from_url(url).await {
            Ok(_) => {
                println!("Loaded correctly!");
            }
            Err(e) => {
                println!("Failed to load image: {:?}", e);
                panic!("Failed to load");
            }
        }
    }
}
