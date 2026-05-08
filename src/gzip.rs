use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use std::path::Path;

pub fn decompress(compressed: &[u8], src_path: &Path) -> Vec<u8> {
    let expected = if compressed.len() >= 4 {
        let f = &compressed[compressed.len() - 4..];
        u32::from_le_bytes([f[0], f[1], f[2], f[3]]) as usize
    } else {
        compressed.len() * 10
    };
    let mut buf = Vec::with_capacity(expected.min(64 * 1024 * 1024));
    match GzDecoder::new(compressed).read_to_end(&mut buf) {
        Ok(_) => buf,
        Err(e) => {
            eprintln!("ERROR decompress {}: {}", src_path.display(), e);
            Vec::new()
        }
    }
}

pub fn compress_and_write(data: &[u8], path: &Path, level: u32) -> bool {
    let mut out = Vec::with_capacity(data.len() / 2);
    {
        let mut enc = GzEncoder::new(&mut out, Compression::new(level));
        if enc.write_all(data).is_err() {
            return false;
        }
        if enc.finish().is_err() {
            return false;
        }
    }
    match std::fs::write(path, &out) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("ERROR write {}: {}", path.display(), e);
            false
        }
    }
}
