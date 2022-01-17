#[macro_use]
extern crate prettytable;

pub use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod error;
pub use error::{Error, ErrorKind};

pub mod sym;
pub use sym::{Guesser, MemoryRegion, RawSymbol, Symbol, SymbolLang, SymbolType};

pub mod report;
pub use report::{TotalMem, LangReport, FuncReport};

#[cfg(test)]
#[path = "./lib_tests.rs"]
mod lib_tests;

// TODO:
// Compare the performance to using other collections (e.g. HashMap,
// BTreeMap, Binary Heap)
#[derive(Debug)]
pub struct Atlas {
    pub nm: PathBuf,
    pub elf: PathBuf,
    pub lib: PathBuf,
    pub syms: Vec<Symbol>,
    pub fails: Vec<(String,String)>,
}

impl Atlas {
    // TODO:
    // Check if some from trait could be implemented for this crate's Error type
    // to get rid of the .map_err() calls. Otherwhise, a private helper function
    // could be created to handle the the canonicalizing and permission
    // checking. The return of this method could then only call .map_err() once.
    pub fn new<N,E,L>(nm: N, elf: E, lib: L) -> Result<Self, Error>
    where
        N: AsRef<Path>,
        E: AsRef<Path>,
        L: AsRef<Path>,
    {
        let curr = std::env::current_dir().unwrap();

        let nm = curr.join(nm.as_ref()).canonicalize()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;
        let elf = curr.join(elf.as_ref()).canonicalize()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;
        let lib = curr.join(lib.as_ref()).canonicalize()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;

        // Check permission by opening and closing files
        let _ = File::open(&nm)
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;
        let _ = File::open(&elf)
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;
        let _ = File::open(&lib)
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;

        Ok(Atlas { nm, elf, lib, syms: Vec::new(), fails: Vec::new() })
    }

    pub fn analyze(&mut self) -> Result<(), Error> {
        let mut gsr = Guesser::new();
        gsr.add_rust_lib(&self.nm, &self.lib).unwrap();

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
            let guess = match gsr.guess(mangled, demangled) {
                Ok(g) => g,
                Err(_) => {
                    self.fails.push((String::from(mangled), String::from(demangled)));
                    continue;
                },
            };
            self.syms.push(guess);
        }

        // The symbols *should* already be sorted but the `is_sorted_by_key`
        // method is not yet stable. Therefore, the symbols are sorted here just
        // to make sure. The `--size-sort` flag from the nm call should also not
        // be removed as this gets rid of a lot of symbols that don't have a
        // size at all (e.g. Kconfigs "00000001 A CONFIG_SHELL").
        self.syms.sort_by_key(|s| s.size);

        Ok(())
    }

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

    pub fn report_func(&self, lang: Vec<SymbolLang>, mem_type: MemoryRegion, max_count: Option<usize>) -> FuncReport<impl Iterator<Item = &Symbol> + Clone>
    {
        let iter = self.syms.iter().rev();
        let iter = iter.filter(move |s| (lang.contains(&SymbolLang::Any)) || (lang.contains(&s.lang)));
        let iter = iter.filter(move |s| (mem_type == MemoryRegion::Both) || (s.sym_type.mem_region() == mem_type));
        let iter = iter.take(if let Some(count) = max_count { count } else { usize::MAX });

        FuncReport::new(iter)
    }
}
