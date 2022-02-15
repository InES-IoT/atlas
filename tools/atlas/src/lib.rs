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

pub use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod detect;
pub use detect::LangDetector;

pub mod error;
pub use error::{Error, ErrorKind};

pub mod sym;
pub use sym::{MemoryRegion, RawSymbol, Symbol, SymbolLang, SymbolType};

pub mod report;
pub use report::{FuncReport, LangReport, TotalMem};

#[cfg(test)]
#[path = "./lib_tests.rs"]
mod lib_tests;

/// Conducts the analysis of the ELF file and generates report type for printing
/// the gathered information.
///
/// Access to the static Rust library used for building the executable allows to
/// definitively determine if any given symbol originated from Rust code, even
/// if `#[no_mangle]` was used. Then, mangled names are identified as Cpp
/// symbols. Lastly, remaining symbols (only symbols without any mangling
/// are remaining) are now identified as C.
// TODO:
// - Compare the performance to using other collections (e.g. HashMap, BTreeMap)
// - Probably put the `syms` field in a Option to signal that nothing has been
//   analyzed yet and prevend the user from calling the report methods.
#[derive(Debug)]
pub struct Atlas {
    /// Canonicalized path to the nm utility
    pub nm: PathBuf,
    /// Absolute path to the ELF binary
    pub elf: PathBuf,
    /// Absolute path to the Rust static library
    pub lib: PathBuf,
    /// Vector containing the symbols with their identified origin language.
    pub syms: Vec<Symbol>,
    /// Vector containing the strings (mangled and demangled) of all symbols
    /// whose language couldn't be determined
    pub fails: Vec<(String, String)>,
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
    /// For the `lib` argument, provide the path to the static Rust library used
    /// when building the ELF file.
    ///
    /// All path provided can either be absolute or relative.
    // TODO:
    // Check if some from trait could be implemented for this crate's Error type
    // to get rid of the .map_err() calls. Otherwhise, a private helper function
    // could be created to handle the the canonicalizing and permission
    // checking. The return of this method could then only call .map_err() once.
    pub fn new<N, E, L>(nm: N, elf: E, lib: L) -> Result<Self, Error>
    where
        N: AsRef<Path>,
        E: AsRef<Path>,
        L: AsRef<Path>,
    {
        let curr = std::env::current_dir().unwrap();

        let nm = curr
            .join(nm.as_ref())
            .canonicalize()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;
        let elf = curr
            .join(elf.as_ref())
            .canonicalize()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;
        let lib = curr
            .join(lib.as_ref())
            .canonicalize()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;

        // Check permission by opening and closing files
        let _ = File::open(&nm).map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;
        let _ = File::open(&elf).map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;
        let _ = File::open(&lib).map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;

        Ok(Atlas {
            nm,
            elf,
            lib,
            syms: Vec::new(),
            fails: Vec::new(),
        })
    }

    /// Analyzes the ELF file using the nm utility and static Rust library, and
    /// stores the created symbols in the `syms` Vec. Failed symbols are stored
    /// in the `fails` Vec as a tuple of Strings (mangled, demangled).
    pub fn analyze(&mut self) -> Result<(), Error> {
        let mut detector = LangDetector::new();
        detector.add_rust_lib(&self.nm, &self.lib).unwrap();

        let mangled_out = Command::new(&self.nm)
            .arg("--print-size")
            .arg("--size-sort")
            .arg(&self.elf)
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Nm).with(io_error))?;

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
            .map_err(|io_error| Error::new(ErrorKind::Nm).with(io_error))?;

        if !demangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let demangled_str = std::str::from_utf8(&demangled_out.stdout)
            .map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        for (mangled, demangled) in mangled_str.lines().zip(demangled_str.lines()) {
            let detected = match detector.detect(mangled, demangled) {
                Ok(g) => g,
                Err(_) => {
                    self.fails
                        .push((String::from(mangled), String::from(demangled)));
                    continue;
                }
            };
            self.syms.push(detected);
        }

        // The symbols *should* already be sorted but the `is_sorted_by_key`
        // method is not yet stable. Therefore, the symbols are sorted here just
        // to make sure. The `--size-sort` flag from the nm call should also not
        // be removed as this gets rid of a lot of symbols that don't have a
        // size at all (e.g. Kconfigs "00000001 A CONFIG_SHELL").
        self.syms.sort_by_key(|s| s.size);

        Ok(())
    }

    /// Creates a language report which contains the absolute and relative
    /// memory usage of C, Cpp, and Rust for the different memory regions (ROM,
    /// RAM, both).
    pub fn report_lang(&self) -> LangReport {
        let c = TotalMem::new(
            self.syms
                .iter()
                .filter(|s| s.lang == SymbolLang::C)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Rom)
                .fold(0, |acc, s| acc + s.size as u64),
            self.syms
                .iter()
                .filter(|s| s.lang == SymbolLang::C)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Ram)
                .fold(0, |acc, s| acc + s.size as u64),
        );

        let cpp = TotalMem::new(
            self.syms
                .iter()
                .filter(|s| s.lang == SymbolLang::Cpp)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Rom)
                .fold(0, |acc, s| acc + s.size as u64),
            self.syms
                .iter()
                .filter(|s| s.lang == SymbolLang::Cpp)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Ram)
                .fold(0, |acc, s| acc + s.size as u64),
        );

        let rust = TotalMem::new(
            self.syms
                .iter()
                .filter(|s| s.lang == SymbolLang::Rust)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Rom)
                .fold(0, |acc, s| acc + s.size as u64),
            self.syms
                .iter()
                .filter(|s| s.lang == SymbolLang::Rust)
                .filter(|s| s.sym_type.mem_region() == MemoryRegion::Ram)
                .fold(0, |acc, s| acc + s.size as u64),
        );
        LangReport::new(c, cpp, rust)
    }

    /// Creates a symbol report starting with the largest symbols for the
    /// selected languages and memory regions. [`SymbolLang::Any`] can be passed
    /// as the only item in the `lang` Vec to select all languages. Otherwise,
    /// one or more specific languages can be used. `max_count` can be used to
    /// limit the amount of symbols in the report. Passing `None` will return a
    /// report with all symbols.
    ///
    /// This will probably be renamed in a future release to "report_symbols" or
    /// something similar.
    // TODO:
    // Rename to report_sym or something similar.
    pub fn report_func(
        &self,
        lang: Vec<SymbolLang>,
        mem_type: MemoryRegion,
        max_count: Option<usize>,
    ) -> FuncReport<impl Iterator<Item = &Symbol> + Clone> {
        let iter = self.syms.iter().rev();
        let iter =
            iter.filter(move |s| (lang.contains(&SymbolLang::Any)) || (lang.contains(&s.lang)));
        let iter = iter.filter(move |s| {
            (mem_type == MemoryRegion::Both) || (s.sym_type.mem_region() == mem_type)
        });
        let iter = iter.take(if let Some(count) = max_count {
            count
        } else {
            usize::MAX
        });

        FuncReport::new(iter)
    }
}
