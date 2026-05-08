use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
struct ConfigFile {
    general: General,
    paths: Paths,
    templates: HashMap<String, String>,
}

#[derive(Deserialize)]
struct General {
    concurrency: Option<usize>,
    high_compression: Option<bool>,
    offset: Option<String>,
}

#[derive(Deserialize)]
struct Paths {
    metadata_dir: Option<String>,
    output_dir: Option<String>,
}

pub struct AppConfig {
    pub concurrency: usize,
    pub high_compression: bool,
    pub compression_level: u32,
    pub offset: String,
    pub metadata_dir: String,
    pub output_dir: String,
    pub type_paths: HashMap<String, String>,
}

impl AppConfig {
    pub fn load(path: &str) -> Self {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Cannot read config file '{}': {}", path, e));
        let cfg: ConfigFile = toml::from_str(&content)
            .unwrap_or_else(|e| panic!("Invalid TOML in '{}': {}", path, e));

        let high_compression = cfg.general.high_compression.unwrap_or(false);
        let concurrency = cfg.general.concurrency.unwrap_or(8);

        Self {
            concurrency,
            high_compression,
            compression_level: if high_compression { 6 } else { 1 },
            offset: cfg.general.offset.unwrap_or_else(|| "+0000".into()),
            metadata_dir: cfg.paths.metadata_dir.unwrap_or_else(|| "metadata".into()),
            output_dir: cfg.paths.output_dir.unwrap_or_else(|| "output".into()),
            type_paths: cfg.templates,
        }
    }
}
