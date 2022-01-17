use crate::error::{Error, ErrorKind};
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

#[cfg(test)]
#[path = "./sym_tests.rs"]
mod sym_tests;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MemoryRegion {
    Unknown,
    Rom,
    Ram,
    Both,
}

impl FromStr for MemoryRegion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        match lower.as_ref() {
            "unknown" => Ok( MemoryRegion::Unknown ),
            "rom" => Ok( MemoryRegion::Rom ),
            "ram" => Ok( MemoryRegion::Ram ),
            "both" => Ok( MemoryRegion::Both ),
            _ => Err(Error::new(ErrorKind::InvalidEnumStr)),
        }
    }
}

impl TryFrom<&str> for MemoryRegion {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        MemoryRegion::from_str(s)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SymbolLang {
    Any,
    Rust,
    C,
    Cpp,
}

impl FromStr for SymbolLang {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        match lower.as_ref() {
            "any" => Ok( SymbolLang::Any ),
            "c" => Ok( SymbolLang::C ),
            "cpp" => Ok( SymbolLang::Cpp ),
            "rust" => Ok( SymbolLang::Rust ),
            _ => Err(Error::new(ErrorKind::InvalidEnumStr)),
        }
    }
}

impl TryFrom<&str> for SymbolLang {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SymbolLang::from_str(s)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SymbolType {
    Absolute,
    BssSection,
    Common,
    DataSection,
    Global,
    IndirectFunction,
    Indirect,
    Debug,
    ReadOnlyDataSection,
    StackUnwindSection,
    UninitializedOrZeroInitialized,
    TextSection,
    Undefined,
    UniqueGlobal,
    TaggedWeak,
    Weak,
    Stabs,
    Unknown,
}

impl FromStr for SymbolType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1 {
            match s {
                "A" => Ok(SymbolType::Absolute),
                "B" | "b" => Ok(SymbolType::BssSection),
                "C" | "c" => Ok(SymbolType::Common),
                "D" | "d" => Ok(SymbolType::DataSection),
                "G" | "g" => Ok(SymbolType::Global),
                "I" => Ok(SymbolType::Indirect),
                "i" => Ok(SymbolType::IndirectFunction),
                "N" => Ok(SymbolType::Debug),
                "n" => Ok(SymbolType::ReadOnlyDataSection),
                "p" => Ok(SymbolType::StackUnwindSection),
                "R" | "r" => Ok(SymbolType::ReadOnlyDataSection),
                "S" | "s" => Ok(SymbolType::UninitializedOrZeroInitialized),
                "T" | "t" => Ok(SymbolType::TextSection),
                "U" => Ok(SymbolType::Undefined),
                "u" => Ok(SymbolType::UniqueGlobal),
                "V" | "v" => Ok(SymbolType::TaggedWeak),
                "W" | "w" => Ok(SymbolType::Weak),
                "-" => Ok(SymbolType::Stabs),
                "?" => Ok(SymbolType::Unknown),
                _ => return Err(Error::new(ErrorKind::InvalidEnumStr)),
            }
        } else {
            match s.to_lowercase().as_ref() {
                "absolute" => Ok(SymbolType::Absolute),
                "bsssection" => Ok(SymbolType::BssSection),
                "common" => Ok(SymbolType::Common),
                "datasection" => Ok(SymbolType::DataSection),
                "global" => Ok(SymbolType::Global),
                "indirect" => Ok(SymbolType::Indirect),
                "indirectfunction" => Ok(SymbolType::IndirectFunction),
                "debug" => Ok(SymbolType::Debug),
                "readonlydatasection" => Ok(SymbolType::ReadOnlyDataSection),
                "stackunwindsection" => Ok(SymbolType::StackUnwindSection),
                "uninitializedorzeroinitialized" => Ok(SymbolType::UninitializedOrZeroInitialized),
                "textsection" => Ok(SymbolType::TextSection),
                "undefined" => Ok(SymbolType::Undefined),
                "uniqueglobal" => Ok(SymbolType::UniqueGlobal),
                "taggedweak" => Ok(SymbolType::TaggedWeak),
                "weak" => Ok(SymbolType::Weak),
                "stabs" => Ok(SymbolType::Stabs),
                "unknown" => Ok(SymbolType::Unknown),
                _ => return Err(Error::new(ErrorKind::InvalidEnumStr)),
            }
        }
    }
}

impl TryFrom<&str> for SymbolType {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SymbolType::from_str(s)
    }
}

impl SymbolType {
    pub fn mem_region(&self) -> MemoryRegion {
        match *self {
            Self::TextSection | Self::Weak => MemoryRegion::Rom,
            Self::BssSection | Self::DataSection | Self::ReadOnlyDataSection => MemoryRegion::Ram,
            // FIXME:
            // Eventually, this should be replaced with by returning a result
            // type. However, for the meantime, let this be a panic to determine
            // during the development phase of this tool, if there are other
            // symbols that could be present in an ELF file. (I assume that some
            // symbol types should never make it to the finally linked ELF file.)
            _ => panic!("The memory region for a symbol of type {:?} is unknown!", self),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct RawSymbol {
    addr: u32,
    size: u32,
    sym_type: SymbolType,
    name: String,
}

impl Default for RawSymbol {
    fn default() -> Self {
        RawSymbol {
            addr: 0,
            size: 0,
            sym_type: SymbolType::Unknown,
            name: String::new(),
        }
    }
}

impl RawSymbol {
    pub fn new(addr: u32, size: u32, sym_type: SymbolType, name: String) -> Self {
        RawSymbol {
            addr,
            size,
            sym_type,
            name,
        }
    }
}

impl FromStr for RawSymbol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^\s*([0-9a-fA-F]{8})\s+([0-9a-fA-F]{8})\s+(\S)\s+(.*?)\s*$")
                    .unwrap();
        }

        let caps = RE.captures(s).ok_or(Error::new(ErrorKind::InvalidSymbol))?;

        let addr = u32::from_str_radix(caps.get(1).unwrap().as_str(), 16)
            .map_err(|_e| Error::new(ErrorKind::InvalidSymbol))?;
        let size = u32::from_str_radix(caps.get(2).unwrap().as_str(), 16)
            .map_err(|_e| Error::new(ErrorKind::InvalidSymbol))?;
        let sym_type = caps.get(3).unwrap().as_str().parse::<SymbolType>()
            .map_err(|_e| Error::new(ErrorKind::InvalidSymbol))?;
        let name = String::from(caps.get(4).unwrap().as_str());

        Ok(RawSymbol::new(addr, size, sym_type, name))
    }
}

impl TryFrom<&str> for RawSymbol {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        RawSymbol::from_str(s)
    }
}

#[derive(PartialEq, Debug)]
pub struct Symbol {
    pub addr: u32,
    pub size: u32,
    pub sym_type: SymbolType,
    pub mangled: String,
    pub demangled: String,
    pub lang: SymbolLang,
}

impl Symbol {
    pub fn new(addr: u32, size: u32, sym_type: SymbolType, mangled: String, demangled: String, lang: SymbolLang) -> Self {
        Symbol {
            addr,
            size,
            sym_type,
            mangled,
            demangled,
            lang,
        }
    }

    pub fn from_rawsymbols<T>(mangled: T, demangled: T) -> Result<Self, Error>
    where
        T: TryInto<RawSymbol>,
    {
        // TODO:
        // Check this again with the ? operator.
        //
        // Old comment:
        // Didn't get the `?` operator to work because of trait requirements
        // revolving around `SymbolParseError`.
        let mangled = match mangled.try_into() {
            Ok(mangled) => mangled,
            Err(_) => return Err(Error::new(ErrorKind::InvalidSymbol)),
        };

        let demangled = match demangled.try_into() {
            Ok(demangled) => demangled,
            Err(_) => return Err(Error::new(ErrorKind::InvalidSymbol)),
        };

        if (mangled.addr != demangled.addr)
            || (mangled.size != demangled.size)
            || (mangled.sym_type != demangled.sym_type)
        {
            return Err(Error::new(ErrorKind::InvalidSymbol));
        }

        Ok(Symbol {
            addr: mangled.addr,
            size: mangled.size,
            sym_type: mangled.sym_type,
            mangled: mangled.name,
            demangled: demangled.name,
            lang: SymbolLang::Any,
        })
    }

    // Maybe not needed...?
    pub fn from_rawsymbols_lang<T>(mangled: T, demangled: T, lang: SymbolLang) -> Result<Self, Error>
    where
        T: TryInto<RawSymbol>,
    {
        let mut s = Symbol::from_rawsymbols(mangled, demangled)?;
        s.lang = lang;
        Ok(s)
    }

    pub fn related(&self, other: &Symbol) -> bool {
        !((self.mangled != other.mangled)
            || (self.demangled != other.demangled)
            || (self.sym_type != other.sym_type)
            || (self.size != other.size))
    }
}

// TODO:
// Rewrite this struct with the builder pattern.
#[derive(Debug, Default)]
pub struct Guesser {
    lib_syms: Vec<Symbol>,
}

impl Guesser {
    pub fn new() -> Self {
        Default::default()
    }

    // FIXME:
    // The name implies that the method only adds a rust library. However, the
    // signature also requires the path to the NM executable.
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

        let mangled_str =
            std::str::from_utf8(&mangled_out.stdout).map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        let demangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg("--demangle")
            .arg(lib.as_ref())
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Nm).with(io_error))?;

        if !demangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let demangled_str =
            std::str::from_utf8(&demangled_out.stdout).map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        for (mangled, demangled) in mangled_str.lines().zip(demangled_str.lines()) {
            let s = match Symbol::from_rawsymbols_lang(mangled,demangled, SymbolLang::Rust) {
                Ok(s) => s,
                // TODO:
                // Differentiate between the various reasons for an error. Some
                // might be expected (e.g lines like "mulvdi3.o:") while others
                // should not fail and should inform the user.
                Err(_) => continue,
            };

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

    pub fn guess<T>(&self, mangled: T, demangled: T) -> Result<Symbol, Error>
    where
        T: TryInto<RawSymbol>,
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
