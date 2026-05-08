mod cli;
mod config;
mod gzip;
mod metadata;
mod processor;
mod rop;

use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let args = cli::parse();

    println!("gzts — Gzip Timestamp Rewriter (pure Rust, zlib-rs)\n");

    let cfg = config::AppConfig::load(&args.config_path);
    let rop_info = rop::RopInfo::from_current_utc();
    let current_rop = args.forced_rop.clone().unwrap_or_else(|| rop_info.current_rop());
    let threads = (cfg.concurrency * 4).min(64);

    println!("--- Configuration ---");
    println!("  config       : {}", args.config_path);
    println!("  metadata dir : {}", cfg.metadata_dir);
    println!("  output dir   : {}", cfg.output_dir);
    println!("  rop          : {} ({})", current_rop, if args.forced_rop.is_some() { "forced" } else { "auto" });
    println!("  compression  : level {} ({})", cfg.compression_level, if cfg.high_compression { "high" } else { "fast" });
    println!("  concurrency  : {} (threads: {})", cfg.concurrency, threads);
    println!("  offset       : {}", cfg.offset);
    for (k, v) in &cfg.type_paths {
        println!("  {} path : {}", k, v);
    }
    println!("---------------------\n");

    let rop_files = metadata::load_all_for_rop(&cfg.metadata_dir, &current_rop);

    if rop_files.is_empty() {
        eprintln!("ERROR: No files found for ROP '{}'", current_rop);
        let available = metadata::available_rops(&cfg.metadata_dir);
        if available.is_empty() {
            eprintln!("  No metadata found in: {}", cfg.metadata_dir);
        } else {
            eprintln!("  Available ROPs: {:?}", &available[..available.len().min(10)]);
        }
        return;
    }

    let mut tasks: Vec<(PathBuf, PathBuf, usize)> = Vec::new();
    let mut dirs_to_create: HashSet<PathBuf> = HashSet::new();
    let mut type_labels: Vec<String> = Vec::new();
    let offset_for_replacement = "+00:00".to_string();

    for (file_type, file_list) in &rop_files {
        let state_key = metadata::state_key_for_type(file_type);
        let template_dir = match cfg.type_paths.get(state_key) {
            Some(p) => p.clone(),
            None => {
                println!("WARN: No template path for '{}' (key='{}')", file_type, state_key);
                continue;
            }
        };

        let type_idx = type_labels.len();
        type_labels.push(file_type.clone());

        let mut seen = HashSet::new();
        for filename in file_list {
            let key = filename.splitn(2, '_').nth(1).unwrap_or(filename);
            if !seen.insert(key.to_string()) {
                continue;
            }

            let template_name = filename.splitn(2, '_').nth(1).unwrap_or(filename);
            let new_name = format!(
                "A{}.{}{}-{}{}_{}",
                rop_info.rop_start_date,
                rop_info.rop_start_hhmm, cfg.offset,
                rop_info.rop_end_hhmm, cfg.offset,
                template_name
            );

            let folder = if file_type.starts_with("nr_ebsn") {
                file_type.to_uppercase()
            } else {
                let under = new_name.find('_');
                if let Some(pos) = under {
                    let after = &new_name[pos + 1..];
                    match after.rfind(',') {
                        Some(cp) => after[..cp].to_string(),
                        None => after.to_string(),
                    }
                } else {
                    new_name.clone()
                }
            };

            let sub = PathBuf::from(&cfg.output_dir)
                .join(&folder)
                .join(&rop_info.epoch_range);
            dirs_to_create.insert(sub.clone());
            tasks.push((
                Path::new(&template_dir).join(filename),
                sub.join(&new_name),
                type_idx,
            ));
        }
    }

    for dir in &dirs_to_create {
        let _ = std::fs::create_dir_all(dir);
    }

    println!("Tasks: {} | Threads: {}", tasks.len(), threads);

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();

    let counts: Vec<AtomicUsize> = (0..type_labels.len())
        .map(|_| AtomicUsize::new(0))
        .collect();

    tasks.par_iter().for_each(|(src, dst, type_idx)| {
        if processor::process_file(src, dst, &rop_info, &offset_for_replacement, cfg.compression_level) {
            counts[*type_idx].fetch_add(1, Ordering::Relaxed);
        }
    });

    let elapsed = start.elapsed();
    for (i, label) in type_labels.iter().enumerate() {
        let c = counts[i].load(Ordering::Relaxed);
        if c > 0 {
            println!("INFO: Updated {} {} files for rop: {}", c, label.to_uppercase(), current_rop);
        }
    }
    println!("Total: {:.4}s", elapsed.as_secs_f64());
}
