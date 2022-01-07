//! Differences between mangled C++ and Rust symbols:
//! - Hypothesis:
//!   All demangled Rust symbols start with the name of the crate or with a "<"
//!   for an implementation of a trait method.
//! - Hypothesis:
//!   All symbols containing "<...>" are Rust symbols. I.e., C++ symbols never
//!   contain these symbols.
//!
//!   Refuted:
//!   ot::AddressResolver::FindCacheEntry(ot::Ip6::Address const&, ot::LinkedList<ot::AddressResolver::CacheEntry>*&, ot::AddressResolver::CacheEntry*&)
//! - Hypothesis: "<" and ">" symbols in Rust mangled names are represented as
//!   $LT$" and "$GT$". C++ seems to NOT do that...
//! - Observation:
//!   Mangled:
//!   "_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17hfae376f5993c24d7E"
//!   rustc_demangle:
//!   core::fmt::num::<impl core::fmt::LowerHex for usize>::fmt::hfae376f5993c24d7
//!   cpp_demangle:
//!   core::fmt::num::_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$::fmt::hfae376f5993c24d7
//!
//!   If both succeed, check for invalid characters. I.e., "$LT$" for C++ as
//!   "Mangled names containing $ or . are reserved for private implementation
//!   use. Names produced using such extensions are inherently non-portable and
//!   should be given internal linkage where possible."
//!
//! - Check if the following panic functions are from Zephyr or Rust's panic handler:
//!   "00029a84 00000052 t panic"
//!   "00002a6c 00000014 t panic"
//!   Both are Zephyr functions! The actual panic handler is called
//!   "rust_begin_unwind". I don't know where the name is coming from.
//! - Hypothesis:
//!   In case both demanglers output something, it is probably a Rust symbol and
//!   not a C++ symbol as the Rust demangler seems more strict.
//!
//! Rust Mangling RFC:
//! https://rust-lang.github.io/rfcs/2603-rust-symbol-name-mangling-v0.html
//!
//! C++ Mangling Reference:
//! https://itanium-cxx-abi.github.io/cxx-abi/abi.html#mangling
//!

use lazy_static::lazy_static;
use regex::Regex;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt;
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

impl SymbolType {
    pub fn mem_region(&self) -> MemoryRegion {
        match *self {
            Self::TextSection | Self::Weak => MemoryRegion::Rom,
            Self::BssSection | Self::DataSection | Self::ReadOnlyDataSection => MemoryRegion::Ram,
            _ => panic!("The memory region for a symbol of type {:?} is unknown!", self),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SymbolLang {
    Any,
    Rust,
    C,
    Cpp,
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
    type Err = SymbolParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^\s*([0-9a-fA-F]{8})\s+([0-9a-fA-F]{8})\s+(\S)\s+(.*?)\s*$")
                    .unwrap();
        }

        let caps = RE.captures(s).ok_or(SymbolParseError(()))?;

        let addr = u32::from_str_radix(caps.get(1).unwrap().as_str(), 16)
            .map_err(|_e| SymbolParseError(()))?;
        let size = u32::from_str_radix(caps.get(2).unwrap().as_str(), 16)
            .map_err(|_e| SymbolParseError(()))?;

        let sym_type = match caps.get(3).unwrap().as_str() {
            "A" => SymbolType::Absolute,
            "B" | "b" => SymbolType::BssSection,
            "C" | "c" => SymbolType::Common,
            "D" | "d" => SymbolType::DataSection,
            "G" | "g" => SymbolType::Global,
            "I" => SymbolType::Indirect,
            "i" => SymbolType::IndirectFunction,
            "N" => SymbolType::Debug,
            "n" => SymbolType::ReadOnlyDataSection,
            "p" => SymbolType::StackUnwindSection,
            "R" | "r" => SymbolType::ReadOnlyDataSection,
            "S" | "s" => SymbolType::UninitializedOrZeroInitialized,
            "T" | "t" => SymbolType::TextSection,
            "U" => SymbolType::Undefined,
            "u" => SymbolType::UniqueGlobal,
            "V" | "v" => SymbolType::TaggedWeak,
            "W" | "w" => SymbolType::Weak,
            "-" => SymbolType::Stabs,
            "?" => SymbolType::Unknown,
            _ => return Err(SymbolParseError(())),
        };

        let name = String::from(caps.get(4).unwrap().as_str());
        Ok(RawSymbol::new(addr, size, sym_type, name))
    }
}

impl TryFrom<&str> for RawSymbol {
    type Error = SymbolParseError;

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

    pub fn from_rawsymbols<T>(mangled: T, demangled: T) -> Result<Self, SymbolParseError>
    where
        T: TryInto<RawSymbol>,
    {
        // Didn't get the `?` operator to work because of trait requirements
        // revolving around `SymbolParseError`.
        let mangled = match mangled.try_into() {
            Ok(mangled) => mangled,
            Err(_) => return Err(SymbolParseError(())),
        };

        let demangled = match demangled.try_into() {
            Ok(demangled) => demangled,
            Err(_) => return Err(SymbolParseError(())),
        };

        if (mangled.addr != demangled.addr)
            || (mangled.size != demangled.size)
            || (mangled.sym_type != demangled.sym_type)
        {
            return Err(SymbolParseError(()));
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
    pub fn from_rawsymbols_lang<T>(mangled: T, demangled: T, lang: SymbolLang) -> Result<Self, SymbolParseError>
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

    pub fn add_rust_lib<T, U>(&mut self, nm: T, lib: U) -> Result<(), SymbolParseError>
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
    {
        let mangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg(lib.as_ref())
            .output()
            .map_err(|_e| SymbolParseError(()))?;

        if !mangled_out.status.success() {
            return Err(SymbolParseError(()));
        }

        let mangled_str =
            std::str::from_utf8(&mangled_out.stdout).map_err(|_e| SymbolParseError(()))?;

        let demangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg("--demangle")
            .arg(lib.as_ref())
            .output()
            .map_err(|_e| SymbolParseError(()))?;

        if !demangled_out.status.success() {
            return Err(SymbolParseError(()));
        }

        let demangled_str =
            std::str::from_utf8(&demangled_out.stdout).map_err(|_e| SymbolParseError(()))?;

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

    pub fn guess<T>(&self, mangled: T, demangled: T) -> Result<Symbol, SymbolParseError>
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

// TODO:
// Rewrite this as a generic atlas error and implement an ErrorKind enum for
// specifying the type of error (including an error message).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolParseError(());

impl Error for SymbolParseError {}

impl fmt::Display for SymbolParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid symbol syntax")
    }
}
