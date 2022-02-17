use crate::error::{Error, ErrorKind};
use crate::sym::{RawSymbol, Symbol, SymbolLang};
use std::convert::TryInto;
use std::path::Path;
use std::process::Command;

#[cfg(test)]
#[path = "./detect_tests.rs"]
mod detect_tests;

/// Struct containing the necessary information to determine the origin language
/// of [`Symbol`]s.
///
/// # Note:
/// The name was given during a time when the origin language of a symbol could
/// not be determined with absolute certainty. This should be renamed in a
/// future release.
// TODO:
// Rewrite this struct with the builder pattern.
#[derive(Debug, Default)]
pub struct LangDetector {
    lib_syms: Vec<Symbol>,
}

impl LangDetector {
    /// Creates a new [`LangDetector`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Parses and stores the symbols contained in the Rust library with the
    /// supplied nm utility. This can then be used by the [`detect`] method for
    /// determining if a symbol stems from a Rust library or not.
    ///
    /// [`detect`]: LangDetector::detect
    pub fn add_rust_lib<T, U>(&mut self, nm: T, lib: U) -> Result<(), Error>
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
    {
        let mangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg(lib.as_ref())
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Nm).with(io_error))?;

        if !mangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let mangled_str = std::str::from_utf8(&mangled_out.stdout)
            .map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        let demangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg("--demangle")
            .arg(lib.as_ref())
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Nm).with(io_error))?;

        if !demangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let demangled_str = std::str::from_utf8(&demangled_out.stdout)
            .map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        for (mangled, demangled) in mangled_str.lines().zip(demangled_str.lines()) {
            let s = match Symbol::from_rawsymbols_lang(mangled, demangled, SymbolLang::Rust) {
                Ok(s) => s,
                // TODO:
                // Differentiate between the various reasons for an error. Some
                // might be expected (e.g lines like "mulvdi3.o:") while others
                // should not fail and should inform the user.
                Err(_) => continue,
            };

            // TODO:
            // Why is this check here? Find out and make a comment here.
            if s.mangled == s.demangled {
                // TODO:
                // Rewrite this using a simple regex and check the performance
                // difference
                let mut chars = s.mangled.chars();
                if let Some(c) = chars.next() {
                    if matches!(c, 'a'..='z' | 'A'..='Z' | '_') {
                        // TODO:
                        // Reuse the iterator here?
                        if s.mangled
                            .chars()
                            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
                        {
                            self.lib_syms.push(s);
                        }
                    }
                }
            } else {
                self.lib_syms.push(s);
            }
        }
        Ok(())
    }

    /// Detect the origin language of symbol. First, this checks if the symbol
    /// is related (using [`Symbol::related`]) to any of the symbols parsed from
    /// the Rust library with [`add_rust_lib`] and set to [`SymbolLang::Rust`].
    /// If it isn't related to any of them, the language is set to
    /// [`SymbolLang::C`] if the mangled and demangled name of the symbol is the
    /// same (C doesn't have name mangling). Otherwise, it is set to
    /// [`SymbolLang::Cpp`].
    ///
    /// [`add_rust_lib`]: LangDetector::add_rust_lib
    // Rename this method `detect_raw` and create a second method called `detect`.
    // This method will then only create the symbol from the rawsymbols and call
    // the `detect` method. This would allow the user to detect the language for
    // an already created Symbol.
    pub fn detect<T>(&self, mangled: T, demangled: T) -> Result<Symbol, Error>
    where
        T: TryInto<RawSymbol>,
        Error: From<<T as TryInto<RawSymbol>>::Error>,
    {
        let mut sym = Symbol::from_rawsymbols(mangled, demangled)?;

        if self.lib_syms.iter().any(|lib_sym| sym.related(lib_sym)) {
            sym.lang = SymbolLang::Rust;
        } else {
            if sym.mangled == sym.demangled {
                sym.lang = SymbolLang::C;
            } else {
                sym.lang = SymbolLang::Cpp;
            }
        }
        Ok(sym)
    }
}
