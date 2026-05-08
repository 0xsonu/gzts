pub struct Args {
    pub config_path: String,
    pub forced_rop: Option<String>,
}

pub fn parse() -> Args {
    let argv: Vec<String> = std::env::args().collect();
    let mut args = Args {
        config_path: "config.toml".into(),
        forced_rop: None,
    };
    let mut i = 1;

    while i < argv.len() {
        match argv[i].as_str() {
            "-h" | "--help" => {
                print_usage(&argv[0]);
                std::process::exit(0);
            }
            "-c" | "--config" => { i += 1; args.config_path = argv.get(i).cloned().unwrap_or(args.config_path); }
            "-r" | "--rop" => { i += 1; args.forced_rop = argv.get(i).cloned(); }
            other => {
                eprintln!("ERROR: Unknown option: {}\n", other);
                print_usage(&argv[0]);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    args
}

fn print_usage(prog: &str) {
    eprintln!("gzts — High-performance parallel gzip XML timestamp rewriter\n");
    eprintln!("Usage: {} [OPTIONS]\n", prog);
    eprintln!("Options:");
    eprintln!("  -c, --config PATH      Path to config.toml (default: config.toml)");
    eprintln!("  -r, --rop HHMM-HHMM    ROP time interval (default: auto from UTC)");
    eprintln!("  -h, --help              Show this help");
}
