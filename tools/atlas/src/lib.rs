//! A tool for analyzing the memory usage in statically linked binaries. It
//! allows the memory usage (ROM, RAM, or both) of C, Cpp, and Rust to be
//! compared within the ELF file. Furthermore, the various symbols contained in
//! the ELF file can be filtered according to language, memory region (ROM,
//! RAM), or even symbol type (BSS data section, text section, ...).
//!
//! Currently, this tool is intended to analyze the memory usage of C, Cpp, and
//! Rust within Zephyr RTOS applications. Zephyr and its associated tools
//! got their names from greek mythology, which is the reason that this tool got
//! its name. It comes from the Titan "Atlas" who was responsible for bearing
//! the weight of the heavens on his shoulders.

#[macro_use]
extern crate prettytable;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod detect;
pub use detect::{LangDetector, Library};

pub mod error;
pub use error::{Error, ErrorKind};

pub mod sym;
pub use sym::{MemoryRegion, RawSymbol, Symbol, SymbolLang, SymbolType};

pub mod report;
pub use report::{LangReport, SymbolReport, TotalMem};

#[cfg(test)]
#[path = "./lib_tests.rs"]
mod lib_tests;

/// Conducts the analysis of the ELF file and generates report type for printing
/// the gathered information.
///
/// Access to the static libraries used for building the executable allows to
/// definitively determine if any given symbol originated from a library, even
/// if `#[no_mangle]` was used (in the case of Rust). The language of all
/// remaining symbols are then set to the default values.
// TODO:
// - Compare the performance to using other collections (e.g. HashMap, BTreeMap)
#[derive(Debug)]
pub struct Atlas {
    /// Canonicalized path to the nm utility
    pub nm: PathBuf,
    /// Absolute path to the ELF binary
    pub elf: PathBuf,
    /// Absolute path to the static libraries
    pub libs: Vec<Library>,
    /// Vector containing the symbols with their identified origin language.
    pub syms: Option<Vec<Symbol>>,
    /// Vector containing the strings (mangled and demangled) of all symbols
    /// whose language couldn't be determined
    pub fails: Option<Vec<(String, String)>>,
}

impl Atlas {
    /// Creates a new instance of the [`Atlas`] struct by checking and storing
    /// all the paths needed for analysis. Returns an [`ErrorKind::Io`] error if
    /// any of the files couldn't be found or a "permission denied" error
    /// occurred.
    ///
    /// It is recommended to use the exact nm utility that was used when
    /// building the ELF file as otherwise errors could occur while demangling
    /// of the Rust and Cpp symbols.
    ///
    ///
    /// All path provided can either be absolute or relative.
    pub fn new<N, E>(nm: N, elf: E) -> Result<Self, Error>
    where
        N: AsRef<Path>,
        E: AsRef<Path>,
    {
        let curr = std::env::current_dir().unwrap();

        let nm = curr
            .join(nm.as_ref())
            .canonicalize()?;
        let elf = curr
            .join(elf.as_ref())
            .canonicalize()?;

        // Check permission by opening and closing files
        let _ = File::open(&nm)?;
        let _ = File::open(&elf)?;

        Ok(Atlas {
            nm,
            elf,
            libs: Vec::new(),
            syms: None,
            fails: None,
        })
    }

    /// Adds libraries to the [`Atlas`] struct which will be used to determine
    /// their origin when calling [`analyze`]. The path can be either absolute
    /// or relative.
    pub fn add_lib<T>(&mut self, lang: SymbolLang, lib_path: T) -> Result<(), Error>
    where
        T: AsRef<Path>,
    {
        let curr = std::env::current_dir().unwrap();

        let lib = curr
            .join(lib_path.as_ref())
            .canonicalize()?;

        // Check permission by opening and closing files
        let _ = File::open(&lib)?;

        let lib = Library::new(lang, lib);

        self.libs.push(lib);

        Ok(())
    }

    /// Analyzes the ELF file using the nm utility and static libraries, and
    /// stores the created symbols in the `syms` Vec. Failed symbols are stored
    /// in the `fails` Vec as a tuple of Strings (mangled, demangled).
    pub fn analyze(&mut self) -> Result<(), Error> {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        for lib in &self.libs {
            detector.add_lib(&self.nm, lib).unwrap();
        }

        let mangled_out = Command::new(&self.nm)
            .arg("--print-size")
            .arg("--size-sort")
            .arg(&self.elf)
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;

        if !mangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let mangled_str = std::str::from_utf8(&mangled_out.stdout)
            .map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        let demangled_out = Command::new(&self.nm)
            .arg("--print-size")
            .arg("--size-sort")
            .arg("--demangle")
            .arg(&self.elf)
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;

        if !demangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let demangled_str = std::str::from_utf8(&demangled_out.stdout)
            .map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        let mut syms = Vec::new();
        let mut fails = Vec::new();

        for (mangled, demangled) in mangled_str.lines().zip(demangled_str.lines()) {
            let detected = match detector.detect(mangled, demangled) {
                Ok(g) => g,
                Err(_) => {
                    fails.push((String::from(mangled), String::from(demangled)));
                    continue;
                }
            };
            syms.push(detected);
        }

        // The symbols *should* already be sorted but the `is_sorted_by_key`
        // method is not yet stable. Therefore, the symbols are sorted here just
        // to make sure. The `--size-sort` flag from the nm call should also not
        // be removed as this gets rid of a lot of symbols that don't have a
        // size at all (e.g. Kconfigs "00000001 A CONFIG_SHELL").
        syms.sort_by_key(|s| s.size);
        self.syms = Some(syms);
        self.fails = Some(fails);

        Ok(())
    }

    /// Creates a language report which contains the absolute and relative
    /// memory usage of C, Cpp, and Rust for the different memory regions (ROM,
    /// RAM, both).
    pub fn report_lang(&self) -> Option<LangReport> {
        let syms = self.syms.as_ref()?;
        let c = TotalMem::new(
            syms.iter()
                .filter(|s| s.lang == SymbolLang::C)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Rom)
                .fold(0, |acc, s| acc + s.size as u64),
            syms.iter()
                .filter(|s| s.lang == SymbolLang::C)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Ram)
                .fold(0, |acc, s| acc + s.size as u64),
        );

        let cpp = TotalMem::new(
            syms.iter()
                .filter(|s| s.lang == SymbolLang::Cpp)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Rom)
                .fold(0, |acc, s| acc + s.size as u64),
            syms.iter()
                .filter(|s| s.lang == SymbolLang::Cpp)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Ram)
                .fold(0, |acc, s| acc + s.size as u64),
        );

        let rust = TotalMem::new(
            syms.iter()
                .filter(|s| s.lang == SymbolLang::Rust)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Rom)
                .fold(0, |acc, s| acc + s.size as u64),
            syms.iter()
                .filter(|s| s.lang == SymbolLang::Rust)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Ram)
                .fold(0, |acc, s| acc + s.size as u64),
        );
        Some(LangReport::new(c, cpp, rust))
    }

    /// Creates a symbol report starting with the largest symbols for the
    /// selected languages and memory regions. [`SymbolLang::Any`] can be passed
    /// as the only item in the `lang` Vec to select all languages. Otherwise,
    /// one or more specific languages can be used. `max_count` can be used to
    /// limit the amount of symbols in the report. Passing `None` will return a
    /// report with all symbols.
    pub fn report_syms(
        &self,
        lang: Vec<SymbolLang>,
        mem_region: MemoryRegion,
        max_count: Option<usize>,
    ) -> Option<SymbolReport<impl Iterator<Item = &Symbol> + Clone>> {
        let iter = self.syms.as_ref()?.iter().rev();
        let iter =
            iter.filter(move |s| (lang.contains(&SymbolLang::Any)) || (lang.contains(&s.lang)));
        let iter = iter.filter(move |s| {
            (mem_region == MemoryRegion::Both) || (s.sym_type.mem_region() == mem_region)
        });
        let iter = iter.take(if let Some(count) = max_count {
            count
        } else {
            usize::MAX
        });

        Some(SymbolReport::new(iter))
    }
}
