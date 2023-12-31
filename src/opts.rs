use std::env;

pub struct Opts {
    pub help: bool,
    pub version: bool,
    pub gen_all: bool,
    pub config: String,
    pub config_dir: String,
    pub usage: String,
}

pub fn parse_opts() -> Result<Opts, getopts::Fail> {
    let args: Vec<String> = env::args().collect();

    let mut opts = getopts::Options::new();

    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "version", "show copyright and version information");

    opts.optflag(
        "G",
        "gen-all",
        "try to generate a uki for all installed kernels",
    );

    opts.optopt("c", "config", "configuration file", "FILE");
    opts.optopt("C", "config-dir", "kernel configuration directory", "DIR");

    let matches = opts.parse(&args[1..])?;

    Ok(Opts {
        help: matches.opt_present("h"),
        version: matches.opt_present("v"),
        gen_all: matches.opt_present("G"),
        config: matches
            .opt_str("c")
            .unwrap_or("/etc/uki-manager.toml".to_owned()),
        config_dir: matches
            .opt_str("C")
            .unwrap_or("/etc/uki-manager.d".to_owned()),
        usage: opts.usage("Usage: uki-manager [options]"),
    })
}
