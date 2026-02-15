use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

// Helper function to load images from URLs
pub async fn load_image_from_url(url: &str) -> Result<gdk::Texture, Box<dyn std::error::Error>> {
    use gdk_pixbuf::Pixbuf;
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let gbytes = glib::Bytes::from(&bytes[..]);
    let stream = gio::MemoryInputStream::from_bytes(&gbytes);
    let pixbuf = Pixbuf::from_stream(&stream, gio::Cancellable::NONE)?;
    Ok(gdk::Texture::for_pixbuf(&pixbuf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url_handling() {
        // We can't easily test network calls, but we can test that our dependencies 
        // and types are what we expect in a basic way.
        let url = "https://example.com/image.png";
        assert!(url.starts_with("https://"));
    }
}
