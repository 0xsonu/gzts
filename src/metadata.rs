use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const FILE_TYPES: &[&str] = &["lte", "nr", "nr_ebsn_cucp", "nr_ebsn_cuup", "nr_ebsn_du"];

pub fn state_key_for_type(file_type: &str) -> &str {
    match file_type {
        "lte" => "LTE",
        "nr" => "NR",
        "nr_ebsn_cucp" | "nr_ebsn_cuup" | "nr_ebsn_du" => "NR_EBSN",
        _ => file_type,
    }
}

pub fn load_rop_files(metadata_dir: &str, file_type: &str, rop: &str) -> Vec<String> {
    let path = Path::new(metadata_dir)
        .join(file_type)
        .join(format!("{}.json", rop));

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn available_rops(metadata_dir: &str) -> Vec<String> {
    let mut rops = std::collections::HashSet::new();
    for ftype in FILE_TYPES {
        let dir = Path::new(metadata_dir).join(ftype);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".json") {
                    rops.insert(name[..name.len() - 5].to_string());
                }
            }
        }
    }
    let mut v: Vec<String> = rops.into_iter().collect();
    v.sort();
    v
}

pub fn load_all_for_rop(metadata_dir: &str, rop: &str) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();
    for ftype in FILE_TYPES {
        let files = load_rop_files(metadata_dir, ftype, rop);
        if !files.is_empty() {
            result.insert(ftype.to_string(), files);
        }
    }
    result
}
