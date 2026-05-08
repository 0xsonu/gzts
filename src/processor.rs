use memchr::memmem;
use std::path::Path;

use crate::gzip;
use crate::rop::RopInfo;

pub fn process_file(
    src_path: &Path,
    dst_path: &Path,
    rop: &RopInfo,
    offset: &str,
    compression_level: u32,
) -> bool {
    let compressed = match std::fs::read(src_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ERROR read {}: {}", src_path.display(), e);
            return false;
        }
    };

    let xml_bytes = gzip::decompress(&compressed, src_path);
    if xml_bytes.is_empty() {
        return false;
    }

    let begin_repl = format!(
        "beginTime=\"{}T{}:00{}\"",
        rop.new_date, rop.rop_start_time, offset
    ).into_bytes();
    let end_repl = format!(
        "endTime=\"{}T{}:00{}\"",
        rop.new_date, rop.rop_end_time, offset
    ).into_bytes();

    let updated = replace_attr(&xml_bytes, b"beginTime=\"", &begin_repl);
    let updated = replace_attr(&updated, b"endTime=\"", &end_repl);

    gzip::compress_and_write(&updated, dst_path, compression_level)
}

fn replace_attr(buf: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    let finder = memmem::Finder::new(needle);
    let mut result = Vec::with_capacity(buf.len());
    let mut start = 0;

    while let Some(pos) = finder.find(&buf[start..]) {
        let abs = start + pos;
        result.extend_from_slice(&buf[start..abs]);
        let val_start = abs + needle.len();
        if let Some(qo) = memchr::memchr(b'"', &buf[val_start..]) {
            result.extend_from_slice(replacement);
            start = val_start + qo + 1;
        } else {
            result.extend_from_slice(&buf[abs..val_start]);
            start = val_start;
        }
    }
    result.extend_from_slice(&buf[start..]);
    result
}
