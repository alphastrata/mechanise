use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

impl ImageSource {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let media_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported image file extension",
            )),
        };

        let data = fs::read(path)?;
        #[allow(deprecated)]
        let base64_data = base64::encode(&data);

        Ok(ImageSource {
            source_type: "base64".to_string(),
            media_type: media_type.to_string(),
            data: base64_data,
        })
    }
}

