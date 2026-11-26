use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub max_file_size_bytes: usize,
    pub tesseract_path: String,
    pub tessdata_prefix: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8081);

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        // Default max file size: 35 MB
        let max_file_size_bytes = env::var("MAX_FILE_SIZE_MB")
            .ok()
            .and_then(|m| m.parse::<usize>().ok())
            .map(|m| m * 1024 * 1024)
            .unwrap_or(35 * 1024 * 1024);

        let tesseract_path = env::var("TESSERACT_PATH")
            .unwrap_or_else(|_| "/usr/bin/tesseract".to_string());

        let tessdata_prefix = env::var("TESSDATA_PREFIX").ok();

        Self {
            port,
            host,
            max_file_size_bytes,
            tesseract_path,
            tessdata_prefix,
        }
    }
}
