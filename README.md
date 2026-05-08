# gzts

High-performance parallel gzip XML timestamp rewriter. Processes thousands of `.xml.gz` files — decompress, patch `beginTime`/`endTime` attributes at byte level, recompress — without parsing XML.

## Use Case

Systems that generate periodic gzipped XML measurement files with ISO 8601 timestamps:

```xml
<measCollec beginTime="2024-03-15T08:00:00+00:00"/>
<granPeriod endTime="2024-03-15T08:15:00+00:00"/>
```

When replaying historical data for testing or load generation, `gzts` rewrites these timestamps to the current time across all files in parallel.

## How It Works

```
fs::read → GzDecoder → memmem byte search + replace → GzEncoder → fs::write
```

- No XML parsing — `memmem` SIMD search for `beginTime="` / `endTime="`, replace to next `"`
- No regex, no UTF-8 decode, no allocations for search
- Rayon work-stealing thread pool, auto-scaled to available cores
- Pure Rust compression via flate2/zlib-rs

## Installation

```bash
git clone https://github.com/yourusername/gzts.git
cd gzts
cargo build --release
```

## Configuration

All settings in a single `config.toml`:

```toml
[general]
concurrency = 8              # base thread count (threads = concurrency * 4, max 64)
high_compression = false     # false = level 1 (fast), true = level 6 (small)
offset = "+0000"             # timezone offset for output filenames

[paths]
metadata_dir = "metadata"
output_dir = "output"

[templates]
LTE = "/data/templates/LTE/day_1"
NR = "/data/templates/NR/day_1"
NR_EBSN = "/data/templates/NR_EBSN/day_1"
```

## Usage

```
gzts [OPTIONS]

Options:
  -c, --config PATH      Path to config.toml (default: config.toml)
  -r, --rop HHMM-HHMM    ROP time interval (default: auto from UTC)
  -h, --help              Show help
```

### Examples

```bash
# Use defaults (config.toml in current dir, auto-detect ROP from UTC)
./gzts

# Force specific ROP interval
./gzts -r 1330-1345

# Custom config path
./gzts -c /etc/gzts/config.toml
```

## Metadata Structure

`gzts` reads pre-generated per-ROP JSON files listing template filenames to process:

```
metadata/
├── lte/
│   ├── 0000-0015.json
│   ├── 0015-0030.json
│   └── ...
├── nr/
│   └── ...
├── nr_ebsn_cucp/
│   └── ...
├── nr_ebsn_cuup/
│   └── ...
└── nr_ebsn_du/
    └── ...
```

Each JSON file is a flat array of template filenames:

```json
["A20260508.1330+0000-1345+0000_Node001_statsfile.xml.gz", ...]
```

## Architecture

```
src/
├── main.rs        — orchestration, parallel dispatch
├── cli.rs         — argument parsing
├── config.rs      — TOML config loading
├── gzip.rs        — decompress/compress via flate2
├── metadata.rs    — per-ROP JSON metadata loading
├── processor.rs   — per-file pipeline (decompress → patch → compress)
└── rop.rs         — ROP time computation from UTC
```

~350 lines total. No macros, no unsafe, no C dependencies.

## Performance

On a 32-core machine processing ~54,000 files:

| Metric | Value |
|---|---|
| Wall time | ~14s |
| Files/second | ~3,800 |
| Throughput (compressed read) | ~370 MB/s |
| Throughput (decompressed) | ~2.7 GB/s |

Scales linearly with cores due to embarrassingly parallel workload.

## License

MIT
