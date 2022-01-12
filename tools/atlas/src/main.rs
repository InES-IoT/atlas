use atlas::Atlas;
use atlas::sym::{MemoryRegion, SymbolLang};
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

/// Atlas analyzes an ELF binary and analyzes the memory usage in regards to
/// languages (C, Cpp, Rust), memory regions (e.g. ROM, RAM), and memory
/// sections (e.g. BSS section, read-only data section, text section).
#[derive(Debug, Parser)]
#[clap(about, author, version)]
struct Args {
    /// Path to NM binary.
    #[clap(long)]
    nm: PathBuf,

    /// Path to application elf.
    #[clap(long)]
    elf: PathBuf,

    /// Path to Rust library.
    #[clap(long)]
    lib: PathBuf,

    /// Select the languages included in the function report. Multiple
    /// selections are possible. (any, c, cpp, rust)
    #[clap(short, long, default_value = "any")]
    lang: Vec<String>,

    /// Select the memory region used for the reports. (both, ram, rom)
    #[clap(short, long, default_value = "rom")]
    region: String,

    /// Max count for printing function reports.
    #[clap(short, long)]
    count: Option<usize>,

    /// Print a size summary of the languages.
    #[clap(short, long)]
    summary: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    println!("{:#?}", &args);

    let region = args.region.to_lowercase().as_str().parse::<MemoryRegion>()?;
    let lang = args.lang.iter().map(|l| l.to_lowercase().as_str().parse::<SymbolLang>()).collect::<Result<Vec<_>,_>>()?;

    let mut at = Atlas::new(&args.nm, &args.elf, &args.lib)?;
    at.analyze()?;

    if args.summary {
        let lang_rep = at.report_lang();
        lang_rep.print(region, &mut std::io::stdout())?;
    } else {
        let func_rep = at.report_func(lang, region, args.count);
        func_rep.print(&mut std::io::stdout())?;
    }

    Ok(())
}
