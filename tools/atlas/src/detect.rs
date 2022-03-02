use crate::error::{Error, ErrorKind};
use crate::sym::{RawSymbol, Symbol, SymbolLang};
use std::convert::TryInto;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(test)]
#[path = "./detect_tests.rs"]
mod detect_tests;

// TODO:
// Path doesn't need to be owned (-> &Path).
#[derive(Debug, PartialEq)]
pub struct Library {
    path: PathBuf,
    lang: SymbolLang,
}

impl Library {
    pub fn new<T>(lang: SymbolLang, path: T) -> Self
    where
        T: AsRef<Path>
    {
        Self {
            path: path.as_ref().to_path_buf(),
            lang
        }
    }
}

#[derive(Debug, PartialEq)]
struct ParsedLibrary {
    path: PathBuf,
    lang: SymbolLang,
    syms: Vec<Symbol>,
}

/// Struct containing the necessary information to determine the origin language
/// of [`Symbol`]s.
#[derive(Debug)]
pub struct LangDetector {
    default_lang: SymbolLang,
    default_mangled_lang: SymbolLang,
    libs: Vec<ParsedLibrary>,
}

impl LangDetector {
    /// Creates a new [`LangDetector`].
    // TODO:
    // Make `default_mangled_lang` optional and return an error (or something) in case a mangled
    // symbol is found that is not present in any of the libraries if this is set to None.
    pub fn new(default_lang: SymbolLang, default_mangled_lang: SymbolLang) -> Self {
        Self {
            default_lang,
            default_mangled_lang,
            libs: Vec::new(),
        }
    }

    /// Parses and stores the symbols contained in the library with the supplied nm utility. This
    /// can then be used by the [`detect`] method for determining if a symbol stems from a library
    /// or not.
    ///
    /// [`detect`]: LangDetector::detect
    pub fn add_lib<T>(&mut self, nm: T, lib: &Library) -> Result<(), Error>
    where
        T: AsRef<Path>,
    {
        // This check makes sure that an ErrorKind::Io error is returned if the libary file cannot
        // be found. Otherwhise, nm would still run but not succeed, thus resulting in an
        // ErrorKind::Nm error.
        let _ = File::open(&lib.path)?;

        let mangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg(&lib.path)
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;

        if !mangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let mangled_str = std::str::from_utf8(&mangled_out.stdout)
            .map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        let demangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg("--demangle")
            .arg(&lib.path)
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))?;

        if !demangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let demangled_str = std::str::from_utf8(&demangled_out.stdout)
            .map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        let mut parsed_lib = ParsedLibrary {
            path: lib.path.clone(),
            lang: lib.lang,
            syms: Vec::new(),
        };

        for (mangled, demangled) in mangled_str.lines().zip(demangled_str.lines()) {
            let s = match Symbol::from_rawsymbols_lang(mangled, demangled, SymbolLang::Rust) {
                Ok(s) => s,
                // TODO:
                // Differentiate between the various reasons for an error. Some
                // might be expected (e.g lines like "mulvdi3.o:") while others
                // should not fail and should inform the user.
                Err(_) => continue,
            };

            // The symbols that have distinct mangled and demangled names are added to the parsed
            // library without any further checks. Symbols, where the mangled and demangled names
            // match, are further checked to be valid C identifiers. I.e., underscores, lower- or
            // uppercase letters, or numbers (not allowed for the first character). Additionally,
            // the dot "." character is also allowed as it seems to be used for symbols in RAM like
            // "000194f0 00000018 b object.8916". This logic thus excludes symbols like
            // ".Lanon.4575732b5f0a476c725a4805a4f03b6f.638" for example, which seem to be unused
            // symbols from Rust static libraries.
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
                            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '.' | '0'..='9'))
                        {
                            parsed_lib.syms.push(s);
                        }
                    }
                }
            } else {
                parsed_lib.syms.push(s);
            }
        }

        self.libs.push(parsed_lib);

        Ok(())
    }

    /// Detect the origin language of symbol. First, this checks if the symbol
    /// is related (using [`Symbol::related`]) to any of the symbols parsed from
    /// the libraries with [`add_lib`].
    /// If it isn't related to any of them, the language is set to the default stored in the
    /// `default_lang` member of Self if the mangled and demangled name of the symbol is the
    /// same. Otherwise, it is set to `default_mangled_lang`.
    ///
    /// [`add_lib`]: LangDetector::add_lib
    // TODO:
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

        for lib in self.libs.iter() {
            if lib.syms.iter().any(|lib_sym| sym.related(lib_sym)) {
                sym.lang = lib.lang;
                return Ok(sym)
            }
        }

        if sym.mangled == sym.demangled {
            sym.lang = self.default_lang;
        } else {
            sym.lang = self.default_mangled_lang;
        }

        Ok(sym)
    }
}
